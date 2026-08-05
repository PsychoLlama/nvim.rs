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
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISlower: C2Rust_Unnamed = 512;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const DO_NOT_FREE_CNT: C2Rust_Unnamed_15 = 1073741823;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
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
    if !tv.is_null() {
        memset(
            tv as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<typval_T>(),
        );
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
static vimvars: GlobalCell<[vimvar; 106]> = GlobalCell::new([
    vimvar {
        vv_name: b"count\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"count1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"prevcount\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"errmsg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"warningmsg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"statusmsg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"shell_error\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"this_session\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"version\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: (1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"lnum\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 4 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"termrequest\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"termresponse\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"fname\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"lang\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"lc_time\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"ctype\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"charconvert_from\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"charconvert_to\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"fname_in\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"fname_out\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"fname_new\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"fname_diff\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"cmdarg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"foldstart\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 4 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"foldend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 4 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"folddashes\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 4 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"foldlevel\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 4 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"progname\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"servername\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"dying\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"exception\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"throwpoint\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"register\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"cmdbang\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"insertmode\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"val\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"key\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"profiling\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"fcs_reason\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"fcs_choice\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"beval_bufnr\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"beval_winnr\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"beval_winid\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"beval_lnum\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"beval_col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"beval_text\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"scrollstart\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"swapname\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"swapchoice\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"swapcommand\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"char\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"mouse_win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"mouse_winid\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"mouse_lnum\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"mouse_col\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"operator\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"searchforward\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"hlsearch\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"oldfiles\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"windowid\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 4 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"progpath\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"completed_item\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_DICT,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"option_new\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"option_old\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"option_oldlocal\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"option_oldglobal\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"option_command\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"option_type\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"errors\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"false\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_BOOL,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"true\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_BOOL,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"null\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_SPECIAL,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"numbermax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"numbermin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"numbersize\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"vim_did_enter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"testing\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 0 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"t_number\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"t_string\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"t_func\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"t_list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"t_dict\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"t_float\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"t_bool\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"t_blob\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_DICT,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"versionlong\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"echospace\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"argf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"argv\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"collate\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"exiting\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"maxcol\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"stacktrace\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"vim_did_init\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"stderr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"msgpack_types\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_DICT,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"_null_string\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"_null_list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"_null_dict\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_DICT,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"_null_blob\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_BLOB,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"lua\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_PARTIAL,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"relnum\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"virtnum\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"starttime\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
    vimvar {
        vv_name: b"exitreason\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0 as uint8_t,
            di_key: [
                0 as ::core::ffi::c_char,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        },
        vv_flags: 2 as ::core::ffi::c_char,
    },
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
pub unsafe extern "C" fn evalvars_init() {
    init_var_dict(get_globvar_dict(), globvars_var.ptr(), VAR_DEF_SCOPE);
    init_var_dict(vimvardict.ptr(), vimvars_var.ptr(), VAR_SCOPE);
    (*vimvardict.ptr()).dv_lock = VAR_FIXED;
    hash_init(compat_hashtab.ptr());
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[vimvar; 106]>()
        .wrapping_div(::core::mem::size_of::<vimvar>())
        .wrapping_div(
            (::core::mem::size_of::<[vimvar; 106]>().wrapping_rem(::core::mem::size_of::<vimvar>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        let mut p: *mut vimvar = (vimvars.ptr() as *mut vimvar).offset(i as isize) as *mut vimvar;
        '_c2rust_label: {
            if strlen((*p).vv_name) <= 16 as size_t {
            } else {
                __assert_fail(
                    b"strlen(p->vv_name) <= VIMVAR_KEY_LEN\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/eval/vars.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    268 as ::core::ffi::c_uint,
                    b"void evalvars_init(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        strcpy(
            &raw mut (*p).vv_di.di_key as *mut ::core::ffi::c_char,
            (*p).vv_name,
        );
        if (*p).vv_flags as ::core::ffi::c_int & VV_RO != 0 {
            (*p).vv_di.di_flags =
                (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
        } else if (*p).vv_flags as ::core::ffi::c_int & VV_RO_SBX != 0 {
            (*p).vv_di.di_flags = (DI_FLAGS_RO_SBX as ::core::ffi::c_int
                | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
        } else {
            (*p).vv_di.di_flags = DI_FLAGS_FIX as ::core::ffi::c_int as uint8_t;
        }
        if (*p).vv_di.di_tv.v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            hash_add(
                &raw mut (*vimvardict.ptr()).dv_hashtab,
                &raw mut (*p).vv_di.di_key as *mut ::core::ffi::c_char,
            );
        }
        if (*p).vv_flags as ::core::ffi::c_int & VV_COMPAT != 0 {
            hash_add(
                compat_hashtab.ptr(),
                &raw mut (*p).vv_di.di_key as *mut ::core::ffi::c_char,
            );
        }
        i = i.wrapping_add(1);
    }
    let vim_version: ::core::ffi::c_int = min_vim_version();
    set_vim_var_nr(VV_VERSION, vim_version as varnumber_T);
    set_vim_var_nr(
        VV_VERSIONLONG,
        (vim_version * 10000 as ::core::ffi::c_int + highest_patch()) as varnumber_T,
    );
    let msgpack_types_dict: *mut dict_T = tv_dict_alloc();
    let mut i_0: size_t = 0 as size_t;
    while i_0
        < ::core::mem::size_of::<[*const ::core::ffi::c_char; 8]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 8]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let type_list: *mut list_T = tv_list_alloc(0 as ptrdiff_t);
        tv_list_set_lock(type_list, VAR_FIXED);
        tv_list_ref(type_list);
        let di: *mut dictitem_T = tv_dict_item_alloc((*msgpack_type_names.ptr())[i_0 as usize]);
        (*di).di_flags = ((*di).di_flags as ::core::ffi::c_int
            | (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int))
            as uint8_t;
        (*di).di_tv = typval_T {
            v_type: VAR_LIST,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_list: type_list },
        };
        (*eval_msgpack_type_lists.ptr())[i_0 as usize] = type_list;
        if tv_dict_add(msgpack_types_dict, di) == FAIL {
            abort();
        }
        i_0 = i_0.wrapping_add(1);
    }
    (*msgpack_types_dict).dv_lock = VAR_FIXED;
    set_vim_var_dict(VV_MSGPACK_TYPES, msgpack_types_dict);
    set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED));
    set_vim_var_dict(VV_EVENT, tv_dict_alloc_lock(VAR_FIXED));
    set_vim_var_list(
        VV_ERRORS,
        tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t),
    );
    set_vim_var_nr(VV_STDERR, CHAN_STDERR as varnumber_T);
    set_vim_var_nr(VV_SEARCHFORWARD, 1 as varnumber_T);
    set_vim_var_nr(VV_HLSEARCH, 1 as varnumber_T);
    set_vim_var_nr(VV_COUNT1, 1 as varnumber_T);
    set_vim_var_special(VV_EXITING, kSpecialVarNull);
    set_vim_var_nr(
        VV_TYPE_NUMBER,
        VAR_TYPE_NUMBER as ::core::ffi::c_int as varnumber_T,
    );
    set_vim_var_nr(
        VV_TYPE_STRING,
        VAR_TYPE_STRING as ::core::ffi::c_int as varnumber_T,
    );
    set_vim_var_nr(
        VV_TYPE_FUNC,
        VAR_TYPE_FUNC as ::core::ffi::c_int as varnumber_T,
    );
    set_vim_var_nr(
        VV_TYPE_LIST,
        VAR_TYPE_LIST as ::core::ffi::c_int as varnumber_T,
    );
    set_vim_var_nr(
        VV_TYPE_DICT,
        VAR_TYPE_DICT as ::core::ffi::c_int as varnumber_T,
    );
    set_vim_var_nr(
        VV_TYPE_FLOAT,
        VAR_TYPE_FLOAT as ::core::ffi::c_int as varnumber_T,
    );
    set_vim_var_nr(
        VV_TYPE_BOOL,
        VAR_TYPE_BOOL as ::core::ffi::c_int as varnumber_T,
    );
    set_vim_var_nr(
        VV_TYPE_BLOB,
        VAR_TYPE_BLOB as ::core::ffi::c_int as varnumber_T,
    );
    set_vim_var_bool(VV_FALSE, kBoolVarFalse);
    set_vim_var_bool(VV_TRUE, kBoolVarTrue);
    set_vim_var_special(VV_NULL, kSpecialVarNull);
    set_vim_var_nr(VV_NUMBERMAX, VARNUMBER_MAX as varnumber_T);
    set_vim_var_nr(VV_NUMBERMIN, VARNUMBER_MIN as varnumber_T);
    set_vim_var_nr(
        VV_NUMBERSIZE,
        ::core::mem::size_of::<varnumber_T>().wrapping_mul(8 as usize) as varnumber_T,
    );
    set_vim_var_nr(VV_MAXCOL, MAXCOL as ::core::ffi::c_int as varnumber_T);
    set_vim_var_nr(
        VV_ECHOSPACE,
        (sc_col.get() - 1 as ::core::ffi::c_int) as varnumber_T,
    );
    let mut vvlua_partial: *mut partial_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
    (*vvlua_partial).pt_name = xmallocz(0 as size_t) as *mut ::core::ffi::c_char;
    (*vvlua_partial).pt_refcount += 1;
    set_vim_var_partial(VV_LUA, vvlua_partial);
    set_reg_var(0 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn garbage_collect_globvars(
    mut copyID: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return set_ref_in_ht(
        &raw mut (*globvardict.ptr()).dv_hashtab,
        copyID,
        ::core::ptr::null_mut::<*mut list_stack_T>(),
    ) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn garbage_collect_vimvars(mut copyID: ::core::ffi::c_int) -> bool {
    return set_ref_in_ht(
        &raw mut (*vimvardict.ptr()).dv_hashtab,
        copyID,
        ::core::ptr::null_mut::<*mut list_stack_T>(),
    );
}
pub unsafe extern "C" fn garbage_collect_scriptvars(mut copyID: ::core::ffi::c_int) -> bool {
    let mut abort_0: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i <= (*script_items.ptr()).ga_len {
        abort_0 = abort_0 as ::core::ffi::c_int != 0
            || set_ref_in_ht(
                &raw mut (*(**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
                    .offset((i - 1 as ::core::ffi::c_int) as isize))
                .sn_vars)
                    .sv_dict
                    .dv_hashtab,
                copyID,
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        i += 1;
    }
    return abort_0;
}
pub unsafe extern "C" fn set_internal_string_var(
    mut name: *const ::core::ffi::c_char,
    mut value: *mut ::core::ffi::c_char,
) {
    let mut tv: typval_T = typval_T {
        v_type: VAR_STRING,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_string: value },
    };
    set_var(name, strlen(name), &raw mut tv, true_0 != 0);
}
pub unsafe extern "C" fn eval_charconvert(
    enc_from: *const ::core::ffi::c_char,
    enc_to: *const ::core::ffi::c_char,
    fname_from: *const ::core::ffi::c_char,
    fname_to: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let saved_sctx: sctx_T = current_sctx.get();
    set_vim_var_string(VV_CC_FROM, enc_from, -1 as ptrdiff_t);
    set_vim_var_string(VV_CC_TO, enc_to, -1 as ptrdiff_t);
    set_vim_var_string(VV_FNAME_IN, fname_from, -1 as ptrdiff_t);
    set_vim_var_string(VV_FNAME_OUT, fname_to, -1 as ptrdiff_t);
    let mut ctx: *mut sctx_T = get_option_sctx(kOptCharconvert);
    if !ctx.is_null() {
        current_sctx.set(*ctx);
    }
    let mut err: bool = false_0 != 0;
    if eval_to_bool(
        p_ccv.get(),
        &raw mut err,
        ::core::ptr::null_mut::<exarg_T>(),
        false_0 != 0,
        true_0 != 0,
    ) {
        err = true_0 != 0;
    }
    set_vim_var_string(
        VV_CC_FROM,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_CC_TO,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_FNAME_IN,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_FNAME_OUT,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    current_sctx.set(saved_sctx);
    if err {
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn eval_diff(
    origfile: *const ::core::ffi::c_char,
    newfile: *const ::core::ffi::c_char,
    outfile: *const ::core::ffi::c_char,
) {
    let saved_sctx: sctx_T = current_sctx.get();
    set_vim_var_string(VV_FNAME_IN, origfile, -1 as ptrdiff_t);
    set_vim_var_string(VV_FNAME_NEW, newfile, -1 as ptrdiff_t);
    set_vim_var_string(VV_FNAME_OUT, outfile, -1 as ptrdiff_t);
    let mut ctx: *mut sctx_T = get_option_sctx(kOptDiffexpr);
    if !ctx.is_null() {
        current_sctx.set(*ctx);
    }
    let mut tv: *mut typval_T =
        eval_expr_ext(p_dex.get(), ::core::ptr::null_mut::<exarg_T>(), true_0 != 0);
    tv_free(tv);
    set_vim_var_string(
        VV_FNAME_IN,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_FNAME_NEW,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_FNAME_OUT,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    current_sctx.set(saved_sctx);
}
pub unsafe extern "C" fn eval_patch(
    origfile: *const ::core::ffi::c_char,
    difffile: *const ::core::ffi::c_char,
    outfile: *const ::core::ffi::c_char,
) {
    let saved_sctx: sctx_T = current_sctx.get();
    set_vim_var_string(VV_FNAME_IN, origfile, -1 as ptrdiff_t);
    set_vim_var_string(VV_FNAME_DIFF, difffile, -1 as ptrdiff_t);
    set_vim_var_string(VV_FNAME_OUT, outfile, -1 as ptrdiff_t);
    let mut ctx: *mut sctx_T = get_option_sctx(kOptPatchexpr);
    if !ctx.is_null() {
        current_sctx.set(*ctx);
    }
    let mut tv: *mut typval_T =
        eval_expr_ext(p_pex.get(), ::core::ptr::null_mut::<exarg_T>(), true_0 != 0);
    tv_free(tv);
    set_vim_var_string(
        VV_FNAME_IN,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_FNAME_DIFF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_FNAME_OUT,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    current_sctx.set(saved_sctx);
}
pub unsafe extern "C" fn eval_spell_expr(
    mut badword: *mut ::core::ffi::c_char,
    mut expr: *mut ::core::ffi::c_char,
) -> *mut list_T {
    let mut save_val: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut list: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut p: *mut ::core::ffi::c_char = skipwhite(expr);
    let saved_sctx: sctx_T = current_sctx.get();
    prepare_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    set_vim_var_string(VV_VAL, badword, -1 as ptrdiff_t);
    if p_verbose.get() == 0 as OptInt {
        (*emsg_off.ptr()) += 1;
    }
    let mut ctx: *mut sctx_T = get_option_sctx(kOptSpellsuggest);
    if !ctx.is_null() {
        current_sctx.set(*ctx);
    }
    let mut r: ::core::ffi::c_int = may_call_simple_func(p, &raw mut rettv);
    if r == NOTDONE {
        r = eval1(&raw mut p, &raw mut rettv, EVALARG_EVALUATE.ptr());
    }
    if r == OK {
        if rettv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_clear(&raw mut rettv);
        } else {
            list = rettv.vval.v_list;
        }
    }
    if p_verbose.get() == 0 as OptInt {
        (*emsg_off.ptr()) -= 1;
    }
    tv_clear(get_vim_var_tv(VV_VAL));
    restore_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
    current_sctx.set(saved_sctx);
    return list;
}
pub unsafe extern "C" fn get_spellword(
    list: *mut list_T,
    mut ret_word: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if tv_list_len(list) != 2 as ::core::ffi::c_int {
        emsg(gettext(
            b"E5700: Expression from 'spellsuggest' must yield lists with exactly two values\0"
                .as_ptr() as *const ::core::ffi::c_char,
        ));
        return -1 as ::core::ffi::c_int;
    }
    *ret_word = tv_list_find_str(list, 0 as ::core::ffi::c_int);
    if (*ret_word).is_null() {
        return -1 as ::core::ffi::c_int;
    }
    return tv_list_find_nr(
        list,
        -1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn prepare_vimvar(mut idx: ::core::ffi::c_int, mut save_tv: *mut typval_T) {
    *save_tv = (*vimvars.ptr())[idx as usize].vv_di.di_tv;
    (*vimvars.ptr())[idx as usize].vv_di.di_tv.vval.v_string =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*vimvars.ptr())[idx as usize].vv_di.di_tv.v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        hash_add(
            &raw mut (*vimvardict.ptr()).dv_hashtab,
            &raw mut (*(vimvars.ptr() as *mut vimvar).offset(idx as isize))
                .vv_di
                .di_key as *mut ::core::ffi::c_char,
        );
    }
}
pub unsafe extern "C" fn restore_vimvar(mut idx: ::core::ffi::c_int, mut save_tv: *mut typval_T) {
    (*vimvars.ptr())[idx as usize].vv_di.di_tv = *save_tv;
    if (*vimvars.ptr())[idx as usize].vv_di.di_tv.v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    let mut hi: *mut hashitem_T = hash_find(
        &raw mut (*vimvardict.ptr()).dv_hashtab,
        &raw mut (*(vimvars.ptr() as *mut vimvar).offset(idx as isize))
            .vv_di
            .di_key as *mut ::core::ffi::c_char,
    );
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
    {
        internal_error(b"restore_vimvar()\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        hash_remove(&raw mut (*vimvardict.ptr()).dv_hashtab, hi);
    };
}
unsafe extern "C" fn list_vim_vars(mut first: *mut ::core::ffi::c_int) {
    list_hashtable_vars(
        &raw mut (*vimvardict.ptr()).dv_hashtab,
        b"v:\0".as_ptr() as *const ::core::ffi::c_char,
        false_0,
        first,
    );
}
unsafe extern "C" fn list_script_vars(mut first: *mut ::core::ffi::c_int) {
    if (*current_sctx.ptr()).sc_sid > 0 as ::core::ffi::c_int
        && (*current_sctx.ptr()).sc_sid <= (*script_items.ptr()).ga_len
    {
        list_hashtable_vars(
            &raw mut (*(**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T).offset(
                ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    as isize,
            ))
            .sn_vars)
                .sv_dict
                .dv_hashtab,
            b"s:\0".as_ptr() as *const ::core::ffi::c_char,
            false_0,
            first,
        );
    }
}
pub unsafe extern "C" fn eval_one_expr_in_str(
    mut p: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
    mut evaluate: bool,
) -> *mut ::core::ffi::c_char {
    let mut block_start: *mut ::core::ffi::c_char =
        skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
    let mut block_end: *mut ::core::ffi::c_char = block_start;
    if *block_start as ::core::ffi::c_int == NUL {
        semsg(
            gettext(&raw const e_missing_close_curly_str as *const ::core::ffi::c_char),
            p,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if skip_expr(&raw mut block_end, ::core::ptr::null_mut::<evalarg_T>()) == FAIL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    block_end = skipwhite(block_end);
    if *block_end as ::core::ffi::c_int != '}' as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_missing_close_curly_str as *const ::core::ffi::c_char),
            p,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if evaluate {
        *block_end = NUL as ::core::ffi::c_char;
        let mut expr_val: *mut ::core::ffi::c_char =
            eval_to_string(block_start, false_0 != 0, false_0 != 0);
        *block_end = '}' as ::core::ffi::c_char;
        if expr_val.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        ga_concat(gap, expr_val);
        xfree(expr_val as *mut ::core::ffi::c_void);
    }
    return block_end.offset(1 as ::core::ffi::c_int as isize);
}
unsafe extern "C" fn eval_all_expr_in_str(
    mut str: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let mut p: *mut ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL {
        let mut escaped_brace: bool = false_0 != 0;
        let mut lit_start: *mut ::core::ffi::c_char = p;
        while *p as ::core::ffi::c_int != '{' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != '}' as ::core::ffi::c_int
            && *p as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int != NUL
            && *p as ::core::ffi::c_int
                == *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            p = p.offset(1);
            escaped_brace = true_0 != 0;
        } else if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_stray_closing_curly_str as *const ::core::ffi::c_char),
                str,
            );
            ga_clear(&raw mut ga);
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        ga_concat_len(&raw mut ga, lit_start, p.offset_from(lit_start) as size_t);
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
        if escaped_brace {
            p = p.offset(1);
        } else {
            p = eval_one_expr_in_str(p, &raw mut ga, true_0 != 0);
            if p.is_null() {
                ga_clear(&raw mut ga);
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        }
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn heredoc_get(
    mut eap: *mut exarg_T,
    mut cmd: *mut ::core::ffi::c_char,
    mut script_get: bool,
) -> *mut list_T {
    let mut marker: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut marker_indent_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut text_indent_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut text_indent: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dot: [::core::ffi::c_char; 2] =
        ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b".\0");
    let mut heredoc_in_string: bool = false_0 != 0;
    let mut line_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut nl_ptr: *mut ::core::ffi::c_char = vim_strchr(cmd, '\n' as ::core::ffi::c_int);
    if !nl_ptr.is_null() {
        heredoc_in_string = true_0 != 0;
        line_arg = nl_ptr.offset(1 as ::core::ffi::c_int as isize);
        *nl_ptr = NUL as ::core::ffi::c_char;
    } else if (*eap).ea_getline.is_none() {
        emsg(gettext(
            (e_cannot_use_heredoc_here.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return ::core::ptr::null_mut::<list_T>();
    }
    cmd = skipwhite(cmd);
    let mut evalstr: bool = false_0 != 0;
    let mut eval_failed: bool = false_0 != 0;
    loop {
        if strncmp(
            cmd,
            b"trim\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
            && (*cmd.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || ascii_iswhite(*cmd.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
        {
            cmd = skipwhite(cmd.offset(4 as ::core::ffi::c_int as isize));
            let mut p: *mut ::core::ffi::c_char = *(*eap).cmdlinep;
            while ascii_iswhite(*p as ::core::ffi::c_int) {
                p = p.offset(1);
                marker_indent_len += 1;
            }
            text_indent_len = -1 as ::core::ffi::c_int;
        } else {
            if !(strncmp(
                cmd,
                b"eval\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
                && (*cmd.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || ascii_iswhite(
                        *cmd.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0))
            {
                break;
            }
            cmd = skipwhite(cmd.offset(4 as ::core::ffi::c_int as isize));
            evalstr = true_0 != 0;
        }
    }
    let comment_char: ::core::ffi::c_char = '"' as ::core::ffi::c_char;
    if *cmd as ::core::ffi::c_int != NUL
        && *cmd as ::core::ffi::c_int != comment_char as ::core::ffi::c_int
    {
        marker = skipwhite(cmd);
        let mut p_0: *mut ::core::ffi::c_char = skiptowhite(marker);
        if *skipwhite(p_0) as ::core::ffi::c_int != NUL
            && *skipwhite(p_0) as ::core::ffi::c_int != comment_char as ::core::ffi::c_int
        {
            semsg(
                gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                p_0,
            );
            return ::core::ptr::null_mut::<list_T>();
        }
        *p_0 = NUL as ::core::ffi::c_char;
        if !script_get
            && *(*__ctype_b_loc()).offset(*marker as uint8_t as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISlower as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
        {
            emsg(gettext(
                b"E221: Marker cannot start with lower case letter\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return ::core::ptr::null_mut::<list_T>();
        }
    } else if script_get {
        marker = &raw mut dot as *mut ::core::ffi::c_char;
    } else {
        emsg(gettext(
            b"E172: Missing marker\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return ::core::ptr::null_mut::<list_T>();
    }
    let mut theline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut l: *mut list_T = tv_list_alloc(0 as ptrdiff_t);
    loop {
        let mut mi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut ti: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if heredoc_in_string {
            if *line_arg as ::core::ffi::c_int == NUL {
                if !script_get {
                    semsg(
                        gettext(
                            (e_missing_end_marker_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        marker,
                    );
                }
                break;
            } else {
                theline = line_arg;
                let mut next_line: *mut ::core::ffi::c_char =
                    vim_strchr(theline, '\n' as ::core::ffi::c_int);
                if next_line.is_null() {
                    line_arg = line_arg.offset(strlen(line_arg) as isize);
                } else {
                    *next_line = NUL as ::core::ffi::c_char;
                    line_arg = next_line.offset(1 as ::core::ffi::c_int as isize);
                }
            }
        } else {
            xfree(theline as *mut ::core::ffi::c_void);
            theline = (*eap).ea_getline.expect("non-null function pointer")(
                NUL,
                (*eap).cookie,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
            if theline.is_null() {
                if !script_get {
                    semsg(
                        gettext(
                            (e_missing_end_marker_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        marker,
                    );
                }
                break;
            }
        }
        if marker_indent_len > 0 as ::core::ffi::c_int
            && strncmp(theline, *(*eap).cmdlinep, marker_indent_len as size_t)
                == 0 as ::core::ffi::c_int
        {
            mi = marker_indent_len;
        }
        if strcmp(marker, theline.offset(mi as isize)) == 0 as ::core::ffi::c_int {
            break;
        }
        if eval_failed {
            continue;
        }
        if text_indent_len == -1 as ::core::ffi::c_int && *theline as ::core::ffi::c_int != NUL {
            let mut p_1: *mut ::core::ffi::c_char = theline;
            text_indent_len = 0 as ::core::ffi::c_int;
            while ascii_iswhite(*p_1 as ::core::ffi::c_int) {
                p_1 = p_1.offset(1);
                text_indent_len += 1;
            }
            text_indent = xmemdupz(
                theline as *const ::core::ffi::c_void,
                text_indent_len as size_t,
            ) as *mut ::core::ffi::c_char;
        }
        if !text_indent.is_null() {
            ti = 0 as ::core::ffi::c_int;
            while ti < text_indent_len {
                if *theline.offset(ti as isize) as ::core::ffi::c_int
                    != *text_indent.offset(ti as isize) as ::core::ffi::c_int
                {
                    break;
                }
                ti += 1;
            }
        }
        let mut str: *mut ::core::ffi::c_char = theline.offset(ti as isize);
        if evalstr as ::core::ffi::c_int != 0 && (*eap).skip == 0 {
            str = eval_all_expr_in_str(str);
            if str.is_null() {
                eval_failed = true_0 != 0;
            } else {
                tv_list_append_allocated_string(l, str);
            }
        } else {
            tv_list_append_string(l, str, -1 as ssize_t);
        }
    }
    if heredoc_in_string {
        (*eap).nextcmd = line_arg;
    } else {
        xfree(theline as *mut ::core::ffi::c_void);
    }
    xfree(text_indent as *mut ::core::ffi::c_void);
    if eval_failed {
        tv_list_free(l);
        return ::core::ptr::null_mut::<list_T>();
    }
    return l;
}
pub unsafe fn ex_let(mut eap: *mut exarg_T) {
    let is_const: bool = (*eap).cmdidx as ::core::ffi::c_int == CMD_const as ::core::ffi::c_int;
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut expr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut var_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut semicolon: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut op: [::core::ffi::c_char; 2] = [0; 2];
    let mut argend: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut first: ::core::ffi::c_int = true_0;
    argend = skip_var_list(arg, &raw mut var_count, &raw mut semicolon, false_0 != 0);
    if argend.is_null() {
        return;
    }
    expr = skipwhite(argend);
    let mut concat: bool = strncmp(
        expr,
        b"..=\0".as_ptr() as *const ::core::ffi::c_char,
        3 as size_t,
    ) == 0 as ::core::ffi::c_int;
    let mut has_assign: bool = *expr as ::core::ffi::c_int == '=' as ::core::ffi::c_int
        || !vim_strchr(
            b"+-*/%.\0".as_ptr() as *const ::core::ffi::c_char,
            *expr as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
            && *expr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int;
    if !has_assign && !concat {
        if *arg as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else if ends_excmd(*arg as ::core::ffi::c_int) == 0 {
            arg = list_arg_vars(eap, arg, &raw mut first) as *mut ::core::ffi::c_char;
        } else if (*eap).skip == 0 {
            list_glob_vars(&raw mut first);
            list_buf_vars(&raw mut first);
            list_win_vars(&raw mut first);
            list_tab_vars(&raw mut first);
            list_script_vars(&raw mut first);
            list_func_vars(&raw mut first);
            list_vim_vars(&raw mut first);
        }
        (*eap).nextcmd = check_nextcmd(arg);
        return;
    }
    if *expr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '=' as ::core::ffi::c_int
        && *expr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '<' as ::core::ffi::c_int
        && *expr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '<' as ::core::ffi::c_int
    {
        let mut l: *mut list_T = heredoc_get(
            eap,
            expr.offset(3 as ::core::ffi::c_int as isize),
            false_0 != 0,
        );
        if !l.is_null() {
            tv_list_set_ret(&raw mut rettv, l);
            if (*eap).skip == 0 {
                op[0 as ::core::ffi::c_int as usize] = '=' as ::core::ffi::c_char;
                op[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                ex_let_vars(
                    (*eap).arg,
                    &raw mut rettv,
                    false_0,
                    semicolon,
                    var_count,
                    is_const as ::core::ffi::c_int,
                    &raw mut op as *mut ::core::ffi::c_char,
                );
            }
            tv_clear(&raw mut rettv);
        }
        return;
    }
    rettv.v_type = VAR_UNKNOWN;
    op[0 as ::core::ffi::c_int as usize] = '=' as ::core::ffi::c_char;
    op[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    if *expr as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
        if !vim_strchr(
            b"+-*/%.\0".as_ptr() as *const ::core::ffi::c_char,
            *expr as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            op[0 as ::core::ffi::c_int as usize] = *expr;
            if *expr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && *expr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
            {
                expr = expr.offset(1);
            }
        }
        expr = expr.offset(2 as ::core::ffi::c_int as isize);
    } else {
        expr = expr.offset(1 as ::core::ffi::c_int as isize);
    }
    expr = skipwhite(expr);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) += 1;
    }
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
    let mut eval_res: ::core::ffi::c_int = eval0(expr, &raw mut rettv, eap, &raw mut evalarg);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) -= 1;
    }
    clear_evalarg(&raw mut evalarg, eap);
    if (*eap).skip == 0 && eval_res != FAIL {
        ex_let_vars(
            (*eap).arg,
            &raw mut rettv,
            false_0,
            semicolon,
            var_count,
            is_const as ::core::ffi::c_int,
            &raw mut op as *mut ::core::ffi::c_char,
        );
    }
    if eval_res != FAIL {
        tv_clear(&raw mut rettv);
    }
}
pub unsafe extern "C" fn ex_let_vars(
    mut arg_start: *mut ::core::ffi::c_char,
    mut tv: *mut typval_T,
    mut copy: ::core::ffi::c_int,
    mut semicolon: ::core::ffi::c_int,
    mut var_count: ::core::ffi::c_int,
    mut is_const: ::core::ffi::c_int,
    mut op: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut arg: *mut ::core::ffi::c_char = arg_start;
    let mut ltv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if *arg as ::core::ffi::c_int != '[' as ::core::ffi::c_int {
        if ex_let_one(arg, tv, copy != 0, is_const != 0, op, op).is_null() {
            return FAIL;
        }
        return OK;
    }
    if (*tv).v_type as ::core::ffi::c_uint != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return FAIL;
    }
    let l: *mut list_T = (*tv).vval.v_list;
    let len: ::core::ffi::c_int = tv_list_len(l);
    if semicolon == 0 as ::core::ffi::c_int && var_count < len {
        emsg(gettext(
            b"E687: Less targets than List items\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return FAIL;
    }
    if var_count - semicolon > len {
        emsg(gettext(
            b"E688: More targets than List items\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return FAIL;
    }
    '_c2rust_label: {
        if !l.is_null() {
        } else {
            __assert_fail(
                b"l != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/vars.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1043 as ::core::ffi::c_uint,
                b"int ex_let_vars(char *, typval_T *, int, int, int, int, char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut item: *mut listitem_T = tv_list_first(l);
    let mut rest_len: size_t = tv_list_len(l) as size_t;
    while *arg as ::core::ffi::c_int != ']' as ::core::ffi::c_int {
        arg = skipwhite(arg.offset(1 as ::core::ffi::c_int as isize));
        arg = ex_let_one(
            arg,
            &raw mut (*item).li_tv,
            true_0 != 0,
            is_const != 0,
            b",;]\0".as_ptr() as *const ::core::ffi::c_char,
            op,
        );
        if arg.is_null() {
            return FAIL;
        }
        rest_len = rest_len.wrapping_sub(1);
        item = (*item).li_next;
        arg = skipwhite(arg);
        if *arg as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
            let rest_list: *mut list_T = tv_list_alloc(rest_len as ptrdiff_t);
            while !item.is_null() {
                tv_list_append_tv(rest_list, &raw mut (*item).li_tv);
                item = (*item).li_next;
            }
            ltv.v_type = VAR_LIST;
            ltv.v_lock = VAR_UNLOCKED;
            ltv.vval.v_list = rest_list;
            tv_list_ref(rest_list);
            arg = ex_let_one(
                skipwhite(arg.offset(1 as ::core::ffi::c_int as isize)),
                &raw mut ltv,
                false_0 != 0,
                is_const != 0,
                b"]\0".as_ptr() as *const ::core::ffi::c_char,
                op,
            );
            tv_clear(&raw mut ltv);
            if arg.is_null() {
                return FAIL;
            }
            break;
        } else if *arg as ::core::ffi::c_int != ',' as ::core::ffi::c_int
            && *arg as ::core::ffi::c_int != ']' as ::core::ffi::c_int
        {
            internal_error(b"ex_let_vars()\0".as_ptr() as *const ::core::ffi::c_char);
            return FAIL;
        }
    }
    return OK;
}
pub unsafe extern "C" fn skip_var_list(
    mut arg: *const ::core::ffi::c_char,
    mut var_count: *mut ::core::ffi::c_int,
    mut semicolon: *mut ::core::ffi::c_int,
    mut silent: bool,
) -> *const ::core::ffi::c_char {
    if *arg as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut p: *const ::core::ffi::c_char = arg;
        loop {
            p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
            s = skip_var_one(p);
            if s == p {
                if !silent {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        p,
                    );
                }
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            *var_count += 1;
            p = skipwhite(s);
            if *p as ::core::ffi::c_int == ']' as ::core::ffi::c_int {
                break;
            }
            if *p as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                if *semicolon == 1 as ::core::ffi::c_int {
                    if !silent {
                        emsg(gettext(
                            (e_double_semicolon_in_list_of_variables.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ));
                    }
                    return ::core::ptr::null::<::core::ffi::c_char>();
                }
                *semicolon = 1 as ::core::ffi::c_int;
            } else if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                if !silent {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        p,
                    );
                }
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
        }
        return p.offset(1 as ::core::ffi::c_int as isize);
    }
    return skip_var_one(arg);
}
unsafe extern "C" fn skip_var_one(
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int
        && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        return arg.offset(2 as ::core::ffi::c_int as isize);
    }
    return find_name_end(
        if *arg as ::core::ffi::c_int == '$' as ::core::ffi::c_int
            || *arg as ::core::ffi::c_int == '&' as ::core::ffi::c_int
        {
            arg.offset(1 as ::core::ffi::c_int as isize)
        } else {
            arg
        },
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        FNE_INCL_BR | FNE_CHECK_START,
    );
}
pub unsafe extern "C" fn list_hashtable_vars(
    mut ht: *mut hashtab_T,
    mut prefix: *const ::core::ffi::c_char,
    mut empty: ::core::ffi::c_int,
    mut first: *mut ::core::ffi::c_int,
) {
    let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut todo: ::core::ffi::c_int = 0;
    todo = (*ht).ht_used as ::core::ffi::c_int;
    hi = (*ht).ht_array;
    while todo > 0 as ::core::ffi::c_int && !got_int.get() {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            todo -= 1;
            di = (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
            let mut buf: [::core::ffi::c_char; 1025] = [0; 1025];
            xstrlcpy(
                &raw mut buf as *mut ::core::ffi::c_char,
                prefix,
                IOSIZE as size_t,
            );
            xstrlcat(
                &raw mut buf as *mut ::core::ffi::c_char,
                &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
            );
            if !message_filtered(&raw mut buf as *mut ::core::ffi::c_char) {
                if empty != 0
                    || (*di).di_tv.v_type as ::core::ffi::c_uint
                        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    || !(*di).di_tv.vval.v_string.is_null()
                {
                    list_one_var(di, prefix, first);
                }
            }
        }
        hi = hi.offset(1);
    }
}
unsafe extern "C" fn list_glob_vars(mut first: *mut ::core::ffi::c_int) {
    list_hashtable_vars(
        &raw mut (*globvardict.ptr()).dv_hashtab,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        true_0,
        first,
    );
}
unsafe extern "C" fn list_buf_vars(mut first: *mut ::core::ffi::c_int) {
    list_hashtable_vars(
        &raw mut (*(*curbuf.get()).b_vars).dv_hashtab,
        b"b:\0".as_ptr() as *const ::core::ffi::c_char,
        true_0,
        first,
    );
}
unsafe extern "C" fn list_win_vars(mut first: *mut ::core::ffi::c_int) {
    list_hashtable_vars(
        &raw mut (*(*curwin.get()).w_vars).dv_hashtab,
        b"w:\0".as_ptr() as *const ::core::ffi::c_char,
        true_0,
        first,
    );
}
unsafe extern "C" fn list_tab_vars(mut first: *mut ::core::ffi::c_int) {
    list_hashtable_vars(
        &raw mut (*(*curtab.get()).tp_vars).dv_hashtab,
        b"t:\0".as_ptr() as *const ::core::ffi::c_char,
        true_0,
        first,
    );
}
unsafe extern "C" fn list_arg_vars(
    mut eap: *mut exarg_T,
    mut arg: *const ::core::ffi::c_char,
    mut first: *mut ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut error: bool = false_0 != 0;
    let mut len: ::core::ffi::c_int = 0;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name_start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    while ends_excmd(*arg as ::core::ffi::c_int) == 0 && !got_int.get() {
        if error as ::core::ffi::c_int != 0 || (*eap).skip != 0 {
            arg = find_name_end(
                arg,
                ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                FNE_INCL_BR | FNE_CHECK_START,
            );
            if !ascii_iswhite(*arg as ::core::ffi::c_int)
                && ends_excmd(*arg as ::core::ffi::c_int) == 0
            {
                emsg_severe.set(true_0 != 0);
                semsg(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    arg,
                );
                break;
            }
        } else {
            name = arg;
            name_start = name;
            let mut tofree: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            len = get_name_len(&raw mut arg, &raw mut tofree, true_0 != 0, true_0 != 0);
            if len <= 0 as ::core::ffi::c_int {
                if len < 0 as ::core::ffi::c_int && !aborting() {
                    emsg_severe.set(true_0 != 0);
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        arg,
                    );
                    break;
                } else {
                    error = true_0 != 0;
                }
            } else {
                if !tofree.is_null() {
                    name = tofree;
                }
                if eval_variable(
                    name,
                    len,
                    &raw mut tv,
                    ::core::ptr::null_mut::<*mut dictitem_T>(),
                    true_0 != 0,
                    false_0 != 0,
                ) == FAIL
                {
                    error = true_0 != 0;
                } else {
                    let arg_subsc: *const ::core::ffi::c_char = arg;
                    if handle_subscript(
                        &raw mut arg,
                        &raw mut tv,
                        EVALARG_EVALUATE.ptr(),
                        true_0 != 0,
                    ) == FAIL
                    {
                        error = true_0 != 0;
                    } else {
                        if arg == arg_subsc
                            && len == 2 as ::core::ffi::c_int
                            && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == ':' as ::core::ffi::c_int
                        {
                            match *name as ::core::ffi::c_int {
                                103 => {
                                    list_glob_vars(first);
                                }
                                98 => {
                                    list_buf_vars(first);
                                }
                                119 => {
                                    list_win_vars(first);
                                }
                                116 => {
                                    list_tab_vars(first);
                                }
                                118 => {
                                    list_vim_vars(first);
                                }
                                115 => {
                                    list_script_vars(first);
                                }
                                108 => {
                                    list_func_vars(first);
                                }
                                _ => {
                                    semsg(
                                        gettext(b"E738: Can't list variables for %s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        name,
                                    );
                                }
                            }
                        } else {
                            let s: *mut ::core::ffi::c_char =
                                encode_tv2echo(&raw mut tv, ::core::ptr::null_mut::<size_t>());
                            let used_name: *const ::core::ffi::c_char =
                                if arg == arg_subsc { name } else { name_start };
                            '_c2rust_label: {
                                if !used_name.is_null() {
                                } else {
                                    __assert_fail(
                                        b"used_name != NULL\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/eval/vars.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        1266 as ::core::ffi::c_uint,
                                        b"const char *list_arg_vars(exarg_T *, const char *, int *)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            let name_size: ptrdiff_t =
                                if used_name == tofree as *const ::core::ffi::c_char {
                                    strlen(used_name) as ptrdiff_t
                                } else {
                                    arg.offset_from(used_name)
                                };
                            list_one_var_a(
                                b"\0".as_ptr() as *const ::core::ffi::c_char,
                                used_name,
                                name_size,
                                tv.v_type,
                                if s.is_null() {
                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                } else {
                                    s as *const ::core::ffi::c_char
                                },
                                first,
                            );
                            xfree(s as *mut ::core::ffi::c_void);
                        }
                        tv_clear(&raw mut tv);
                    }
                }
            }
            xfree(tofree as *mut ::core::ffi::c_void);
        }
        arg = skipwhite(arg);
    }
    return arg;
}
unsafe extern "C" fn ex_let_env(
    mut arg: *mut ::core::ffi::c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const ::core::ffi::c_char,
    op: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if is_const {
        emsg(gettext(
            b"E996: Cannot lock an environment variable\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    arg = arg.offset(1);
    let mut name: *mut ::core::ffi::c_char = arg;
    let mut len: ::core::ffi::c_int = get_env_len(&raw mut arg as *mut *const ::core::ffi::c_char);
    if len == 0 as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            name.offset(-(1 as ::core::ffi::c_int as isize)),
        );
    } else if !op.is_null()
        && !vim_strchr(
            b"+-*/%\0".as_ptr() as *const ::core::ffi::c_char,
            *op as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
    {
        semsg(
            gettext(&raw const e_letwrong as *const ::core::ffi::c_char),
            op,
        );
    } else if !endchars.is_null()
        && vim_strchr(endchars, *skipwhite(arg) as uint8_t as ::core::ffi::c_int).is_null()
    {
        emsg(gettext(e_letunexp.get()));
    } else if !check_secure() {
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let c1: ::core::ffi::c_char = *name.offset(len as isize);
        *name.offset(len as isize) = NUL as ::core::ffi::c_char;
        let mut p: *const ::core::ffi::c_char = tv_get_string_chk(tv);
        if !p.is_null() && !op.is_null() && *op as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            let mut s: *mut ::core::ffi::c_char = vim_getenv(name);
            if !s.is_null() {
                tofree = concat_str(s, p);
                p = tofree;
                xfree(s as *mut ::core::ffi::c_void);
            }
        }
        if !p.is_null() {
            vim_setenv_ext(name, p);
            arg_end = arg;
        }
        *name.offset(len as isize) = c1;
        xfree(tofree as *mut ::core::ffi::c_void);
    }
    return arg_end;
}
unsafe extern "C" fn ex_let_option(
    mut arg: *mut ::core::ffi::c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const ::core::ffi::c_char,
    op: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut error: bool = false;
    let mut is_num: bool = false;
    let mut is_string: bool = false;
    let mut err: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if is_const {
        emsg(gettext(
            b"E996: Cannot lock an option\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut opt_idx: OptIndex = kOptAleph;
    let mut opt_flags: ::core::ffi::c_int = 0;
    let p: *mut ::core::ffi::c_char = find_option_var_end(
        &raw mut arg as *mut *const ::core::ffi::c_char,
        &raw mut opt_idx,
        &raw mut opt_flags,
    ) as *mut ::core::ffi::c_char;
    if p.is_null()
        || !endchars.is_null()
            && vim_strchr(endchars, *skipwhite(p) as uint8_t as ::core::ffi::c_int).is_null()
    {
        emsg(gettext(e_letunexp.get()));
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let c1: ::core::ffi::c_char = *p;
    *p = NUL as ::core::ffi::c_char;
    let mut is_tty_opt: bool = is_tty_option(arg);
    let mut hidden: bool = is_option_hidden(opt_idx);
    let mut curval: OptVal = if is_tty_opt as ::core::ffi::c_int != 0 {
        get_tty_option(arg)
    } else {
        get_option_value(opt_idx, opt_flags)
    };
    let mut newval: OptVal = OptVal {
        type_0: kOptValTypeNil,
        data: OptValData { boolean: kFalse },
    };
    if curval.type_0 as ::core::ffi::c_int == kOptValTypeNil as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_unknown_option2 as *const ::core::ffi::c_char),
            arg,
        );
    } else if !op.is_null()
        && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int
        && (curval.type_0 as ::core::ffi::c_int != kOptValTypeString as ::core::ffi::c_int
            && *op as ::core::ffi::c_int == '.' as ::core::ffi::c_int
            || curval.type_0 as ::core::ffi::c_int == kOptValTypeString as ::core::ffi::c_int
                && *op as ::core::ffi::c_int != '.' as ::core::ffi::c_int)
    {
        semsg(
            gettext(&raw const e_letwrong as *const ::core::ffi::c_char),
            op,
        );
    } else {
        error = false;
        newval = tv_to_optval(tv, opt_idx, arg, &raw mut error);
        if !error {
            '_c2rust_label: {
                if curval.type_0 as ::core::ffi::c_int == newval.type_0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"curval.type == newval.type\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/eval/vars.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1383 as ::core::ffi::c_uint,
                        b"char *ex_let_option(char *, typval_T *const, const _Bool, const char *const, const char *const)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            is_num = curval.type_0 as ::core::ffi::c_int == kOptValTypeNumber as ::core::ffi::c_int
                || curval.type_0 as ::core::ffi::c_int == kOptValTypeBoolean as ::core::ffi::c_int;
            is_string =
                curval.type_0 as ::core::ffi::c_int == kOptValTypeString as ::core::ffi::c_int;
            if !op.is_null() && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                if !hidden && is_num as ::core::ffi::c_int != 0 {
                    let mut cur_n: OptInt = if curval.type_0 as ::core::ffi::c_int
                        == kOptValTypeNumber as ::core::ffi::c_int
                    {
                        curval.data.number
                    } else {
                        curval.data.boolean as OptInt
                    };
                    let mut new_n: OptInt = if newval.type_0 as ::core::ffi::c_int
                        == kOptValTypeNumber as ::core::ffi::c_int
                    {
                        newval.data.number
                    } else {
                        newval.data.boolean as OptInt
                    };
                    match *op as ::core::ffi::c_int {
                        43 => {
                            new_n = cur_n + new_n;
                        }
                        45 => {
                            new_n = cur_n - new_n;
                        }
                        42 => {
                            new_n = cur_n * new_n;
                        }
                        47 => {
                            new_n =
                                num_divide(cur_n as varnumber_T, new_n as varnumber_T) as OptInt;
                        }
                        37 => {
                            new_n =
                                num_modulus(cur_n as varnumber_T, new_n as varnumber_T) as OptInt;
                        }
                        _ => {}
                    }
                    if curval.type_0 as ::core::ffi::c_int
                        == kOptValTypeNumber as ::core::ffi::c_int
                    {
                        newval = OptVal {
                            type_0: kOptValTypeNumber,
                            data: OptValData { number: new_n },
                        };
                    } else {
                        newval = OptVal {
                            type_0: kOptValTypeBoolean,
                            data: OptValData {
                                boolean: (if new_n == 0 as OptInt {
                                    kFalse as ::core::ffi::c_int
                                } else if new_n >= 1 as OptInt {
                                    kTrue as ::core::ffi::c_int
                                } else {
                                    kNone as ::core::ffi::c_int
                                }) as TriState,
                            },
                        };
                    }
                } else if !hidden && is_string as ::core::ffi::c_int != 0 {
                    let mut curval_data: *const ::core::ffi::c_char = curval.data.string.data;
                    let mut newval_data: *const ::core::ffi::c_char = newval.data.string.data;
                    if !curval_data.is_null() && !newval_data.is_null() {
                        let mut newval_old: OptVal = newval;
                        newval = OptVal {
                            type_0: kOptValTypeString,
                            data: OptValData {
                                string: cstr_as_string(concat_str(curval_data, newval_data)),
                            },
                        };
                        optval_free(newval_old);
                    }
                }
            }
            err = set_option_value_handle_tty(arg, opt_idx, newval, opt_flags);
            arg_end = p;
            if !err.is_null() {
                emsg(gettext(err));
            }
        }
    }
    *p = c1;
    optval_free(curval);
    optval_free(newval);
    return arg_end;
}
unsafe extern "C" fn ex_let_register(
    mut arg: *mut ::core::ffi::c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const ::core::ffi::c_char,
    op: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if is_const {
        emsg(gettext(
            b"E996: Cannot lock a register\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    arg = arg.offset(1);
    if !op.is_null()
        && !vim_strchr(
            b"+-*/%\0".as_ptr() as *const ::core::ffi::c_char,
            *op as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
    {
        semsg(
            gettext(&raw const e_letwrong as *const ::core::ffi::c_char),
            op,
        );
    } else if !endchars.is_null()
        && vim_strchr(
            endchars,
            *skipwhite(arg.offset(1 as ::core::ffi::c_int as isize)) as uint8_t
                as ::core::ffi::c_int,
        )
        .is_null()
    {
        emsg(gettext(e_letunexp.get()));
    } else {
        let mut ptofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *const ::core::ffi::c_char = tv_get_string_chk(tv);
        if !p.is_null() && !op.is_null() && *op as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            let mut s: *mut ::core::ffi::c_char = get_reg_contents(
                if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                    '"' as ::core::ffi::c_int
                } else {
                    *arg as ::core::ffi::c_int
                },
                kGRegExprSrc as ::core::ffi::c_int,
            ) as *mut ::core::ffi::c_char;
            if !s.is_null() {
                ptofree = concat_str(s, p);
                p = ptofree;
                xfree(s as *mut ::core::ffi::c_void);
            }
        }
        if !p.is_null() {
            write_reg_contents(
                if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                    '"' as ::core::ffi::c_int
                } else {
                    *arg as ::core::ffi::c_int
                },
                p,
                strlen(p) as ssize_t,
                false_0,
            );
            arg_end = arg.offset(1 as ::core::ffi::c_int as isize);
        }
        xfree(ptofree as *mut ::core::ffi::c_void);
    }
    return arg_end;
}
unsafe extern "C" fn ex_let_one(
    mut arg: *mut ::core::ffi::c_char,
    tv: *mut typval_T,
    copy: bool,
    is_const: bool,
    endchars: *const ::core::ffi::c_char,
    op: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *arg as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
        return ex_let_env(arg, tv, is_const, endchars, op);
    } else if *arg as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
        return ex_let_option(arg, tv, is_const, endchars, op);
    } else if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
        return ex_let_register(arg, tv, is_const, endchars, op);
    } else if eval_isnamec1(*arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        || *arg as ::core::ffi::c_int == '{' as ::core::ffi::c_int
    {
        let mut lv: lval_T = lval_T {
            ll_name: ::core::ptr::null::<::core::ffi::c_char>(),
            ll_name_len: 0,
            ll_exp_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ll_tv: ::core::ptr::null_mut::<typval_T>(),
            ll_li: ::core::ptr::null_mut::<listitem_T>(),
            ll_list: ::core::ptr::null_mut::<list_T>(),
            ll_range: false,
            ll_empty2: false,
            ll_n1: 0,
            ll_n2: 0,
            ll_dict: ::core::ptr::null_mut::<dict_T>(),
            ll_di: ::core::ptr::null_mut::<dictitem_T>(),
            ll_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ll_blob: ::core::ptr::null_mut::<blob_T>(),
        };
        let p: *mut ::core::ffi::c_char = get_lval(
            arg,
            tv,
            &raw mut lv,
            false_0 != 0,
            false_0 != 0,
            0 as ::core::ffi::c_int,
            FNE_CHECK_START,
        );
        if !p.is_null() && !lv.ll_name.is_null() {
            if !endchars.is_null()
                && vim_strchr(endchars, *skipwhite(p) as uint8_t as ::core::ffi::c_int).is_null()
            {
                emsg(gettext(e_letunexp.get()));
            } else {
                set_var_lval(&raw mut lv, p, tv, copy, is_const, op);
                arg_end = p;
            }
        }
        clear_lval(&raw mut lv);
    } else {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            arg,
        );
    }
    return arg_end;
}
pub unsafe fn ex_unlet(mut eap: *mut exarg_T) {
    ex_unletlock(
        eap,
        (*eap).arg,
        0 as ::core::ffi::c_int,
        if (*eap).forceit != 0 {
            GLV_QUIET as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
        Some(
            do_unlet_var
                as unsafe extern "C" fn(
                    *mut lval_T,
                    *mut ::core::ffi::c_char,
                    *mut exarg_T,
                    ::core::ffi::c_int,
                ) -> ::core::ffi::c_int,
        ),
    );
}
pub unsafe fn ex_lockvar(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut deep: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    if (*eap).forceit != 0 {
        deep = -1 as ::core::ffi::c_int;
    } else if ascii_isdigit(*arg as ::core::ffi::c_int) {
        deep = getdigits_int(&raw mut arg, false_0 != 0, -1 as ::core::ffi::c_int);
        arg = skipwhite(arg);
    }
    ex_unletlock(
        eap,
        arg,
        deep,
        0 as ::core::ffi::c_int,
        Some(
            do_lock_var
                as unsafe extern "C" fn(
                    *mut lval_T,
                    *mut ::core::ffi::c_char,
                    *mut exarg_T,
                    ::core::ffi::c_int,
                ) -> ::core::ffi::c_int,
        ),
    );
}
unsafe extern "C" fn ex_unletlock(
    mut eap: *mut exarg_T,
    mut argstart: *mut ::core::ffi::c_char,
    mut deep: ::core::ffi::c_int,
    mut glv_flags: ::core::ffi::c_int,
    mut callback: ex_unletlock_callback,
) {
    let mut arg: *mut ::core::ffi::c_char = argstart;
    let mut name_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut error: bool = false_0 != 0;
    let mut lv: lval_T = lval_T {
        ll_name: ::core::ptr::null::<::core::ffi::c_char>(),
        ll_name_len: 0,
        ll_exp_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ll_tv: ::core::ptr::null_mut::<typval_T>(),
        ll_li: ::core::ptr::null_mut::<listitem_T>(),
        ll_list: ::core::ptr::null_mut::<list_T>(),
        ll_range: false,
        ll_empty2: false,
        ll_n1: 0,
        ll_n2: 0,
        ll_dict: ::core::ptr::null_mut::<dict_T>(),
        ll_di: ::core::ptr::null_mut::<dictitem_T>(),
        ll_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ll_blob: ::core::ptr::null_mut::<blob_T>(),
    };
    loop {
        if *arg as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
            lv.ll_name = arg;
            lv.ll_tv = ::core::ptr::null_mut::<typval_T>();
            arg = arg.offset(1);
            if get_env_len(&raw mut arg as *mut *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    arg.offset(-(1 as ::core::ffi::c_int as isize)),
                );
                return;
            }
            '_c2rust_label: {
                if *lv.ll_name as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"*lv.ll_name == '$'\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/vars.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1570 as ::core::ffi::c_uint,
                        b"void ex_unletlock(exarg_T *, char *, int, int, ex_unletlock_callback)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if !error
                && (*eap).skip == 0
                && callback.expect("non-null function pointer")(&raw mut lv, arg, eap, deep) == FAIL
            {
                error = true_0 != 0;
            }
            name_end = arg;
        } else {
            name_end = get_lval(
                arg,
                ::core::ptr::null_mut::<typval_T>(),
                &raw mut lv,
                true_0 != 0,
                (*eap).skip != 0 || error as ::core::ffi::c_int != 0,
                glv_flags,
                FNE_CHECK_START,
            );
            if lv.ll_name.is_null() {
                error = true_0 != 0;
            }
            if name_end.is_null()
                || !ascii_iswhite(*name_end as ::core::ffi::c_int)
                    && ends_excmd(*name_end as ::core::ffi::c_int) == 0
            {
                if !name_end.is_null() {
                    emsg_severe.set(true_0 != 0);
                    semsg(
                        gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                        name_end,
                    );
                }
                if !((*eap).skip != 0 || error as ::core::ffi::c_int != 0) {
                    clear_lval(&raw mut lv);
                }
                break;
            } else {
                if !error
                    && (*eap).skip == 0
                    && callback.expect("non-null function pointer")(
                        &raw mut lv,
                        name_end,
                        eap,
                        deep,
                    ) == FAIL
                {
                    error = true_0 != 0;
                }
                if (*eap).skip == 0 {
                    clear_lval(&raw mut lv);
                }
            }
        }
        arg = skipwhite(name_end);
        if ends_excmd(*arg as ::core::ffi::c_int) != 0 {
            break;
        }
    }
    (*eap).nextcmd = check_nextcmd(arg);
}
unsafe extern "C" fn do_unlet_var(
    mut lp: *mut lval_T,
    mut name_end: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut _deep: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut forceit: ::core::ffi::c_int = (*eap).forceit;
    let mut ret: ::core::ffi::c_int = OK;
    if (*lp).ll_tv.is_null() {
        let mut cc: ::core::ffi::c_int = *name_end as uint8_t as ::core::ffi::c_int;
        *name_end = NUL as ::core::ffi::c_char;
        if *(*lp).ll_name as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
            vim_unsetenv_ext((*lp).ll_name.offset(1 as ::core::ffi::c_int as isize));
        } else if do_unlet((*lp).ll_name, (*lp).ll_name_len, forceit != 0) == FAIL {
            ret = FAIL;
        }
        *name_end = cc as ::core::ffi::c_char;
    } else if !(*lp).ll_list.is_null()
        && value_check_lock(
            tv_list_locked((*lp).ll_list),
            (*lp).ll_name,
            (*lp).ll_name_len,
        ) as ::core::ffi::c_int
            != 0
        || !(*lp).ll_dict.is_null()
            && value_check_lock((*(*lp).ll_dict).dv_lock, (*lp).ll_name, (*lp).ll_name_len)
                as ::core::ffi::c_int
                != 0
    {
        return FAIL;
    } else if (*lp).ll_range {
        tv_list_unlet_range(
            (*lp).ll_list,
            (*lp).ll_li,
            (*lp).ll_n1,
            !(*lp).ll_empty2,
            (*lp).ll_n2,
        );
    } else if !(*lp).ll_list.is_null() {
        tv_list_item_remove((*lp).ll_list, (*lp).ll_li);
    } else {
        let mut d: *mut dict_T = (*lp).ll_dict;
        '_c2rust_label: {
            if !d.is_null() {
            } else {
                __assert_fail(
                    b"d != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/vars.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1652 as ::core::ffi::c_uint,
                    b"int do_unlet_var(lval_T *, char *, exarg_T *, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut di: *mut dictitem_T = (*lp).ll_di;
        let mut watched: bool = tv_dict_is_watched(d);
        let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut oldtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if watched {
            tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            key = xstrdup(&raw mut (*di).di_key as *mut ::core::ffi::c_char);
        }
        tv_dict_item_remove(d, di);
        if watched {
            tv_dict_watcher_notify(d, key, ::core::ptr::null_mut::<typval_T>(), &raw mut oldtv);
            tv_clear(&raw mut oldtv);
            xfree(key as *mut ::core::ffi::c_void);
        }
    }
    return ret;
}
unsafe extern "C" fn tv_list_unlet_range(
    l: *mut list_T,
    li_first: *mut listitem_T,
    n1_arg: ::core::ffi::c_int,
    has_n2: bool,
    n2: ::core::ffi::c_int,
) {
    '_c2rust_label: {
        if !l.is_null() {
        } else {
            __assert_fail(
                b"l != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/vars.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1681 as ::core::ffi::c_uint,
                b"void tv_list_unlet_range(list_T *const, listitem_T *const, const int, const _Bool, const int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut li_last: *mut listitem_T = li_first;
    let mut n1: ::core::ffi::c_int = n1_arg;
    loop {
        let li: *mut listitem_T = (*li_last).li_next;
        n1 += 1;
        if li.is_null() || has_n2 as ::core::ffi::c_int != 0 && n2 < n1 {
            break;
        }
        li_last = li;
    }
    tv_list_remove_items(l, li_first, li_last);
}
pub unsafe extern "C" fn do_unlet(
    name: *const ::core::ffi::c_char,
    name_len: size_t,
    forceit: bool,
) -> ::core::ffi::c_int {
    let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut ht: *mut hashtab_T = find_var_ht_dict(name, name_len, &raw mut varname, &raw mut dict);
    if !ht.is_null() && *varname as ::core::ffi::c_int != NUL {
        let mut d: *mut dict_T = get_current_funccal_dict(ht);
        if d.is_null() {
            if ht == &raw mut (*globvardict.ptr()).dv_hashtab {
                d = globvardict.ptr();
            } else if ht == compat_hashtab.ptr() {
                d = vimvardict.ptr();
            } else {
                let di: *mut dictitem_T = find_var_in_ht(
                    ht,
                    *name as ::core::ffi::c_int,
                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as size_t,
                    false_0,
                );
                d = (*di).di_tv.vval.v_dict;
            }
            if d.is_null() {
                internal_error(b"do_unlet()\0".as_ptr() as *const ::core::ffi::c_char);
                return FAIL;
            }
        }
        let mut hi: *mut hashitem_T = hash_find(ht, varname);
        if (*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            hi = find_hi_in_scoped_ht(name, &raw mut ht);
        }
        if !hi.is_null()
            && !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            let di_0: *mut dictitem_T =
                (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
            if var_check_fixed(
                (*di_0).di_flags as ::core::ffi::c_int,
                name,
                TV_CSTRING as size_t,
            ) as ::core::ffi::c_int
                != 0
                || var_check_ro(
                    (*di_0).di_flags as ::core::ffi::c_int,
                    name,
                    TV_CSTRING as size_t,
                ) as ::core::ffi::c_int
                    != 0
                || value_check_lock((*d).dv_lock, name, TV_CSTRING as size_t) as ::core::ffi::c_int
                    != 0
            {
                return FAIL;
            }
            if value_check_lock((*d).dv_lock, name, TV_CSTRING as size_t) {
                return FAIL;
            }
            let mut oldtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut watched: bool = tv_dict_is_watched(dict);
            if watched {
                tv_copy(&raw mut (*di_0).di_tv, &raw mut oldtv);
            }
            delete_var(ht, hi);
            if watched {
                tv_dict_watcher_notify(
                    dict,
                    varname,
                    ::core::ptr::null_mut::<typval_T>(),
                    &raw mut oldtv,
                );
                tv_clear(&raw mut oldtv);
            }
            return OK;
        }
    }
    if forceit {
        return OK;
    }
    semsg(
        gettext(b"E108: No such variable: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
        name,
    );
    return FAIL;
}
unsafe extern "C" fn do_lock_var(
    mut lp: *mut lval_T,
    mut _name_end: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut deep: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lock: bool = (*eap).cmdidx as ::core::ffi::c_int == CMD_lockvar as ::core::ffi::c_int;
    let mut ret: ::core::ffi::c_int = OK;
    if (*lp).ll_tv.is_null() {
        if *(*lp).ll_name as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
            semsg(gettext(e_lock_unlock.get()), (*lp).ll_name);
            ret = FAIL;
        } else {
            let di: *mut dictitem_T = find_var(
                (*lp).ll_name,
                (*lp).ll_name_len,
                ::core::ptr::null_mut::<*mut hashtab_T>(),
                true_0,
            );
            if di.is_null() {
                ret = FAIL;
            } else if (*di).di_flags as ::core::ffi::c_int & DI_FLAGS_FIX as ::core::ffi::c_int != 0
                && (*di).di_tv.v_type as ::core::ffi::c_uint
                    != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*di).di_tv.v_type as ::core::ffi::c_uint
                    != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                semsg(gettext(e_lock_unlock.get()), (*lp).ll_name);
                ret = FAIL;
            } else {
                if lock {
                    (*di).di_flags = ((*di).di_flags as ::core::ffi::c_int
                        | DI_FLAGS_LOCK as ::core::ffi::c_int)
                        as uint8_t;
                } else {
                    (*di).di_flags = ((*di).di_flags as ::core::ffi::c_int
                        & !(DI_FLAGS_LOCK as ::core::ffi::c_int) as uint8_t as ::core::ffi::c_int)
                        as uint8_t;
                }
                if deep != 0 as ::core::ffi::c_int {
                    tv_item_lock(&raw mut (*di).di_tv, deep, lock, false_0 != 0);
                }
            }
        }
    } else if deep != 0 as ::core::ffi::c_int {
        if (*lp).ll_range {
            let mut li: *mut listitem_T = (*lp).ll_li;
            while !li.is_null()
                && ((*lp).ll_empty2 as ::core::ffi::c_int != 0 || (*lp).ll_n2 >= (*lp).ll_n1)
            {
                tv_item_lock(&raw mut (*li).li_tv, deep, lock, false_0 != 0);
                li = (*li).li_next;
                (*lp).ll_n1 += 1;
            }
        } else if !(*lp).ll_list.is_null() {
            tv_item_lock(&raw mut (*(*lp).ll_li).li_tv, deep, lock, false_0 != 0);
        } else {
            tv_item_lock(&raw mut (*(*lp).ll_di).di_tv, deep, lock, false_0 != 0);
        }
    }
    return ret;
}
pub unsafe extern "C" fn del_menutrans_vars() {
    hash_lock(&raw mut (*globvardict.ptr()).dv_hashtab);
    let hiht_: *mut hashtab_T = &raw mut (*globvardict.ptr()).dv_hashtab;
    let mut hitodo_: size_t = (*hiht_).ht_used;
    let mut hi: *mut hashitem_T = (*hiht_).ht_array;
    while hitodo_ != 0 {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            hitodo_ = hitodo_.wrapping_sub(1);
            if strncmp(
                (*hi).hi_key,
                b"menutrans_\0".as_ptr() as *const ::core::ffi::c_char,
                10 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                delete_var(&raw mut (*globvardict.ptr()).dv_hashtab, hi);
            }
        }
        hi = hi.offset(1);
    }
    hash_unlock(&raw mut (*globvardict.ptr()).dv_hashtab);
}
pub unsafe extern "C" fn get_globvar_dict() -> *mut dict_T {
    return globvardict.ptr();
}
pub unsafe extern "C" fn get_globvar_ht() -> *mut hashtab_T {
    return &raw mut (*globvardict.ptr()).dv_hashtab;
}
pub unsafe extern "C" fn get_vimvar_dict() -> *mut dict_T {
    return vimvardict.ptr();
}
pub unsafe extern "C" fn set_vim_var_tv(idx: VimVarIndex, tv: *mut typval_T) {
    let mut tv_out: *mut typval_T = get_vim_var_tv(idx);
    tv_clear(tv_out);
    tv_copy(tv, tv_out);
}
pub unsafe extern "C" fn get_vim_var_name(idx: VimVarIndex) -> *mut ::core::ffi::c_char {
    return (*vimvars.ptr())[idx as usize].vv_name;
}
pub unsafe extern "C" fn get_vim_var_tv(idx: VimVarIndex) -> *mut typval_T {
    return &raw mut (*(vimvars.ptr() as *mut vimvar).offset(idx as isize))
        .vv_di
        .di_tv;
}
pub unsafe extern "C" fn get_vim_var_nr(idx: VimVarIndex) -> varnumber_T {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    return (*tv).vval.v_number;
}
pub unsafe extern "C" fn get_vim_var_list(idx: VimVarIndex) -> *mut list_T {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    return (*tv).vval.v_list;
}
pub unsafe extern "C" fn get_vim_var_dict(idx: VimVarIndex) -> *mut dict_T {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    return (*tv).vval.v_dict;
}
pub unsafe extern "C" fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char {
    return tv_get_string(get_vim_var_tv(idx)) as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn get_vim_var_partial(idx: VimVarIndex) -> *mut partial_T {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    return (*tv).vval.v_partial;
}
static varnamebuf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static varnamebuflen: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub unsafe extern "C" fn cat_prefix_varname(
    mut prefix: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = strlen(name).wrapping_add(3 as size_t);
    if len > varnamebuflen.get() {
        xfree(varnamebuf.get() as *mut ::core::ffi::c_void);
        len = len.wrapping_add(10 as size_t);
        varnamebuf.set(xmalloc(len) as *mut ::core::ffi::c_char);
        varnamebuflen.set(len);
    }
    *varnamebuf.get() = prefix as ::core::ffi::c_char;
    *(*varnamebuf.ptr()).offset(1 as ::core::ffi::c_int as isize) = ':' as ::core::ffi::c_char;
    strcpy(
        (*varnamebuf.ptr()).offset(2 as ::core::ffi::c_int as isize),
        name as *mut ::core::ffi::c_char,
    );
    return varnamebuf.get();
}
pub unsafe extern "C" fn get_user_var_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static gdone: GlobalCell<size_t> = GlobalCell::new(0);
    static bdone: GlobalCell<size_t> = GlobalCell::new(0);
    static wdone: GlobalCell<size_t> = GlobalCell::new(0);
    static tdone: GlobalCell<size_t> = GlobalCell::new(0);
    static vidx: GlobalCell<size_t> = GlobalCell::new(0);
    static hi: GlobalCell<*mut hashitem_T> = GlobalCell::new(::core::ptr::null_mut::<hashitem_T>());
    if idx == 0 as ::core::ffi::c_int {
        vidx.set(0 as size_t);
        wdone.set(vidx.get());
        bdone.set(wdone.get());
        gdone.set(bdone.get());
        tdone.set(0 as size_t);
    }
    if gdone.get() < (*globvardict.ptr()).dv_hashtab.ht_used {
        let c2rust_fresh0 = gdone.get();
        gdone.set((*gdone.ptr()).wrapping_add(1));
        if c2rust_fresh0 == 0 as size_t {
            hi.set((*globvardict.ptr()).dv_hashtab.ht_array);
        } else {
            hi.set((*hi.ptr()).offset(1));
        }
        while (*hi.get()).hi_key.is_null()
            || (*hi.get()).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            hi.set((*hi.ptr()).offset(1));
        }
        if strncmp(
            b"g:\0".as_ptr() as *const ::core::ffi::c_char,
            (*xp).xp_pattern,
            2 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            return cat_prefix_varname('g' as ::core::ffi::c_int, (*hi.get()).hi_key);
        }
        return (*hi.get()).hi_key;
    }
    let mut ht: *const hashtab_T =
        &raw mut (*(*(*(prevwin_curwin as unsafe extern "C" fn() -> *mut win_T)()).w_buffer)
            .b_vars)
            .dv_hashtab;
    if bdone.get() < (*ht).ht_used {
        let c2rust_fresh1 = bdone.get();
        bdone.set((*bdone.ptr()).wrapping_add(1));
        if c2rust_fresh1 == 0 as size_t {
            hi.set((*ht).ht_array);
        } else {
            hi.set((*hi.ptr()).offset(1));
        }
        while (*hi.get()).hi_key.is_null()
            || (*hi.get()).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            hi.set((*hi.ptr()).offset(1));
        }
        return cat_prefix_varname('b' as ::core::ffi::c_int, (*hi.get()).hi_key);
    }
    ht =
        &raw mut (*(*(prevwin_curwin as unsafe extern "C" fn() -> *mut win_T)()).w_vars).dv_hashtab;
    if wdone.get() < (*ht).ht_used {
        let c2rust_fresh2 = wdone.get();
        wdone.set((*wdone.ptr()).wrapping_add(1));
        if c2rust_fresh2 == 0 as size_t {
            hi.set((*ht).ht_array);
        } else {
            hi.set((*hi.ptr()).offset(1));
        }
        while (*hi.get()).hi_key.is_null()
            || (*hi.get()).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            hi.set((*hi.ptr()).offset(1));
        }
        return cat_prefix_varname('w' as ::core::ffi::c_int, (*hi.get()).hi_key);
    }
    ht = &raw mut (*(*curtab.get()).tp_vars).dv_hashtab;
    if tdone.get() < (*ht).ht_used {
        let c2rust_fresh3 = tdone.get();
        tdone.set((*tdone.ptr()).wrapping_add(1));
        if c2rust_fresh3 == 0 as size_t {
            hi.set((*ht).ht_array);
        } else {
            hi.set((*hi.ptr()).offset(1));
        }
        while (*hi.get()).hi_key.is_null()
            || (*hi.get()).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            hi.set((*hi.ptr()).offset(1));
        }
        return cat_prefix_varname('t' as ::core::ffi::c_int, (*hi.get()).hi_key);
    }
    if vidx.get()
        < ::core::mem::size_of::<[vimvar; 106]>()
            .wrapping_div(::core::mem::size_of::<vimvar>())
            .wrapping_div(
                (::core::mem::size_of::<[vimvar; 106]>()
                    .wrapping_rem(::core::mem::size_of::<vimvar>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let c2rust_fresh4 = vidx.get();
        vidx.set((*vidx.ptr()).wrapping_add(1));
        return cat_prefix_varname(
            'v' as ::core::ffi::c_int,
            get_vim_var_name(c2rust_fresh4 as VimVarIndex),
        );
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void = varnamebuf.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    varnamebuflen.set(0 as size_t);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn set_vim_var_type(idx: VimVarIndex, type_0: VarType) {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    (*tv).v_type = type_0;
}
pub unsafe extern "C" fn set_vim_var_nr(idx: VimVarIndex, val: varnumber_T) {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    tv_clear(tv);
    (*tv).vval.v_number = val;
}
pub unsafe extern "C" fn set_vim_var_bool(idx: VimVarIndex, val: BoolVarValue) {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    tv_clear(tv);
    (*tv).v_type = VAR_BOOL;
    (*tv).vval.v_bool = val;
}
pub unsafe extern "C" fn set_vim_var_special(idx: VimVarIndex, val: SpecialVarValue) {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    tv_clear(tv);
    (*tv).v_type = VAR_SPECIAL;
    (*tv).vval.v_special = val;
}
pub unsafe extern "C" fn set_vim_var_char(mut c: ::core::ffi::c_int) {
    let mut buf: [::core::ffi::c_char; 7] = [0; 7];
    let mut buflen: ::core::ffi::c_int =
        utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char);
    buf[buflen as usize] = NUL as ::core::ffi::c_char;
    set_vim_var_string(
        VV_CHAR,
        &raw mut buf as *mut ::core::ffi::c_char,
        buflen as ptrdiff_t,
    );
}
pub unsafe extern "C" fn set_vim_var_string(
    idx: VimVarIndex,
    val: *const ::core::ffi::c_char,
    len: ptrdiff_t,
) {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    tv_clear(tv);
    (*tv).v_type = VAR_STRING;
    if val.is_null() {
        (*tv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else if len == -1 as ptrdiff_t {
        (*tv).vval.v_string = xstrdup(val);
    } else {
        (*tv).vval.v_string = xstrndup(val, len as size_t);
    };
}
pub unsafe extern "C" fn set_vim_var_list(idx: VimVarIndex, val: *mut list_T) {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    tv_clear(tv);
    (*tv).v_type = VAR_LIST;
    (*tv).vval.v_list = val;
    if !val.is_null() {
        tv_list_ref(val);
    }
}
pub unsafe extern "C" fn set_vim_var_dict(idx: VimVarIndex, val: *mut dict_T) {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    tv_clear(tv);
    (*tv).v_type = VAR_DICT;
    (*tv).vval.v_dict = val;
    if val.is_null() {
        return;
    }
    (*val).dv_refcount += 1;
    tv_dict_set_keys_readonly(val);
}
pub unsafe extern "C" fn set_vim_var_partial(idx: VimVarIndex, mut val: *mut partial_T) {
    let mut tv: *mut typval_T = get_vim_var_tv(idx);
    (*tv).vval.v_partial = val;
}
pub unsafe extern "C" fn set_reg_var(mut c: ::core::ffi::c_int) {
    let mut regname: [::core::ffi::c_char; 2] = [0; 2];
    if c == 0 as ::core::ffi::c_int || c == ' ' as ::core::ffi::c_int {
        regname[0 as ::core::ffi::c_int as usize] = '"' as ::core::ffi::c_char;
    } else {
        regname[0 as ::core::ffi::c_int as usize] = c as ::core::ffi::c_char;
    }
    regname[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    let mut tv: *mut typval_T = get_vim_var_tv(VV_REG);
    if (*tv).vval.v_string.is_null()
        || *(*tv).vval.v_string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != c
    {
        set_vim_var_string(
            VV_REG,
            &raw mut regname as *mut ::core::ffi::c_char,
            1 as ptrdiff_t,
        );
    }
}
pub unsafe extern "C" fn v_exception(
    mut oldval: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut tv: *mut typval_T = get_vim_var_tv(VV_EXCEPTION);
    if oldval.is_null() {
        return (*tv).vval.v_string;
    }
    (*tv).vval.v_string = oldval;
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn set_cmdarg(
    mut eap: *mut exarg_T,
    mut oldarg: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = 0;
    let mut newval_len: size_t = 0;
    let mut newval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut xlen: size_t = 0;
    let mut rc: ::core::ffi::c_int = 0;
    let mut tv: *mut typval_T = get_vim_var_tv(VV_CMDARG);
    let mut oldval: *mut ::core::ffi::c_char = (*tv).vval.v_string;
    '_error: {
        if !eap.is_null() {
            len = 0 as size_t;
            if (*eap).force_bin == FORCE_BIN {
                len = len.wrapping_add(6 as size_t);
            } else if (*eap).force_bin == FORCE_NOBIN {
                len = len.wrapping_add(8 as size_t);
            }
            if (*eap).read_edit != 0 {
                len = len.wrapping_add(7 as size_t);
            }
            if (*eap).force_ff != 0 as ::core::ffi::c_int {
                len = len.wrapping_add(10 as size_t);
            }
            if (*eap).force_enc != 0 as ::core::ffi::c_int {
                len = len.wrapping_add(
                    strlen((*eap).cmd.offset((*eap).force_enc as isize)).wrapping_add(7 as size_t),
                );
            }
            if (*eap).bad_char != 0 as ::core::ffi::c_int {
                len =
                    len.wrapping_add((7 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as size_t);
            }
            if (*eap).mkdir_p != 0 as ::core::ffi::c_int {
                len = len.wrapping_add(4 as size_t);
            }
            newval_len = len.wrapping_add(1 as size_t);
            newval = xmalloc(newval_len) as *mut ::core::ffi::c_char;
            xlen = 0 as size_t;
            rc = 0 as ::core::ffi::c_int;
            if (*eap).force_bin == FORCE_BIN {
                rc = snprintf(
                    newval,
                    newval_len,
                    b" ++bin\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else if (*eap).force_bin == FORCE_NOBIN {
                rc = snprintf(
                    newval,
                    newval_len,
                    b" ++nobin\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                *newval = NUL as ::core::ffi::c_char;
            }
            if rc >= 0 as ::core::ffi::c_int {
                xlen = xlen.wrapping_add(rc as size_t);
                if (*eap).read_edit != 0 {
                    rc = snprintf(
                        newval.offset(xlen as isize),
                        newval_len.wrapping_sub(xlen),
                        b" ++edit\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    if rc < 0 as ::core::ffi::c_int {
                        break '_error;
                    } else {
                        xlen = xlen.wrapping_add(rc as size_t);
                    }
                }
                if (*eap).force_ff != 0 as ::core::ffi::c_int {
                    rc = snprintf(
                        newval.offset(xlen as isize),
                        newval_len.wrapping_sub(xlen),
                        b" ++ff=%s\0".as_ptr() as *const ::core::ffi::c_char,
                        if (*eap).force_ff == 'u' as ::core::ffi::c_int {
                            b"unix\0".as_ptr() as *const ::core::ffi::c_char
                        } else if (*eap).force_ff == 'd' as ::core::ffi::c_int {
                            b"dos\0".as_ptr() as *const ::core::ffi::c_char
                        } else {
                            b"mac\0".as_ptr() as *const ::core::ffi::c_char
                        },
                    );
                    if rc < 0 as ::core::ffi::c_int {
                        break '_error;
                    } else {
                        xlen = xlen.wrapping_add(rc as size_t);
                    }
                }
                if (*eap).force_enc != 0 as ::core::ffi::c_int {
                    rc = snprintf(
                        newval.offset(xlen as isize),
                        newval_len.wrapping_sub(xlen),
                        b" ++enc=%s\0".as_ptr() as *const ::core::ffi::c_char,
                        (*eap).cmd.offset((*eap).force_enc as isize),
                    );
                    if rc < 0 as ::core::ffi::c_int {
                        break '_error;
                    } else {
                        xlen = xlen.wrapping_add(rc as size_t);
                    }
                }
                if (*eap).bad_char == BAD_KEEP {
                    rc = snprintf(
                        newval.offset(xlen as isize),
                        newval_len.wrapping_sub(xlen),
                        b" ++bad=keep\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    if rc < 0 as ::core::ffi::c_int {
                        break '_error;
                    } else {
                        xlen = xlen.wrapping_add(rc as size_t);
                    }
                } else if (*eap).bad_char == BAD_DROP {
                    rc = snprintf(
                        newval.offset(xlen as isize),
                        newval_len.wrapping_sub(xlen),
                        b" ++bad=drop\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    if rc < 0 as ::core::ffi::c_int {
                        break '_error;
                    } else {
                        xlen = xlen.wrapping_add(rc as size_t);
                    }
                } else if (*eap).bad_char != 0 as ::core::ffi::c_int {
                    rc = snprintf(
                        newval.offset(xlen as isize),
                        newval_len.wrapping_sub(xlen),
                        b" ++bad=%c\0".as_ptr() as *const ::core::ffi::c_char,
                        (*eap).bad_char,
                    );
                    if rc < 0 as ::core::ffi::c_int {
                        break '_error;
                    } else {
                        xlen = xlen.wrapping_add(rc as size_t);
                    }
                }
                if (*eap).mkdir_p != 0 as ::core::ffi::c_int {
                    rc = snprintf(
                        newval.offset(xlen as isize),
                        newval_len.wrapping_sub(xlen),
                        b" ++p\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    if rc < 0 as ::core::ffi::c_int {
                        break '_error;
                    } else {
                        xlen = xlen.wrapping_add(rc as size_t);
                    }
                }
                '_c2rust_label: {
                    if xlen <= newval_len {
                    } else {
                        __assert_fail(
                            b"xlen <= newval_len\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/vars.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2297 as ::core::ffi::c_uint,
                            b"char *set_cmdarg(exarg_T *, char *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                (*tv).vval.v_string = newval;
                return oldval;
            }
        }
    }
    xfree(oldval as *mut ::core::ffi::c_void);
    (*tv).vval.v_string = oldarg;
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn v_throwpoint(
    mut oldval: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut tv: *mut typval_T = get_vim_var_tv(VV_THROWPOINT);
    if oldval.is_null() {
        return (*tv).vval.v_string;
    }
    (*tv).vval.v_string = oldval;
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn set_vcount(
    mut count: int64_t,
    mut count1: int64_t,
    mut set_prevcount: bool,
) {
    if set_prevcount {
        (*get_vim_var_tv(VV_PREVCOUNT)).vval.v_number = get_vim_var_nr(VV_COUNT);
    }
    (*get_vim_var_tv(VV_COUNT)).vval.v_number = count as varnumber_T;
    (*get_vim_var_tv(VV_COUNT1)).vval.v_number = count1 as varnumber_T;
}
pub unsafe extern "C" fn eval_variable(
    mut name: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
    mut dip: *mut *mut dictitem_T,
    mut verbose: bool,
    mut no_autoload: bool,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = OK;
    let mut tv: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
    let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    v = find_var(
        name,
        len as size_t,
        ::core::ptr::null_mut::<*mut hashtab_T>(),
        no_autoload as ::core::ffi::c_int,
    );
    if !v.is_null() {
        tv = &raw mut (*v).di_tv;
        if !dip.is_null() {
            *dip = v;
        }
    }
    if tv.is_null() {
        if !rettv.is_null() && verbose as ::core::ffi::c_int != 0 {
            semsg(
                gettext(b"E121: Undefined variable: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
                len,
                name,
            );
        }
        ret = FAIL;
    } else if !rettv.is_null() {
        tv_copy(tv, rettv);
    }
    return ret;
}
pub unsafe extern "C" fn check_vars(mut name: *const ::core::ffi::c_char, mut len: size_t) {
    if (*eval_lavars_used.ptr()).is_null() {
        return;
    }
    let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ht: *mut hashtab_T = find_var_ht(name, len, &raw mut varname);
    if ht == get_funccal_local_ht() || ht == get_funccal_args_ht() {
        if !find_var(name, len, ::core::ptr::null_mut::<*mut hashtab_T>(), true_0).is_null() {
            *eval_lavars_used.get() = true_0 != 0;
        }
    }
}
pub unsafe extern "C" fn find_var(
    name: *const ::core::ffi::c_char,
    name_len: size_t,
    mut htp: *mut *mut hashtab_T,
    mut no_autoload: ::core::ffi::c_int,
) -> *mut dictitem_T {
    let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let ht: *mut hashtab_T = find_var_ht(name, name_len, &raw mut varname);
    if !htp.is_null() {
        *htp = ht;
    }
    if ht.is_null() {
        return ::core::ptr::null_mut::<dictitem_T>();
    }
    let ret: *mut dictitem_T = find_var_in_ht(
        ht,
        *name as ::core::ffi::c_int,
        varname,
        name_len.wrapping_sub(varname.offset_from(name) as size_t),
        (no_autoload != 0 || !htp.is_null()) as ::core::ffi::c_int,
    );
    if !ret.is_null() {
        return ret;
    }
    return find_var_in_scoped_ht(
        name,
        name_len,
        (no_autoload != 0 || !htp.is_null()) as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn find_var_in_ht(
    ht: *mut hashtab_T,
    mut htname: ::core::ffi::c_int,
    varname: *const ::core::ffi::c_char,
    varname_len: size_t,
    mut no_autoload: ::core::ffi::c_int,
) -> *mut dictitem_T {
    if varname_len == 0 as size_t {
        match htname {
            115 => {
                return &raw mut (*(**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
                    .offset(
                        ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int) as isize,
                    ))
                .sn_vars)
                    .sv_var as *mut dictitem_T;
            }
            103 => return globvars_var.ptr() as *mut dictitem_T,
            118 => return vimvars_var.ptr() as *mut dictitem_T,
            98 => return &raw mut (*curbuf.get()).b_bufvar as *mut dictitem_T,
            119 => return &raw mut (*curwin.get()).w_winvar as *mut dictitem_T,
            116 => return &raw mut (*curtab.get()).tp_winvar as *mut dictitem_T,
            108 => return get_funccal_local_var(),
            97 => return get_funccal_args_var(),
            _ => {}
        }
        return ::core::ptr::null_mut::<dictitem_T>();
    }
    let mut hi: *mut hashitem_T = hash_find_len(ht, varname, varname_len);
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
    {
        if ht == get_globvar_ht() && no_autoload == 0 {
            if !script_autoload(varname, varname_len, false_0 != 0)
                || aborting() as ::core::ffi::c_int != 0
            {
                return ::core::ptr::null_mut::<dictitem_T>();
            }
            hi = hash_find_len(ht, varname, varname_len);
        }
        if (*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            return ::core::ptr::null_mut::<dictitem_T>();
        }
    }
    return (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
}
unsafe extern "C" fn find_var_ht_dict(
    mut name: *const ::core::ffi::c_char,
    name_len: size_t,
    mut varname: *mut *const ::core::ffi::c_char,
    mut d: *mut *mut dict_T,
) -> *mut hashtab_T {
    *d = ::core::ptr::null_mut::<dict_T>();
    if name_len == 0 as size_t {
        return ::core::ptr::null_mut::<hashtab_T>();
    }
    if name_len == 1 as size_t
        || *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != ':' as ::core::ffi::c_int
    {
        if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
            || *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == AUTOLOAD_CHAR
        {
            return ::core::ptr::null_mut::<hashtab_T>();
        }
        *varname = name;
        let mut hi: *mut hashitem_T = hash_find_len(compat_hashtab.ptr(), name, name_len);
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            return compat_hashtab.ptr();
        }
        *d = get_funccal_local_dict();
        if (*d).is_null() {
            *d = get_globvar_dict();
        }
    } else {
        *varname = name.offset(2 as ::core::ffi::c_int as isize);
        if *name as ::core::ffi::c_int == 'g' as ::core::ffi::c_int {
            *d = get_globvar_dict();
        } else if name_len > 2 as size_t
            && (!memchr(
                name.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                ':' as ::core::ffi::c_int,
                name_len.wrapping_sub(2 as size_t),
            )
            .is_null()
                || !memchr(
                    name.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    AUTOLOAD_CHAR,
                    name_len.wrapping_sub(2 as size_t),
                )
                .is_null())
        {
            return ::core::ptr::null_mut::<hashtab_T>();
        }
        if *name as ::core::ffi::c_int == 'b' as ::core::ffi::c_int {
            *d = (*curbuf.get()).b_vars;
        } else if *name as ::core::ffi::c_int == 'w' as ::core::ffi::c_int {
            *d = (*curwin.get()).w_vars;
        } else if *name as ::core::ffi::c_int == 't' as ::core::ffi::c_int {
            *d = (*curtab.get()).tp_vars;
        } else if *name as ::core::ffi::c_int == 'v' as ::core::ffi::c_int {
            *d = get_vimvar_dict();
        } else if *name as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
            *d = get_funccal_args_dict();
        } else if *name as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
            *d = get_funccal_local_dict();
        } else if *name as ::core::ffi::c_int == 's' as ::core::ffi::c_int
            && ((*current_sctx.ptr()).sc_sid > 0 as ::core::ffi::c_int
                || (*current_sctx.ptr()).sc_sid == SID_STR
                || (*current_sctx.ptr()).sc_sid == SID_LUA)
            && (*current_sctx.ptr()).sc_sid <= (*script_items.ptr()).ga_len
        {
            nlua_set_sctx(current_sctx.ptr());
            if (*current_sctx.ptr()).sc_sid == SID_STR || (*current_sctx.ptr()).sc_sid == SID_LUA {
                new_script_item(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    &raw mut (*current_sctx.ptr()).sc_sid,
                );
            }
            *d = &raw mut (*(**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T).offset(
                ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    as isize,
            ))
            .sn_vars)
                .sv_dict;
        }
    }
    return if !(*d).is_null() {
        &raw mut (**d).dv_hashtab
    } else {
        ::core::ptr::null_mut::<hashtab_T>()
    };
}
pub unsafe extern "C" fn find_var_ht(
    mut name: *const ::core::ffi::c_char,
    name_len: size_t,
    mut varname: *mut *const ::core::ffi::c_char,
) -> *mut hashtab_T {
    let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    return find_var_ht_dict(name, name_len, varname, &raw mut d);
}
pub unsafe extern "C" fn get_var_value(
    name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    v = find_var(
        name,
        strlen(name),
        ::core::ptr::null_mut::<*mut hashtab_T>(),
        false_0,
    );
    if v.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return tv_get_string(&raw mut (*v).di_tv) as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn new_script_vars(mut id: scid_T) {
    let mut sv: *mut scriptvar_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<scriptvar_T>()) as *mut scriptvar_T;
    init_var_dict(&raw mut (*sv).sv_dict, &raw mut (*sv).sv_var, VAR_SCOPE);
    (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
        .offset((id as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize))
    .sn_vars = sv;
}
pub unsafe extern "C" fn init_var_dict(
    mut dict: *mut dict_T,
    mut dict_var: *mut ScopeDictDictItem,
    mut scope: ScopeType,
) {
    hash_init(&raw mut (*dict).dv_hashtab);
    (*dict).dv_lock = VAR_UNLOCKED;
    (*dict).dv_scope = scope;
    (*dict).dv_refcount = DO_NOT_FREE_CNT as ::core::ffi::c_int;
    (*dict).dv_copyID = 0 as ::core::ffi::c_int;
    (*dict_var).di_tv.vval.v_dict = dict;
    (*dict_var).di_tv.v_type = VAR_DICT;
    (*dict_var).di_tv.v_lock = VAR_FIXED;
    (*dict_var).di_flags =
        (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
    *(&raw mut (*dict_var).di_key as *mut ::core::ffi::c_char)
        .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    QUEUE_INIT(&raw mut (*dict).watchers);
}
pub unsafe extern "C" fn unref_var_dict(mut dict: *mut dict_T) {
    (*dict).dv_refcount -= DO_NOT_FREE_CNT as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    tv_dict_unref(dict);
}
pub unsafe extern "C" fn vars_clear(mut ht: *mut hashtab_T) {
    vars_clear_ext(ht, true_0 != 0);
}
pub unsafe extern "C" fn vars_clear_ext(mut ht: *mut hashtab_T, mut free_val: bool) {
    let mut todo: ::core::ffi::c_int = 0;
    let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
    let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    hash_lock(ht);
    todo = (*ht).ht_used as ::core::ffi::c_int;
    hi = (*ht).ht_array;
    while todo > 0 as ::core::ffi::c_int {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            todo -= 1;
            v = (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
            if free_val {
                tv_clear(&raw mut (*v).di_tv);
            }
            if (*v).di_flags as ::core::ffi::c_int & DI_FLAGS_ALLOC as ::core::ffi::c_int != 0 {
                xfree(v as *mut ::core::ffi::c_void);
            }
        }
        hi = hi.offset(1);
    }
    hash_clear(ht);
    hash_init(ht);
}
unsafe extern "C" fn delete_var(mut ht: *mut hashtab_T, mut hi: *mut hashitem_T) {
    let mut di: *mut dictitem_T =
        (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
    hash_remove(ht, hi);
    tv_clear(&raw mut (*di).di_tv);
    xfree(di as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn list_one_var(
    mut v: *mut dictitem_T,
    mut prefix: *const ::core::ffi::c_char,
    mut first: *mut ::core::ffi::c_int,
) {
    let s: *mut ::core::ffi::c_char =
        encode_tv2echo(&raw mut (*v).di_tv, ::core::ptr::null_mut::<size_t>());
    list_one_var_a(
        prefix,
        &raw mut (*v).di_key as *mut ::core::ffi::c_char,
        strlen(&raw mut (*v).di_key as *mut ::core::ffi::c_char) as ptrdiff_t,
        (*v).di_tv.v_type,
        if s.is_null() {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            s as *const ::core::ffi::c_char
        },
        first,
    );
    xfree(s as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn list_one_var_a(
    mut prefix: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
    name_len: ptrdiff_t,
    type_0: VarType,
    mut string: *const ::core::ffi::c_char,
    mut first: *mut ::core::ffi::c_int,
) {
    if *first != 0 {
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        msg_start();
    } else {
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    if *prefix as ::core::ffi::c_int != NUL {
        msg_puts(prefix);
    }
    if !name.is_null() {
        msg_puts_len(name, name_len, 0 as ::core::ffi::c_int, false_0 != 0);
    }
    msg_putchar(' ' as ::core::ffi::c_int);
    msg_advance(22 as ::core::ffi::c_int);
    if type_0 as ::core::ffi::c_uint == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint {
        msg_putchar('#' as ::core::ffi::c_int);
    } else if type_0 as ::core::ffi::c_uint == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || type_0 as ::core::ffi::c_uint == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        msg_putchar('*' as ::core::ffi::c_int);
    } else if type_0 as ::core::ffi::c_uint == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        msg_putchar('[' as ::core::ffi::c_int);
        if *string as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
            string = string.offset(1);
        }
    } else if type_0 as ::core::ffi::c_uint == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        msg_putchar('{' as ::core::ffi::c_int);
        if *string as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
            string = string.offset(1);
        }
    } else {
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    msg_outtrans(string, 0 as ::core::ffi::c_int, false_0 != 0);
    if type_0 as ::core::ffi::c_uint == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || type_0 as ::core::ffi::c_uint == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        msg_puts(b"()\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if *first != 0 {
        msg_clr_eos();
        *first = false_0;
    }
}
pub unsafe extern "C" fn before_set_vvar(
    varname: *const ::core::ffi::c_char,
    di: *mut dictitem_T,
    tv: *mut typval_T,
    copy: bool,
    watched: bool,
    type_error: *mut bool,
) -> bool {
    if (*di).di_tv.v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut oldtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if watched {
            tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*di).di_tv.vval.v_string as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        if copy as ::core::ffi::c_int != 0
            || (*tv).v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let val: *const ::core::ffi::c_char = tv_get_string(tv);
            if (*di).di_tv.vval.v_string.is_null() {
                (*di).di_tv.vval.v_string = xstrdup(val);
            }
        } else {
            (*di).di_tv.vval.v_string = (*tv).vval.v_string;
            (*tv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if watched {
            tv_dict_watcher_notify(
                vimvardict.ptr(),
                varname,
                &raw mut (*di).di_tv,
                &raw mut oldtv,
            );
            tv_clear(&raw mut oldtv);
        }
        return false_0 != 0;
    } else if (*di).di_tv.v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut oldtv_0: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if watched {
            tv_copy(&raw mut (*di).di_tv, &raw mut oldtv_0);
        }
        (*di).di_tv.vval.v_number = tv_get_number(tv);
        if strcmp(
            varname,
            b"searchforward\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            set_search_direction(if (*di).di_tv.vval.v_number != 0 {
                '/' as ::core::ffi::c_int
            } else {
                '?' as ::core::ffi::c_int
            });
        } else if strcmp(
            varname,
            b"hlsearch\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            no_hlsearch.set((*di).di_tv.vval.v_number == 0);
            redraw_all_later(UPD_SOME_VALID);
        }
        if watched {
            tv_dict_watcher_notify(
                vimvardict.ptr(),
                varname,
                &raw mut (*di).di_tv,
                &raw mut oldtv_0,
            );
            tv_clear(&raw mut oldtv_0);
        }
        return false_0 != 0;
    } else if (*di).di_tv.v_type as ::core::ffi::c_uint != (*tv).v_type as ::core::ffi::c_uint {
        *type_error = true_0 != 0;
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn set_var(
    mut name: *const ::core::ffi::c_char,
    name_len: size_t,
    tv: *mut typval_T,
    copy: bool,
) {
    set_var_const(name, name_len, tv, copy, false_0 != 0);
}
pub unsafe extern "C" fn set_var_const(
    mut name: *const ::core::ffi::c_char,
    name_len: size_t,
    tv: *mut typval_T,
    copy: bool,
    is_const: bool,
) {
    let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut ht: *mut hashtab_T = find_var_ht_dict(name, name_len, &raw mut varname, &raw mut dict);
    let watched: bool = tv_dict_is_watched(dict);
    if ht.is_null() || *varname as ::core::ffi::c_int == NUL {
        semsg(
            gettext(&raw const e_illvar as *const ::core::ffi::c_char),
            name,
        );
        return;
    }
    let varname_len: size_t = name_len.wrapping_sub(varname.offset_from(name) as size_t);
    let mut di: *mut dictitem_T =
        find_var_in_ht(ht, 0 as ::core::ffi::c_int, varname, varname_len, true_0);
    if di.is_null() {
        di = find_var_in_scoped_ht(name, name_len, true_0);
    }
    if tv_is_func(*tv) as ::core::ffi::c_int != 0
        && var_wrong_func_name(name, di.is_null()) as ::core::ffi::c_int != 0
    {
        return;
    }
    let mut oldtv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if !di.is_null() {
        if is_const {
            emsg(gettext(
                &raw const e_cannot_mod as *const ::core::ffi::c_char,
            ));
            return;
        }
        if var_check_ro((*di).di_flags as ::core::ffi::c_int, name, name_len) as ::core::ffi::c_int
            != 0
            || value_check_lock((*di).di_tv.v_lock, name, name_len) as ::core::ffi::c_int != 0
            || var_check_lock((*di).di_flags as ::core::ffi::c_int, name, name_len)
                as ::core::ffi::c_int
                != 0
        {
            return;
        }
        let mut type_error: bool = false_0 != 0;
        if ht == &raw mut (*vimvardict.ptr()).dv_hashtab
            && !before_set_vvar(varname, di, tv, copy, watched, &raw mut type_error)
        {
            if type_error {
                semsg(
                    gettext(
                        (e_setting_v_str_to_value_with_wrong_type.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    varname,
                );
            }
            return;
        }
        if watched {
            tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
        }
        tv_clear(&raw mut (*di).di_tv);
    } else {
        if ht == &raw mut (*vimvardict.ptr()).dv_hashtab || ht == get_funccal_args_ht() {
            semsg(
                gettext(&raw const e_illvar as *const ::core::ffi::c_char),
                name,
            );
            return;
        }
        if !valid_varname(varname) {
            return;
        }
        '_c2rust_label: {
            if !dict.is_null() {
            } else {
                __assert_fail(
                    b"dict != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/vars.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    2883 as ::core::ffi::c_uint,
                    b"void set_var_const(const char *, const size_t, typval_T *const, const _Bool, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        di = xmalloc(
            (17 as size_t)
                .wrapping_add(varname_len)
                .wrapping_add(1 as size_t),
        ) as *mut dictitem_T;
        memcpy(
            &raw mut (*di).di_key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            varname as *const ::core::ffi::c_void,
            varname_len.wrapping_add(1 as size_t),
        );
        if hash_add(ht, &raw mut (*di).di_key as *mut ::core::ffi::c_char) == FAIL {
            xfree(di as *mut ::core::ffi::c_void);
            return;
        }
        (*di).di_flags = DI_FLAGS_ALLOC as ::core::ffi::c_int as uint8_t;
        if is_const {
            (*di).di_flags = ((*di).di_flags as ::core::ffi::c_int
                | DI_FLAGS_LOCK as ::core::ffi::c_int) as uint8_t;
        }
    }
    if copy as ::core::ffi::c_int != 0
        || (*tv).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_copy(tv, &raw mut (*di).di_tv);
    } else {
        (*di).di_tv = *tv;
        (*di).di_tv.v_lock = VAR_UNLOCKED;
        tv_init(tv);
    }
    if watched {
        tv_dict_watcher_notify(
            dict,
            &raw mut (*di).di_key as *mut ::core::ffi::c_char,
            &raw mut (*di).di_tv,
            &raw mut oldtv,
        );
        tv_clear(&raw mut oldtv);
    }
    if is_const {
        tv_item_lock(&raw mut (*di).di_tv, DICT_MAXNEST, true_0 != 0, true_0 != 0);
    }
}
pub unsafe extern "C" fn var_check_ro(
    flags: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    let mut error_message: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if flags & DI_FLAGS_RO as ::core::ffi::c_int != 0 {
        error_message =
            &raw const e_cannot_change_readonly_variable_str as *const ::core::ffi::c_char;
    } else if flags & DI_FLAGS_RO_SBX as ::core::ffi::c_int != 0 && sandbox.get() != 0 {
        error_message =
            &raw const e_cannot_set_variable_in_sandbox_str as *const ::core::ffi::c_char;
    }
    if error_message.is_null() {
        return false_0 != 0;
    }
    if name_len == TV_TRANSLATE as size_t {
        name = gettext(name);
        name_len = strlen(name);
    } else if name_len == TV_CSTRING as size_t {
        name_len = strlen(name);
    }
    semsg(gettext(error_message), name_len as ::core::ffi::c_int, name);
    return true_0 != 0;
}
pub unsafe extern "C" fn var_check_lock(
    flags: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    if flags & DI_FLAGS_LOCK as ::core::ffi::c_int == 0 {
        return false_0 != 0;
    }
    if name_len == TV_TRANSLATE as size_t {
        name = gettext(name);
        name_len = strlen(name);
    } else if name_len == TV_CSTRING as size_t {
        name_len = strlen(name);
    }
    semsg(
        gettext(b"E1122: Variable is locked: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        name_len as ::core::ffi::c_int,
        name,
    );
    return true_0 != 0;
}
pub unsafe extern "C" fn var_check_fixed(
    flags: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    if flags & DI_FLAGS_FIX as ::core::ffi::c_int != 0 {
        if name_len == TV_TRANSLATE as size_t {
            name = gettext(name);
            name_len = strlen(name);
        } else if name_len == TV_CSTRING as size_t {
            name_len = strlen(name);
        }
        semsg(
            gettext(&raw const e_cannot_delete_variable_str as *const ::core::ffi::c_char),
            name_len as ::core::ffi::c_int,
            name,
        );
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn var_wrong_func_name(
    name: *const ::core::ffi::c_char,
    new_var: bool,
) -> bool {
    if !(!vim_strchr(
        b"wbst\0".as_ptr() as *const ::core::ffi::c_char,
        *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int)
        && !((if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '\0' as ::core::ffi::c_int
            && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
        {
            *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        }) as ::core::ffi::c_uint
            >= 'A' as ::core::ffi::c_uint
            && (if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\0' as ::core::ffi::c_int
                && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
            {
                *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            }) as ::core::ffi::c_uint
                <= 'Z' as ::core::ffi::c_uint)
        && vim_strchr(name, '#' as ::core::ffi::c_int).is_null()
    {
        semsg(
            gettext(
                b"E704: Funcref variable name must start with a capital: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            name,
        );
        return true_0 != 0;
    }
    if new_var as ::core::ffi::c_int != 0
        && function_exists(name, false_0 != 0) as ::core::ffi::c_int != 0
    {
        semsg(
            gettext(
                b"E705: Variable name conflicts with existing function: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            name,
        );
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn valid_varname(mut varname: *const ::core::ffi::c_char) -> bool {
    let mut p: *const ::core::ffi::c_char = varname;
    while *p as ::core::ffi::c_int != NUL {
        if !eval_isnamec1(*p as uint8_t as ::core::ffi::c_int)
            && (p == varname || !ascii_isdigit(*p as ::core::ffi::c_int))
            && *p as ::core::ffi::c_int != AUTOLOAD_CHAR
        {
            semsg(
                gettext(&raw const e_illvar as *const ::core::ffi::c_char),
                varname,
            );
            return false_0 != 0;
        }
        p = p.offset(1);
    }
    return true_0 != 0;
}
unsafe extern "C" fn get_var_from(
    mut varname: *const ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut deftv: *mut typval_T,
    mut htname: ::core::ffi::c_int,
    mut tp: *mut tabpage_T,
    mut win: *mut win_T,
    mut buf: *mut buf_T,
) {
    let mut done: bool = false_0 != 0;
    let do_change_curbuf: bool = !buf.is_null() && htname == 'b' as ::core::ffi::c_int;
    (*emsg_off.ptr()) += 1;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !varname.is_null()
        && !tp.is_null()
        && !win.is_null()
        && (htname != 'b' as ::core::ffi::c_int || !buf.is_null())
    {
        let need_switch_win: bool =
            !(tp == curtab.get() && win == curwin.get()) && !do_change_curbuf;
        let mut switchwin: switchwin_T = switchwin_T {
            sw_curwin: ::core::ptr::null_mut::<win_T>(),
            sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
            sw_same_win: false,
            sw_visual_active: false,
        };
        if !need_switch_win || switch_win(&raw mut switchwin, win, tp, true_0 != 0) == OK {
            if *varname as ::core::ffi::c_int == '&' as ::core::ffi::c_int
                && htname != 't' as ::core::ffi::c_int
            {
                let save_curbuf: *mut buf_T = curbuf.get();
                if do_change_curbuf {
                    curbuf.set(buf);
                }
                if *varname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                    let mut opts: *mut dict_T = get_winbuf_options(
                        (htname == 'b' as ::core::ffi::c_int) as ::core::ffi::c_int,
                    );
                    if !opts.is_null() {
                        tv_dict_set_ret(rettv, opts);
                        done = true_0 != 0;
                    }
                } else if eval_option(&raw mut varname, rettv, true_0 != 0) == OK {
                    done = true_0 != 0;
                }
                curbuf.set(save_curbuf);
            } else if *varname as ::core::ffi::c_int == NUL {
                let mut v: *const ScopeDictDictItem = ::core::ptr::null::<ScopeDictDictItem>();
                if htname == 'b' as ::core::ffi::c_int {
                    v = &raw mut (*buf).b_bufvar;
                } else if htname == 'w' as ::core::ffi::c_int {
                    v = &raw mut (*win).w_winvar;
                } else {
                    v = &raw mut (*tp).tp_winvar;
                }
                tv_copy(&raw const (*v).di_tv, rettv);
                done = true_0 != 0;
            } else {
                let mut ht: *mut hashtab_T = ::core::ptr::null_mut::<hashtab_T>();
                if htname == 'b' as ::core::ffi::c_int {
                    ht = &raw mut (*(*buf).b_vars).dv_hashtab;
                } else if htname == 'w' as ::core::ffi::c_int {
                    ht = &raw mut (*(*win).w_vars).dv_hashtab;
                } else {
                    ht = &raw mut (*(*tp).tp_vars).dv_hashtab;
                }
                let v_0: *const dictitem_T =
                    find_var_in_ht(ht, htname, varname, strlen(varname), false_0);
                if !v_0.is_null() {
                    tv_copy(&raw const (*v_0).di_tv, rettv);
                    done = true_0 != 0;
                }
            }
        }
        if need_switch_win {
            restore_win(&raw mut switchwin, true_0 != 0);
        }
    }
    if !done
        && (*deftv).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_copy(deftv, rettv);
    }
    (*emsg_off.ptr()) -= 1;
}
unsafe extern "C" fn getwinvar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut off: ::core::ffi::c_int,
) {
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if off == 1 as ::core::ffi::c_int {
        tp = find_tabpage(tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) as ::core::ffi::c_int);
    } else {
        tp = curtab.get();
    }
    let win: *mut win_T = find_win_by_nr(argvars.offset(off as isize), tp);
    let varname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset((off + 1 as ::core::ffi::c_int) as isize));
    get_var_from(
        varname,
        rettv,
        argvars.offset((off + 2 as ::core::ffi::c_int) as isize),
        'w' as ::core::ffi::c_int,
        tp,
        win,
        ::core::ptr::null_mut::<buf_T>(),
    );
}
unsafe extern "C" fn tv_to_optval(
    mut tv: *mut typval_T,
    mut opt_idx: OptIndex,
    mut option: *const ::core::ffi::c_char,
    mut error: *mut bool,
) -> OptVal {
    let mut value: OptVal = OptVal {
        type_0: kOptValTypeNil,
        data: OptValData { boolean: kFalse },
    };
    let mut nbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut err: bool = false_0 != 0;
    let is_tty_opt: bool = is_tty_option(option);
    let option_has_bool: bool =
        !is_tty_opt && option_has_type(opt_idx, kOptValTypeBoolean) as ::core::ffi::c_int != 0;
    let option_has_num: bool =
        !is_tty_opt && option_has_type(opt_idx, kOptValTypeNumber) as ::core::ffi::c_int != 0;
    let option_has_str: bool = is_tty_opt as ::core::ffi::c_int != 0
        || option_has_type(opt_idx, kOptValTypeString) as ::core::ffi::c_int != 0;
    if !is_tty_opt
        && (*get_option(opt_idx)).flags & kOptFlagFunc as ::core::ffi::c_int as uint32_t != 0
        && tv_is_func(*tv) as ::core::ffi::c_int != 0
    {
        let mut strval: *mut ::core::ffi::c_char =
            encode_tv2string(tv, ::core::ptr::null_mut::<size_t>());
        err = strval.is_null();
        value = OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(strval),
            },
        };
    } else if option_has_bool as ::core::ffi::c_int != 0
        || option_has_num as ::core::ffi::c_int != 0
    {
        let mut n: varnumber_T = if option_has_num as ::core::ffi::c_int != 0 {
            tv_get_number_chk(tv, &raw mut err)
        } else {
            tv_get_bool_chk(tv, &raw mut err)
        };
        if !err
            && (*tv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && n == 0 as varnumber_T
        {
            let mut idx: ::core::ffi::c_uint = 0;
            idx = 0 as ::core::ffi::c_uint;
            while !(*tv).vval.v_string.is_null()
                && *(*tv).vval.v_string.offset(idx as isize) as ::core::ffi::c_int
                    == '0' as ::core::ffi::c_int
            {
                idx = idx.wrapping_add(1);
            }
            if idx == 0 as ::core::ffi::c_uint
                || *(*tv).vval.v_string.offset(idx as isize) as ::core::ffi::c_int != NUL
            {
                err = true_0 != 0;
                semsg(
                    gettext(b"E521: Number required: &%s = '%s'\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    option,
                    if (*tv).vval.v_string.is_null() {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        (*tv).vval.v_string as *const ::core::ffi::c_char
                    },
                );
            }
        }
        value = if option_has_num as ::core::ffi::c_int != 0 {
            OptVal {
                type_0: kOptValTypeNumber,
                data: OptValData { number: n },
            }
        } else {
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData {
                    boolean: (if n == 0 as varnumber_T {
                        kFalse as ::core::ffi::c_int
                    } else if n >= 1 as varnumber_T {
                        kTrue as ::core::ffi::c_int
                    } else {
                        kNone as ::core::ffi::c_int
                    }) as TriState,
                },
            }
        };
    } else if option_has_str {
        if (*tv).v_type as ::core::ffi::c_uint
            != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*tv).v_type as ::core::ffi::c_uint
                != VAR_SPECIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut strval_0: *const ::core::ffi::c_char =
                tv_get_string_buf_chk(tv, &raw mut nbuf as *mut ::core::ffi::c_char);
            err = strval_0.is_null();
            value = OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_to_string(strval_0),
                },
            };
        } else if !is_tty_opt {
            err = true_0 != 0;
            emsg(gettext(
                &raw const e_string_required as *const ::core::ffi::c_char,
            ));
        }
    } else {
        abort();
    }
    if !error.is_null() {
        *error = err;
    }
    return value;
}
pub unsafe extern "C" fn optval_as_tv(mut value: OptVal, mut numbool: bool) -> typval_T {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_SPECIAL,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union {
            v_special: kSpecialVarNull,
        },
    };
    match value.type_0 as ::core::ffi::c_int {
        0 => {
            if numbool {
                rettv.v_type = VAR_NUMBER;
                rettv.vval.v_number = value.data.boolean as varnumber_T;
            } else if value.data.boolean as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
                rettv.v_type = VAR_BOOL;
                rettv.vval.v_bool = (value.data.boolean as ::core::ffi::c_int
                    == kTrue as ::core::ffi::c_int)
                    as ::core::ffi::c_int as BoolVarValue;
            }
        }
        1 => {
            rettv.v_type = VAR_NUMBER;
            rettv.vval.v_number = value.data.number as varnumber_T;
        }
        2 => {
            rettv.v_type = VAR_STRING;
            rettv.vval.v_string = value.data.string.data;
        }
        -1 | _ => {}
    }
    return rettv;
}
unsafe extern "C" fn set_option_from_tv(
    mut varname: *const ::core::ffi::c_char,
    mut varp: *mut typval_T,
) {
    let mut opt_idx: OptIndex = find_option(varname);
    if opt_idx as ::core::ffi::c_int == kOptInvalid as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_unknown_option2 as *const ::core::ffi::c_char),
            varname,
        );
        return;
    }
    let mut error: bool = false_0 != 0;
    let mut value: OptVal = tv_to_optval(varp, opt_idx, varname, &raw mut error);
    if !error {
        let mut errmsg: *const ::core::ffi::c_char =
            set_option_value_handle_tty(varname, opt_idx, value, OPT_LOCAL as ::core::ffi::c_int);
        if !errmsg.is_null() {
            emsg(errmsg);
        }
    }
    optval_free(value);
}
unsafe extern "C" fn setwinvar(mut argvars: *mut typval_T, mut off: ::core::ffi::c_int) {
    if check_secure() {
        return;
    }
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if off == 1 as ::core::ffi::c_int {
        tp = find_tabpage(tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) as ::core::ffi::c_int);
    } else {
        tp = curtab.get();
    }
    let win: *mut win_T = find_win_by_nr(argvars.offset(off as isize), tp);
    let mut varname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset((off + 1 as ::core::ffi::c_int) as isize));
    let mut varp: *mut typval_T = argvars.offset((off + 2 as ::core::ffi::c_int) as isize);
    if win.is_null() || varname.is_null() {
        return;
    }
    let mut need_switch_win: bool = !(tp == curtab.get() && win == curwin.get());
    let mut switchwin: switchwin_T = switchwin_T {
        sw_curwin: ::core::ptr::null_mut::<win_T>(),
        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
        sw_same_win: false,
        sw_visual_active: false,
    };
    if !need_switch_win || switch_win(&raw mut switchwin, win, tp, true_0 != 0) == OK {
        if *varname as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
            set_option_from_tv(varname.offset(1 as ::core::ffi::c_int as isize), varp);
        } else {
            let varname_len: size_t = strlen(varname);
            let winvarname: *mut ::core::ffi::c_char =
                xmalloc(varname_len.wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
            memcpy(
                winvarname as *mut ::core::ffi::c_void,
                b"w:\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                2 as size_t,
            );
            memcpy(
                winvarname.offset(2 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                varname as *const ::core::ffi::c_void,
                varname_len.wrapping_add(1 as size_t),
            );
            set_var(
                winvarname,
                varname_len.wrapping_add(2 as size_t),
                varp,
                true_0 != 0,
            );
            xfree(winvarname as *mut ::core::ffi::c_void);
        }
    }
    if need_switch_win {
        restore_win(&raw mut switchwin, true_0 != 0);
    }
}
pub unsafe extern "C" fn reset_v_option_vars() {
    set_vim_var_string(
        VV_OPTION_NEW,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_OPTION_OLD,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_OPTION_OLDLOCAL,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_OPTION_OLDGLOBAL,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_OPTION_COMMAND,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    set_vim_var_string(
        VV_OPTION_TYPE,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
}
pub unsafe extern "C" fn assert_error(mut gap: *mut garray_T) {
    let mut tv: *mut typval_T = get_vim_var_tv(VV_ERRORS);
    if (*tv).v_type as ::core::ffi::c_uint != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv).vval.v_list.is_null()
    {
        set_vim_var_list(VV_ERRORS, tv_list_alloc(1 as ptrdiff_t));
    }
    tv_list_append_string(
        get_vim_var_list(VV_ERRORS),
        (*gap).ga_data as *const ::core::ffi::c_char,
        (*gap).ga_len as ssize_t,
    );
}
pub unsafe extern "C" fn var_exists(mut var: *const ::core::ffi::c_char) -> bool {
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: bool = false_0 != 0;
    let mut name: *const ::core::ffi::c_char = var;
    let len: ::core::ffi::c_int =
        get_name_len(&raw mut var, &raw mut tofree, true_0 != 0, false_0 != 0);
    if len > 0 as ::core::ffi::c_int {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if !tofree.is_null() {
            name = tofree;
        }
        n = eval_variable(
            name,
            len,
            &raw mut tv,
            ::core::ptr::null_mut::<*mut dictitem_T>(),
            false_0 != 0,
            true_0 != 0,
        ) == OK;
        if n {
            n = handle_subscript(
                &raw mut var,
                &raw mut tv,
                EVALARG_EVALUATE.ptr(),
                false_0 != 0,
            ) == OK;
            if n {
                tv_clear(&raw mut tv);
            }
        }
    }
    if *var as ::core::ffi::c_int != NUL {
        n = false_0 != 0;
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    return n;
}
static redir_lval: GlobalCell<*mut lval_T> = GlobalCell::new(::core::ptr::null_mut::<lval_T>());
static redir_ga: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
});
static redir_endp: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static redir_varname: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub unsafe extern "C" fn var_redir_start(
    mut name: *mut ::core::ffi::c_char,
    mut append: bool,
) -> ::core::ffi::c_int {
    if !eval_isnamec1(*name as ::core::ffi::c_int) {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return FAIL;
    }
    redir_varname.set(xstrdup(name));
    redir_lval.set(xcalloc(1 as size_t, ::core::mem::size_of::<lval_T>()) as *mut lval_T);
    ga_init(
        redir_ga.ptr(),
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        500 as ::core::ffi::c_int,
    );
    redir_endp.set(get_lval(
        redir_varname.get(),
        ::core::ptr::null_mut::<typval_T>(),
        redir_lval.get(),
        false_0 != 0,
        false_0 != 0,
        0 as ::core::ffi::c_int,
        FNE_CHECK_START,
    ));
    if (*redir_endp.ptr()).is_null()
        || (*redir_lval.get()).ll_name.is_null()
        || *redir_endp.get() as ::core::ffi::c_int != NUL
    {
        clear_lval(redir_lval.get());
        if !(*redir_endp.ptr()).is_null() && *redir_endp.get() as ::core::ffi::c_int != NUL {
            semsg(
                gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                redir_endp.get(),
            );
        } else {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                name,
            );
        }
        redir_endp.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        var_redir_stop();
        return FAIL;
    }
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    did_emsg.set(false_0);
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    tv.v_type = VAR_STRING;
    tv.vval.v_string = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    if append {
        set_var_lval(
            redir_lval.get(),
            redir_endp.get(),
            &raw mut tv,
            true_0 != 0,
            false_0 != 0,
            b".\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        set_var_lval(
            redir_lval.get(),
            redir_endp.get(),
            &raw mut tv,
            true_0 != 0,
            false_0 != 0,
            b"=\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    clear_lval(redir_lval.get());
    if called_emsg.get() > called_emsg_before {
        redir_endp.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        var_redir_stop();
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn var_redir_str(
    mut value: *const ::core::ffi::c_char,
    mut value_len: ::core::ffi::c_int,
) {
    if (*redir_lval.ptr()).is_null() {
        return;
    }
    let mut len: ::core::ffi::c_int = 0;
    if value_len == -1 as ::core::ffi::c_int {
        len = strlen(value) as ::core::ffi::c_int;
    } else {
        len = value_len;
    }
    ga_grow(redir_ga.ptr(), len);
    memmove(
        ((*redir_ga.ptr()).ga_data as *mut ::core::ffi::c_char)
            .offset((*redir_ga.ptr()).ga_len as isize) as *mut ::core::ffi::c_void,
        value as *const ::core::ffi::c_void,
        len as size_t,
    );
    (*redir_ga.ptr()).ga_len += len;
}
pub unsafe extern "C" fn var_redir_stop() {
    if !(*redir_lval.ptr()).is_null() {
        if !(*redir_endp.ptr()).is_null() {
            ga_append(redir_ga.ptr(), NUL as uint8_t);
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            tv.v_type = VAR_STRING;
            tv.vval.v_string = (*redir_ga.ptr()).ga_data as *mut ::core::ffi::c_char;
            redir_endp.set(get_lval(
                redir_varname.get(),
                ::core::ptr::null_mut::<typval_T>(),
                redir_lval.get(),
                false_0 != 0,
                false_0 != 0,
                0 as ::core::ffi::c_int,
                FNE_CHECK_START,
            ));
            if !(*redir_endp.ptr()).is_null() && !(*redir_lval.get()).ll_name.is_null() {
                set_var_lval(
                    redir_lval.get(),
                    redir_endp.get(),
                    &raw mut tv,
                    false_0 != 0,
                    false_0 != 0,
                    b".\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            clear_lval(redir_lval.get());
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*redir_ga.ptr()).ga_data;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            redir_lval.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
    }
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        redir_varname.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL;
    let _ = *ptr__1;
}
pub unsafe extern "C" fn f_gettabvar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let varname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    let tp: *mut tabpage_T = find_tabpage(tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int);
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if !tp.is_null() {
        win = if tp == curtab.get() || (*tp).tp_firstwin.is_null() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
    }
    get_var_from(
        varname,
        rettv,
        argvars.offset(2 as ::core::ffi::c_int as isize),
        't' as ::core::ffi::c_int,
        tp,
        win,
        ::core::ptr::null_mut::<buf_T>(),
    );
}
pub unsafe extern "C" fn f_gettabwinvar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getwinvar(argvars, rettv, 1 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn f_getwinvar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getwinvar(argvars, rettv, 0 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn f_getbufvar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let varname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    get_var_from(
        varname,
        rettv,
        argvars.offset(2 as ::core::ffi::c_int as isize),
        'b' as ::core::ffi::c_int,
        curtab.get(),
        curwin.get(),
        buf,
    );
}
pub unsafe extern "C" fn f_settabvar(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    let tp: *mut tabpage_T = find_tabpage(tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int);
    let varname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    let varp: *mut typval_T = argvars.offset(2 as ::core::ffi::c_int as isize);
    if varname.is_null() || tp.is_null() {
        return;
    }
    let save_curtab: *mut tabpage_T = curtab.get();
    let save_lu_tp: *mut tabpage_T = lastused_tabpage.get();
    goto_tabpage_tp(tp, false_0 != 0, false_0 != 0);
    let varname_len: size_t = strlen(varname);
    let tabvarname: *mut ::core::ffi::c_char =
        xmalloc(varname_len.wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
    memcpy(
        tabvarname as *mut ::core::ffi::c_void,
        b"t:\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        2 as size_t,
    );
    memcpy(
        tabvarname.offset(2 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        varname as *const ::core::ffi::c_void,
        varname_len.wrapping_add(1 as size_t),
    );
    set_var(
        tabvarname,
        varname_len.wrapping_add(2 as size_t),
        varp,
        true_0 != 0,
    );
    xfree(tabvarname as *mut ::core::ffi::c_void);
    if valid_tabpage(save_curtab) {
        goto_tabpage_tp(save_curtab, false_0 != 0, false_0 != 0);
        if valid_tabpage(save_lu_tp) {
            lastused_tabpage.set(save_lu_tp);
        }
    }
}
pub unsafe extern "C" fn f_settabwinvar(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    setwinvar(argvars, 1 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn f_setwinvar(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    setwinvar(argvars, 0 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn f_setbufvar(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() as ::core::ffi::c_int != 0
        || !tv_check_str_or_nr(argvars.offset(0 as ::core::ffi::c_int as isize))
    {
        return;
    }
    let mut varname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    let buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    let mut varp: *mut typval_T = argvars.offset(2 as ::core::ffi::c_int as isize);
    if buf.is_null() || varname.is_null() {
        return;
    }
    if *varname as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
        let mut aco: aco_save_T = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, buf);
        set_option_from_tv(varname.offset(1 as ::core::ffi::c_int as isize), varp);
        aucmd_restbuf(&raw mut aco);
    } else {
        let varname_len: size_t = strlen(varname);
        let bufvarname: *mut ::core::ffi::c_char =
            xmalloc(varname_len.wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
        let save_curbuf: *mut buf_T = curbuf.get();
        curbuf.set(buf);
        memcpy(
            bufvarname as *mut ::core::ffi::c_void,
            b"b:\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            2 as size_t,
        );
        memcpy(
            bufvarname.offset(2 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            varname as *const ::core::ffi::c_void,
            varname_len.wrapping_add(1 as size_t),
        );
        set_var(
            bufvarname,
            varname_len.wrapping_add(2 as size_t),
            varp,
            true_0 != 0,
        );
        xfree(bufvarname as *mut ::core::ffi::c_void);
        curbuf.set(save_curbuf);
    };
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SID_LUA: ::core::ffi::c_int = -8 as ::core::ffi::c_int;
pub const SID_STR: ::core::ffi::c_int = -10 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
