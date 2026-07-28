use crate::src::nvim::api::private::converter::vim_to_object;
use crate::src::nvim::api::private::helpers::{cstr_as_string, cstr_to_string};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite, ascii_iswhite_or_nul, ascii_isxdigit};
use crate::src::nvim::buffer::{bt_prompt, buflist_findnr};
use crate::src::nvim::change::appended_lines_mark;
use crate::src::nvim::channel::callback_reader_free;
use crate::src::nvim::channel::channel_proc;
use crate::src::nvim::channel::find_channel;
use crate::src::nvim::charset::{
    hex2nr, skipdigits, skiptowhite, skipwhite, vim_isIDc, vim_str2nr,
};
use crate::src::nvim::eval::encode::{encode_list_write, encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::executor::eexe_mod_op;
use crate::src::nvim::eval::gc::{gc_first_dict, gc_first_list};
use crate::src::nvim::eval::typval::{
    callback_free, callback_put, tv_blob_alloc, tv_blob_alloc_ret, tv_blob_check_index,
    tv_blob_check_range, tv_blob_copy, tv_blob_equal, tv_blob_set_append, tv_blob_set_range,
    tv_blob_slice_or_index, tv_blob_unref, tv_check_lock, tv_check_num, tv_check_str, tv_clear,
    tv_copy, tv_dict_add, tv_dict_add_nr, tv_dict_alloc, tv_dict_copy, tv_dict_equal, tv_dict_find,
    tv_dict_free, tv_dict_free_contents, tv_dict_free_dict, tv_dict_get_callback,
    tv_dict_get_number, tv_dict_item_alloc, tv_dict_item_free, tv_dict_unref,
    tv_dict_watcher_notify, tv_dict_wrong_func_name, tv_empty_string, tv_equal, tv_get_float,
    tv_get_number, tv_get_number_chk, tv_get_string, tv_get_string_buf, tv_get_string_buf_chk,
    tv_get_string_chk, tv_in_free_unref_items, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_dict, tv_list_append_owned_tv, tv_list_append_string, tv_list_assign_range,
    tv_list_check_range_index_one, tv_list_check_range_index_two, tv_list_concat, tv_list_copy,
    tv_list_equal, tv_list_find, tv_list_find_nr, tv_list_free, tv_list_free_contents,
    tv_list_free_list, tv_list_join, tv_list_slice_or_index, tv_list_unref, tv_list_watch_add,
    tv_list_watch_remove, tv2bool, value_check_lock,
};
use crate::src::nvim::eval::typval::{
    tv_blob_get, tv_blob_len, tv_blob_set_ret, tv_dict_is_watched, tv_dict_set_ret,
    tv_dict_watcher_node_data, tv_is_func, tv_list_copyid, tv_list_first, tv_list_last,
    tv_list_len, tv_list_ref, tv_list_set_lock, tv_list_set_ret,
};
use crate::src::nvim::eval::userfunc::{
    call_func, call_simple_func, call_simple_luafunc, deref_func_name, eval_fname_script,
    find_func, free_unref_funccal, func_init, func_ptr_unref, func_ref, func_unref,
    get_current_funccal, get_func_tv, get_funccal_args_ht, get_lambda_tv, get_scriptlocal_funcname,
    make_partial, restore_funccal, save_funccal, set_ref_in_call_stack, set_ref_in_func,
    set_ref_in_func_args, set_ref_in_functions, set_ref_in_previous_funccal,
};
use crate::src::nvim::eval::vars::{
    check_vars, eval_one_expr_in_str, eval_variable, evalvars_init, ex_let_vars, find_var,
    garbage_collect_globvars, garbage_collect_scriptvars, garbage_collect_vimvars,
    get_vim_var_dict, get_vim_var_partial, get_vimvar_dict, optval_as_tv, set_var, set_var_const,
    set_vim_var_list, set_vim_var_nr, skip_var_list, valid_varname, var_check_lock, var_check_ro,
    var_wrong_func_name,
};
use crate::src::nvim::event::multiqueue::{multiqueue_free, multiqueue_new_child};
use crate::src::nvim::event::proc::proc_is_stopped;
use crate::src::nvim::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_docmd::{
    check_nextcmd, cmd_has_expr_args, do_cmdline, ends_excmd, get_pressedreturn, set_pressedreturn,
    set_ref_in_findfunc,
};
use crate::src::nvim::ex_eval::{aborting, discard_current_exception};
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_init;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::highlight_group::syn_name2id;
use crate::src::nvim::insexpand::{set_ref_in_cpt_callbacks, set_ref_in_insexpand_funcs};
use crate::src::nvim::keycodes::{find_special_key, trans_special};
use crate::src::nvim::lua::executor::{
    nlua_call_ref, nlua_is_deferred_safe, nlua_is_table_from_lua, nlua_register_table_as_callable,
};
use crate::src::nvim::main::aucmd_win_vec;
use crate::src::nvim::main::{
    EVALARG_EVALUATE, VIsual, VIsual_active, autocmd_bufnr, autocmd_fname, autocmd_fname_full,
    autocmd_match, called_emsg, channels, curbuf, current_sctx, curtab, curwin, did_emsg,
    did_throw, do_profiling, e_cannot_mod, e_command_too_recursive, e_dictkey, e_dictkey_len,
    e_fast_api_disabled, e_illvar, e_invalblob, e_invalid_value_for_blob_nr, e_invarg, e_invarg2,
    e_invargNval, e_invchan, e_invchanjob, e_invexpr2, e_letwrong, e_missingparen, e_nobufnr,
    e_not_callable_type_str, e_stray_closing_curly_str, e_trailing_arg, e_using_float_as_string,
    empty_string_option, emsg_off, emsg_severe, emsg_skip, first_tabpage, firstbuf, firstwin,
    force_abort, garbage_collect_at_exit, got_int, line_msg, main_loop, may_garbage_collect,
    msg_didout, msg_ext_skip_verbose, need_clr_eos, p_cpo, p_ic, p_lpl, p_mfd, p_verbose,
    provider_call_nesting, provider_caller_scope, sandbox, textlock, want_garbage_collect,
};
use crate::src::nvim::map::{map_del_uint64_t_ptr_t, map_put_ref_uint64_t_ptr_t, mh_get_uint64_t};
use crate::src::nvim::mark::{mark_get, mark_global_iter};
use crate::src::nvim::mbyte::{
    mb_charlen, mb_copy_char, mb_strcmp_ic, string_convert, utf_char2bytes, utf_head_off,
    utfc_ptr2len,
};
use crate::src::nvim::memline::{ml_append, ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{
    memchrsub, strchrsub, strequal, strnequal, xcalloc, xfree, xmalloc, xmemdupz, xrealloc, xstrdup,
};
use crate::src::nvim::message::{
    emsg, emsg_multiline, iemsg, internal_error, msg, msg_clr_eos, msg_end, msg_ext_set_append,
    msg_ext_set_kind, msg_multiline, msg_outnum, msg_puts, msg_puts_hl, msg_puts_len, msg_sb_eol,
    msg_start, semsg, smsg, verb_msg, verbose_enter, verbose_enter_scroll, verbose_leave,
    verbose_leave_scroll,
};
use crate::src::nvim::r#move::{check_cursor_moved, update_topline, validate_botline_win};
use crate::src::nvim::ops::set_ref_in_opfunc;
use crate::src::nvim::option::{
    find_option_end, get_option_value, get_tty_option, is_option_hidden, is_tty_option,
    set_option_value_give_err, was_set_insecurely,
};
use crate::src::nvim::options::{
    kOptAleph, kOptCpoptions, kOptFoldexpr, kOptFoldtext, kOptInvalid,
};
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::env::{expand_env_save, vim_getenv};
use crate::src::nvim::os::fs::os_can_exe;
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, abort, atol, gettext, memcmp, memcpy, memmove, memset, snprintf,
    strcmp, strcpy, strlen, strncasecmp, strncmp, strpbrk, strstr, strtod, toupper,
};
use crate::src::nvim::os::shell::{
    os_system, shell_argv_to_str, shell_build_argv, shell_free_argv,
};
use crate::src::nvim::profile::{prof_child_enter, prof_child_exit};
use crate::src::nvim::quickfix::set_ref_in_quickfix;
use crate::src::nvim::regexp::{vim_regcomp, vim_regexec_nl, vim_regfree, vim_regsub};
use crate::src::nvim::register::{get_reg_contents, op_global_reg_iter};
use crate::src::nvim::runtime::{
    exestack, get_scriptname, script_autoload, script_is_lua, sourcing_a_script,
};
use crate::src::nvim::strings::{concat_str, vim_snprintf, vim_strchr, xstrnsave};
use crate::src::nvim::tag::set_ref_in_tagfunc;
use crate::src::nvim::types::{
    AdditionalData, Arena, Array, BoolVarValue, CMD_index, Callback, CallbackReader, CallbackType,
    Channel, ChannelStreamType, DictWatcher, Error, EvalFuncData, GRegFlags, ListLenSpecials,
    LuaRetMode, Map_uint64_t_ptr_t, MapHash, MarkGet, MotionType, Object, ObjectType, OptIndex,
    OptInt, OptVal, OptValData, OptValType, QUEUE, ScopeType, Set_uint64_t, String_0, TimeWatcher,
    UIExtension, VarLockStatus, VarType, VimVarIndex, blob_T, buf_T, caller_scope, cmdidx_T,
    colnr_T, dict_T, dictitem_T, estack_T, evalarg_T, exarg_T, expand_T, exprtype_T, float_T,
    fmark_T, fmarkv_T, funccal_entry_T, funcexe_T, garray_T, hashitem_T, hashtab_T, ht_stack_S,
    ht_stack_T, int64_t, key_extra, linenr_T, list_T, list_stack_S, list_stack_T, listitem_T,
    listwatch_T, lval_T, object, object_data, partial_T, pos_T, proftime_T, ptr_t, ptrdiff_t,
    regmatch_T, regprog_T, save_v_event_T, sctx_T, size_t, ssize_t, tabpage_T, timer_T, typval_T,
    typval_vval_union, ufunc_T, uint8_t, uint32_t, uint64_t, uvarnumber_T, var_flavour_T,
    varnumber_T, vimconv_T, win_T, xfmark_T, yankreg_T,
};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::undo::u_clearallandblockfree;
use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_ushort, c_void};
pub const _ISalnum: c_uint = 8;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub const HLF_E: c_uint = 6;
pub const EXPAND_ENV_VARS: c_int = 26;
pub const EXPAND_EXPRESSION: c_int = 20;
pub const EXPAND_FUNCTIONS: c_int = 18;
pub const EXPAND_USER_VARS: c_int = 15;
pub const EXPAND_SETTINGS: c_int = 4;
pub const EXPAND_COMMANDS: c_int = 1;
pub const EXPAND_NOTHING: c_int = 0;
pub const REGSUB_MAGIC: c_uint = 2;
pub const REGSUB_COPY: c_uint = 1;
pub const kWinOptFoldexpr: c_int = 15;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNil: OptValType = -1;
pub const kMarkAll: MarkGet = 1;
pub const CMD_let: CMD_index = 231;
pub const CMD_execute: CMD_index = 151;
pub const CMD_echon: CMD_index = 139;
pub const CMD_echomsg: CMD_index = 138;
pub const CMD_echoerr: CMD_index = 136;
pub const CMD_echo: CMD_index = 135;
pub const CMD_const: CMD_index = 99;
pub const CMD_call: CMD_index = 53;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub const kUIMessages: UIExtension = 4;
pub const STR2NR_ALL: c_uint = 15;
pub const VV_LUA: VimVarIndex = 101;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_EVENT: VimVarIndex = 85;
pub const VV_SHELL_ERROR: VimVarIndex = 6;
pub const CONV_NONE: c_uint = 0;
pub const VAR_FLAVOUR_SHADA: var_flavour_T = 4;
pub const VAR_FLAVOUR_SESSION: var_flavour_T = 2;
pub const VAR_FLAVOUR_DEFAULT: var_flavour_T = 1;
pub const GLV_READ_ONLY: c_uint = 16;
pub const GLV_NO_AUTOLOAD: c_uint = 4;
pub const GLV_QUIET: c_uint = 2;
pub const EXPR_ISNOT: exprtype_T = 10;
pub const EXPR_IS: exprtype_T = 9;
pub const EXPR_NOMATCH: exprtype_T = 8;
pub const EXPR_MATCH: exprtype_T = 7;
pub const EXPR_SEQUAL: exprtype_T = 6;
pub const EXPR_SMALLER: exprtype_T = 5;
pub const EXPR_GEQUAL: exprtype_T = 4;
pub const EXPR_GREATER: exprtype_T = 3;
pub const EXPR_NEQUAL: exprtype_T = 2;
pub const EXPR_EQUAL: exprtype_T = 1;
pub const EXPR_UNKNOWN: exprtype_T = 0;
pub const EVAL_EVALUATE: c_uint = 1;
pub const KE_SNR: key_extra = 82;
pub const kGRegExprSrc: GRegFlags = 2;
pub const FSK_IN_STRING: c_uint = 4;
pub const FSK_KEYCODE: c_uint = 1;
pub const FSK_SIMPLIFY: c_uint = 8;
pub const OPT_LOCAL: c_uint = 2;
pub const OPT_GLOBAL: c_uint = 1;
pub const GLV_STOP: glv_status_T = 2;
pub type glv_status_T = c_uint;
pub const GLV_OK: glv_status_T = 1;
pub const GLV_FAIL: glv_status_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct forinfo_T {
    pub fi_semicolon: c_int,
    pub fi_varcount: c_int,
    pub fi_lw: listwatch_T,
    pub fi_list: *mut list_T,
    pub fi_bi: c_int,
    pub fi_blob: *mut blob_T,
    pub fi_string: *mut c_char,
    pub fi_byte_idx: c_int,
}
pub const kMTCharWise: MotionType = 0;
pub const kRetNilBool: LuaRetMode = 1;
pub const DOCMD_VERBOSE: c_uint = 1;
pub const DOCMD_NOWAIT: c_uint = 2;
pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const NULL_0: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const INT64_MIN: c_long = -9223372036854775807 as c_long - 1 as c_long;
pub const INT64_MAX: c_long = 9223372036854775807 as c_long;
pub const UINT32_MAX: c_uint = 4294967295 as c_uint;
pub const SIZE_MAX: c_ulong = 18446744073709551615 as c_ulong;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_uint64_t = Set_uint64_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint64_t>(),
};
pub const MAP_INIT: Map_uint64_t_ptr_t = Map_uint64_t_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_get_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_put_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_uint64_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut uint64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
pub const VARNUMBER_MAX: c_long = INT64_MAX;
pub const VARNUMBER_MIN: c_long = INT64_MIN;
pub const NUL: c_int = '\0' as c_int;
pub const BS: c_int = '\u{8}' as c_int;
pub const TAB: c_int = '\t' as c_int;
pub const NL: c_int = '\n' as c_int;
pub const FF: c_int = '\u{c}' as c_int;
pub const CAR: c_int = '\r' as c_int;
pub const ESC: c_int = '\u{1b}' as c_int;
pub const OK: c_int = 1 as c_int;
pub const FAIL: c_int = 0 as c_int;
pub const NOTDONE: c_int = 2 as c_int;
pub const COPYID_INC: c_int = 2 as c_int;
pub const COPYID_MASK: c_int = !(0x1 as c_int);
pub const FNE_INCL_BR: c_int = 1 as c_int;
pub const FNE_CHECK_START: c_int = 2 as c_int;
pub const AUTOLOAD_CHAR: c_int = '#' as c_int;
pub const DICT_MAXNEST: c_int = 100 as c_int;
static e_missbrac: GlobalCell<*const c_char> =
    GlobalCell::new(b"E111: Missing ']'\0".as_ptr() as *const c_char);
static e_list_end: GlobalCell<*const c_char> =
    GlobalCell::new(b"E697: Missing end of List ']': %s\0".as_ptr() as *const c_char);
static e_cannot_slice_dictionary: GlobalCell<[c_char; 32]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 32], [c_char; 32]>(*b"E719: Cannot slice a Dictionary\0")
});
static e_cannot_index_special_variable: GlobalCell<[c_char; 38]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 38], [c_char; 38]>(*b"E909: Cannot index a special variable\0")
});
static e_nowhitespace: GlobalCell<*const c_char> =
    GlobalCell::new(b"E274: No white space allowed before parenthesis\0".as_ptr() as *const c_char);
static e_cannot_index_a_funcref: GlobalCell<[c_char; 29]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 29], [c_char; 29]>(*b"E695: Cannot index a Funcref\0")
});
static e_variable_nested_too_deep_for_making_copy: GlobalCell<[c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [c_char; 49]>(
            *b"E698: Variable nested too deep for making a copy\0",
        )
    });
static e_string_list_or_blob_required: GlobalCell<[c_char; 37]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 37], [c_char; 37]>(*b"E1098: String, List or Blob required\0")
});
static e_expression_too_recursive_str: GlobalCell<[c_char; 36]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 36], [c_char; 36]>(*b"E1169: Expression too recursive: %s\0")
});
static e_dot_can_only_be_used_on_dictionary_str: GlobalCell<[c_char; 48]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 48], [c_char; 48]>(
            *b"E1203: Dot can only be used on a dictionary: %s\0",
        )
    });
static e_empty_function_name: GlobalCell<[c_char; 27]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 27], [c_char; 27]>(*b"E1192: Empty function name\0")
});
static e_cannot_use_partial_here: GlobalCell<[c_char; 33]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 33], [c_char; 33]>(*b"E1265: Cannot use a partial here\0")
});
static namespace_char: GlobalCell<*mut c_char> =
    GlobalCell::new(b"abglstvw\0".as_ptr() as *const c_char as *mut c_char);
pub static eval_lavars_used: GlobalCell<*mut bool> =
    GlobalCell::new(::core::ptr::null_mut::<bool>());
static echo_hl_id: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static last_timer_id: GlobalCell<uint64_t> = GlobalCell::new(1 as uint64_t);
static timers: GlobalCell<Map_uint64_t_ptr_t> = GlobalCell::new(MAP_INIT);
pub unsafe extern "C" fn get_v_event(mut sve: *mut save_v_event_T) -> *mut dict_T {
    let mut v_event: *mut dict_T = get_vim_var_dict(VV_EVENT);
    if (*v_event).dv_hashtab.ht_used > 0 as size_t {
        (*sve).sve_did_save = true_0 != 0;
        (*sve).sve_hashtab = (*v_event).dv_hashtab;
        hash_init(&raw mut (*v_event).dv_hashtab);
    } else {
        (*sve).sve_did_save = false_0 != 0;
    }
    return v_event;
}
pub unsafe extern "C" fn restore_v_event(mut v_event: *mut dict_T, mut sve: *mut save_v_event_T) {
    tv_dict_free_contents(v_event);
    if (*sve).sve_did_save {
        (*v_event).dv_hashtab = (*sve).sve_hashtab;
    } else {
        hash_init(&raw mut (*v_event).dv_hashtab);
    };
}
pub unsafe extern "C" fn num_divide(mut n1: varnumber_T, mut n2: varnumber_T) -> varnumber_T {
    let mut result: varnumber_T = 0;
    if n2 == 0 as varnumber_T {
        if n1 == 0 as varnumber_T {
            result = VARNUMBER_MIN as varnumber_T;
        } else if n1 < 0 as varnumber_T {
            result = -VARNUMBER_MAX as varnumber_T;
        } else {
            result = VARNUMBER_MAX as varnumber_T;
        }
    } else if n1 == VARNUMBER_MIN as varnumber_T && n2 == -1 as varnumber_T {
        result = VARNUMBER_MAX as varnumber_T;
    } else {
        result = n1 / n2;
    }
    return result;
}
pub unsafe extern "C" fn num_modulus(mut n1: varnumber_T, mut n2: varnumber_T) -> varnumber_T {
    return if n2 == 0 as varnumber_T {
        0 as varnumber_T
    } else {
        n1 % n2
    };
}
pub unsafe extern "C" fn eval_init() {
    evalvars_init();
    func_init();
}
pub unsafe extern "C" fn fill_evalarg_from_eap(
    mut evalarg: *mut evalarg_T,
    mut eap: *mut exarg_T,
    mut skip: bool,
) {
    *evalarg = evalarg_T {
        eval_flags: if skip as c_int != 0 {
            0 as c_int
        } else {
            EVAL_EVALUATE as c_int
        },
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    if eap.is_null() {
        return;
    }
    if sourcing_a_script(eap) != 0 {
        (*evalarg).eval_getline = (*eap).ea_getline;
        (*evalarg).eval_cookie = (*eap).cookie;
    }
}
pub unsafe extern "C" fn eval_to_bool(
    mut arg: *mut c_char,
    mut error: *mut bool,
    mut eap: *mut exarg_T,
    skip: bool,
    use_simple_function: bool,
) -> bool {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: bool = false_0 != 0;
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, skip);
    if skip {
        (*emsg_skip.ptr()) += 1;
    }
    let mut r: c_int = if use_simple_function as c_int != 0 {
        eval0_simple_funccal(arg, &raw mut tv, eap, &raw mut evalarg)
    } else {
        eval0(arg, &raw mut tv, eap, &raw mut evalarg)
    };
    if r == FAIL {
        *error = true_0 != 0;
    } else {
        *error = false_0 != 0;
        if !skip {
            retval = tv_get_number_chk(&raw mut tv, error) != 0 as varnumber_T;
            tv_clear(&raw mut tv);
        }
    }
    if skip {
        (*emsg_skip.ptr()) -= 1;
    }
    clear_evalarg(&raw mut evalarg, eap);
    return retval;
}
unsafe extern "C" fn eval1_emsg(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut eap: *mut exarg_T,
) -> c_int {
    let start: *const c_char = *arg;
    let did_emsg_before: c_int = did_emsg.get();
    let called_emsg_before: c_int = called_emsg.get();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
    let ret: c_int = eval1(arg, rettv, &raw mut evalarg);
    if ret == FAIL {
        if !aborting()
            && did_emsg.get() == did_emsg_before
            && called_emsg.get() == called_emsg_before
        {
            semsg(gettext(&raw const e_invexpr2 as *const c_char), start);
        }
    }
    clear_evalarg(&raw mut evalarg, eap);
    return ret;
}
pub unsafe extern "C" fn eval_expr_valid_arg(tv: *const typval_T) -> bool {
    return (*tv).v_type as c_uint != VAR_UNKNOWN as c_int as c_uint
        && ((*tv).v_type as c_uint != VAR_STRING as c_int as c_uint
            || !(*tv).vval.v_string.is_null() && *(*tv).vval.v_string as c_int != NUL);
}
unsafe extern "C" fn eval_expr_partial(
    mut expr: *const typval_T,
    mut argv: *mut typval_T,
    mut argc: c_int,
    mut rettv: *mut typval_T,
) -> c_int {
    let partial: *mut partial_T = (*expr).vval.v_partial;
    if partial.is_null() {
        return FAIL;
    }
    let s: *const c_char = partial_name(partial);
    if s.is_null() || *s as c_int == NUL {
        return FAIL;
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_evaluate = true_0 != 0;
    funcexe.fe_partial = partial;
    if call_func(s, -1 as c_int, rettv, argc, argv, &raw mut funcexe) == FAIL {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn eval_expr_func(
    mut expr: *const typval_T,
    mut argv: *mut typval_T,
    mut argc: c_int,
    mut rettv: *mut typval_T,
) -> c_int {
    let mut buf: [c_char; 65] = [0; 65];
    let s: *const c_char = if (*expr).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        (*expr).vval.v_string as *const c_char
    } else {
        tv_get_string_buf_chk(expr, &raw mut buf as *mut c_char)
    };
    if s.is_null() || *s as c_int == NUL {
        return FAIL;
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_evaluate = true_0 != 0;
    if call_func(s, -1 as c_int, rettv, argc, argv, &raw mut funcexe) == FAIL {
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn eval_expr_string(
    mut expr: *const typval_T,
    mut rettv: *mut typval_T,
) -> c_int {
    let mut buf: [c_char; 65] = [0; 65];
    let mut s: *mut c_char =
        tv_get_string_buf_chk(expr, &raw mut buf as *mut c_char) as *mut c_char;
    if s.is_null() {
        return FAIL;
    }
    s = skipwhite(s);
    if eval1_emsg(&raw mut s, rettv, ::core::ptr::null_mut::<exarg_T>()) == FAIL {
        return FAIL;
    }
    if *skipwhite(s) as c_int != NUL {
        tv_clear(rettv);
        semsg(gettext(&raw const e_invexpr2 as *const c_char), s);
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn eval_expr_typval(
    mut expr: *const typval_T,
    mut want_func: bool,
    mut argv: *mut typval_T,
    mut argc: c_int,
    mut rettv: *mut typval_T,
) -> c_int {
    if (*expr).v_type as c_uint == VAR_PARTIAL as c_int as c_uint {
        return eval_expr_partial(expr, argv, argc, rettv);
    }
    if (*expr).v_type as c_uint == VAR_FUNC as c_int as c_uint || want_func as c_int != 0 {
        return eval_expr_func(expr, argv, argc, rettv);
    }
    return eval_expr_string(expr, rettv);
}
pub unsafe extern "C" fn eval_expr_to_bool(
    mut expr: *const typval_T,
    mut error: *mut bool,
) -> bool {
    let mut argv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if eval_expr_typval(
        expr,
        false_0 != 0,
        &raw mut argv,
        0 as c_int,
        &raw mut rettv,
    ) == FAIL
    {
        *error = true_0 != 0;
        return false_0 != 0;
    }
    let res: bool = tv_get_number_chk(&raw mut rettv, error) != 0 as varnumber_T;
    tv_clear(&raw mut rettv);
    return res;
}
pub unsafe extern "C" fn eval_to_string_skip(
    mut arg: *mut c_char,
    mut eap: *mut exarg_T,
    skip: bool,
) -> *mut c_char {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, skip);
    if skip {
        (*emsg_skip.ptr()) += 1;
    }
    if eval0(arg, &raw mut tv, eap, &raw mut evalarg) == FAIL || skip as c_int != 0 {
        retval = ::core::ptr::null_mut::<c_char>();
    } else {
        retval = xstrdup(tv_get_string(&raw mut tv));
        tv_clear(&raw mut tv);
    }
    if skip {
        (*emsg_skip.ptr()) -= 1;
    }
    clear_evalarg(&raw mut evalarg, eap);
    return retval;
}
pub unsafe extern "C" fn skip_expr(mut pp: *mut *mut c_char, evalarg: *mut evalarg_T) -> c_int {
    let save_flags: c_int = if evalarg.is_null() {
        0 as c_int
    } else {
        (*evalarg).eval_flags
    };
    if !evalarg.is_null() {
        (*evalarg).eval_flags &= !(EVAL_EVALUATE as c_int);
    }
    *pp = skipwhite(*pp);
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut res: c_int = eval1(pp, &raw mut rettv, ::core::ptr::null_mut::<evalarg_T>());
    if !evalarg.is_null() {
        (*evalarg).eval_flags = save_flags;
    }
    return res;
}
unsafe extern "C" fn typval2string(mut tv: *mut typval_T, mut join_list: bool) -> *mut c_char {
    if join_list as c_int != 0 && (*tv).v_type as c_uint == VAR_LIST as c_int as c_uint {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<c_char>() as c_int,
            80 as c_int,
        );
        if !(*tv).vval.v_list.is_null() {
            tv_list_join(
                &raw mut ga,
                (*tv).vval.v_list,
                b"\n\0".as_ptr() as *const c_char,
            );
            if tv_list_len((*tv).vval.v_list) > 0 as c_int {
                ga_append(&raw mut ga, NL as uint8_t);
            }
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        return ga.ga_data as *mut c_char;
    } else if (*tv).v_type as c_uint == VAR_LIST as c_int as c_uint
        || (*tv).v_type as c_uint == VAR_DICT as c_int as c_uint
    {
        return encode_tv2string(tv, ::core::ptr::null_mut::<size_t>());
    }
    return xstrdup(tv_get_string(tv));
}
pub unsafe extern "C" fn eval_to_string_eap(
    mut arg: *mut c_char,
    join_list: bool,
    mut eap: *mut exarg_T,
    use_simple_function: bool,
) -> *mut c_char {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
    let mut r: c_int = if use_simple_function as c_int != 0 {
        eval0_simple_funccal(
            arg,
            &raw mut tv,
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut evalarg,
        )
    } else {
        eval0(
            arg,
            &raw mut tv,
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut evalarg,
        )
    };
    if r == FAIL {
        retval = ::core::ptr::null_mut::<c_char>();
    } else {
        retval = typval2string(&raw mut tv, join_list);
        tv_clear(&raw mut tv);
    }
    clear_evalarg(&raw mut evalarg, ::core::ptr::null_mut::<exarg_T>());
    return retval;
}
pub unsafe extern "C" fn eval_to_string(
    mut arg: *mut c_char,
    join_list: bool,
    use_simple_function: bool,
) -> *mut c_char {
    return eval_to_string_eap(
        arg,
        join_list,
        ::core::ptr::null_mut::<exarg_T>(),
        use_simple_function,
    );
}
pub unsafe extern "C" fn eval_to_string_safe(
    mut arg: *mut c_char,
    use_sandbox: bool,
    use_simple_function: bool,
) -> *mut c_char {
    let mut retval: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccal_entry);
    if use_sandbox {
        (*sandbox.ptr()) += 1;
    }
    (*textlock.ptr()) += 1;
    retval = eval_to_string(arg, false_0 != 0, use_simple_function);
    if use_sandbox {
        (*sandbox.ptr()) -= 1;
    }
    (*textlock.ptr()) -= 1;
    restore_funccal();
    return retval;
}
pub unsafe extern "C" fn eval_to_number(
    mut expr: *mut c_char,
    use_simple_function: bool,
) -> varnumber_T {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: varnumber_T = 0;
    let mut p: *mut c_char = skipwhite(expr);
    let mut r: c_int = NOTDONE;
    (*emsg_off.ptr()) += 1;
    if use_simple_function {
        r = may_call_simple_func(expr, &raw mut rettv);
    }
    if r == NOTDONE {
        r = eval1(&raw mut p, &raw mut rettv, EVALARG_EVALUATE.ptr());
    }
    if r == FAIL {
        retval = -1 as varnumber_T;
    } else {
        retval = tv_get_number_chk(&raw mut rettv, ::core::ptr::null_mut::<bool>());
        tv_clear(&raw mut rettv);
    }
    (*emsg_off.ptr()) -= 1;
    return retval;
}
pub unsafe extern "C" fn eval_expr(mut arg: *mut c_char, mut eap: *mut exarg_T) -> *mut typval_T {
    return eval_expr_ext(arg, eap, false_0 != 0);
}
pub unsafe extern "C" fn eval_expr_ext(
    mut arg: *mut c_char,
    mut eap: *mut exarg_T,
    use_simple_function: bool,
) -> *mut typval_T {
    let mut tv: *mut typval_T = xmalloc(::core::mem::size_of::<typval_T>()) as *mut typval_T;
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
    let mut r: c_int = NOTDONE;
    if use_simple_function {
        r = eval0_simple_funccal(arg, tv, eap, &raw mut evalarg);
    }
    if r == NOTDONE {
        r = eval0(arg, tv, eap, &raw mut evalarg);
    }
    if r == FAIL {
        let mut ptr_: *mut *mut c_void = &raw mut tv as *mut *mut c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
    clear_evalarg(&raw mut evalarg, eap);
    return tv;
}
pub unsafe extern "C" fn call_vim_function(
    mut func: *const c_char,
    mut argc: c_int,
    mut argv: *mut typval_T,
    mut rettv: *mut typval_T,
) -> c_int {
    let mut funcexe: funcexe_T = funcexe_T {
        fe_argv_func: None,
        fe_firstline: 0,
        fe_lastline: 0,
        fe_doesrange: ::core::ptr::null_mut::<bool>(),
        fe_evaluate: false,
        fe_partial: ::core::ptr::null_mut::<partial_T>(),
        fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
        fe_basetv: ::core::ptr::null_mut::<typval_T>(),
        fe_found_var: false,
    };
    let mut ret: c_int = 0;
    let mut len: c_int = strlen(func) as c_int;
    let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    '_fail: {
        if len >= 6 as c_int
            && memcmp(
                func as *const c_void,
                b"v:lua.\0".as_ptr() as *const c_char as *const c_void,
                6 as size_t,
            ) == 0
        {
            func = func.offset(6 as c_int as isize);
            len = check_luafunc_name(func, false_0 != 0);
            if len == 0 as c_int {
                ret = FAIL;
                break '_fail;
            } else {
                pt = get_vim_var_partial(VV_LUA);
            }
        }
        (*rettv).v_type = VAR_UNKNOWN;
        funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = true_0 != 0;
        funcexe.fe_partial = pt;
        ret = call_func(func, len, rettv, argc, argv, &raw mut funcexe);
    }
    if ret == FAIL {
        tv_clear(rettv);
    }
    return ret;
}
pub unsafe extern "C" fn call_func_retstr(
    func: *const c_char,
    mut argc: c_int,
    mut argv: *mut typval_T,
) -> *mut c_void {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if call_vim_function(func, argc, argv, &raw mut rettv) == FAIL {
        return NULL_0;
    }
    let retval: *mut c_char = xstrdup(tv_get_string(&raw mut rettv));
    tv_clear(&raw mut rettv);
    return retval as *mut c_void;
}
pub unsafe extern "C" fn call_func_retlist(
    mut func: *const c_char,
    mut argc: c_int,
    mut argv: *mut typval_T,
) -> *mut c_void {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if call_vim_function(func, argc, argv, &raw mut rettv) == FAIL {
        return NULL_0;
    }
    if rettv.v_type as c_uint != VAR_LIST as c_int as c_uint {
        tv_clear(&raw mut rettv);
        return NULL_0;
    }
    return rettv.vval.v_list as *mut c_void;
}
pub unsafe extern "C" fn eval_foldexpr(mut wp: *mut win_T, mut cp: *mut c_int) -> c_int {
    let saved_sctx: sctx_T = current_sctx.get();
    let use_sandbox: bool = was_set_insecurely(wp, kOptFoldexpr, OPT_LOCAL as c_int);
    let mut arg: *mut c_char = skipwhite((*wp).w_onebuf_opt.wo_fde);
    current_sctx.set((*wp).w_onebuf_opt.wo_script_ctx[kWinOptFoldexpr as c_int as usize]);
    (*emsg_off.ptr()) += 1;
    if use_sandbox {
        (*sandbox.ptr()) += 1;
    }
    (*textlock.ptr()) += 1;
    *cp = NUL;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: varnumber_T = 0;
    if eval0_simple_funccal(
        arg,
        &raw mut tv,
        ::core::ptr::null_mut::<exarg_T>(),
        EVALARG_EVALUATE.ptr(),
    ) == FAIL
    {
        retval = 0 as varnumber_T;
    } else {
        if tv.v_type as c_uint == VAR_NUMBER as c_int as c_uint {
            retval = tv.vval.v_number;
        } else if tv.v_type as c_uint != VAR_STRING as c_int as c_uint || tv.vval.v_string.is_null()
        {
            retval = 0 as varnumber_T;
        } else {
            let mut s: *mut c_char = tv.vval.v_string;
            if *s as c_int != NUL && !ascii_isdigit(*s as c_int) && *s as c_int != '-' as c_int {
                let c2rust_fresh10 = s;
                s = s.offset(1);
                *cp = *c2rust_fresh10 as uint8_t as c_int;
            }
            retval = atol(s) as varnumber_T;
        }
        tv_clear(&raw mut tv);
    }
    (*emsg_off.ptr()) -= 1;
    if use_sandbox {
        (*sandbox.ptr()) -= 1;
    }
    (*textlock.ptr()) -= 1;
    clear_evalarg(EVALARG_EVALUATE.ptr(), ::core::ptr::null_mut::<exarg_T>());
    current_sctx.set(saved_sctx);
    return retval as c_int;
}
pub unsafe extern "C" fn eval_foldtext(mut wp: *mut win_T) -> Object {
    let use_sandbox: bool = was_set_insecurely(wp, kOptFoldtext, OPT_LOCAL as c_int);
    let mut arg: *mut c_char = (*wp).w_onebuf_opt.wo_fdt;
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccal_entry);
    if use_sandbox {
        (*sandbox.ptr()) += 1;
    }
    (*textlock.ptr()) += 1;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: Object = Object {
        type_0: kObjectTypeNil,
        data: object_data { boolean: false },
    };
    if eval0_simple_funccal(
        arg,
        &raw mut tv,
        ::core::ptr::null_mut::<exarg_T>(),
        EVALARG_EVALUATE.ptr(),
    ) == FAIL
    {
        retval = object {
            type_0: kObjectTypeString,
            data: object_data {
                string: String_0 {
                    data: ::core::ptr::null_mut::<c_char>(),
                    size: 0 as size_t,
                },
            },
        };
    } else {
        if tv.v_type as c_uint == VAR_LIST as c_int as c_uint {
            retval = vim_to_object(&raw mut tv, ::core::ptr::null_mut::<Arena>(), false_0 != 0);
        } else {
            retval = object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: cstr_to_string(tv_get_string(&raw mut tv)),
                },
            };
        }
        tv_clear(&raw mut tv);
    }
    clear_evalarg(EVALARG_EVALUATE.ptr(), ::core::ptr::null_mut::<exarg_T>());
    if use_sandbox {
        (*sandbox.ptr()) -= 1;
    }
    (*textlock.ptr()) -= 1;
    restore_funccal();
    return retval;
}
unsafe extern "C" fn to_name_end(mut arg: *const c_char, mut use_namespace: bool) -> *const c_char {
    if !eval_isnamec1(*arg as c_int) {
        return arg;
    }
    let mut p: *const c_char = ::core::ptr::null::<c_char>();
    p = arg.offset(1 as c_int as isize);
    while *p as c_int != NUL && eval_isnamec(*p as c_int) as c_int != 0 {
        if *p as c_int == ':' as c_int
            && (p != arg.offset(1 as c_int as isize)
                || !use_namespace
                || vim_strchr(b"bgstvw\0".as_ptr() as *const c_char, *arg as c_int).is_null())
        {
            break;
        }
        p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
    }
    return p;
}
unsafe extern "C" fn get_lval_dict_item(
    mut lp: *mut lval_T,
    mut name: *mut c_char,
    mut key: *mut c_char,
    mut len: c_int,
    mut key_end: *mut *mut c_char,
    mut var1: *mut typval_T,
    mut flags: c_int,
    mut unlet: bool,
    mut rettv: *mut typval_T,
) -> glv_status_T {
    let mut quiet: bool = flags & GLV_QUIET as c_int != 0;
    let mut p: *mut c_char = *key_end;
    if len == -1 as c_int {
        key = tv_get_string(var1) as *mut c_char;
    }
    (*lp).ll_list = ::core::ptr::null_mut::<list_T>();
    if (*(*lp).ll_tv).vval.v_dict.is_null() {
        (*(*lp).ll_tv).vval.v_dict = tv_dict_alloc();
        (*(*(*lp).ll_tv).vval.v_dict).dv_refcount += 1;
    }
    (*lp).ll_dict = (*(*lp).ll_tv).vval.v_dict;
    (*lp).ll_di = tv_dict_find((*lp).ll_dict, key, len as ptrdiff_t);
    if !rettv.is_null() && (*(*lp).ll_dict).dv_scope as c_uint != 0 as c_uint {
        let mut prevval: c_char = 0;
        if len != -1 as c_int {
            prevval = *key.offset(len as isize);
            *key.offset(len as isize) = NUL as c_char;
        } else {
            prevval = 0 as c_char;
        }
        let mut wrong: bool = (*(*lp).ll_dict).dv_scope as c_uint
            == VAR_DEF_SCOPE as c_int as c_uint
            && tv_is_func(*rettv) as c_int != 0
            && var_wrong_func_name(key, (*lp).ll_di.is_null()) as c_int != 0
            || !valid_varname(key);
        if len != -1 as c_int {
            *key.offset(len as isize) = prevval;
        }
        if wrong {
            return GLV_FAIL;
        }
    }
    if !(*lp).ll_di.is_null()
        && tv_is_luafunc(&raw mut (*(*lp).ll_di).di_tv) as c_int != 0
        && len == -1 as c_int
        && rettv.is_null()
    {
        semsg(
            &raw const e_illvar as *const c_char,
            b"v:['lua']\0".as_ptr() as *const c_char,
        );
        return GLV_FAIL;
    }
    if (*lp).ll_di.is_null() {
        if (*lp).ll_dict == get_vimvar_dict()
            || &raw mut (*(*lp).ll_dict).dv_hashtab == get_funccal_args_ht()
        {
            semsg(gettext(&raw const e_illvar as *const c_char), name);
            return GLV_FAIL;
        }
        if *p as c_int == '[' as c_int || *p as c_int == '.' as c_int || unlet as c_int != 0 {
            if !quiet {
                semsg(gettext(&raw const e_dictkey as *const c_char), key);
            }
            return GLV_FAIL;
        }
        if len == -1 as c_int {
            (*lp).ll_newkey = xstrdup(key);
        } else {
            (*lp).ll_newkey = xmemdupz(key as *const c_void, len as size_t) as *mut c_char;
        }
        *key_end = p;
        return GLV_STOP;
    } else if flags & GLV_READ_ONLY as c_int == 0
        && (var_check_ro(
            (*(*lp).ll_di).di_flags as c_int,
            name,
            p.offset_from(name) as size_t,
        ) as c_int
            != 0
            || var_check_lock(
                (*(*lp).ll_di).di_flags as c_int,
                name,
                p.offset_from(name) as size_t,
            ) as c_int
                != 0)
    {
        return GLV_FAIL;
    }
    (*lp).ll_tv = &raw mut (*(*lp).ll_di).di_tv;
    return GLV_OK;
}
unsafe extern "C" fn get_lval_blob(
    mut lp: *mut lval_T,
    mut var1: *mut typval_T,
    mut var2: *mut typval_T,
    mut empty1: bool,
    mut quiet: bool,
) -> c_int {
    let bloblen: c_int = tv_blob_len((*(*lp).ll_tv).vval.v_blob);
    if empty1 {
        (*lp).ll_n1 = 0 as c_int;
    } else {
        (*lp).ll_n1 = tv_get_number(var1) as c_int;
    }
    if tv_blob_check_index(bloblen, (*lp).ll_n1 as varnumber_T, quiet) == FAIL {
        return FAIL;
    }
    if (*lp).ll_range as c_int != 0 && !(*lp).ll_empty2 {
        (*lp).ll_n2 = tv_get_number(var2) as c_int;
        if tv_blob_check_range(
            bloblen,
            (*lp).ll_n1 as varnumber_T,
            (*lp).ll_n2 as varnumber_T,
            quiet,
        ) == FAIL
        {
            return FAIL;
        }
    }
    (*lp).ll_blob = (*(*lp).ll_tv).vval.v_blob;
    (*lp).ll_tv = ::core::ptr::null_mut::<typval_T>();
    return OK;
}
unsafe extern "C" fn get_lval_list(
    mut lp: *mut lval_T,
    mut var1: *mut typval_T,
    mut var2: *mut typval_T,
    mut empty1: bool,
    mut _flags: c_int,
    mut quiet: bool,
) -> c_int {
    if empty1 {
        (*lp).ll_n1 = 0 as c_int;
    } else {
        (*lp).ll_n1 = tv_get_number(var1) as c_int;
    }
    (*lp).ll_dict = ::core::ptr::null_mut::<dict_T>();
    (*lp).ll_list = (*(*lp).ll_tv).vval.v_list;
    (*lp).ll_li = tv_list_check_range_index_one((*lp).ll_list, &raw mut (*lp).ll_n1, quiet);
    if (*lp).ll_li.is_null() {
        return FAIL;
    }
    if (*lp).ll_range as c_int != 0 && !(*lp).ll_empty2 {
        (*lp).ll_n2 = tv_get_number(var2) as c_int;
        if tv_list_check_range_index_two(
            (*lp).ll_list,
            &raw mut (*lp).ll_n1,
            (*lp).ll_li,
            &raw mut (*lp).ll_n2,
            quiet,
        ) == FAIL
        {
            return FAIL;
        }
    }
    (*lp).ll_tv = &raw mut (*(*lp).ll_li).li_tv;
    return OK;
}
unsafe extern "C" fn get_lval_subscript(
    mut lp: *mut lval_T,
    mut p: *mut c_char,
    mut name: *mut c_char,
    mut rettv: *mut typval_T,
    mut _ht: *mut hashtab_T,
    mut _v: *mut dictitem_T,
    mut unlet: bool,
    mut flags: c_int,
) -> *mut c_char {
    let mut quiet: bool = flags & GLV_QUIET as c_int != 0;
    let mut var1: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    var1.v_type = VAR_UNKNOWN;
    let mut var2: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    var2.v_type = VAR_UNKNOWN;
    let mut empty1: bool = false_0 != 0;
    let mut rc: c_int = FAIL;
    '_done: {
        while *p as c_int == '[' as c_int
            || *p as c_int == '.' as c_int
                && *p.offset(1 as c_int as isize) as c_int != '=' as c_int
                && *p.offset(1 as c_int as isize) as c_int != '.' as c_int
        {
            if *p as c_int == '.' as c_int
                && (*(*lp).ll_tv).v_type as c_uint != VAR_DICT as c_int as c_uint
            {
                if !quiet {
                    semsg(
                        gettext(
                            (e_dot_can_only_be_used_on_dictionary_str.ptr() as *const _)
                                as *const c_char,
                        ),
                        name,
                    );
                }
                return ::core::ptr::null_mut::<c_char>();
            }
            if (*(*lp).ll_tv).v_type as c_uint != VAR_LIST as c_int as c_uint
                && (*(*lp).ll_tv).v_type as c_uint != VAR_DICT as c_int as c_uint
                && (*(*lp).ll_tv).v_type as c_uint != VAR_BLOB as c_int as c_uint
            {
                if !quiet {
                    emsg(gettext(
                        b"E689: Can only index a List, Dictionary or Blob\0".as_ptr()
                            as *const c_char,
                    ));
                }
                return ::core::ptr::null_mut::<c_char>();
            }
            if (*(*lp).ll_tv).v_type as c_uint == VAR_LIST as c_int as c_uint
                && (*(*lp).ll_tv).vval.v_list.is_null()
            {
                tv_list_alloc_ret((*lp).ll_tv, kListLenUnknown as c_int as ptrdiff_t);
            } else if (*(*lp).ll_tv).v_type as c_uint == VAR_BLOB as c_int as c_uint
                && (*(*lp).ll_tv).vval.v_blob.is_null()
            {
                tv_blob_alloc_ret((*lp).ll_tv);
            }
            if (*lp).ll_range {
                if !quiet {
                    emsg(gettext(
                        b"E708: [:] must come last\0".as_ptr() as *const c_char
                    ));
                }
                break '_done;
            } else {
                let mut len: c_int = -1 as c_int;
                let mut key: *mut c_char = ::core::ptr::null_mut::<c_char>();
                if *p as c_int == '.' as c_int {
                    key = p.offset(1 as c_int as isize);
                    len = 0 as c_int;
                    while *key.offset(len as isize) as c_uint >= 'A' as c_uint
                        && *key.offset(len as isize) as c_uint <= 'Z' as c_uint
                        || *key.offset(len as isize) as c_uint >= 'a' as c_uint
                            && *key.offset(len as isize) as c_uint <= 'z' as c_uint
                        || ascii_isdigit(*key.offset(len as isize) as c_int) as c_int != 0
                        || *key.offset(len as isize) as c_int == '_' as c_int
                    {
                        len += 1;
                    }
                    if len == 0 as c_int {
                        if !quiet {
                            emsg(gettext(
                                b"E713: Cannot use empty key after .\0".as_ptr() as *const c_char
                            ));
                        }
                        return ::core::ptr::null_mut::<c_char>();
                    }
                    p = key.offset(len as isize);
                } else {
                    p = skipwhite(p.offset(1 as c_int as isize));
                    if *p as c_int == ':' as c_int {
                        empty1 = true_0 != 0;
                    } else {
                        empty1 = false_0 != 0;
                        if eval1(&raw mut p, &raw mut var1, EVALARG_EVALUATE.ptr()) == FAIL {
                            break '_done;
                        }
                        if !tv_check_str(&raw mut var1) {
                            break '_done;
                        }
                        p = skipwhite(p);
                    }
                    if *p as c_int == ':' as c_int {
                        if (*(*lp).ll_tv).v_type as c_uint == VAR_DICT as c_int as c_uint {
                            if !quiet {
                                emsg(gettext(
                                    (e_cannot_slice_dictionary.ptr() as *const _) as *const c_char,
                                ));
                            }
                            break '_done;
                        } else if !rettv.is_null()
                            && !((*rettv).v_type as c_uint == VAR_LIST as c_int as c_uint
                                && !(*rettv).vval.v_list.is_null())
                            && !((*rettv).v_type as c_uint == VAR_BLOB as c_int as c_uint
                                && !(*rettv).vval.v_blob.is_null())
                        {
                            if !quiet {
                                emsg(gettext(
                                    b"E709: [:] requires a List or Blob value\0".as_ptr()
                                        as *const c_char,
                                ));
                            }
                            break '_done;
                        } else {
                            p = skipwhite(p.offset(1 as c_int as isize));
                            if *p as c_int == ']' as c_int {
                                (*lp).ll_empty2 = true_0 != 0;
                            } else {
                                (*lp).ll_empty2 = false_0 != 0;
                                if eval1(&raw mut p, &raw mut var2, EVALARG_EVALUATE.ptr()) == FAIL
                                {
                                    break '_done;
                                }
                                if !tv_check_str(&raw mut var2) {
                                    break '_done;
                                }
                            }
                            (*lp).ll_range = true_0 != 0;
                        }
                    } else {
                        (*lp).ll_range = false_0 != 0;
                    }
                    if *p as c_int != ']' as c_int {
                        if !quiet {
                            emsg(gettext(e_missbrac.get()));
                        }
                        break '_done;
                    } else {
                        p = p.offset(1);
                    }
                }
                if (*(*lp).ll_tv).v_type as c_uint == VAR_DICT as c_int as c_uint {
                    let mut glv_status: glv_status_T = get_lval_dict_item(
                        lp,
                        name,
                        key,
                        len,
                        &raw mut p,
                        &raw mut var1,
                        flags,
                        unlet,
                        rettv,
                    );
                    if glv_status as c_uint == GLV_FAIL as c_int as c_uint {
                        break '_done;
                    }
                    if glv_status as c_uint == GLV_STOP as c_int as c_uint {
                        break;
                    }
                } else if (*(*lp).ll_tv).v_type as c_uint == VAR_BLOB as c_int as c_uint {
                    if get_lval_blob(lp, &raw mut var1, &raw mut var2, empty1, quiet) == FAIL {
                        break '_done;
                    } else {
                        break;
                    }
                } else if get_lval_list(lp, &raw mut var1, &raw mut var2, empty1, flags, quiet)
                    == FAIL
                {
                    break '_done;
                }
                tv_clear(&raw mut var1);
                tv_clear(&raw mut var2);
                var1.v_type = VAR_UNKNOWN;
                var2.v_type = VAR_UNKNOWN;
            }
        }
        rc = OK;
    }
    tv_clear(&raw mut var1);
    tv_clear(&raw mut var2);
    return if rc == OK {
        p
    } else {
        ::core::ptr::null_mut::<c_char>()
    };
}
pub unsafe extern "C" fn get_lval(
    name: *mut c_char,
    rettv: *mut typval_T,
    lp: *mut lval_T,
    unlet: bool,
    skip: bool,
    flags: c_int,
    fne_flags: c_int,
) -> *mut c_char {
    let mut quiet: c_int = flags & GLV_QUIET as c_int;
    memset(
        lp as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<lval_T>(),
    );
    if skip {
        (*lp).ll_name = name;
        return find_name_end(
            name,
            ::core::ptr::null_mut::<*const c_char>(),
            ::core::ptr::null_mut::<*const c_char>(),
            FNE_INCL_BR | fne_flags,
        ) as *mut c_char;
    }
    let mut expr_start: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut expr_end: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut p: *mut c_char = find_name_end(
        name,
        &raw mut expr_start as *mut *const c_char,
        &raw mut expr_end as *mut *const c_char,
        fne_flags,
    ) as *mut c_char;
    if !expr_start.is_null() {
        if unlet as c_int != 0
            && !ascii_iswhite(*p as c_int)
            && ends_excmd(*p as c_int) == 0
            && *p as c_int != '[' as c_int
            && *p as c_int != '.' as c_int
        {
            semsg(gettext(&raw const e_trailing_arg as *const c_char), p);
            return ::core::ptr::null_mut::<c_char>();
        }
        (*lp).ll_exp_name = make_expanded_name(name, expr_start, expr_end, p);
        (*lp).ll_name = (*lp).ll_exp_name;
        if (*lp).ll_exp_name.is_null() {
            if !aborting() && quiet == 0 {
                emsg_severe.set(true_0 != 0);
                semsg(gettext(&raw const e_invarg2 as *const c_char), name);
                return ::core::ptr::null_mut::<c_char>();
            }
            (*lp).ll_name_len = 0 as size_t;
        } else {
            (*lp).ll_name_len = strlen((*lp).ll_name);
        }
    } else {
        (*lp).ll_name = name;
        (*lp).ll_name_len = p.offset_from((*lp).ll_name) as size_t;
    }
    if *p as c_int != '[' as c_int && *p as c_int != '.' as c_int || (*lp).ll_name.is_null() {
        return p;
    }
    let mut ht: *mut hashtab_T = ::core::ptr::null_mut::<hashtab_T>();
    let mut v: *mut dictitem_T = find_var(
        (*lp).ll_name,
        (*lp).ll_name_len,
        if flags & GLV_READ_ONLY as c_int != 0 {
            ::core::ptr::null_mut::<*mut hashtab_T>()
        } else {
            &raw mut ht
        },
        flags & GLV_NO_AUTOLOAD as c_int,
    );
    if v.is_null() && quiet == 0 {
        semsg(
            gettext(b"E121: Undefined variable: %.*s\0".as_ptr() as *const c_char),
            (*lp).ll_name_len as c_int,
            (*lp).ll_name,
        );
    }
    if v.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    (*lp).ll_tv = &raw mut (*v).di_tv;
    if tv_is_luafunc((*lp).ll_tv) {
        return p;
    }
    p = get_lval_subscript(lp, p, name, rettv, ht, v, unlet, flags);
    if p.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    (*lp).ll_name_len = p.offset_from((*lp).ll_name) as size_t;
    return p;
}
pub unsafe extern "C" fn clear_lval(mut lp: *mut lval_T) {
    xfree((*lp).ll_exp_name as *mut c_void);
    xfree((*lp).ll_newkey as *mut c_void);
}
pub unsafe extern "C" fn set_var_lval(
    mut lp: *mut lval_T,
    mut endp: *mut c_char,
    mut rettv: *mut typval_T,
    mut copy: bool,
    is_const: bool,
    mut op: *const c_char,
) {
    let mut cc: c_int = 0;
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    if (*lp).ll_tv.is_null() {
        cc = *endp as uint8_t as c_int;
        *endp = NUL as c_char;
        if !(*lp).ll_blob.is_null() {
            if !op.is_null() && *op as c_int != '=' as c_int {
                semsg(gettext(&raw const e_letwrong as *const c_char), op);
                return;
            }
            if value_check_lock(
                (*(*lp).ll_blob).bv_lock,
                (*lp).ll_name,
                TV_CSTRING as size_t,
            ) {
                return;
            }
            if (*lp).ll_range as c_int != 0
                && (*rettv).v_type as c_uint == VAR_BLOB as c_int as c_uint
            {
                if (*lp).ll_empty2 {
                    (*lp).ll_n2 = tv_blob_len((*lp).ll_blob) - 1 as c_int;
                }
                if tv_blob_set_range(
                    (*lp).ll_blob,
                    (*lp).ll_n1 as varnumber_T,
                    (*lp).ll_n2 as varnumber_T,
                    rettv,
                ) == FAIL
                {
                    return;
                }
            } else {
                let mut error: bool = false_0 != 0;
                let val: varnumber_T = tv_get_number_chk(rettv, &raw mut error);
                if !error {
                    if val < 0 as varnumber_T || val > 255 as varnumber_T {
                        semsg(
                            gettext(&raw const e_invalid_value_for_blob_nr as *const c_char),
                            val,
                        );
                    } else {
                        tv_blob_set_append((*lp).ll_blob, (*lp).ll_n1, val as uint8_t);
                    }
                }
            }
        } else if !op.is_null() && *op as c_int != '=' as c_int {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if is_const {
                emsg(gettext(&raw const e_cannot_mod as *const c_char));
                *endp = cc as c_char;
                return;
            }
            di = ::core::ptr::null_mut::<dictitem_T>();
            if eval_variable(
                (*lp).ll_name,
                (*lp).ll_name_len as c_int,
                &raw mut tv,
                &raw mut di,
                true_0 != 0,
                false_0 != 0,
            ) == OK
            {
                if (di.is_null()
                    || !var_check_ro((*di).di_flags as c_int, (*lp).ll_name, TV_CSTRING as size_t)
                        && !tv_check_lock(
                            &raw mut (*di).di_tv,
                            (*lp).ll_name,
                            TV_CSTRING as size_t,
                        ))
                    && eexe_mod_op(&raw mut tv, rettv, op) == OK
                {
                    set_var((*lp).ll_name, (*lp).ll_name_len, &raw mut tv, false_0 != 0);
                }
                tv_clear(&raw mut tv);
            }
        } else {
            set_var_const((*lp).ll_name, (*lp).ll_name_len, rettv, copy, is_const);
        }
        *endp = cc as c_char;
    } else if !value_check_lock(
        (if (*lp).ll_newkey.is_null() {
            (*(*lp).ll_tv).v_lock as c_uint
        } else {
            (*(*(*lp).ll_tv).vval.v_dict).dv_lock as c_uint
        }) as VarLockStatus,
        (*lp).ll_name,
        TV_CSTRING as size_t,
    ) {
        if (*lp).ll_range {
            if is_const {
                emsg(gettext(
                    b"E996: Cannot lock a range\0".as_ptr() as *const c_char
                ));
                return;
            }
            tv_list_assign_range(
                (*lp).ll_list,
                (*rettv).vval.v_list,
                (*lp).ll_n1,
                (*lp).ll_n2,
                (*lp).ll_empty2,
                op,
                (*lp).ll_name,
            );
        } else {
            let mut oldtv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let mut dict: *mut dict_T = (*lp).ll_dict;
            let mut watched: bool = tv_dict_is_watched(dict);
            if is_const {
                emsg(gettext(
                    b"E996: Cannot lock a list or dict\0".as_ptr() as *const c_char
                ));
                return;
            }
            '_notify: {
                if !(*lp).ll_newkey.is_null() {
                    if !op.is_null() && *op as c_int != '=' as c_int {
                        semsg(
                            gettext(&raw const e_dictkey as *const c_char),
                            (*lp).ll_newkey,
                        );
                        return;
                    }
                    if tv_dict_wrong_func_name((*(*lp).ll_tv).vval.v_dict, rettv, (*lp).ll_newkey)
                        != 0
                    {
                        return;
                    }
                    di = tv_dict_item_alloc((*lp).ll_newkey);
                    if tv_dict_add((*(*lp).ll_tv).vval.v_dict, di) == FAIL {
                        xfree(di as *mut c_void);
                        return;
                    }
                    (*lp).ll_tv = &raw mut (*di).di_tv;
                } else {
                    if watched {
                        tv_copy((*lp).ll_tv, &raw mut oldtv);
                    }
                    if !op.is_null() && *op as c_int != '=' as c_int {
                        eexe_mod_op((*lp).ll_tv, rettv, op);
                        break '_notify;
                    } else {
                        tv_clear((*lp).ll_tv);
                    }
                }
                if copy {
                    tv_copy(rettv, (*lp).ll_tv);
                } else {
                    *(*lp).ll_tv = *rettv;
                    (*(*lp).ll_tv).v_lock = VAR_UNLOCKED;
                    tv_init(rettv);
                }
            }
            if watched {
                if oldtv.v_type as c_uint == VAR_UNKNOWN as c_int as c_uint {
                    '_c2rust_label: {
                        if !(*lp).ll_newkey.is_null() {
                        } else {
                            __assert_fail(
                                b"lp->ll_newkey != NULL\0".as_ptr()
                                    as *const c_char,
                                b"src/nvim/eval.rs\0"
                                    .as_ptr() as *const c_char,
                                1418 as c_uint,
                                b"void set_var_lval(lval_T *, char *, typval_T *, _Bool, const _Bool, const char *)\0"
                                    .as_ptr() as *const c_char,
                            );
                        }
                    };
                    tv_dict_watcher_notify(
                        dict,
                        (*lp).ll_newkey,
                        (*lp).ll_tv,
                        ::core::ptr::null_mut::<typval_T>(),
                    );
                } else {
                    let mut di_: *mut dictitem_T = (*lp).ll_di;
                    '_c2rust_label_0: {
                        if !(&raw mut (*di_).di_key as *mut c_char).is_null() {
                        } else {
                            __assert_fail(
                                b"di_->di_key != NULL\0".as_ptr()
                                    as *const c_char,
                                b"src/nvim/eval.rs\0"
                                    .as_ptr() as *const c_char,
                                1422 as c_uint,
                                b"void set_var_lval(lval_T *, char *, typval_T *, _Bool, const _Bool, const char *)\0"
                                    .as_ptr() as *const c_char,
                            );
                        }
                    };
                    tv_dict_watcher_notify(
                        dict,
                        &raw mut (*di_).di_key as *mut c_char,
                        (*lp).ll_tv,
                        &raw mut oldtv,
                    );
                    tv_clear(&raw mut oldtv);
                }
            }
        }
    }
}
pub unsafe extern "C" fn eval_for_line(
    mut arg: *const c_char,
    mut errp: *mut bool,
    mut eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> *mut c_void {
    let mut fi: *mut forinfo_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<forinfo_T>()) as *mut forinfo_T;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let skip: bool = (*evalarg).eval_flags & EVAL_EVALUATE as c_int == 0;
    *errp = true_0 != 0;
    let mut expr: *const c_char = skip_var_list(
        arg,
        &raw mut (*fi).fi_varcount,
        &raw mut (*fi).fi_semicolon,
        false_0 != 0,
    );
    if expr.is_null() {
        return fi as *mut c_void;
    }
    expr = skipwhite(expr);
    if *expr.offset(0 as c_int as isize) as c_int != 'i' as c_int
        || *expr.offset(1 as c_int as isize) as c_int != 'n' as c_int
        || !(*expr.offset(2 as c_int as isize) as c_int == NUL
            || ascii_iswhite(*expr.offset(2 as c_int as isize) as c_int) as c_int != 0)
    {
        emsg(gettext(
            b"E690: Missing \"in\" after :for\0".as_ptr() as *const c_char
        ));
        return fi as *mut c_void;
    }
    if skip {
        (*emsg_skip.ptr()) += 1;
    }
    expr = skipwhite(expr.offset(2 as c_int as isize));
    if eval0(expr as *mut c_char, &raw mut tv, eap, evalarg) == OK {
        *errp = false_0 != 0;
        if !skip {
            if tv.v_type as c_uint == VAR_LIST as c_int as c_uint {
                l = tv.vval.v_list;
                if l.is_null() {
                    tv_clear(&raw mut tv);
                } else {
                    (*fi).fi_list = l;
                    tv_list_watch_add(l, &raw mut (*fi).fi_lw);
                    (*fi).fi_lw.lw_item = tv_list_first(l);
                }
            } else if tv.v_type as c_uint == VAR_BLOB as c_int as c_uint {
                (*fi).fi_bi = 0 as c_int;
                if !tv.vval.v_blob.is_null() {
                    let mut btv: typval_T = typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    tv_blob_copy(tv.vval.v_blob, &raw mut btv);
                    (*fi).fi_blob = btv.vval.v_blob;
                }
                tv_clear(&raw mut tv);
            } else if tv.v_type as c_uint == VAR_STRING as c_int as c_uint {
                (*fi).fi_byte_idx = 0 as c_int;
                (*fi).fi_string = tv.vval.v_string;
                tv.vval.v_string = ::core::ptr::null_mut::<c_char>();
                if (*fi).fi_string.is_null() {
                    (*fi).fi_string = xstrdup(b"\0".as_ptr() as *const c_char);
                }
            } else {
                emsg(gettext(
                    (e_string_list_or_blob_required.ptr() as *const _) as *const c_char,
                ));
                tv_clear(&raw mut tv);
            }
        }
    }
    if skip {
        (*emsg_skip.ptr()) -= 1;
    }
    return fi as *mut c_void;
}
pub unsafe extern "C" fn next_for_item(mut fi_void: *mut c_void, mut arg: *mut c_char) -> bool {
    let mut fi: *mut forinfo_T = fi_void as *mut forinfo_T;
    if !(*fi).fi_blob.is_null() {
        if (*fi).fi_bi >= tv_blob_len((*fi).fi_blob) {
            return false_0 != 0;
        }
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv.v_type = VAR_NUMBER;
        tv.v_lock = VAR_FIXED;
        tv.vval.v_number = tv_blob_get((*fi).fi_blob, (*fi).fi_bi) as varnumber_T;
        (*fi).fi_bi += 1;
        return ex_let_vars(
            arg,
            &raw mut tv,
            true_0,
            (*fi).fi_semicolon,
            (*fi).fi_varcount,
            false_0,
            ::core::ptr::null_mut::<c_char>(),
        ) == OK;
    }
    if !(*fi).fi_string.is_null() {
        let len: c_int = utfc_ptr2len((*fi).fi_string.offset((*fi).fi_byte_idx as isize));
        if len == 0 as c_int {
            return false_0 != 0;
        }
        let mut tv_0: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv_0.v_type = VAR_STRING;
        tv_0.v_lock = VAR_FIXED;
        tv_0.vval.v_string = xmemdupz(
            (*fi).fi_string.offset((*fi).fi_byte_idx as isize) as *const c_void,
            len as size_t,
        ) as *mut c_char;
        (*fi).fi_byte_idx += len;
        let result: c_int = (ex_let_vars(
            arg,
            &raw mut tv_0,
            true_0,
            (*fi).fi_semicolon,
            (*fi).fi_varcount,
            false_0,
            ::core::ptr::null_mut::<c_char>(),
        ) == OK) as c_int;
        xfree(tv_0.vval.v_string as *mut c_void);
        return result != 0;
    }
    let mut item: *mut listitem_T = (*fi).fi_lw.lw_item;
    if item.is_null() {
        return false_0 != 0;
    }
    (*fi).fi_lw.lw_item = (*item).li_next;
    return ex_let_vars(
        arg,
        &raw mut (*item).li_tv,
        true_0,
        (*fi).fi_semicolon,
        (*fi).fi_varcount,
        false_0,
        ::core::ptr::null_mut::<c_char>(),
    ) == OK;
}
pub unsafe extern "C" fn free_for_info(mut fi_void: *mut c_void) {
    let mut fi: *mut forinfo_T = fi_void as *mut forinfo_T;
    if fi.is_null() {
        return;
    }
    if !(*fi).fi_list.is_null() {
        tv_list_watch_remove((*fi).fi_list, &raw mut (*fi).fi_lw);
        tv_list_unref((*fi).fi_list);
    } else if !(*fi).fi_blob.is_null() {
        tv_blob_unref((*fi).fi_blob);
    } else {
        xfree((*fi).fi_string as *mut c_void);
    }
    xfree(fi as *mut c_void);
}
pub unsafe extern "C" fn set_context_for_expression(
    mut xp: *mut expand_T,
    mut arg: *mut c_char,
    mut cmdidx: cmdidx_T,
) {
    let mut got_eq: bool = false_0 != 0;
    if cmdidx as c_int == CMD_let as c_int || cmdidx as c_int == CMD_const as c_int {
        (*xp).xp_context = EXPAND_USER_VARS as c_int;
        if strpbrk(arg, b"\"'+-*/%.=!?~|&$([<>,#\0".as_ptr() as *const c_char).is_null() {
            let mut p: *mut c_char = arg.offset(strlen(arg) as isize);
            while p >= arg {
                (*xp).xp_pattern = p;
                p = p.offset(
                    -((utf_head_off(arg, p.offset(-(1 as c_int as isize))) + 1 as c_int) as isize),
                );
                if ascii_iswhite(*p as c_int) {
                    break;
                }
            }
            return;
        }
    } else {
        (*xp).xp_context = if cmdidx as c_int == CMD_call as c_int {
            EXPAND_FUNCTIONS as c_int
        } else {
            EXPAND_EXPRESSION as c_int
        };
    }
    loop {
        (*xp).xp_pattern = strpbrk(arg, b"\"'+-*/%.=!?~|&$([<>,#\0".as_ptr() as *const c_char);
        if (*xp).xp_pattern.is_null() {
            break;
        }
        let mut c: c_int = *(*xp).xp_pattern as uint8_t as c_int;
        if c == '&' as c_int {
            c = *(*xp).xp_pattern.offset(1 as c_int as isize) as uint8_t as c_int;
            if c == '&' as c_int {
                (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                (*xp).xp_context = if cmdidx as c_int != CMD_let as c_int || got_eq as c_int != 0 {
                    EXPAND_EXPRESSION as c_int
                } else {
                    EXPAND_NOTHING as c_int
                };
            } else if c != ' ' as c_int {
                (*xp).xp_context = EXPAND_SETTINGS as c_int;
                if (c == 'l' as c_int || c == 'g' as c_int)
                    && *(*xp).xp_pattern.offset(2 as c_int as isize) as c_int == ':' as c_int
                {
                    (*xp).xp_pattern = (*xp).xp_pattern.offset(2 as c_int as isize);
                }
            }
        } else if c == '$' as c_int {
            (*xp).xp_context = EXPAND_ENV_VARS as c_int;
        } else if c == '=' as c_int {
            got_eq = true_0 != 0;
            (*xp).xp_context = EXPAND_EXPRESSION as c_int;
        } else {
            if c == '#' as c_int && (*xp).xp_context == EXPAND_EXPRESSION as c_int {
                break;
            }
            if (c == '<' as c_int || c == '#' as c_int)
                && (*xp).xp_context == EXPAND_FUNCTIONS as c_int
                && vim_strchr((*xp).xp_pattern, '(' as c_int).is_null()
            {
                break;
            }
            if cmdidx as c_int != CMD_let as c_int || got_eq as c_int != 0 {
                if c == '"' as c_int {
                    loop {
                        (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        c = *(*xp).xp_pattern as uint8_t as c_int;
                        if !(c != NUL && c != '"' as c_int) {
                            break;
                        }
                        if c == '\\' as c_int
                            && *(*xp).xp_pattern.offset(1 as c_int as isize) as c_int != NUL
                        {
                            (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        }
                    }
                    (*xp).xp_context = EXPAND_NOTHING as c_int;
                } else if c == '\'' as c_int {
                    loop {
                        (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        c = *(*xp).xp_pattern as uint8_t as c_int;
                        if !(c != NUL && c != '\'' as c_int) {
                            break;
                        }
                    }
                    (*xp).xp_context = EXPAND_NOTHING as c_int;
                } else if c == '|' as c_int {
                    if *(*xp).xp_pattern.offset(1 as c_int as isize) as c_int == '|' as c_int {
                        (*xp).xp_pattern = (*xp).xp_pattern.offset(1);
                        (*xp).xp_context = EXPAND_EXPRESSION as c_int;
                    } else {
                        (*xp).xp_context = EXPAND_COMMANDS as c_int;
                    }
                } else {
                    (*xp).xp_context = EXPAND_EXPRESSION as c_int;
                }
            } else {
                (*xp).xp_context = EXPAND_EXPRESSION as c_int;
            }
        }
        arg = (*xp).xp_pattern;
        if *arg as c_int != NUL {
            loop {
                arg = arg.offset(1);
                c = *arg as uint8_t as c_int;
                if !(c != NUL && (c == ' ' as c_int || c == '\t' as c_int)) {
                    break;
                }
            }
        }
    }
    if cmd_has_expr_args(cmdidx) as c_int != 0 && (*xp).xp_context == EXPAND_EXPRESSION as c_int {
        loop {
            let n: *mut c_char = skiptowhite(arg);
            if n == arg || ascii_iswhite_or_nul(*skipwhite(n) as c_int) as c_int != 0 {
                break;
            }
            arg = skipwhite(n);
        }
    }
    (*xp).xp_pattern = arg;
}
pub unsafe extern "C" fn pattern_match(
    mut pat: *const c_char,
    mut text: *const c_char,
    mut ic: bool,
) -> c_int {
    let mut matches: c_int = 0 as c_int;
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<c_char>(); 10],
        endp: [::core::ptr::null_mut::<c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut save_cpo: *mut c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut c_char);
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !regmatch.regprog.is_null() {
        regmatch.rm_ic = ic;
        matches = vim_regexec_nl(&raw mut regmatch, text, 0 as colnr_T) as c_int;
        vim_regfree(regmatch.regprog);
    }
    p_cpo.set(save_cpo);
    return matches;
}
unsafe extern "C" fn eval_func(
    arg: *mut *mut c_char,
    evalarg: *mut evalarg_T,
    name: *mut c_char,
    name_len: c_int,
    rettv: *mut typval_T,
    flags: c_int,
    basetv: *mut typval_T,
) -> c_int {
    let evaluate: bool = flags & EVAL_EVALUATE as c_int != 0;
    let mut s: *mut c_char = name;
    let mut len: c_int = name_len;
    let mut found_var: bool = false_0 != 0;
    if !evaluate {
        check_vars(s, len as size_t);
    }
    let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    s = deref_func_name(
        s,
        &raw mut len,
        &raw mut partial,
        !evaluate,
        &raw mut found_var,
    );
    s = xmemdupz(s as *const c_void, len as size_t) as *mut c_char;
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_evaluate = evaluate;
    funcexe.fe_partial = partial;
    funcexe.fe_basetv = basetv;
    funcexe.fe_found_var = found_var;
    let mut ret: c_int = get_func_tv(s, len, rettv, arg, evalarg, &raw mut funcexe);
    xfree(s as *mut c_void);
    if (*rettv).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint
        && !evaluate
        && **arg as c_int == '(' as c_int
    {
        (*rettv).vval.v_string = tv_empty_string.get() as *mut c_char;
        (*rettv).v_type = VAR_FUNC;
    }
    if evaluate as c_int != 0 && aborting() as c_int != 0 {
        if ret == OK {
            tv_clear(rettv);
        }
        ret = FAIL;
    }
    return ret;
}
pub unsafe extern "C" fn clear_evalarg(mut evalarg: *mut evalarg_T, mut eap: *mut exarg_T) {
    if evalarg.is_null() {
        return;
    }
    if !(*evalarg).eval_tofree.is_null() {
        if !eap.is_null() {
            xfree((*eap).cmdline_tofree as *mut c_void);
            (*eap).cmdline_tofree = *(*eap).cmdlinep;
            *(*eap).cmdlinep = (*evalarg).eval_tofree;
        } else {
            xfree((*evalarg).eval_tofree as *mut c_void);
        }
        (*evalarg).eval_tofree = ::core::ptr::null_mut::<c_char>();
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eval0(
    mut arg: *mut c_char,
    mut rettv: *mut typval_T,
    mut eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let did_emsg_before: c_int = did_emsg.get();
    let called_emsg_before: c_int = called_emsg.get();
    let mut end_error: bool = false_0 != 0;
    let mut p: *mut c_char = skipwhite(arg);
    let mut ret: c_int = eval1(&raw mut p, rettv, evalarg);
    if ret != FAIL {
        end_error = ends_excmd(*p as c_int) == 0;
    }
    if ret == FAIL || end_error as c_int != 0 {
        if ret != FAIL {
            tv_clear(rettv);
        }
        if !aborting()
            && did_emsg.get() == did_emsg_before
            && called_emsg.get() == called_emsg_before
        {
            if end_error {
                semsg(gettext(&raw const e_trailing_arg as *const c_char), p);
            } else {
                semsg(gettext(&raw const e_invexpr2 as *const c_char), arg);
            }
        }
        if !eap.is_null() && !p.is_null() {
            let mut nextcmd: *mut c_char = check_nextcmd(p);
            if !nextcmd.is_null() && *nextcmd as c_int != '|' as c_int {
                (*eap).nextcmd = nextcmd;
            }
        }
        return FAIL;
    }
    if !eap.is_null() {
        (*eap).nextcmd = check_nextcmd(p);
    }
    return ret;
}
pub unsafe extern "C" fn may_call_simple_func(
    mut arg: *const c_char,
    mut rettv: *mut typval_T,
) -> c_int {
    let mut parens: *const c_char = strstr(arg, b"()\0".as_ptr() as *const c_char);
    let mut r: c_int = NOTDONE;
    if !parens.is_null() && *skipwhite(parens.offset(2 as c_int as isize)) as c_int == NUL {
        if strnequal(arg, b"v:lua.\0".as_ptr() as *const c_char, 6 as size_t) {
            let mut p: *const c_char = arg.offset(6 as c_int as isize);
            if p != parens && skip_luafunc_name(p) == parens {
                r = call_simple_luafunc(p, parens.offset_from(p) as size_t, rettv);
            }
        } else {
            let mut p_0: *const c_char =
                if strncmp(arg, b"<SNR>\0".as_ptr() as *const c_char, 5 as size_t) == 0 as c_int {
                    skipdigits(arg.offset(5 as c_int as isize)) as *const c_char
                } else {
                    arg
                };
            if to_name_end(p_0, true_0 != 0) == parens {
                r = call_simple_func(arg, parens.offset_from(arg) as size_t, rettv);
            }
        }
    }
    return r;
}
unsafe extern "C" fn eval0_simple_funccal(
    mut arg: *mut c_char,
    mut rettv: *mut typval_T,
    mut eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let mut r: c_int = may_call_simple_func(arg, rettv);
    if r == NOTDONE {
        r = eval0(arg, rettv, eap, evalarg);
    }
    return r;
}
pub unsafe extern "C" fn eval1(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    memset(
        rettv as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<typval_T>(),
    );
    if eval2(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut c_char = *arg;
    if *p as c_int == '?' as c_int {
        let op_falsy: bool = *p.offset(1 as c_int as isize) as c_int == '?' as c_int;
        let mut evalarg_used: *mut evalarg_T = evalarg;
        let mut local_evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<c_void>(),
            eval_tofree: ::core::ptr::null_mut::<c_char>(),
        };
        if evalarg.is_null() {
            local_evalarg = evalarg_T {
                eval_flags: 0 as c_int,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<c_void>(),
                eval_tofree: ::core::ptr::null_mut::<c_char>(),
            };
            evalarg_used = &raw mut local_evalarg;
        }
        let orig_flags: c_int = (*evalarg_used).eval_flags;
        let evaluate: bool = (*evalarg_used).eval_flags & EVAL_EVALUATE as c_int != 0;
        let mut result: bool = false_0 != 0;
        if evaluate {
            let mut error: bool = false_0 != 0;
            if op_falsy {
                result = tv2bool(rettv);
            } else if tv_get_number_chk(rettv, &raw mut error) != 0 as varnumber_T {
                result = true_0 != 0;
            }
            if error as c_int != 0 || !op_falsy || !result {
                tv_clear(rettv);
            }
            if error {
                return FAIL;
            }
        }
        if op_falsy {
            *arg = (*arg).offset(1);
        }
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
        (*evalarg_used).eval_flags = if if op_falsy as c_int != 0 {
            !result as c_int
        } else {
            result as c_int
        } != 0
        {
            orig_flags
        } else {
            orig_flags & !(EVAL_EVALUATE as c_int)
        };
        let mut var2: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if eval1(arg, &raw mut var2, evalarg_used) == FAIL {
            (*evalarg_used).eval_flags = orig_flags;
            return FAIL;
        }
        if !op_falsy || !result {
            *rettv = var2;
        }
        if !op_falsy {
            p = *arg;
            if *p as c_int != ':' as c_int {
                emsg(gettext(
                    b"E109: Missing ':' after '?'\0".as_ptr() as *const c_char
                ));
                if evaluate as c_int != 0 && result as c_int != 0 {
                    tv_clear(rettv);
                }
                (*evalarg_used).eval_flags = orig_flags;
                return FAIL;
            }
            *arg = skipwhite((*arg).offset(1 as c_int as isize));
            (*evalarg_used).eval_flags = if !result {
                orig_flags
            } else {
                orig_flags & !(EVAL_EVALUATE as c_int)
            };
            if eval1(arg, &raw mut var2, evalarg_used) == FAIL {
                if evaluate as c_int != 0 && result as c_int != 0 {
                    tv_clear(rettv);
                }
                (*evalarg_used).eval_flags = orig_flags;
                return FAIL;
            }
            if evaluate as c_int != 0 && !result {
                *rettv = var2;
            }
        }
        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, ::core::ptr::null_mut::<exarg_T>());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
    }
    return OK;
}
unsafe extern "C" fn eval2(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    if eval3(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut c_char = *arg;
    if *p.offset(0 as c_int as isize) as c_int == '|' as c_int
        && *p.offset(1 as c_int as isize) as c_int == '|' as c_int
    {
        let mut evalarg_used: *mut evalarg_T = evalarg;
        let mut local_evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<c_void>(),
            eval_tofree: ::core::ptr::null_mut::<c_char>(),
        };
        if evalarg.is_null() {
            local_evalarg = evalarg_T {
                eval_flags: 0 as c_int,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<c_void>(),
                eval_tofree: ::core::ptr::null_mut::<c_char>(),
            };
            evalarg_used = &raw mut local_evalarg;
        }
        let orig_flags: c_int = (*evalarg_used).eval_flags;
        let evaluate: bool = (*evalarg_used).eval_flags & EVAL_EVALUATE as c_int != 0;
        let mut result: bool = false_0 != 0;
        if evaluate {
            let mut error: bool = false_0 != 0;
            if tv_get_number_chk(rettv, &raw mut error) != 0 as varnumber_T {
                result = true_0 != 0;
            }
            tv_clear(rettv);
            if error {
                return FAIL;
            }
        }
        while *p.offset(0 as c_int as isize) as c_int == '|' as c_int
            && *p.offset(1 as c_int as isize) as c_int == '|' as c_int
        {
            *arg = skipwhite((*arg).offset(2 as c_int as isize));
            (*evalarg_used).eval_flags = if !result {
                orig_flags
            } else {
                orig_flags & !(EVAL_EVALUATE as c_int)
            };
            let mut var2: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if eval3(arg, &raw mut var2, evalarg_used) == FAIL {
                return FAIL;
            }
            if evaluate as c_int != 0 && !result {
                let mut error_0: bool = false_0 != 0;
                if tv_get_number_chk(&raw mut var2, &raw mut error_0) != 0 as varnumber_T {
                    result = true_0 != 0;
                }
                tv_clear(&raw mut var2);
                if error_0 {
                    return FAIL;
                }
            }
            if evaluate {
                (*rettv).v_type = VAR_NUMBER;
                (*rettv).vval.v_number = result as varnumber_T;
            }
            p = *arg;
        }
        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, ::core::ptr::null_mut::<exarg_T>());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
    }
    return OK;
}
unsafe extern "C" fn eval3(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    if eval4(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut c_char = *arg;
    if *p.offset(0 as c_int as isize) as c_int == '&' as c_int
        && *p.offset(1 as c_int as isize) as c_int == '&' as c_int
    {
        let mut evalarg_used: *mut evalarg_T = evalarg;
        let mut local_evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<c_void>(),
            eval_tofree: ::core::ptr::null_mut::<c_char>(),
        };
        if evalarg.is_null() {
            local_evalarg = evalarg_T {
                eval_flags: 0 as c_int,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<c_void>(),
                eval_tofree: ::core::ptr::null_mut::<c_char>(),
            };
            evalarg_used = &raw mut local_evalarg;
        }
        let orig_flags: c_int = (*evalarg_used).eval_flags;
        let evaluate: bool = (*evalarg_used).eval_flags & EVAL_EVALUATE as c_int != 0;
        let mut result: bool = true_0 != 0;
        if evaluate {
            let mut error: bool = false_0 != 0;
            if tv_get_number_chk(rettv, &raw mut error) == 0 as varnumber_T {
                result = false_0 != 0;
            }
            tv_clear(rettv);
            if error {
                return FAIL;
            }
        }
        while *p.offset(0 as c_int as isize) as c_int == '&' as c_int
            && *p.offset(1 as c_int as isize) as c_int == '&' as c_int
        {
            *arg = skipwhite((*arg).offset(2 as c_int as isize));
            (*evalarg_used).eval_flags = if result as c_int != 0 {
                orig_flags
            } else {
                orig_flags & !(EVAL_EVALUATE as c_int)
            };
            let mut var2: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if eval4(arg, &raw mut var2, evalarg_used) == FAIL {
                return FAIL;
            }
            if evaluate as c_int != 0 && result as c_int != 0 {
                let mut error_0: bool = false_0 != 0;
                if tv_get_number_chk(&raw mut var2, &raw mut error_0) == 0 as varnumber_T {
                    result = false_0 != 0;
                }
                tv_clear(&raw mut var2);
                if error_0 {
                    return FAIL;
                }
            }
            if evaluate {
                (*rettv).v_type = VAR_NUMBER;
                (*rettv).vval.v_number = result as varnumber_T;
            }
            p = *arg;
        }
        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, ::core::ptr::null_mut::<exarg_T>());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
    }
    return OK;
}
unsafe extern "C" fn eval4(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let mut var2: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut type_0: exprtype_T = EXPR_UNKNOWN;
    let mut len: c_int = 2 as c_int;
    if eval5(arg, rettv, evalarg) == FAIL {
        return FAIL;
    }
    let mut p: *mut c_char = *arg;
    match *p.offset(0 as c_int as isize) as c_int {
        61 => {
            if *p.offset(1 as c_int as isize) as c_int == '=' as c_int {
                type_0 = EXPR_EQUAL;
            } else if *p.offset(1 as c_int as isize) as c_int == '~' as c_int {
                type_0 = EXPR_MATCH;
            }
        }
        33 => {
            if *p.offset(1 as c_int as isize) as c_int == '=' as c_int {
                type_0 = EXPR_NEQUAL;
            } else if *p.offset(1 as c_int as isize) as c_int == '~' as c_int {
                type_0 = EXPR_NOMATCH;
            }
        }
        62 => {
            if *p.offset(1 as c_int as isize) as c_int != '=' as c_int {
                type_0 = EXPR_GREATER;
                len = 1 as c_int;
            } else {
                type_0 = EXPR_GEQUAL;
            }
        }
        60 => {
            if *p.offset(1 as c_int as isize) as c_int != '=' as c_int {
                type_0 = EXPR_SMALLER;
                len = 1 as c_int;
            } else {
                type_0 = EXPR_SEQUAL;
            }
        }
        105 => {
            if *p.offset(1 as c_int as isize) as c_int == 's' as c_int {
                if *p.offset(2 as c_int as isize) as c_int == 'n' as c_int
                    && *p.offset(3 as c_int as isize) as c_int == 'o' as c_int
                    && *p.offset(4 as c_int as isize) as c_int == 't' as c_int
                {
                    len = 5 as c_int;
                }
                if *(*__ctype_b_loc()).offset(*p.offset(len as isize) as uint8_t as c_int as isize)
                    as c_int
                    & _ISalnum as c_int as c_ushort as c_int
                    == 0
                    && *p.offset(len as isize) as c_int != '_' as c_int
                {
                    type_0 = (if len == 2 as c_int {
                        EXPR_IS as c_int
                    } else {
                        EXPR_ISNOT as c_int
                    }) as exprtype_T;
                }
            }
        }
        _ => {}
    }
    if type_0 as c_uint != EXPR_UNKNOWN as c_int as c_uint {
        let mut ic: bool = false;
        if *p.offset(len as isize) as c_int == '?' as c_int {
            ic = true_0 != 0;
            len += 1;
        } else if *p.offset(len as isize) as c_int == '#' as c_int {
            ic = false_0 != 0;
            len += 1;
        } else {
            ic = p_ic.get() != 0;
        }
        *arg = skipwhite(p.offset(len as isize));
        if eval5(arg, &raw mut var2, evalarg) == FAIL {
            tv_clear(rettv);
            return FAIL;
        }
        if !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0 {
            let ret: c_int = typval_compare(rettv, &raw mut var2, type_0, ic);
            tv_clear(&raw mut var2);
            return ret;
        }
    }
    return OK;
}
unsafe extern "C" fn eval_addblob(mut tv1: *mut typval_T, mut tv2: *mut typval_T) {
    let b1: *const blob_T = (*tv1).vval.v_blob;
    let b2: *const blob_T = (*tv2).vval.v_blob;
    let b: *mut blob_T = tv_blob_alloc();
    let mut len1: int64_t = tv_blob_len(b1) as int64_t;
    let mut len2: int64_t = tv_blob_len(b2) as int64_t;
    let mut totallen: int64_t = len1 + len2;
    if totallen >= 0 as int64_t && totallen <= INT_MAX as int64_t {
        ga_grow(&raw mut (*b).bv_ga, totallen as c_int);
        if len1 > 0 as int64_t {
            memmove(
                (*b).bv_ga.ga_data as *mut c_char as *mut c_void,
                (*b1).bv_ga.ga_data,
                len1 as size_t,
            );
        }
        if len2 > 0 as int64_t {
            memmove(
                ((*b).bv_ga.ga_data as *mut c_char).offset(len1 as isize) as *mut c_void,
                (*b2).bv_ga.ga_data,
                len2 as size_t,
            );
        }
        (*b).bv_ga.ga_len = totallen as c_int;
    }
    tv_clear(tv1);
    tv_blob_set_ret(tv1, b);
}
unsafe extern "C" fn eval_addlist(mut tv1: *mut typval_T, mut tv2: *mut typval_T) -> c_int {
    let mut var3: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if tv_list_concat((*tv1).vval.v_list, (*tv2).vval.v_list, &raw mut var3) == FAIL {
        tv_clear(tv1);
        tv_clear(tv2);
        return FAIL;
    }
    tv_clear(tv1);
    *tv1 = var3;
    return OK;
}
pub unsafe extern "C" fn grow_string_tv(mut tv1: *mut typval_T, mut s2: *const c_char) -> c_int {
    if (*tv1).v_type as c_uint != VAR_STRING as c_int as c_uint || (*tv1).vval.v_string.is_null() {
        return FAIL;
    }
    let mut len1: size_t = strlen((*tv1).vval.v_string);
    let mut len2: size_t = strlen(s2);
    let mut p: *mut c_char = xrealloc(
        (*tv1).vval.v_string as *mut c_void,
        len1.wrapping_add(len2).wrapping_add(1 as size_t),
    ) as *mut c_char;
    memmove(
        p.offset(len1 as isize) as *mut c_void,
        s2 as *const c_void,
        len2.wrapping_add(1 as size_t),
    );
    (*tv1).vval.v_string = p;
    return OK;
}
unsafe extern "C" fn eval_concat_str(mut tv1: *mut typval_T, mut tv2: *mut typval_T) -> c_int {
    let mut buf1: [c_char; 65] = [0; 65];
    let mut buf2: [c_char; 65] = [0; 65];
    let s1: *const c_char = tv_get_string_buf(tv1, &raw mut buf1 as *mut c_char);
    let s2: *const c_char = tv_get_string_buf_chk(tv2, &raw mut buf2 as *mut c_char);
    if s2.is_null() {
        tv_clear(tv1);
        tv_clear(tv2);
        return FAIL;
    }
    if grow_string_tv(tv1, s2) == OK {
        return OK;
    }
    let mut p: *mut c_char = concat_str(s1, s2);
    tv_clear(tv1);
    (*tv1).v_type = VAR_STRING;
    (*tv1).vval.v_string = p;
    return OK;
}
unsafe extern "C" fn eval_addsub_number(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
    mut op: c_int,
) -> c_int {
    let mut error: bool = false_0 != 0;
    let mut n1: varnumber_T = 0;
    let mut n2: varnumber_T = 0;
    let mut f1: float_T = 0 as c_int as float_T;
    let mut f2: float_T = 0 as c_int as float_T;
    if (*tv1).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
        f1 = (*tv1).vval.v_float;
        n1 = 0 as varnumber_T;
    } else {
        n1 = tv_get_number_chk(tv1, &raw mut error);
        if error {
            tv_clear(tv1);
            tv_clear(tv2);
            return FAIL;
        }
        if (*tv2).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
            f1 = n1 as float_T;
        }
    }
    if (*tv2).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
        f2 = (*tv2).vval.v_float;
        n2 = 0 as varnumber_T;
    } else {
        n2 = tv_get_number_chk(tv2, &raw mut error);
        if error {
            tv_clear(tv1);
            tv_clear(tv2);
            return FAIL;
        }
        if (*tv1).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
            f2 = n2 as float_T;
        }
    }
    tv_clear(tv1);
    if (*tv1).v_type as c_uint == VAR_FLOAT as c_int as c_uint
        || (*tv2).v_type as c_uint == VAR_FLOAT as c_int as c_uint
    {
        if op == '+' as c_int {
            f1 = f1 + f2;
        } else {
            f1 = f1 - f2;
        }
        (*tv1).v_type = VAR_FLOAT;
        (*tv1).vval.v_float = f1;
    } else {
        if op == '+' as c_int {
            n1 = n1 + n2;
        } else {
            n1 = n1 - n2;
        }
        (*tv1).v_type = VAR_NUMBER;
        (*tv1).vval.v_number = n1;
    }
    return OK;
}
unsafe extern "C" fn eval5(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    if eval6(arg, rettv, evalarg, false_0 != 0) == FAIL {
        return FAIL;
    }
    loop {
        let mut op: c_int = **arg as uint8_t as c_int;
        let mut concat: bool = op == '.' as c_int;
        if op != '+' as c_int && op != '-' as c_int && !concat {
            break;
        }
        let evaluate: bool = if evalarg.is_null() {
            0 as c_int
        } else {
            (*evalarg).eval_flags & EVAL_EVALUATE as c_int
        } != 0;
        if (op != '+' as c_int
            || (*rettv).v_type as c_uint != VAR_LIST as c_int as c_uint
                && (*rettv).v_type as c_uint != VAR_BLOB as c_int as c_uint)
            && (op == '.' as c_int || (*rettv).v_type as c_uint != VAR_FLOAT as c_int as c_uint)
            && evaluate as c_int != 0
        {
            if op == '.' as c_int && !tv_check_str(rettv)
                || op != '.' as c_int && !tv_check_num(rettv)
            {
                tv_clear(rettv);
                return FAIL;
            }
        }
        if op == '.' as c_int && *(*arg).offset(1 as c_int as isize) as c_int == '.' as c_int {
            *arg = (*arg).offset(1);
        }
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
        let mut var2: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if eval6(arg, &raw mut var2, evalarg, op == '.' as c_int) == FAIL {
            tv_clear(rettv);
            return FAIL;
        }
        if evaluate {
            if op == '.' as c_int {
                if eval_concat_str(rettv, &raw mut var2) == FAIL {
                    return FAIL;
                }
            } else if op == '+' as c_int
                && (*rettv).v_type as c_uint == VAR_BLOB as c_int as c_uint
                && var2.v_type as c_uint == VAR_BLOB as c_int as c_uint
            {
                eval_addblob(rettv, &raw mut var2);
            } else if op == '+' as c_int
                && (*rettv).v_type as c_uint == VAR_LIST as c_int as c_uint
                && var2.v_type as c_uint == VAR_LIST as c_int as c_uint
            {
                if eval_addlist(rettv, &raw mut var2) == FAIL {
                    return FAIL;
                }
            } else if eval_addsub_number(rettv, &raw mut var2, op) == FAIL {
                return FAIL;
            }
            tv_clear(&raw mut var2);
        }
    }
    return OK;
}
unsafe extern "C" fn eval_multdiv_number(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
    mut op: c_int,
) -> c_int {
    let mut n1: varnumber_T = 0;
    let mut n2: varnumber_T = 0;
    let mut use_float: bool = false_0 != 0;
    let mut f1: float_T = 0 as c_int as float_T;
    let mut f2: float_T = 0 as c_int as float_T;
    let mut error: bool = false_0 != 0;
    if (*tv1).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
        f1 = (*tv1).vval.v_float;
        use_float = true_0 != 0;
        n1 = 0 as varnumber_T;
    } else {
        n1 = tv_get_number_chk(tv1, &raw mut error);
    }
    tv_clear(tv1);
    if error {
        tv_clear(tv2);
        return FAIL;
    }
    if (*tv2).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
        if !use_float {
            f1 = n1 as float_T;
            use_float = true_0 != 0;
        }
        f2 = (*tv2).vval.v_float;
        n2 = 0 as varnumber_T;
    } else {
        n2 = tv_get_number_chk(tv2, &raw mut error);
        tv_clear(tv2);
        if error {
            return FAIL;
        }
        if use_float {
            f2 = n2 as float_T;
        }
    }
    if use_float {
        if op == '*' as c_int {
            f1 = f1 * f2;
        } else if op == '/' as c_int {
            f1 = if f2 == 0 as c_int as float_T {
                if f1 == 0 as c_int as float_T {
                    ::core::f32::NAN as float_T
                } else if f1 > 0 as c_int as float_T {
                    ::core::f32::INFINITY as float_T
                } else {
                    -::core::f32::INFINITY as float_T
                }
            } else {
                f1 / f2
            };
        } else {
            emsg(gettext(
                b"E804: Cannot use '%' with Float\0".as_ptr() as *const c_char
            ));
            return FAIL;
        }
        (*tv1).v_type = VAR_FLOAT;
        (*tv1).vval.v_float = f1;
    } else {
        if op == '*' as c_int {
            // Vimscript arithmetic wraps on overflow (C two's-complement).
            n1 = n1.wrapping_mul(n2);
        } else if op == '/' as c_int {
            n1 = num_divide(n1, n2);
        } else {
            n1 = num_modulus(n1, n2);
        }
        (*tv1).v_type = VAR_NUMBER;
        (*tv1).vval.v_number = n1;
    }
    return OK;
}
unsafe extern "C" fn eval6(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut want_string: bool,
) -> c_int {
    if eval7(arg, rettv, evalarg, want_string) == FAIL {
        return FAIL;
    }
    loop {
        let mut op: c_int = **arg as uint8_t as c_int;
        if op != '*' as c_int && op != '/' as c_int && op != '%' as c_int {
            break;
        }
        let evaluate: bool = if evalarg.is_null() {
            0 as c_int
        } else {
            (*evalarg).eval_flags & EVAL_EVALUATE as c_int
        } != 0;
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
        let mut var2: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if eval7(arg, &raw mut var2, evalarg, false_0 != 0) == FAIL {
            return FAIL;
        }
        if evaluate {
            if eval_multdiv_number(rettv, &raw mut var2, op) == FAIL {
                return FAIL;
            }
        }
    }
    return OK;
}
unsafe extern "C" fn eval7(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut want_string: bool,
) -> c_int {
    let evaluate: bool = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0;
    let mut ret: c_int = OK;
    static recurse: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    (*rettv).v_type = VAR_UNKNOWN;
    let mut start_leader: *const c_char = *arg;
    while **arg as c_int == '!' as c_int
        || **arg as c_int == '-' as c_int
        || **arg as c_int == '+' as c_int
    {
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
    }
    let mut end_leader: *const c_char = *arg;
    if recurse.get() == 1000 as c_int {
        semsg(
            gettext((e_expression_too_recursive_str.ptr() as *const _) as *const c_char),
            *arg,
        );
        return FAIL;
    }
    (*recurse.ptr()) += 1;
    match **arg as c_int {
        48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
            ret = eval_number(arg, rettv, evaluate, want_string);
            if ret == OK && evaluate as c_int != 0 && end_leader > start_leader {
                ret = eval7_leader(rettv, true_0 != 0, start_leader, &raw mut end_leader);
            }
        }
        34 => {
            ret = eval_string(arg, rettv, evaluate, false_0 != 0);
        }
        39 => {
            ret = eval_lit_string(arg, rettv, evaluate, false_0 != 0);
        }
        91 => {
            ret = eval_list(arg, rettv, evalarg);
        }
        35 => {
            ret = eval_lit_dict(arg, rettv, evalarg);
        }
        123 => {
            ret = get_lambda_tv(arg, rettv, evalarg);
            if ret == NOTDONE {
                ret = eval_dict(arg, rettv, evalarg, false_0 != 0);
            }
        }
        38 => {
            ret = eval_option(arg as *mut *const c_char, rettv, evaluate);
        }
        36 => {
            if *(*arg).offset(1 as c_int as isize) as c_int == '"' as c_int
                || *(*arg).offset(1 as c_int as isize) as c_int == '\'' as c_int
            {
                ret = eval_interp_string(arg, rettv, evaluate);
            } else {
                ret = eval_env_var(arg, rettv, evaluate as c_int);
            }
        }
        64 => {
            *arg = (*arg).offset(1);
            if evaluate {
                (*rettv).v_type = VAR_STRING;
                (*rettv).vval.v_string =
                    get_reg_contents(**arg as c_int, kGRegExprSrc as c_int) as *mut c_char;
            }
            if **arg as c_int != NUL {
                *arg = (*arg).offset(1);
            }
        }
        40 => {
            *arg = skipwhite((*arg).offset(1 as c_int as isize));
            ret = eval1(arg, rettv, evalarg);
            if **arg as c_int == ')' as c_int {
                *arg = (*arg).offset(1);
            } else if ret == OK {
                emsg(gettext(b"E110: Missing ')'\0".as_ptr() as *const c_char));
                tv_clear(rettv);
                ret = FAIL;
            }
        }
        _ => {
            ret = NOTDONE;
        }
    }
    if ret == NOTDONE {
        let mut s: *mut c_char = *arg;
        let mut alias: *mut c_char = ::core::ptr::null_mut::<c_char>();
        let mut len: c_int = get_name_len(
            arg as *mut *const c_char,
            &raw mut alias,
            evaluate,
            true_0 != 0,
        );
        if !alias.is_null() {
            s = alias;
        }
        if len <= 0 as c_int {
            ret = FAIL;
        } else {
            let flags: c_int = if evalarg.is_null() {
                0 as c_int
            } else {
                (*evalarg).eval_flags
            };
            if *skipwhite(*arg) as c_int == '(' as c_int {
                *arg = skipwhite(*arg);
                ret = eval_func(
                    arg,
                    evalarg,
                    s,
                    len,
                    rettv,
                    flags,
                    ::core::ptr::null_mut::<typval_T>(),
                );
            } else if evaluate {
                ret = eval_variable(
                    s,
                    len,
                    rettv,
                    ::core::ptr::null_mut::<*mut dictitem_T>(),
                    true_0 != 0,
                    false_0 != 0,
                );
            } else {
                check_vars(s, len as size_t);
                if (*rettv).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint
                    && !evaluate
                    && strnequal(s, b"v:lua.\0".as_ptr() as *const c_char, 6 as size_t) as c_int
                        != 0
                {
                    (*rettv).v_type = VAR_PARTIAL;
                    (*rettv).vval.v_partial = get_vim_var_partial(VV_LUA);
                    (*(*rettv).vval.v_partial).pt_refcount += 1;
                }
                ret = OK;
            }
        }
        xfree(alias as *mut c_void);
    }
    *arg = skipwhite(*arg);
    if ret == OK {
        ret = handle_subscript(arg as *mut *const c_char, rettv, evalarg, true_0 != 0);
    }
    if ret == OK && evaluate as c_int != 0 && end_leader > start_leader {
        ret = eval7_leader(rettv, false_0 != 0, start_leader, &raw mut end_leader);
    }
    (*recurse.ptr()) -= 1;
    return ret;
}
unsafe extern "C" fn eval7_leader(
    rettv: *mut typval_T,
    numeric_only: bool,
    start_leader: *const c_char,
    end_leaderp: *mut *const c_char,
) -> c_int {
    let mut end_leader: *const c_char = *end_leaderp;
    let mut ret: c_int = OK;
    let mut error: bool = false_0 != 0;
    let mut val: varnumber_T = 0 as varnumber_T;
    let mut f: float_T = 0.0f64;
    if (*rettv).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
        f = (*rettv).vval.v_float;
    } else {
        val = tv_get_number_chk(rettv, &raw mut error);
    }
    if error {
        tv_clear(rettv);
        ret = FAIL;
    } else {
        while end_leader > start_leader {
            end_leader = end_leader.offset(-1);
            if *end_leader as c_int == '!' as c_int {
                if numeric_only {
                    end_leader = end_leader.offset(1);
                    break;
                } else if (*rettv).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
                    (*rettv).v_type = VAR_BOOL;
                    val = (if f == 0.0f64 {
                        kBoolVarTrue as c_int
                    } else {
                        kBoolVarFalse as c_int
                    }) as varnumber_T;
                } else {
                    val = (val == 0) as c_int as varnumber_T;
                }
            } else if *end_leader as c_int == '-' as c_int {
                if (*rettv).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
                    f = -f;
                } else {
                    val = -val;
                }
            }
        }
        if (*rettv).v_type as c_uint == VAR_FLOAT as c_int as c_uint {
            tv_clear(rettv);
            (*rettv).vval.v_float = f;
        } else {
            tv_clear(rettv);
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = val;
        }
    }
    *end_leaderp = end_leader;
    return ret;
}
unsafe extern "C" fn call_func_rettv(
    arg: *mut *mut c_char,
    evalarg: *mut evalarg_T,
    rettv: *mut typval_T,
    evaluate: bool,
    selfdict: *mut dict_T,
    basetv: *mut typval_T,
    lua_funcname: *const c_char,
) -> c_int {
    let mut funcexe: funcexe_T = funcexe_T {
        fe_argv_func: None,
        fe_firstline: 0,
        fe_lastline: 0,
        fe_doesrange: ::core::ptr::null_mut::<bool>(),
        fe_evaluate: false,
        fe_partial: ::core::ptr::null_mut::<partial_T>(),
        fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
        fe_basetv: ::core::ptr::null_mut::<typval_T>(),
        fe_found_var: false,
    };
    let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut functv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut funcname: *const c_char = ::core::ptr::null::<c_char>();
    let mut is_lua: bool = false_0 != 0;
    let mut ret: c_int = 0;
    '_theend: {
        if evaluate {
            functv = *rettv;
            (*rettv).v_type = VAR_UNKNOWN;
            if functv.v_type as c_uint == VAR_PARTIAL as c_int as c_uint {
                pt = functv.vval.v_partial;
                is_lua = is_luafunc(pt);
                funcname = if is_lua as c_int != 0 {
                    lua_funcname
                } else {
                    partial_name(pt) as *const c_char
                };
            } else {
                funcname = functv.vval.v_string;
                if funcname.is_null() || *funcname as c_int == NUL {
                    emsg(gettext(
                        (e_empty_function_name.ptr() as *const _) as *const c_char,
                    ));
                    ret = FAIL;
                    break '_theend;
                }
            }
        } else {
            funcname = b"\0".as_ptr() as *const c_char;
        }
        funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = evaluate;
        funcexe.fe_partial = pt;
        funcexe.fe_selfdict = selfdict;
        funcexe.fe_basetv = basetv;
        ret = get_func_tv(
            funcname,
            if is_lua as c_int != 0 {
                (*arg).offset_from(funcname) as c_int
            } else {
                -1 as c_int
            },
            rettv,
            arg,
            evalarg,
            &raw mut funcexe,
        );
    }
    if evaluate {
        tv_clear(&raw mut functv);
    }
    return ret;
}
unsafe extern "C" fn eval_lambda(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> c_int {
    let evaluate: bool = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0;
    *arg = (*arg).offset(2 as c_int as isize);
    let mut base: typval_T = *rettv;
    (*rettv).v_type = VAR_UNKNOWN;
    let mut ret: c_int = get_lambda_tv(arg, rettv, evalarg);
    if ret != OK {
        return FAIL;
    } else if **arg as c_int != '(' as c_int {
        if verbose {
            if *skipwhite(*arg) as c_int == '(' as c_int {
                emsg(gettext(e_nowhitespace.get()));
            } else {
                semsg(
                    gettext(&raw const e_missingparen as *const c_char),
                    b"lambda\0".as_ptr() as *const c_char,
                );
            }
        }
        tv_clear(rettv);
        ret = FAIL;
    } else {
        ret = call_func_rettv(
            arg,
            evalarg,
            rettv,
            evaluate,
            ::core::ptr::null_mut::<dict_T>(),
            &raw mut base,
            ::core::ptr::null::<c_char>(),
        );
    }
    if evaluate {
        tv_clear(&raw mut base);
    }
    return ret;
}
unsafe extern "C" fn eval_method(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> c_int {
    let evaluate: bool = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0;
    *arg = (*arg).offset(2 as c_int as isize);
    let mut base: typval_T = *rettv;
    (*rettv).v_type = VAR_UNKNOWN;
    let mut len: c_int = 0;
    let mut name: *mut c_char = *arg;
    let mut lua_funcname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut alias: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if strnequal(name, b"v:lua.\0".as_ptr() as *const c_char, 6 as size_t) {
        lua_funcname = name.offset(6 as c_int as isize);
        *arg = skip_luafunc_name(lua_funcname) as *mut c_char;
        *arg = skipwhite(*arg);
        len = (*arg).offset_from(lua_funcname) as c_int;
    } else {
        len = get_name_len(
            arg as *mut *const c_char,
            &raw mut alias,
            evaluate,
            true_0 != 0,
        );
        if !alias.is_null() {
            name = alias;
        }
    }
    let mut tofree: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut ret: c_int = OK;
    if len <= 0 as c_int {
        if verbose {
            if lua_funcname.is_null() {
                emsg(gettext(
                    b"E260: Missing name after ->\0".as_ptr() as *const c_char
                ));
            } else {
                semsg(gettext(&raw const e_invexpr2 as *const c_char), name);
            }
        }
        ret = FAIL;
    } else {
        *arg = skipwhite(*arg);
        let mut paren: *mut c_char = ::core::ptr::null_mut::<c_char>();
        if **arg as c_int != '(' as c_int && lua_funcname.is_null() && alias.is_null() && {
            paren = vim_strchr(*arg, '(' as c_int);
            !paren.is_null()
        } {
            *arg = name;
            *paren = NUL as c_char;
            let mut ref_0: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            ref_0.v_type = VAR_UNKNOWN;
            if eval7(arg, &raw mut ref_0, evalarg, false_0 != 0) == FAIL {
                *arg = name.offset(len as isize);
                ret = FAIL;
            } else if *skipwhite(*arg) as c_int != NUL {
                if verbose {
                    semsg(gettext(&raw const e_trailing_arg as *const c_char), *arg);
                }
                ret = FAIL;
            } else if ref_0.v_type as c_uint == VAR_FUNC as c_int as c_uint
                && !ref_0.vval.v_string.is_null()
            {
                name = ref_0.vval.v_string;
                ref_0.vval.v_string = ::core::ptr::null_mut::<c_char>();
                tofree = name;
                len = strlen(name) as c_int;
            } else if ref_0.v_type as c_uint == VAR_PARTIAL as c_int as c_uint
                && !ref_0.vval.v_partial.is_null()
            {
                if (*ref_0.vval.v_partial).pt_argc > 0 as c_int
                    || !(*ref_0.vval.v_partial).pt_dict.is_null()
                {
                    if verbose {
                        emsg(gettext(
                            (e_cannot_use_partial_here.ptr() as *const _) as *const c_char,
                        ));
                    }
                    ret = FAIL;
                } else {
                    name = xstrdup(partial_name(ref_0.vval.v_partial));
                    tofree = name;
                    if name.is_null() {
                        ret = FAIL;
                        name = *arg;
                    } else {
                        len = strlen(name) as c_int;
                    }
                }
            } else {
                if verbose {
                    semsg(
                        gettext(&raw const e_not_callable_type_str as *const c_char),
                        name,
                    );
                }
                ret = FAIL;
            }
            tv_clear(&raw mut ref_0);
            *paren = '(' as c_char;
        }
        if ret == OK {
            if **arg as c_int != '(' as c_int {
                if verbose {
                    semsg(gettext(&raw const e_missingparen as *const c_char), name);
                }
                ret = FAIL;
            } else if ascii_iswhite(*(*arg).offset(-1 as c_int as isize) as c_int) {
                if verbose {
                    emsg(gettext(e_nowhitespace.get()));
                }
                ret = FAIL;
            } else if !lua_funcname.is_null() {
                if evaluate {
                    (*rettv).v_type = VAR_PARTIAL;
                    (*rettv).vval.v_partial = get_vim_var_partial(VV_LUA);
                    (*(*rettv).vval.v_partial).pt_refcount += 1;
                }
                ret = call_func_rettv(
                    arg,
                    evalarg,
                    rettv,
                    evaluate,
                    ::core::ptr::null_mut::<dict_T>(),
                    &raw mut base,
                    lua_funcname,
                );
            } else {
                ret = eval_func(
                    arg,
                    evalarg,
                    name,
                    len,
                    rettv,
                    if evaluate as c_int != 0 {
                        EVAL_EVALUATE as c_int
                    } else {
                        0 as c_int
                    },
                    &raw mut base,
                );
            }
        }
    }
    if evaluate {
        tv_clear(&raw mut base);
    }
    xfree(tofree as *mut c_void);
    if !alias.is_null() {
        xfree(alias as *mut c_void);
    }
    return ret;
}
unsafe extern "C" fn eval_index(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut verbose: bool,
) -> c_int {
    let evaluate: bool = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0;
    let mut empty1: bool = false_0 != 0;
    let mut empty2: bool = false_0 != 0;
    let mut range: bool = false_0 != 0;
    let mut key: *const c_char = ::core::ptr::null::<c_char>();
    let mut keylen: ptrdiff_t = -1 as ptrdiff_t;
    if check_can_index(rettv, evaluate, verbose) == FAIL {
        return FAIL;
    }
    let mut var1: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut var2: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if **arg as c_int == '.' as c_int {
        key = (*arg).offset(1 as c_int as isize);
        keylen = 0 as ptrdiff_t;
        while eval_isdictc(*key.offset(keylen as isize) as c_int) {
            keylen += 1;
        }
        if keylen == 0 as ptrdiff_t {
            return FAIL;
        }
        *arg = skipwhite(key.offset(keylen as isize));
    } else {
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
        if **arg as c_int == ':' as c_int {
            empty1 = true_0 != 0;
        } else if eval1(arg, &raw mut var1, evalarg) == FAIL {
            return FAIL;
        } else if evaluate as c_int != 0 && !tv_check_str(&raw mut var1) {
            tv_clear(&raw mut var1);
            return FAIL;
        }
        if **arg as c_int == ':' as c_int {
            range = true_0 != 0;
            *arg = skipwhite((*arg).offset(1 as c_int as isize));
            if **arg as c_int == ']' as c_int {
                empty2 = true_0 != 0;
            } else if eval1(arg, &raw mut var2, evalarg) == FAIL {
                if !empty1 {
                    tv_clear(&raw mut var1);
                }
                return FAIL;
            } else if evaluate as c_int != 0 && !tv_check_str(&raw mut var2) {
                if !empty1 {
                    tv_clear(&raw mut var1);
                }
                tv_clear(&raw mut var2);
                return FAIL;
            }
        }
        if **arg as c_int != ']' as c_int {
            if verbose {
                emsg(gettext(e_missbrac.get()));
            }
            tv_clear(&raw mut var1);
            if range {
                tv_clear(&raw mut var2);
            }
            return FAIL;
        }
        *arg = skipwhite((*arg).offset(1 as c_int as isize));
    }
    if evaluate {
        let mut res: c_int = eval_index_inner(
            rettv,
            range,
            if empty1 as c_int != 0 {
                ::core::ptr::null_mut::<typval_T>()
            } else {
                &raw mut var1
            },
            if empty2 as c_int != 0 {
                ::core::ptr::null_mut::<typval_T>()
            } else {
                &raw mut var2
            },
            false_0 != 0,
            key,
            keylen,
            verbose,
        );
        if !empty1 {
            tv_clear(&raw mut var1);
        }
        if range {
            tv_clear(&raw mut var2);
        }
        return res;
    }
    return OK;
}
unsafe extern "C" fn check_can_index(
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut verbose: bool,
) -> c_int {
    match (*rettv).v_type as c_uint {
        3 | 9 => {
            if verbose {
                emsg(gettext(
                    (e_cannot_index_a_funcref.ptr() as *const _) as *const c_char,
                ));
            }
            return FAIL;
        }
        6 => {
            if verbose {
                emsg(gettext(&raw const e_using_float_as_string as *const c_char));
            }
            return FAIL;
        }
        7 | 8 => {
            if verbose {
                emsg(gettext(
                    (e_cannot_index_special_variable.ptr() as *const _) as *const c_char,
                ));
            }
            return FAIL;
        }
        0 => {
            if evaluate {
                emsg(gettext(
                    (e_cannot_index_special_variable.ptr() as *const _) as *const c_char,
                ));
                return FAIL;
            }
        }
        2 | 1 | 4 | 5 | 10 | _ => {}
    }
    return OK;
}
pub unsafe extern "C" fn f_slice(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_can_index(
        argvars.offset(0 as c_int as isize),
        true_0 != 0,
        false_0 != 0,
    ) != OK
    {
        return;
    }
    tv_copy(argvars, rettv);
    eval_index_inner(
        rettv,
        true_0 != 0,
        argvars.offset(1 as c_int as isize),
        if (*argvars.offset(2 as c_int as isize)).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint
        {
            ::core::ptr::null_mut::<typval_T>()
        } else {
            argvars.offset(2 as c_int as isize)
        },
        true_0 != 0,
        ::core::ptr::null::<c_char>(),
        0 as ptrdiff_t,
        false_0 != 0,
    );
}
unsafe extern "C" fn eval_index_inner(
    mut rettv: *mut typval_T,
    mut is_range: bool,
    mut var1: *mut typval_T,
    mut var2: *mut typval_T,
    mut exclusive: bool,
    mut key: *const c_char,
    mut keylen: ptrdiff_t,
    mut verbose: bool,
) -> c_int {
    let mut n1: varnumber_T = 0 as varnumber_T;
    let mut n2: varnumber_T = 0 as varnumber_T;
    if !var1.is_null() && (*rettv).v_type as c_uint != VAR_DICT as c_int as c_uint {
        n1 = tv_get_number(var1);
    }
    if is_range {
        if (*rettv).v_type as c_uint == VAR_DICT as c_int as c_uint {
            if verbose {
                emsg(gettext(
                    (e_cannot_slice_dictionary.ptr() as *const _) as *const c_char,
                ));
            }
            return FAIL;
        }
        if !var2.is_null() {
            n2 = tv_get_number(var2);
        } else {
            n2 = VARNUMBER_MAX as varnumber_T;
        }
    }
    match (*rettv).v_type as c_uint {
        1 | 2 => {
            let s: *const c_char = tv_get_string(rettv);
            let mut v: *mut c_char = ::core::ptr::null_mut::<c_char>();
            let mut len: c_int = strlen(s) as c_int;
            if exclusive {
                if is_range {
                    v = string_slice(s, n1, n2, exclusive);
                } else {
                    v = char_from_string(s, n1);
                }
            } else if is_range {
                if n1 < 0 as varnumber_T {
                    n1 = len as varnumber_T + n1;
                    if n1 < 0 as varnumber_T {
                        n1 = 0 as varnumber_T;
                    }
                }
                if n2 < 0 as varnumber_T {
                    n2 = len as varnumber_T + n2;
                } else if n2 >= len as varnumber_T {
                    n2 = len as varnumber_T;
                }
                if n1 >= len as varnumber_T || n2 < 0 as varnumber_T || n1 > n2 {
                    v = ::core::ptr::null_mut::<c_char>();
                } else {
                    v = xmemdupz(
                        s.offset(n1 as isize) as *const c_void,
                        (n2 as size_t)
                            .wrapping_sub(n1 as size_t)
                            .wrapping_add(1 as size_t),
                    ) as *mut c_char;
                }
            } else if n1 >= len as varnumber_T || n1 < 0 as varnumber_T {
                v = ::core::ptr::null_mut::<c_char>();
            } else {
                v = xmemdupz(s.offset(n1 as isize) as *const c_void, 1 as size_t) as *mut c_char;
            }
            tv_clear(rettv);
            (*rettv).v_type = VAR_STRING;
            (*rettv).vval.v_string = v;
        }
        10 => {
            tv_blob_slice_or_index((*rettv).vval.v_blob, is_range, n1, n2, exclusive, rettv);
        }
        4 => {
            if var1.is_null() {
                n1 = 0 as varnumber_T;
            }
            if var2.is_null() {
                n2 = VARNUMBER_MAX as varnumber_T;
            }
            if tv_list_slice_or_index(
                (*rettv).vval.v_list,
                is_range,
                n1,
                n2,
                exclusive,
                rettv,
                verbose,
            ) == FAIL
            {
                return FAIL;
            }
        }
        5 => {
            if key.is_null() {
                key = tv_get_string_chk(var1);
                if key.is_null() {
                    return FAIL;
                }
            }
            let item: *mut dictitem_T = tv_dict_find((*rettv).vval.v_dict, key, keylen);
            if item.is_null() && verbose as c_int != 0 {
                if keylen > 0 as ptrdiff_t {
                    semsg(
                        gettext(&raw const e_dictkey_len as *const c_char),
                        keylen,
                        key,
                    );
                } else {
                    semsg(gettext(&raw const e_dictkey as *const c_char), key);
                }
            }
            if item.is_null() || tv_is_luafunc(&raw mut (*item).di_tv) as c_int != 0 {
                return FAIL;
            }
            let mut tmp: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            tv_copy(&raw mut (*item).di_tv, &raw mut tmp);
            tv_clear(rettv);
            *rettv = tmp;
        }
        7 | 8 | 3 | 6 | 9 | 0 | _ => {}
    }
    return OK;
}
pub unsafe extern "C" fn eval_option(
    arg: *mut *const c_char,
    rettv: *mut typval_T,
    evaluate: bool,
) -> c_int {
    let working: bool = **arg as c_int == '+' as c_int;
    let mut opt_idx: OptIndex = kOptAleph;
    let mut opt_flags: c_int = 0;
    let option_end: *mut c_char =
        find_option_var_end(arg, &raw mut opt_idx, &raw mut opt_flags) as *mut c_char;
    if option_end.is_null() {
        if !rettv.is_null() {
            semsg(
                gettext(b"E112: Option name missing: %s\0".as_ptr() as *const c_char),
                *arg,
            );
        }
        return FAIL;
    }
    if !evaluate {
        *arg = option_end;
        return OK;
    }
    let mut c: c_char = *option_end;
    *option_end = NUL as c_char;
    let mut ret: c_int = OK;
    let mut is_tty_opt: bool = is_tty_option(*arg);
    if opt_idx as c_int == kOptInvalid as c_int && !is_tty_opt {
        if !rettv.is_null() {
            semsg(
                gettext(b"E113: Unknown option: %s\0".as_ptr() as *const c_char),
                *arg,
            );
        }
        ret = FAIL;
    } else if !rettv.is_null() {
        let mut value: OptVal = if is_tty_opt as c_int != 0 {
            get_tty_option(*arg)
        } else {
            get_option_value(opt_idx, opt_flags)
        };
        '_c2rust_label: {
            if value.type_0 as c_int != kOptValTypeNil as c_int {
            } else {
                __assert_fail(
                    b"value.type != kOptValTypeNil\0".as_ptr() as *const c_char,
                    b"src/nvim/eval.rs\0".as_ptr() as *const c_char,
                    3409 as c_uint,
                    b"int eval_option(const char **const, typval_T *const, const _Bool)\0".as_ptr()
                        as *const c_char,
                );
            }
        };
        *rettv = optval_as_tv(value, true_0 != 0);
    } else if working as c_int != 0 && !is_tty_opt && is_option_hidden(opt_idx) as c_int != 0 {
        ret = FAIL;
    }
    *option_end = c;
    *arg = option_end;
    return ret;
}
unsafe extern "C" fn eval_number(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut want_string: bool,
) -> c_int {
    let mut p: *mut c_char = skipdigits((*arg).offset(1 as c_int as isize));
    let mut get_float: bool = false_0 != 0;
    if !want_string
        && *p.offset(0 as c_int as isize) as c_int == '.' as c_int
        && ascii_isdigit(*p.offset(1 as c_int as isize) as c_int) as c_int != 0
    {
        get_float = true_0 != 0;
        p = skipdigits(p.offset(2 as c_int as isize));
        if *p as c_int == 'e' as c_int || *p as c_int == 'E' as c_int {
            p = p.offset(1);
            if *p as c_int == '-' as c_int || *p as c_int == '+' as c_int {
                p = p.offset(1);
            }
            if !ascii_isdigit(*p as c_int) {
                get_float = false_0 != 0;
            } else {
                p = skipdigits(p.offset(1 as c_int as isize));
            }
        }
        if *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
            || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
            || *p as c_int == '.' as c_int
        {
            get_float = false_0 != 0;
        }
    }
    if get_float {
        let mut f: float_T = 0.;
        *arg = (*arg).offset(string2float(*arg, &raw mut f) as isize);
        if evaluate {
            (*rettv).v_type = VAR_FLOAT;
            (*rettv).vval.v_float = f;
        }
    } else if **arg as c_int == '0' as c_int
        && (*(*arg).offset(1 as c_int as isize) as c_int == 'z' as c_int
            || *(*arg).offset(1 as c_int as isize) as c_int == 'Z' as c_int)
    {
        let mut blob: *mut blob_T = ::core::ptr::null_mut::<blob_T>();
        if evaluate {
            blob = tv_blob_alloc();
        }
        let mut bp: *mut c_char = ::core::ptr::null_mut::<c_char>();
        bp = (*arg).offset(2 as c_int as isize);
        while ascii_isxdigit(*bp.offset(0 as c_int as isize) as c_int) {
            if !ascii_isxdigit(*bp.offset(1 as c_int as isize) as c_int) {
                if !blob.is_null() {
                    emsg(gettext(
                        b"E973: Blob literal should have an even number of hex characters\0"
                            .as_ptr() as *const c_char,
                    ));
                    ga_clear(&raw mut (*blob).bv_ga);
                    let mut ptr_: *mut *mut c_void = &raw mut blob as *mut *mut c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL_0;
                    let _ = *ptr_;
                }
                return FAIL;
            }
            if !blob.is_null() {
                ga_append(
                    &raw mut (*blob).bv_ga,
                    ((hex2nr(*bp as c_int) << 4 as c_int)
                        + hex2nr(*bp.offset(1 as c_int as isize) as c_int))
                        as uint8_t,
                );
            }
            if *bp.offset(2 as c_int as isize) as c_int == '.' as c_int
                && ascii_isxdigit(*bp.offset(3 as c_int as isize) as c_int) as c_int != 0
            {
                bp = bp.offset(1);
            }
            bp = bp.offset(2 as c_int as isize);
        }
        if !blob.is_null() {
            tv_blob_set_ret(rettv, blob);
        }
        *arg = bp;
    } else {
        let mut len: c_int = 0;
        let mut n: varnumber_T = 0;
        vim_str2nr(
            *arg,
            ::core::ptr::null_mut::<c_int>(),
            &raw mut len,
            STR2NR_ALL as c_int,
            &raw mut n,
            ::core::ptr::null_mut::<uvarnumber_T>(),
            0 as c_int,
            true_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        if len == 0 as c_int {
            if evaluate {
                semsg(gettext(&raw const e_invexpr2 as *const c_char), *arg);
            }
            return FAIL;
        }
        *arg = (*arg).offset(len as isize);
        if evaluate {
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = n;
        }
    }
    return OK;
}
unsafe extern "C" fn eval_string(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut interpolate: bool,
) -> c_int {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let arg_end: *const c_char = (*arg).offset(strlen(*arg) as isize);
    let mut extra: c_uint = (if interpolate as c_int != 0 {
        1 as c_int
    } else {
        0 as c_int
    }) as c_uint;
    let off: c_int = if interpolate as c_int != 0 {
        0 as c_int
    } else {
        1 as c_int
    };
    p = (*arg).offset(off as isize);
    while *p as c_int != NUL && *p as c_int != '"' as c_int {
        if *p as c_int == '\\' as c_int && *p.offset(1 as c_int as isize) as c_int != NUL {
            p = p.offset(1);
            if *p as c_int == '<' as c_int {
                let mut modifiers: c_int = 0 as c_int;
                let mut flags: c_int = FSK_KEYCODE as c_int | FSK_IN_STRING as c_int;
                extra = extra.wrapping_add(5 as c_uint);
                if *p.offset(1 as c_int as isize) as c_int != '*' as c_int {
                    flags |= FSK_SIMPLIFY as c_int;
                }
                if find_special_key(
                    &raw mut p as *mut *const c_char,
                    arg_end.offset_from(p) as size_t,
                    &raw mut modifiers,
                    flags,
                    ::core::ptr::null_mut::<bool>(),
                ) != 0 as c_int
                {
                    p = p.offset(-1);
                }
            }
        } else if interpolate as c_int != 0
            && (*p as c_int == '{' as c_int || *p as c_int == '}' as c_int)
        {
            if *p as c_int == '{' as c_int
                && *p.offset(1 as c_int as isize) as c_int != '{' as c_int
            {
                break;
            }
            p = p.offset(1);
            if *p.offset(-1 as c_int as isize) as c_int == '}' as c_int
                && *p as c_int != '}' as c_int
            {
                semsg(
                    gettext(&raw const e_stray_closing_curly_str as *const c_char),
                    *arg,
                );
                return FAIL;
            }
            extra = extra.wrapping_sub(1);
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if *p as c_int != '"' as c_int && !(interpolate as c_int != 0 && *p as c_int == '{' as c_int) {
        semsg(
            gettext(b"E114: Missing quote: %s\0".as_ptr() as *const c_char),
            *arg,
        );
        return FAIL;
    }
    if !evaluate {
        *arg = p.offset(off as isize);
        return OK;
    }
    (*rettv).v_type = VAR_STRING;
    let len: c_int = (p.offset_from(*arg) + extra as isize) as c_int;
    (*rettv).vval.v_string = xmalloc(len as size_t) as *mut c_char;
    let mut end: *mut c_char = (*rettv).vval.v_string;
    p = (*arg).offset(off as isize);
    while *p as c_int != NUL && *p as c_int != '"' as c_int {
        if *p as c_int == '\\' as c_int {
            's_424: {
                p = p.offset(1);
                match *p as c_int {
                    98 => {
                        let c2rust_fresh0 = end;
                        end = end.offset(1);
                        *c2rust_fresh0 = BS as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    101 => {
                        let c2rust_fresh1 = end;
                        end = end.offset(1);
                        *c2rust_fresh1 = ESC as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    102 => {
                        let c2rust_fresh2 = end;
                        end = end.offset(1);
                        *c2rust_fresh2 = FF as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    110 => {
                        let c2rust_fresh3 = end;
                        end = end.offset(1);
                        *c2rust_fresh3 = NL as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    114 => {
                        let c2rust_fresh4 = end;
                        end = end.offset(1);
                        *c2rust_fresh4 = CAR as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    116 => {
                        let c2rust_fresh5 = end;
                        end = end.offset(1);
                        *c2rust_fresh5 = TAB as c_char;
                        p = p.offset(1);
                        break 's_424;
                    }
                    88 | 120 | 117 | 85 => {
                        if ascii_isxdigit(*p.offset(1 as c_int as isize) as c_int) {
                            let mut n: c_int = 0;
                            let mut nr: c_int = 0;
                            let mut c: c_int = toupper(*p as uint8_t as c_int);
                            if c == 'X' as c_int {
                                n = 2 as c_int;
                            } else if *p as c_int == 'u' as c_int {
                                n = 4 as c_int;
                            } else {
                                n = 8 as c_int;
                            }
                            nr = 0 as c_int;
                            loop {
                                n -= 1;
                                if !(n >= 0 as c_int
                                    && ascii_isxdigit(*p.offset(1 as c_int as isize) as c_int)
                                        as c_int
                                        != 0)
                                {
                                    break;
                                }
                                p = p.offset(1);
                                nr = (nr << 4 as c_int) + hex2nr(*p as c_int);
                            }
                            p = p.offset(1);
                            if c != 'X' as c_int {
                                end = end.offset(utf_char2bytes(nr, end) as isize);
                            } else {
                                let c2rust_fresh6 = end;
                                end = end.offset(1);
                                *c2rust_fresh6 = nr as c_char;
                            }
                        }
                        break 's_424;
                    }
                    48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 => {
                        let c2rust_fresh7 = p;
                        p = p.offset(1);
                        *end = (*c2rust_fresh7 as c_int - '0' as c_int) as c_char;
                        if *p as c_int >= '0' as c_int && *p as c_int <= '7' as c_int {
                            let c2rust_fresh8 = p;
                            p = p.offset(1);
                            *end = (((*end as c_int) << 3 as c_int) + *c2rust_fresh8 as c_int
                                - '0' as c_int) as c_char;
                            if *p as c_int >= '0' as c_int && *p as c_int <= '7' as c_int {
                                let c2rust_fresh9 = p;
                                p = p.offset(1);
                                *end = (((*end as c_int) << 3 as c_int) + *c2rust_fresh9 as c_int
                                    - '0' as c_int)
                                    as c_char;
                            }
                        }
                        end = end.offset(1);
                        break 's_424;
                    }
                    60 => {
                        let mut flags_0: c_int = FSK_KEYCODE as c_int | FSK_IN_STRING as c_int;
                        if *p.offset(1 as c_int as isize) as c_int != '*' as c_int {
                            flags_0 |= FSK_SIMPLIFY as c_int;
                        }
                        extra = trans_special(
                            &raw mut p as *mut *const c_char,
                            arg_end.offset_from(p) as size_t,
                            end,
                            flags_0,
                            false_0 != 0,
                            ::core::ptr::null_mut::<bool>(),
                        );
                        if extra != 0 as c_uint {
                            end = end.offset(extra as isize);
                            if end >= (*rettv).vval.v_string.offset(len as isize) {
                                iemsg(b"eval_string() used more space than allocated\0".as_ptr()
                                    as *const c_char);
                            }
                            break 's_424;
                        }
                    }
                    _ => {}
                }
                mb_copy_char(&raw mut p as *mut *const c_char, &raw mut end);
            }
        } else {
            if interpolate as c_int != 0
                && (*p as c_int == '{' as c_int || *p as c_int == '}' as c_int)
            {
                if *p as c_int == '{' as c_int
                    && *p.offset(1 as c_int as isize) as c_int != '{' as c_int
                {
                    break;
                }
                p = p.offset(1);
            }
            mb_copy_char(&raw mut p as *mut *const c_char, &raw mut end);
        }
    }
    *end = NUL as c_char;
    if *p as c_int == '"' as c_int && !interpolate {
        p = p.offset(1);
    }
    *arg = p;
    return OK;
}
unsafe extern "C" fn eval_lit_string(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
    mut interpolate: bool,
) -> c_int {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut reduce: c_int = if interpolate as c_int != 0 {
        -1 as c_int
    } else {
        0 as c_int
    };
    let off: c_int = if interpolate as c_int != 0 {
        0 as c_int
    } else {
        1 as c_int
    };
    p = (*arg).offset(off as isize);
    while *p as c_int != NUL {
        if *p as c_int == '\'' as c_int {
            if *p.offset(1 as c_int as isize) as c_int != '\'' as c_int {
                break;
            }
            reduce += 1;
            p = p.offset(1);
        } else if interpolate {
            if *p as c_int == '{' as c_int {
                if *p.offset(1 as c_int as isize) as c_int != '{' as c_int {
                    break;
                }
                p = p.offset(1);
                reduce += 1;
            } else if *p as c_int == '}' as c_int {
                p = p.offset(1);
                if *p as c_int != '}' as c_int {
                    semsg(
                        gettext(&raw const e_stray_closing_curly_str as *const c_char),
                        *arg,
                    );
                    return FAIL;
                }
                reduce += 1;
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if *p as c_int != '\'' as c_int && !(interpolate as c_int != 0 && *p as c_int == '{' as c_int) {
        semsg(
            gettext(b"E115: Missing quote: %s\0".as_ptr() as *const c_char),
            *arg,
        );
        return FAIL;
    }
    if !evaluate {
        *arg = p.offset(off as isize);
        return OK;
    }
    let mut str: *mut c_char =
        xmalloc((p.offset_from(*arg) - reduce as isize) as size_t) as *mut c_char;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = str;
    p = (*arg).offset(off as isize);
    while *p as c_int != NUL {
        if *p as c_int == '\'' as c_int {
            if *p.offset(1 as c_int as isize) as c_int != '\'' as c_int {
                break;
            }
            p = p.offset(1);
        } else if interpolate as c_int != 0
            && (*p as c_int == '{' as c_int || *p as c_int == '}' as c_int)
        {
            if *p as c_int == '{' as c_int
                && *p.offset(1 as c_int as isize) as c_int != '{' as c_int
            {
                break;
            }
            p = p.offset(1);
        }
        mb_copy_char(&raw mut p as *mut *const c_char, &raw mut str);
    }
    *str = NUL as c_char;
    *arg = p.offset(off as isize);
    return OK;
}
pub unsafe extern "C" fn eval_interp_string(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut evaluate: bool,
) -> c_int {
    let mut ret: c_int = OK;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    };
    ga_init(&raw mut ga, 1 as c_int, 80 as c_int);
    *arg = (*arg).offset(1);
    let quote: c_int = **arg as uint8_t as c_int;
    *arg = (*arg).offset(1);
    loop {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if quote == '"' as c_int {
            ret = eval_string(arg, &raw mut tv, evaluate, true_0 != 0);
        } else {
            ret = eval_lit_string(arg, &raw mut tv, evaluate, true_0 != 0);
        }
        if ret == FAIL {
            break;
        }
        if evaluate {
            ga_concat(&raw mut ga, tv.vval.v_string);
            tv_clear(&raw mut tv);
        }
        if **arg as c_int != '{' as c_int {
            *arg = (*arg).offset(1);
            break;
        } else {
            let mut p: *mut c_char = eval_one_expr_in_str(*arg, &raw mut ga, evaluate);
            if p.is_null() {
                ret = FAIL;
                break;
            } else {
                *arg = p;
            }
        }
    }
    (*rettv).v_type = VAR_STRING;
    if ret != FAIL && evaluate as c_int != 0 {
        ga_append(&raw mut ga, NUL as uint8_t);
    }
    (*rettv).vval.v_string = ga.ga_data as *mut c_char;
    return OK;
}
pub unsafe extern "C" fn partial_name(mut pt: *mut partial_T) -> *mut c_char {
    if !pt.is_null() {
        if !(*pt).pt_name.is_null() {
            return (*pt).pt_name;
        }
        if !(*pt).pt_func.is_null() {
            return &raw mut (*(*pt).pt_func).uf_name as *mut c_char;
        }
    }
    return b"\0".as_ptr() as *const c_char as *mut c_char;
}
unsafe extern "C" fn partial_free(mut pt: *mut partial_T) {
    let mut i: c_int = 0 as c_int;
    while i < (*pt).pt_argc {
        tv_clear((*pt).pt_argv.offset(i as isize));
        i += 1;
    }
    xfree((*pt).pt_argv as *mut c_void);
    tv_dict_unref((*pt).pt_dict);
    if !(*pt).pt_name.is_null() {
        func_unref((*pt).pt_name);
        xfree((*pt).pt_name as *mut c_void);
    } else {
        func_ptr_unref((*pt).pt_func);
    }
    xfree(pt as *mut c_void);
}
pub unsafe extern "C" fn partial_unref(mut pt: *mut partial_T) {
    if pt.is_null() {
        return;
    }
    (*pt).pt_refcount -= 1;
    if (*pt).pt_refcount <= 0 as c_int {
        partial_free(pt);
    }
}
unsafe extern "C" fn eval_list(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let evaluate: bool = if evalarg.is_null() {
        false_0
    } else {
        (*evalarg).eval_flags & EVAL_EVALUATE as c_int
    } != 0;
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if evaluate {
        l = tv_list_alloc(kListLenShouldKnow as c_int as ptrdiff_t);
    }
    *arg = skipwhite((*arg).offset(1 as c_int as isize));
    '_failret: {
        while **arg as c_int != ']' as c_int && **arg as c_int != NUL {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if eval1(arg, &raw mut tv, evalarg) == FAIL {
                break '_failret;
            }
            if evaluate {
                tv.v_lock = VAR_UNLOCKED;
                tv_list_append_owned_tv(l, tv);
            }
            let mut had_comma: bool = **arg as c_int == ',' as c_int;
            if had_comma {
                *arg = skipwhite((*arg).offset(1 as c_int as isize));
            }
            if **arg as c_int == ']' as c_int {
                break;
            }
            if had_comma {
                continue;
            }
            semsg(
                gettext(b"E696: Missing comma in List: %s\0".as_ptr() as *const c_char),
                *arg,
            );
            break '_failret;
        }
        if **arg as c_int != ']' as c_int {
            semsg(gettext(e_list_end.get()), *arg);
        } else {
            *arg = skipwhite((*arg).offset(1 as c_int as isize));
            if evaluate {
                tv_list_set_ret(rettv, l);
            }
            return OK;
        }
    }
    if evaluate {
        tv_list_free(l);
    }
    return FAIL;
}
pub unsafe extern "C" fn func_equal(
    mut tv1: *mut typval_T,
    mut tv2: *mut typval_T,
    mut ic: bool,
) -> bool {
    let mut s1: *mut c_char = if (*tv1).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        (*tv1).vval.v_string
    } else {
        partial_name((*tv1).vval.v_partial)
    };
    if !s1.is_null() && *s1 as c_int == NUL {
        s1 = ::core::ptr::null_mut::<c_char>();
    }
    let mut s2: *mut c_char = if (*tv2).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        (*tv2).vval.v_string
    } else {
        partial_name((*tv2).vval.v_partial)
    };
    if !s2.is_null() && *s2 as c_int == NUL {
        s2 = ::core::ptr::null_mut::<c_char>();
    }
    if s1.is_null() || s2.is_null() {
        if s1 != s2 {
            return false_0 != 0;
        }
    } else if strcmp(s1, s2) != 0 as c_int {
        return false_0 != 0;
    }
    let mut d1: *mut dict_T = if (*tv1).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        ::core::ptr::null_mut::<dict_T>()
    } else {
        (*(*tv1).vval.v_partial).pt_dict
    };
    let mut d2: *mut dict_T = if (*tv2).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        ::core::ptr::null_mut::<dict_T>()
    } else {
        (*(*tv2).vval.v_partial).pt_dict
    };
    if d1.is_null() || d2.is_null() {
        if d1 != d2 {
            return false_0 != 0;
        }
    } else if !tv_dict_equal(d1, d2, ic) {
        return false_0 != 0;
    }
    let mut a1: c_int = if (*tv1).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        0 as c_int
    } else {
        (*(*tv1).vval.v_partial).pt_argc
    };
    let mut a2: c_int = if (*tv2).v_type as c_uint == VAR_FUNC as c_int as c_uint {
        0 as c_int
    } else {
        (*(*tv2).vval.v_partial).pt_argc
    };
    if a1 != a2 {
        return false_0 != 0;
    }
    let mut i: c_int = 0 as c_int;
    while i < a1 {
        if !tv_equal(
            (*(*tv1).vval.v_partial).pt_argv.offset(i as isize),
            (*(*tv2).vval.v_partial).pt_argv.offset(i as isize),
            ic,
        ) {
            return false_0 != 0;
        }
        i += 1;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn get_copyID() -> c_int {
    static current_copyID: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    (*current_copyID.ptr()) += COPYID_INC;
    return current_copyID.get();
}
pub unsafe extern "C" fn garbage_collect(mut testing: bool) -> bool {
    let mut abort_0: bool = false_0 != 0;
    if !testing {
        want_garbage_collect.set(false_0 != 0);
        may_garbage_collect.set(false_0 != 0);
        garbage_collect_at_exit.set(false_0 != 0);
    }
    if (*exestack.ptr()).ga_maxlen - (*exestack.ptr()).ga_len > 500 as c_int {
        let mut n: c_int = (*exestack.ptr()).ga_len / 2 as c_int;
        if n < (*exestack.ptr()).ga_growsize {
            n = (*exestack.ptr()).ga_growsize;
        }
        if (*exestack.ptr()).ga_len + n < (*exestack.ptr()).ga_maxlen {
            let mut new_len: size_t = ((*exestack.ptr()).ga_itemsize as size_t)
                .wrapping_mul(((*exestack.ptr()).ga_len + n) as size_t);
            let mut pp: *mut c_char = xrealloc((*exestack.ptr()).ga_data, new_len) as *mut c_char;
            (*exestack.ptr()).ga_maxlen = (*exestack.ptr()).ga_len + n;
            (*exestack.ptr()).ga_data = pp as *mut c_void;
        }
    }
    let copyID: c_int = get_copyID();
    abort_0 = abort_0 as c_int != 0 || set_ref_in_previous_funccal(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || garbage_collect_scriptvars(copyID) as c_int != 0;
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_item(
                &raw mut (*buf).b_bufvar.di_tv,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_prompt_callback,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_prompt_interrupt,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_cfu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_ofu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_tsrfu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_tfu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_ffu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        if !abort_0 && !(*buf).b_p_cpt_cb.is_null() {
            abort_0 = abort_0 as c_int != 0
                || set_ref_in_cpt_callbacks((*buf).b_p_cpt_cb, (*buf).b_p_cpt_count, copyID)
                    as c_int
                    != 0;
        }
        buf = (*buf).b_next;
    }
    abort_0 = abort_0 as c_int != 0 || set_ref_in_insexpand_funcs(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_opfunc(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_tagfunc(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_findfunc(copyID) as c_int != 0;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            abort_0 = abort_0 as c_int != 0
                || set_ref_in_item(
                    &raw mut (*wp).w_winvar.di_tv,
                    copyID,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as c_int
                    != 0;
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    let mut i: c_int = 0 as c_int;
    while i < (*aucmd_win_vec.ptr()).size as c_int {
        if !(*(*aucmd_win_vec.ptr()).items.offset(i as isize))
            .auc_win
            .is_null()
        {
            abort_0 = abort_0 as c_int != 0
                || set_ref_in_item(
                    &raw mut (*(*(*aucmd_win_vec.ptr()).items.offset(i as isize)).auc_win)
                        .w_winvar
                        .di_tv,
                    copyID,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as c_int
                    != 0;
        }
        i += 1;
    }
    let mut reg_iter: *const c_void = ::core::ptr::null::<c_void>();
    loop {
        let mut reg: yankreg_T = yankreg_T {
            y_array: ::core::ptr::null_mut::<String_0>(),
            y_size: 0,
            y_type: kMTCharWise,
            y_width: 0,
            timestamp: 0,
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        };
        let mut name: c_char = NUL as c_char;
        let mut is_unnamed: bool = false_0 != 0;
        reg_iter = op_global_reg_iter(reg_iter, &raw mut name, &raw mut reg, &raw mut is_unnamed);
        if reg_iter.is_null() {
            break;
        }
    }
    let mut mark_iter: *const c_void = ::core::ptr::null::<c_void>();
    loop {
        let mut fm: xfmark_T = xfmark_T {
            fmark: fmark_T {
                mark: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                fnum: 0,
                timestamp: 0,
                view: fmarkv_T {
                    topline_offset: 0,
                    skipcol: 0,
                },
                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
            },
            fname: ::core::ptr::null_mut::<c_char>(),
        };
        let mut name_0: c_char = NUL as c_char;
        mark_iter = mark_global_iter(mark_iter, &raw mut name_0, &raw mut fm);
        if mark_iter.is_null() {
            break;
        }
    }
    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp_0.is_null() {
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_item(
                &raw mut (*tp_0).tp_winvar.di_tv,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    abort_0 = abort_0 as c_int != 0 || garbage_collect_globvars(copyID) != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_call_stack(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_functions(copyID) as c_int != 0;
    let mut data: *mut Channel = ::core::ptr::null_mut::<Channel>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*channels.ptr()).set.h.n_keys {
        data = *(*channels.ptr()).values.offset(__i as isize) as *mut Channel;
        set_ref_in_callback_reader(
            &raw mut (*data).on_data,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        set_ref_in_callback_reader(
            &raw mut (*data).on_stderr,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        set_ref_in_callback(
            &raw mut (*data).on_exit,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        __i = __i.wrapping_add(1);
    }
    let mut timer: *mut timer_T = ::core::ptr::null_mut::<timer_T>();
    let mut __i_0: uint32_t = 0;
    __i_0 = 0 as uint32_t;
    while __i_0 < (*timers.ptr()).set.h.n_keys {
        timer = *(*timers.ptr()).values.offset(__i_0 as isize) as *mut timer_T;
        set_ref_in_callback(
            &raw mut (*timer).callback,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        __i_0 = __i_0.wrapping_add(1);
    }
    abort_0 = abort_0 as c_int != 0 || set_ref_in_func_args(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || garbage_collect_vimvars(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_quickfix(copyID) as c_int != 0;
    let mut did_free: bool = false_0 != 0;
    if !abort_0 {
        did_free = free_unref_items(copyID) != 0;
        did_free =
            free_unref_funccal(copyID, testing as c_int) as c_int != 0 || did_free as c_int != 0;
    } else if p_verbose.get() > 0 as OptInt {
        verb_msg(gettext(
            b"Not enough memory to set references, garbage collection aborted!\0".as_ptr()
                as *const c_char,
        ));
    }
    return did_free;
}
unsafe extern "C" fn free_unref_items(mut copyID: c_int) -> c_int {
    let mut did_free: bool = false_0 != 0;
    tv_in_free_unref_items.set(true_0 != 0);
    let mut dd: *mut dict_T = gc_first_dict.get();
    while !dd.is_null() {
        if (*dd).dv_copyID & COPYID_MASK != copyID & COPYID_MASK {
            tv_dict_free_contents(dd);
            did_free = true_0 != 0;
        }
        dd = (*dd).dv_used_next;
    }
    let mut ll: *mut list_T = gc_first_list.get();
    while !ll.is_null() {
        if tv_list_copyid(ll) & COPYID_MASK != copyID & COPYID_MASK && !tv_list_has_watchers(ll) {
            tv_list_free_contents(ll);
            did_free = true_0 != 0;
        }
        ll = (*ll).lv_used_next;
    }
    let mut dd_next: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut dd_0: *mut dict_T = gc_first_dict.get();
    while !dd_0.is_null() {
        dd_next = (*dd_0).dv_used_next;
        if (*dd_0).dv_copyID & COPYID_MASK != copyID & COPYID_MASK {
            tv_dict_free_dict(dd_0);
        }
        dd_0 = dd_next;
    }
    let mut ll_next: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut ll_0: *mut list_T = gc_first_list.get();
    while !ll_0.is_null() {
        ll_next = (*ll_0).lv_used_next;
        if (*ll_0).lv_copyID & COPYID_MASK != copyID & COPYID_MASK && !tv_list_has_watchers(ll_0) {
            tv_list_free_list(ll_0);
        }
        ll_0 = ll_next;
    }
    tv_in_free_unref_items.set(false_0 != 0);
    return did_free as c_int;
}
pub unsafe extern "C" fn set_ref_in_ht(
    mut ht: *mut hashtab_T,
    mut copyID: c_int,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    let mut ht_stack: *mut ht_stack_T = ::core::ptr::null_mut::<ht_stack_T>();
    let mut cur_ht: *mut hashtab_T = ht;
    loop {
        if !abort_0 {
            let hiht_: *mut hashtab_T = cur_ht;
            let mut hitodo_: size_t = (*hiht_).ht_used;
            let mut hi: *mut hashitem_T = (*hiht_).ht_array;
            while hitodo_ != 0 {
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut c_char)
                {
                    hitodo_ = hitodo_.wrapping_sub(1);
                    abort_0 = abort_0 as c_int != 0
                        || set_ref_in_item(
                            &raw mut (*((*hi).hi_key.offset(-(17 as c_ulong as isize))
                                as *mut dictitem_T))
                                .di_tv,
                            copyID,
                            &raw mut ht_stack,
                            list_stack,
                        ) as c_int
                            != 0;
                }
                hi = hi.offset(1);
            }
        }
        if ht_stack.is_null() {
            break;
        }
        cur_ht = (*ht_stack).ht;
        let mut tempitem: *mut ht_stack_T = ht_stack;
        ht_stack = (*ht_stack).prev as *mut ht_stack_T;
        xfree(tempitem as *mut c_void);
    }
    return abort_0;
}
pub unsafe extern "C" fn set_ref_in_list_items(
    mut l: *mut list_T,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    let mut list_stack: *mut list_stack_T = ::core::ptr::null_mut::<list_stack_T>();
    let mut cur_l: *mut list_T = l;
    loop {
        let l_: *mut list_T = cur_l;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if abort_0 {
                    break;
                }
                abort_0 =
                    set_ref_in_item(&raw mut (*li).li_tv, copyID, ht_stack, &raw mut list_stack);
                li = (*li).li_next;
            }
        }
        if list_stack.is_null() {
            break;
        }
        cur_l = (*list_stack).list;
        let mut tempitem: *mut list_stack_T = list_stack;
        list_stack = (*list_stack).prev as *mut list_stack_T;
        xfree(tempitem as *mut c_void);
    }
    return abort_0;
}
unsafe extern "C" fn set_ref_in_item_dict(
    mut dd: *mut dict_T,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if dd.is_null() || (*dd).dv_copyID == copyID {
        return false_0 != 0;
    }
    (*dd).dv_copyID = copyID;
    if ht_stack.is_null() {
        return set_ref_in_ht(&raw mut (*dd).dv_hashtab, copyID, list_stack);
    }
    let newitem: *mut ht_stack_T = xmalloc(::core::mem::size_of::<ht_stack_T>()) as *mut ht_stack_T;
    (*newitem).ht = &raw mut (*dd).dv_hashtab;
    (*newitem).prev = *ht_stack as *mut ht_stack_S;
    *ht_stack = newitem;
    let mut w: *mut QUEUE = ::core::ptr::null_mut::<QUEUE>();
    let mut watcher: *mut DictWatcher = ::core::ptr::null_mut::<DictWatcher>();
    w = (*dd).watchers.next as *mut QUEUE;
    while w != &raw mut (*dd).watchers {
        let mut next: *mut QUEUE = (*w).next as *mut QUEUE;
        watcher = tv_dict_watcher_node_data(w);
        set_ref_in_callback(&raw mut (*watcher).callback, copyID, ht_stack, list_stack);
        w = next;
    }
    return false_0 != 0;
}
unsafe extern "C" fn set_ref_in_item_list(
    mut ll: *mut list_T,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if ll.is_null() || (*ll).lv_copyID == copyID {
        return false_0 != 0;
    }
    (*ll).lv_copyID = copyID;
    if list_stack.is_null() {
        return set_ref_in_list_items(ll, copyID, ht_stack);
    }
    let newitem: *mut list_stack_T =
        xmalloc(::core::mem::size_of::<list_stack_T>()) as *mut list_stack_T;
    (*newitem).list = ll;
    (*newitem).prev = *list_stack as *mut list_stack_S;
    *list_stack = newitem;
    return false_0 != 0;
}
unsafe extern "C" fn set_ref_in_item_partial(
    mut pt: *mut partial_T,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if pt.is_null() || (*pt).pt_copyID == copyID {
        return false_0 != 0;
    }
    (*pt).pt_copyID = copyID;
    let mut abort_0: bool = set_ref_in_func((*pt).pt_name, (*pt).pt_func, copyID);
    if !(*pt).pt_dict.is_null() {
        let mut dtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        dtv.v_type = VAR_DICT;
        dtv.vval.v_dict = (*pt).pt_dict;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_item(&raw mut dtv, copyID, ht_stack, list_stack) as c_int != 0;
    }
    let mut i: c_int = 0 as c_int;
    while i < (*pt).pt_argc {
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_item(
                (*pt).pt_argv.offset(i as isize),
                copyID,
                ht_stack,
                list_stack,
            ) as c_int
                != 0;
        i += 1;
    }
    return abort_0;
}
pub unsafe extern "C" fn set_ref_in_item(
    mut tv: *mut typval_T,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    match (*tv).v_type as c_uint {
        5 => return set_ref_in_item_dict((*tv).vval.v_dict, copyID, ht_stack, list_stack),
        4 => return set_ref_in_item_list((*tv).vval.v_list, copyID, ht_stack, list_stack),
        3 => {
            abort_0 = set_ref_in_func(
                (*tv).vval.v_string,
                ::core::ptr::null_mut::<ufunc_T>(),
                copyID,
            );
        }
        9 => {
            return set_ref_in_item_partial((*tv).vval.v_partial, copyID, ht_stack, list_stack);
        }
        0 | 7 | 8 | 6 | 1 | 2 | 10 | _ => {}
    }
    return abort_0;
}
unsafe extern "C" fn get_literal_key(mut arg: *mut *mut c_char, mut tv: *mut typval_T) -> c_int {
    let mut p: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if !(**arg as c_uint >= 'A' as c_uint && **arg as c_uint <= 'Z' as c_uint
        || **arg as c_uint >= 'a' as c_uint && **arg as c_uint <= 'z' as c_uint
        || ascii_isdigit(**arg as c_int) as c_int != 0)
        && **arg as c_int != '_' as c_int
        && **arg as c_int != '-' as c_int
    {
        return FAIL;
    }
    p = *arg;
    while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
        || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
        || ascii_isdigit(*p as c_int) as c_int != 0
        || *p as c_int == '_' as c_int
        || *p as c_int == '-' as c_int
    {
        p = p.offset(1);
    }
    (*tv).v_type = VAR_STRING;
    (*tv).vval.v_string =
        xmemdupz(*arg as *const c_void, p.offset_from(*arg) as size_t) as *mut c_char;
    *arg = skipwhite(p);
    return OK;
}
unsafe extern "C" fn eval_dict(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut literal: bool,
) -> c_int {
    let evaluate: bool = if evalarg.is_null() {
        false_0
    } else {
        (*evalarg).eval_flags & EVAL_EVALUATE as c_int
    } != 0;
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut key: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut curly_expr: *mut c_char = skipwhite((*arg).offset(1 as c_int as isize));
    let mut buf: [c_char; 65] = [0; 65];
    if *curly_expr as c_int != '}' as c_int
        && !literal
        && eval1(
            &raw mut curly_expr,
            &raw mut tv,
            ::core::ptr::null_mut::<evalarg_T>(),
        ) == OK
        && *skipwhite(curly_expr) as c_int == '}' as c_int
    {
        return NOTDONE;
    }
    let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    if evaluate {
        d = tv_dict_alloc();
    }
    let mut tvkey: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    tvkey.v_type = VAR_UNKNOWN;
    tv.v_type = VAR_UNKNOWN;
    *arg = skipwhite((*arg).offset(1 as c_int as isize));
    '_failret: {
        while **arg as c_int != '}' as c_int && **arg as c_int != NUL {
            if (if literal as c_int != 0 {
                get_literal_key(arg, &raw mut tvkey)
            } else {
                eval1(arg, &raw mut tvkey, evalarg)
            }) == FAIL
            {
                break '_failret;
            }
            if **arg as c_int != ':' as c_int {
                semsg(
                    gettext(b"E720: Missing colon in Dictionary: %s\0".as_ptr() as *const c_char),
                    *arg,
                );
                tv_clear(&raw mut tvkey);
                break '_failret;
            } else {
                if evaluate {
                    key = tv_get_string_buf_chk(&raw mut tvkey, &raw mut buf as *mut c_char)
                        as *mut c_char;
                    if key.is_null() {
                        tv_clear(&raw mut tvkey);
                        break '_failret;
                    }
                }
                *arg = skipwhite((*arg).offset(1 as c_int as isize));
                if eval1(arg, &raw mut tv, evalarg) == FAIL {
                    tv_clear(&raw mut tvkey);
                    break '_failret;
                } else {
                    if evaluate {
                        let mut item: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
                        if !item.is_null() {
                            semsg(
                                gettext(b"E721: Duplicate key in Dictionary: \"%s\"\0".as_ptr()
                                    as *const c_char),
                                key,
                            );
                            tv_clear(&raw mut tvkey);
                            tv_clear(&raw mut tv);
                            break '_failret;
                        } else {
                            item = tv_dict_item_alloc(key);
                            (*item).di_tv = tv;
                            (*item).di_tv.v_lock = VAR_UNLOCKED;
                            if tv_dict_add(d, item) == FAIL {
                                tv_dict_item_free(item);
                            }
                        }
                    }
                    tv_clear(&raw mut tvkey);
                    let mut had_comma: bool = **arg as c_int == ',' as c_int;
                    if had_comma {
                        *arg = skipwhite((*arg).offset(1 as c_int as isize));
                    }
                    if **arg as c_int == '}' as c_int {
                        break;
                    }
                    if had_comma {
                        continue;
                    }
                    semsg(
                        gettext(
                            b"E722: Missing comma in Dictionary: %s\0".as_ptr() as *const c_char
                        ),
                        *arg,
                    );
                    break '_failret;
                }
            }
        }
        if **arg as c_int != '}' as c_int {
            semsg(
                gettext(b"E723: Missing end of Dictionary '}': %s\0".as_ptr() as *const c_char),
                *arg,
            );
        } else {
            *arg = skipwhite((*arg).offset(1 as c_int as isize));
            if evaluate {
                tv_dict_set_ret(rettv, d);
            }
            return OK;
        }
    }
    if !d.is_null() {
        tv_dict_free(d);
    }
    return FAIL;
}
unsafe extern "C" fn eval_lit_dict(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    let mut ret: c_int = OK;
    if *(*arg).offset(1 as c_int as isize) as c_int == '{' as c_int {
        *arg = (*arg).offset(1);
        ret = eval_dict(arg, rettv, evalarg, true_0 != 0);
    } else {
        ret = NOTDONE;
    }
    return ret;
}
pub unsafe extern "C" fn string2float(text: *const c_char, ret_value: *mut float_T) -> size_t {
    if strncasecmp(
        text as *mut c_char,
        b"inf\0".as_ptr() as *const c_char as *mut c_char,
        3 as c_int as size_t,
    ) == 0 as c_int
    {
        *ret_value = ::core::f32::INFINITY as float_T;
        return 3 as size_t;
    }
    if strncasecmp(
        text as *mut c_char,
        b"-inf\0".as_ptr() as *const c_char as *mut c_char,
        4 as c_int as size_t,
    ) == 0 as c_int
    {
        *ret_value = -::core::f32::INFINITY as float_T;
        return 4 as size_t;
    }
    if strncasecmp(
        text as *mut c_char,
        b"nan\0".as_ptr() as *const c_char as *mut c_char,
        3 as c_int as size_t,
    ) == 0 as c_int
    {
        *ret_value = ::core::f32::NAN as float_T;
        return 3 as size_t;
    }
    let mut s: *mut c_char = ::core::ptr::null_mut::<c_char>();
    *ret_value = strtod(text, &raw mut s) as float_T;
    return s.offset_from(text) as size_t;
}
unsafe extern "C" fn eval_env_var(
    mut arg: *mut *mut c_char,
    mut rettv: *mut typval_T,
    mut evaluate: c_int,
) -> c_int {
    *arg = (*arg).offset(1);
    let mut name: *mut c_char = *arg;
    let mut len: c_int = get_env_len(arg as *mut *const c_char);
    if evaluate != 0 {
        if len == 0 as c_int {
            return FAIL;
        }
        let mut cc: c_int = *name.offset(len as isize) as c_int;
        *name.offset(len as isize) = NUL as c_char;
        let mut string: *mut c_char = vim_getenv(name);
        if string.is_null() || *string as c_int == NUL {
            xfree(string as *mut c_void);
            string = expand_env_save(name.offset(-(1 as c_int as isize)));
            if !string.is_null() && *string as c_int == '$' as c_int {
                let mut ptr_: *mut *mut c_void = &raw mut string as *mut *mut c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                let _ = *ptr_;
            }
        }
        *name.offset(len as isize) = cc as c_char;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = string;
        (*rettv).v_lock = VAR_UNLOCKED;
    }
    return OK;
}
pub unsafe extern "C" fn tv_to_argv(
    mut cmd_tv: *mut typval_T,
    mut cmd: *mut *const c_char,
    mut executable: *mut bool,
) -> *mut *mut c_char {
    if (*cmd_tv).v_type as c_uint == VAR_STRING as c_int as c_uint {
        let mut cmd_str: *const c_char = tv_get_string(cmd_tv);
        if !cmd.is_null() {
            *cmd = cmd_str;
        }
        return shell_build_argv(cmd_str, ::core::ptr::null::<c_char>());
    }
    if (*cmd_tv).v_type as c_uint != VAR_LIST as c_int as c_uint {
        semsg(
            gettext(&raw const e_invarg2 as *const c_char),
            b"expected String or List\0".as_ptr() as *const c_char,
        );
        return ::core::ptr::null_mut::<*mut c_char>();
    }
    let mut argl: *mut list_T = (*cmd_tv).vval.v_list;
    let mut argc: c_int = tv_list_len(argl);
    if argc == 0 {
        emsg(gettext(&raw const e_invarg as *const c_char));
        return ::core::ptr::null_mut::<*mut c_char>();
    }
    let mut arg0: *const c_char = tv_get_string_chk(&raw mut (*tv_list_first(argl)).li_tv);
    let mut exe_resolved: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if arg0.is_null() || !os_can_exe(arg0, &raw mut exe_resolved, true_0 != 0) {
        if !arg0.is_null() && !executable.is_null() {
            let mut buf: [c_char; 1025] = [0; 1025];
            snprintf(
                &raw mut buf as *mut c_char,
                ::core::mem::size_of::<[c_char; 1025]>(),
                b"'%s' is not executable\0".as_ptr() as *const c_char,
                arg0,
            );
            semsg(
                gettext(&raw const e_invargNval as *const c_char),
                b"cmd\0".as_ptr() as *const c_char,
                &raw mut buf as *mut c_char,
            );
            *executable = false_0 != 0;
        }
        return ::core::ptr::null_mut::<*mut c_char>();
    }
    if !cmd.is_null() {
        *cmd = exe_resolved;
    }
    let mut i: c_int = 0 as c_int;
    let mut argv: *mut *mut c_char = xcalloc(
        (argc as size_t).wrapping_add(1 as size_t),
        ::core::mem::size_of::<*mut c_char>(),
    ) as *mut *mut c_char;
    let l_: *const list_T = argl;
    if !l_.is_null() {
        let mut arg: *const listitem_T = (*l_).lv_first;
        while !arg.is_null() {
            let mut a: *const c_char = tv_get_string_chk(&raw const (*arg).li_tv);
            if a.is_null() {
                shell_free_argv(argv);
                xfree(exe_resolved as *mut c_void);
                return ::core::ptr::null_mut::<*mut c_char>();
            }
            let c2rust_fresh11 = i;
            i = i + 1;
            let c2rust_lvalue_ptr = &raw mut *argv.offset(c2rust_fresh11 as isize);
            *c2rust_lvalue_ptr = xstrdup(a);
            arg = (*arg).li_next;
        }
    }
    xfree(*argv.offset(0 as c_int as isize) as *mut c_void);
    *argv.offset(0 as c_int as isize) = exe_resolved;
    return argv;
}
unsafe extern "C" fn string_to_list(
    mut str: *const c_char,
    mut len: size_t,
    keepempty: bool,
) -> *mut list_T {
    if !keepempty && *str.offset(len.wrapping_sub(1 as size_t) as isize) as c_int == NL {
        len = len.wrapping_sub(1);
    }
    let list: *mut list_T = tv_list_alloc(kListLenMayKnow as c_int as ptrdiff_t);
    encode_list_write(list as *mut c_void, str, len);
    return list;
}
unsafe extern "C" fn get_system_output_as_rettv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut retlist: bool,
) {
    let mut wait_time: proftime_T = 0;
    let mut profiling: bool = do_profiling.get() == PROF_YES;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<c_char>();
    if check_secure() {
        return;
    }
    let mut input_len: ptrdiff_t = 0;
    let mut input: *mut c_char = save_tv_as_string(
        argvars.offset(1 as c_int as isize),
        &raw mut input_len,
        false_0 != 0,
        false_0 != 0,
    );
    if input_len < 0 as ptrdiff_t {
        '_c2rust_label: {
            if input.is_null() {
            } else {
                __assert_fail(
                    b"input == NULL\0".as_ptr() as *const c_char,
                    b"src/nvim/eval.rs\0".as_ptr() as *const c_char,
                    4731 as c_uint,
                    b"void get_system_output_as_rettv(typval_T *, typval_T *, _Bool)\0".as_ptr()
                        as *const c_char,
                );
            }
        };
        return;
    }
    let mut executable: bool = true_0 != 0;
    let mut argv: *mut *mut c_char = tv_to_argv(
        argvars.offset(0 as c_int as isize),
        ::core::ptr::null_mut::<*const c_char>(),
        &raw mut executable,
    );
    if argv.is_null() {
        if !executable {
            set_vim_var_nr(VV_SHELL_ERROR, -1 as varnumber_T);
        }
        xfree(input as *mut c_void);
        return;
    }
    if p_verbose.get() > 3 as OptInt {
        let mut cmdstr: *mut c_char = shell_argv_to_str(argv);
        verbose_enter_scroll();
        smsg(
            0 as c_int,
            gettext(b"Executing command: \"%s\"\0".as_ptr() as *const c_char),
            cmdstr,
        );
        msg_puts(b"\n\n\0".as_ptr() as *const c_char);
        verbose_leave_scroll();
        xfree(cmdstr as *mut c_void);
    }
    if profiling {
        wait_time = prof_child_enter();
    }
    let mut nread: size_t = 0 as size_t;
    let mut res: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut status: c_int = os_system(
        argv,
        input,
        input_len as size_t,
        &raw mut res,
        &raw mut nread,
    );
    if profiling {
        prof_child_exit(wait_time);
    }
    xfree(input as *mut c_void);
    set_vim_var_nr(VV_SHELL_ERROR, status as varnumber_T);
    if res.is_null() {
        if retlist {
            tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
        } else {
            (*rettv).vval.v_string = xstrdup(b"\0".as_ptr() as *const c_char);
        }
        return;
    }
    if retlist {
        let mut keepempty: c_int = 0 as c_int;
        if (*argvars.offset(1 as c_int as isize)).v_type as c_uint != VAR_UNKNOWN as c_int as c_uint
            && (*argvars.offset(2 as c_int as isize)).v_type as c_uint
                != VAR_UNKNOWN as c_int as c_uint
        {
            keepempty = tv_get_number(argvars.offset(2 as c_int as isize)) as c_int;
        }
        (*rettv).vval.v_list = string_to_list(res, nread, keepempty != 0);
        tv_list_ref((*rettv).vval.v_list);
        (*rettv).v_type = VAR_LIST;
        xfree(res as *mut c_void);
    } else {
        memchrsub(res as *mut c_void, NUL as c_char, 1 as c_char, nread);
        (*rettv).vval.v_string = res;
    };
}
pub unsafe extern "C" fn f_system(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_system_output_as_rettv(argvars, rettv, false_0 != 0);
}
pub unsafe extern "C" fn f_systemlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_system_output_as_rettv(argvars, rettv, true_0 != 0);
}
pub unsafe extern "C" fn callback_from_typval(
    callback: *mut Callback,
    arg: *const typval_T,
) -> bool {
    let mut r: c_int = OK;
    if (*arg).v_type as c_uint == VAR_PARTIAL as c_int as c_uint && !(*arg).vval.v_partial.is_null()
    {
        (*callback).data.partial = (*arg).vval.v_partial;
        (*(*callback).data.partial).pt_refcount += 1;
        (*callback).type_0 = kCallbackPartial;
    } else if (*arg).v_type as c_uint == VAR_STRING as c_int as c_uint
        && !(*arg).vval.v_string.is_null()
        && ascii_isdigit(*(*arg).vval.v_string as c_int) as c_int != 0
    {
        r = FAIL;
    } else if (*arg).v_type as c_uint == VAR_FUNC as c_int as c_uint
        || (*arg).v_type as c_uint == VAR_STRING as c_int as c_uint
    {
        let mut name: *mut c_char = (*arg).vval.v_string;
        if name.is_null() {
            r = FAIL;
        } else if *name as c_int == NUL {
            (*callback).type_0 = kCallbackNone;
            (*callback).data.funcref = ::core::ptr::null_mut::<c_char>();
        } else {
            (*callback).data.funcref = ::core::ptr::null_mut::<c_char>();
            if (*arg).v_type as c_uint == VAR_STRING as c_int as c_uint {
                (*callback).data.funcref = get_scriptlocal_funcname(name);
            }
            if (*callback).data.funcref.is_null() {
                (*callback).data.funcref = xstrdup(name);
            }
            func_ref((*callback).data.funcref);
            (*callback).type_0 = kCallbackFuncref;
        }
    } else if nlua_is_table_from_lua(arg) {
        let mut name_0: *mut c_char = nlua_register_table_as_callable(arg);
        if !name_0.is_null() {
            (*callback).data.funcref = xstrdup(name_0);
            (*callback).type_0 = kCallbackFuncref;
        } else {
            r = FAIL;
        }
    } else if (*arg).v_type as c_uint == VAR_SPECIAL as c_int as c_uint
        || (*arg).v_type as c_uint == VAR_NUMBER as c_int as c_uint
            && (*arg).vval.v_number == 0 as varnumber_T
    {
        (*callback).type_0 = kCallbackNone;
        (*callback).data.funcref = ::core::ptr::null_mut::<c_char>();
    } else {
        r = FAIL;
    }
    if r == FAIL {
        emsg(gettext(
            b"E921: Invalid callback argument\0".as_ptr() as *const c_char
        ));
        return false_0 != 0;
    }
    return true_0 != 0;
}
static callback_depth: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub unsafe extern "C" fn get_callback_depth() -> c_int {
    return callback_depth.get();
}
pub unsafe extern "C" fn callback_call(
    callback: *mut Callback,
    argcount_in: c_int,
    argvars_in: *mut typval_T,
    rettv: *mut typval_T,
) -> bool {
    if callback_depth.get() as OptInt > p_mfd.get() {
        emsg(gettext(&raw const e_command_too_recursive as *const c_char));
        return false_0 != 0;
    }
    let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut name: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut args: Array = ARRAY_DICT_INIT;
    let mut rv: Object = Object {
        type_0: kObjectTypeNil,
        data: object_data { boolean: false },
    };
    let mut len: c_int = 0;
    match (*callback).type_0 as c_uint {
        1 => {
            name = (*callback).data.funcref;
            len = strlen(name) as c_int;
            if len >= 6 as c_int
                && memcmp(
                    name as *const c_void,
                    b"v:lua.\0".as_ptr() as *const c_char as *const c_void,
                    6 as size_t,
                ) == 0
            {
                name = name.offset(6 as c_int as isize);
                len = check_luafunc_name(name, false_0 != 0);
                if len == 0 as c_int {
                    return false_0 != 0;
                }
                partial = get_vim_var_partial(VV_LUA);
            } else {
                partial = ::core::ptr::null_mut::<partial_T>();
            }
        }
        2 => {
            partial = (*callback).data.partial;
            name = partial_name(partial);
        }
        3 => {
            rv = nlua_call_ref(
                (*callback).data.luaref,
                ::core::ptr::null::<c_char>(),
                args,
                kRetNilBool,
                ::core::ptr::null_mut::<Arena>(),
                ::core::ptr::null_mut::<Error>(),
            );
            return rv.type_0 as c_uint == kObjectTypeBoolean as c_int as c_uint
                && rv.data.boolean as c_int == true_0;
        }
        0 => return false_0 != 0,
        _ => {}
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_evaluate = true_0 != 0;
    funcexe.fe_partial = partial;
    (*callback_depth.ptr()) += 1;
    let mut ret: c_int = call_func(
        name,
        -1 as c_int,
        rettv,
        argcount_in,
        argvars_in,
        &raw mut funcexe,
    );
    (*callback_depth.ptr()) -= 1;
    return ret != 0;
}
pub unsafe extern "C" fn set_ref_in_callback(
    mut callback: *mut Callback,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    match (*callback).type_0 as c_uint {
        2 => {
            tv.v_type = VAR_PARTIAL;
            tv.vval.v_partial = (*callback).data.partial;
            return set_ref_in_item(&raw mut tv, copyID, ht_stack, list_stack);
        }
        3 => {
            abort();
        }
        1 | 0 | _ => {}
    }
    return false_0 != 0;
}
unsafe extern "C" fn set_ref_in_callback_reader(
    mut reader: *mut CallbackReader,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if set_ref_in_callback(&raw mut (*reader).cb, copyID, ht_stack, list_stack) {
        return true_0 != 0;
    }
    if !(*reader).self_0.is_null() {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv.v_type = VAR_DICT;
        tv.vval.v_dict = (*reader).self_0;
        return set_ref_in_item(&raw mut tv, copyID, ht_stack, list_stack);
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn find_timer_by_nr(mut xx: varnumber_T) -> *mut timer_T {
    return map_get_uint64_t_ptr_t(timers.ptr(), xx as uint64_t) as *mut timer_T;
}
pub unsafe extern "C" fn add_timer_info(mut rettv: *mut typval_T, mut timer: *mut timer_T) {
    let mut list: *mut list_T = (*rettv).vval.v_list;
    let mut dict: *mut dict_T = tv_dict_alloc();
    tv_list_append_dict(list, dict);
    tv_dict_add_nr(
        dict,
        b"id\0".as_ptr() as *const c_char,
        ::core::mem::size_of::<[c_char; 3]>().wrapping_sub(1 as size_t),
        (*timer).timer_id as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"time\0".as_ptr() as *const c_char,
        ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as size_t),
        (*timer).timeout as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"paused\0".as_ptr() as *const c_char,
        ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as size_t),
        (*timer).paused as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"repeat\0".as_ptr() as *const c_char,
        ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as size_t),
        (if (*timer).repeat_count < 0 as c_int {
            -1 as c_int
        } else {
            (*timer).repeat_count
        }) as varnumber_T,
    );
    let mut di: *mut dictitem_T = tv_dict_item_alloc(b"callback\0".as_ptr() as *const c_char);
    if tv_dict_add(dict, di) == FAIL {
        xfree(di as *mut c_void);
        return;
    }
    callback_put(&raw mut (*timer).callback, &raw mut (*di).di_tv);
}
pub unsafe extern "C" fn add_timer_info_all(mut rettv: *mut typval_T) {
    tv_list_alloc_ret(rettv, (*timers.ptr()).set.h.size as ptrdiff_t);
    let mut timer: *mut timer_T = ::core::ptr::null_mut::<timer_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*timers.ptr()).set.h.n_keys {
        timer = *(*timers.ptr()).values.offset(__i as isize) as *mut timer_T;
        if !(*timer).stopped || (*timer).refcount > 1 as c_int {
            add_timer_info(rettv, timer);
        }
        __i = __i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn timer_due_cb(mut _tw: *mut TimeWatcher, mut data: *mut c_void) {
    let mut timer: *mut timer_T = data as *mut timer_T;
    let mut save_did_emsg: c_int = did_emsg.get();
    let called_emsg_before: c_int = called_emsg.get();
    let save_ex_pressedreturn: bool = get_pressedreturn();
    if (*timer).stopped as c_int != 0 || (*timer).paused as c_int != 0 {
        return;
    }
    (*timer).refcount += 1;
    if (*timer).repeat_count >= 0 as c_int && {
        (*timer).repeat_count -= 1;
        (*timer).repeat_count == 0 as c_int
    } {
        timer_stop(timer);
    }
    let mut argv: [typval_T; 2] = [
        typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        },
        typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        },
    ];
    argv[0 as c_int as usize].v_type = VAR_NUMBER;
    argv[0 as c_int as usize].vval.v_number = (*timer).timer_id as varnumber_T;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    callback_call(
        &raw mut (*timer).callback,
        1 as c_int,
        &raw mut argv as *mut typval_T,
        &raw mut rettv,
    );
    if called_emsg.get() > called_emsg_before && did_emsg.get() != 0 {
        (*timer).emsg_count += 1;
        if did_throw.get() {
            discard_current_exception();
        }
    }
    did_emsg.set(save_did_emsg);
    set_pressedreturn(save_ex_pressedreturn);
    if (*timer).emsg_count >= 3 as c_int {
        timer_stop(timer);
    }
    tv_clear(&raw mut rettv);
    if !(*timer).stopped && (*timer).timeout == 0 as int64_t {
        time_watcher_start(
            &raw mut (*timer).tw,
            Some(timer_due_cb as unsafe extern "C" fn(*mut TimeWatcher, *mut c_void) -> ()),
            0 as uint64_t,
            0 as uint64_t,
        );
    }
    timer_decref(timer);
}
pub unsafe extern "C" fn timer_start(
    timeout: int64_t,
    repeat_count: c_int,
    callback: *const Callback,
) -> uint64_t {
    let mut timer: *mut timer_T = xmalloc(::core::mem::size_of::<timer_T>()) as *mut timer_T;
    (*timer).refcount = 1 as c_int;
    (*timer).stopped = false_0 != 0;
    (*timer).paused = false_0 != 0;
    (*timer).emsg_count = 0 as c_int;
    (*timer).repeat_count = repeat_count;
    (*timer).timeout = timeout;
    let c2rust_fresh17 = last_timer_id.get();
    last_timer_id.set((*last_timer_id.ptr()).wrapping_add(1));
    (*timer).timer_id = c2rust_fresh17 as c_int;
    (*timer).callback = *callback;
    time_watcher_init(main_loop.ptr(), &raw mut (*timer).tw, timer as *mut c_void);
    (*timer).tw.events = multiqueue_new_child((*main_loop.ptr()).events);
    (*timer).tw.blockable = true_0 != 0;
    time_watcher_start(
        &raw mut (*timer).tw,
        Some(timer_due_cb as unsafe extern "C" fn(*mut TimeWatcher, *mut c_void) -> ()),
        timeout as uint64_t,
        timeout as uint64_t,
    );
    map_put_uint64_t_ptr_t(timers.ptr(), (*timer).timer_id as uint64_t, timer as ptr_t);
    return (*timer).timer_id as uint64_t;
}
pub unsafe extern "C" fn timer_stop(mut timer: *mut timer_T) {
    if (*timer).stopped {
        return;
    }
    (*timer).stopped = true_0 != 0;
    time_watcher_stop(&raw mut (*timer).tw);
    time_watcher_close(
        &raw mut (*timer).tw,
        Some(timer_close_cb as unsafe extern "C" fn(*mut TimeWatcher, *mut c_void) -> ()),
    );
}
unsafe extern "C" fn timer_close_cb(mut _tw: *mut TimeWatcher, mut data: *mut c_void) {
    let mut timer: *mut timer_T = data as *mut timer_T;
    multiqueue_free((*timer).tw.events);
    callback_free(&raw mut (*timer).callback);
    map_del_uint64_t_ptr_t(
        timers.ptr(),
        (*timer).timer_id as uint64_t,
        ::core::ptr::null_mut::<uint64_t>(),
    );
    timer_decref(timer);
}
unsafe extern "C" fn timer_decref(mut timer: *mut timer_T) {
    (*timer).refcount -= 1;
    if (*timer).refcount == 0 as c_int {
        xfree(timer as *mut c_void);
    }
}
pub unsafe extern "C" fn timer_stop_all() {
    let mut timer: *mut timer_T = ::core::ptr::null_mut::<timer_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*timers.ptr()).set.h.n_keys {
        timer = *(*timers.ptr()).values.offset(__i as isize) as *mut timer_T;
        timer_stop(timer);
        __i = __i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn timer_teardown() {
    timer_stop_all();
}
pub unsafe extern "C" fn save_tv_as_string(
    mut tv: *mut typval_T,
    len: *mut ptrdiff_t,
    mut endnl: bool,
    mut crlf: bool,
) -> *mut c_char {
    *len = 0 as ptrdiff_t;
    if (*tv).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint {
        return ::core::ptr::null_mut::<c_char>();
    }
    if (*tv).v_type as c_uint != VAR_LIST as c_int as c_uint
        && (*tv).v_type as c_uint != VAR_NUMBER as c_int as c_uint
    {
        let mut ret: *const c_char = tv_get_string_chk(tv);
        if !ret.is_null() {
            *len = strlen(ret) as ptrdiff_t;
            return xmemdupz(ret as *const c_void, *len as size_t) as *mut c_char;
        } else {
            *len = -1 as ptrdiff_t;
            return ::core::ptr::null_mut::<c_char>();
        }
    }
    if (*tv).v_type as c_uint == VAR_NUMBER as c_int as c_uint {
        let mut buf: *mut buf_T = buflist_findnr((*tv).vval.v_number as c_int);
        if !buf.is_null() {
            let mut lnum: linenr_T = 1 as linenr_T;
            while lnum <= (*buf).b_ml.ml_line_count {
                let mut p: *mut c_char = ml_get_buf(buf, lnum);
                while *p as c_int != NUL {
                    *len += 1 as ptrdiff_t;
                    p = p.offset(1);
                }
                *len += 1 as ptrdiff_t;
                lnum += 1;
            }
        } else {
            semsg(
                gettext(&raw const e_nobufnr as *const c_char),
                (*tv).vval.v_number,
            );
            *len = -1 as ptrdiff_t;
            return ::core::ptr::null_mut::<c_char>();
        }
        if *len == 0 as ptrdiff_t {
            return ::core::ptr::null_mut::<c_char>();
        }
        let mut ret_0: *mut c_char =
            xmalloc((*len as size_t).wrapping_add(1 as size_t)) as *mut c_char;
        let mut end: *mut c_char = ret_0;
        let mut lnum_0: linenr_T = 1 as linenr_T;
        while lnum_0 <= (*buf).b_ml.ml_line_count {
            let mut p_0: *mut c_char = ml_get_buf(buf, lnum_0);
            while *p_0 as c_int != NUL {
                let c2rust_fresh12 = end;
                end = end.offset(1);
                *c2rust_fresh12 = (if *p_0 as c_int == '\n' as c_int {
                    NUL
                } else {
                    *p_0 as c_int
                }) as c_char;
                p_0 = p_0.offset(1);
            }
            let c2rust_fresh13 = end;
            end = end.offset(1);
            *c2rust_fresh13 = '\n' as c_char;
            lnum_0 += 1;
        }
        *end = NUL as c_char;
        *len = end.offset_from(ret_0) as ptrdiff_t;
        return ret_0;
    }
    '_c2rust_label: {
        if (*tv).v_type as c_uint == VAR_LIST as c_int as c_uint {
        } else {
            __assert_fail(
                b"tv->v_type == VAR_LIST\0".as_ptr() as *const c_char,
                b"src/nvim/eval.rs\0".as_ptr() as *const c_char,
                5197 as c_uint,
                b"char *save_tv_as_string(typval_T *, ptrdiff_t *const, _Bool, _Bool)\0".as_ptr()
                    as *const c_char,
            );
        }
    };
    let mut list: *mut list_T = (*tv).vval.v_list;
    let l_: *const list_T = list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            *len += strlen(tv_get_string(&raw const (*li).li_tv)) as ptrdiff_t
                + (if crlf as c_int != 0 {
                    2 as c_int
                } else {
                    1 as c_int
                }) as ptrdiff_t;
            li = (*li).li_next;
        }
    }
    if *len == 0 as ptrdiff_t {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut ret_1: *mut c_char = xmalloc((*len as size_t).wrapping_add(
        (if endnl as c_int != 0 {
            if crlf as c_int != 0 {
                2 as c_int
            } else {
                1 as c_int
            }
        } else {
            0 as c_int
        }) as size_t,
    )) as *mut c_char;
    let mut end_0: *mut c_char = ret_1;
    let l__0: *const list_T = list;
    if !l__0.is_null() {
        let mut li_0: *const listitem_T = (*l__0).lv_first;
        while !li_0.is_null() {
            let mut s: *const c_char = tv_get_string(&raw const (*li_0).li_tv);
            while *s as c_int != '\0' as c_int {
                let c2rust_fresh14 = end_0;
                end_0 = end_0.offset(1);
                *c2rust_fresh14 = (if *s as c_int == '\n' as c_int {
                    '\0' as c_int
                } else {
                    *s as c_int
                }) as c_char;
                s = s.offset(1);
            }
            if endnl as c_int != 0 || !(*li_0).li_next.is_null() {
                if crlf {
                    let c2rust_fresh15 = end_0;
                    end_0 = end_0.offset(1);
                    *c2rust_fresh15 = '\r' as c_char;
                }
                let c2rust_fresh16 = end_0;
                end_0 = end_0.offset(1);
                *c2rust_fresh16 = '\n' as c_char;
            }
            li_0 = (*li_0).li_next;
        }
    }
    *end_0 = NUL as c_char;
    *len = end_0.offset_from(ret_1) as ptrdiff_t;
    return ret_1;
}
pub unsafe extern "C" fn buf_byteidx_to_charidx(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut byteidx: c_int,
) -> c_int {
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
        return -1 as c_int;
    }
    if lnum > (*buf).b_ml.ml_line_count {
        lnum = (*buf).b_ml.ml_line_count;
    }
    let mut str: *mut c_char = ml_get_buf(buf, lnum);
    if *str as c_int == NUL {
        return 0 as c_int;
    }
    let mut t: *mut c_char = str;
    let mut count: c_int = 0;
    count = 0 as c_int;
    while *t as c_int != NUL && t <= str.offset(byteidx as isize) {
        t = t.offset(utfc_ptr2len(t) as isize);
        count += 1;
    }
    if *t as c_int == NUL && byteidx != 0 as c_int && t == str.offset(byteidx as isize) {
        count += 1;
    }
    return count - 1 as c_int;
}
pub unsafe extern "C" fn buf_charidx_to_byteidx(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut charidx: c_int,
) -> c_int {
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
        return -1 as c_int;
    }
    if lnum > (*buf).b_ml.ml_line_count {
        lnum = (*buf).b_ml.ml_line_count;
    }
    let mut str: *mut c_char = ml_get_buf(buf, lnum);
    let mut t: *mut c_char = str;
    while *t as c_int != NUL && {
        charidx -= 1;
        charidx > 0 as c_int
    } {
        t = t.offset(utfc_ptr2len(t) as isize);
    }
    return t.offset_from(str) as c_int;
}
pub unsafe extern "C" fn var2fpos(
    tv: *const typval_T,
    dollar_lnum: bool,
    ret_fnum: *mut c_int,
    charcol: bool,
    mut wp: *mut win_T,
) -> *mut pos_T {
    static pos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    });
    let mut bp: *mut buf_T = (*wp).w_buffer;
    if (*tv).v_type as c_uint == VAR_LIST as c_int as c_uint {
        let mut error: bool = false_0 != 0;
        let mut l: *mut list_T = (*tv).vval.v_list;
        if l.is_null() {
            return ::core::ptr::null_mut::<pos_T>();
        }
        (*pos.ptr()).lnum = tv_list_find_nr(l, 0 as c_int, &raw mut error) as linenr_T;
        if error as c_int != 0
            || (*pos.ptr()).lnum <= 0 as linenr_T
            || (*pos.ptr()).lnum > (*bp).b_ml.ml_line_count
        {
            return ::core::ptr::null_mut::<pos_T>();
        }
        (*pos.ptr()).col = tv_list_find_nr(l, 1 as c_int, &raw mut error) as colnr_T;
        if error {
            return ::core::ptr::null_mut::<pos_T>();
        }
        let mut len: c_int = 0;
        if charcol {
            len = mb_charlen(ml_get_buf(bp, (*pos.ptr()).lnum));
        } else {
            len = ml_get_buf_len(bp, (*pos.ptr()).lnum) as c_int;
        }
        let mut li: *mut listitem_T = tv_list_find(l, 1 as c_int);
        if !li.is_null()
            && (*li).li_tv.v_type as c_uint == VAR_STRING as c_int as c_uint
            && !(*li).li_tv.vval.v_string.is_null()
            && strcmp((*li).li_tv.vval.v_string, b"$\0".as_ptr() as *const c_char) == 0 as c_int
        {
            (*pos.ptr()).col = (len + 1 as c_int) as colnr_T;
        }
        if (*pos.ptr()).col == 0 as c_int || (*pos.ptr()).col > len + 1 as c_int {
            return ::core::ptr::null_mut::<pos_T>();
        }
        (*pos.ptr()).col -= 1;
        (*pos.ptr()).coladd = tv_list_find_nr(l, 2 as c_int, &raw mut error) as colnr_T;
        if error {
            (*pos.ptr()).coladd = 0 as c_int as colnr_T;
        }
        return pos.ptr();
    }
    let name: *const c_char = tv_get_string_chk(tv);
    if name.is_null() {
        return ::core::ptr::null_mut::<pos_T>();
    }
    (*pos.ptr()).lnum = 0 as c_int as linenr_T;
    if *name.offset(0 as c_int as isize) as c_int == '.' as c_int {
        pos.set((*wp).w_cursor);
    } else if *name.offset(0 as c_int as isize) as c_int == 'v' as c_int
        && *name.offset(1 as c_int as isize) as c_int == NUL
    {
        if VIsual_active.get() as c_int != 0 && wp == curwin.get() {
            pos.set(VIsual.get());
        } else {
            pos.set((*wp).w_cursor);
        }
    } else if *name.offset(0 as c_int as isize) as c_int == '\'' as c_int {
        let mut mname: c_int = *name.offset(1 as c_int as isize) as uint8_t as c_int;
        let fm: *const fmark_T =
            mark_get(bp, wp, ::core::ptr::null_mut::<fmark_T>(), kMarkAll, mname);
        if fm.is_null() || (*fm).mark.lnum <= 0 as linenr_T {
            return ::core::ptr::null_mut::<pos_T>();
        }
        pos.set((*fm).mark);
        *ret_fnum = if mname as c_uint >= 'A' as c_uint && mname as c_uint <= 'Z' as c_uint
            || ascii_isdigit(mname) as c_int != 0
        {
            (*fm).fnum
        } else {
            *ret_fnum
        };
    }
    if (*pos.ptr()).lnum != 0 as linenr_T {
        if charcol {
            (*pos.ptr()).col =
                buf_byteidx_to_charidx(bp, (*pos.ptr()).lnum, (*pos.ptr()).col as c_int) as colnr_T;
        }
        return pos.ptr();
    }
    (*pos.ptr()).coladd = 0 as c_int as colnr_T;
    if *name.offset(0 as c_int as isize) as c_int == 'w' as c_int && dollar_lnum as c_int != 0 {
        check_cursor_moved(wp);
        (*pos.ptr()).col = 0 as c_int as colnr_T;
        if *name.offset(1 as c_int as isize) as c_int == '0' as c_int {
            update_topline(wp);
            (*pos.ptr()).lnum = if (*wp).w_topline > 0 as linenr_T {
                (*wp).w_topline
            } else {
                1 as linenr_T
            };
            return pos.ptr();
        } else if *name.offset(1 as c_int as isize) as c_int == '$' as c_int {
            validate_botline_win(wp);
            (*pos.ptr()).lnum = if (*wp).w_botline > 0 as linenr_T {
                (*wp).w_botline - 1 as linenr_T
            } else {
                0 as linenr_T
            };
            return pos.ptr();
        }
    } else if *name.offset(0 as c_int as isize) as c_int == '$' as c_int {
        if dollar_lnum {
            (*pos.ptr()).lnum = (*bp).b_ml.ml_line_count;
            (*pos.ptr()).col = 0 as c_int as colnr_T;
        } else {
            (*pos.ptr()).lnum = (*wp).w_cursor.lnum;
            if charcol {
                (*pos.ptr()).col = mb_charlen(ml_get_buf(bp, (*wp).w_cursor.lnum));
            } else {
                (*pos.ptr()).col = ml_get_buf_len(bp, (*wp).w_cursor.lnum);
            }
        }
        return pos.ptr();
    }
    return ::core::ptr::null_mut::<pos_T>();
}
pub unsafe extern "C" fn list2fpos(
    mut arg: *mut typval_T,
    mut posp: *mut pos_T,
    mut fnump: *mut c_int,
    mut curswantp: *mut colnr_T,
    mut charcol: bool,
) -> c_int {
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if (*arg).v_type as c_uint != VAR_LIST as c_int as c_uint
        || {
            l = (*arg).vval.v_list;
            l.is_null()
        }
        || tv_list_len(l)
            < (if fnump.is_null() {
                2 as c_int
            } else {
                3 as c_int
            })
        || tv_list_len(l)
            > (if fnump.is_null() {
                4 as c_int
            } else {
                5 as c_int
            })
    {
        return FAIL;
    }
    let mut i: c_int = 0 as c_int;
    let mut n: c_int = 0;
    if !fnump.is_null() {
        let c2rust_fresh18 = i;
        i = i + 1;
        n = tv_list_find_nr(l, c2rust_fresh18, ::core::ptr::null_mut::<bool>()) as c_int;
        if n < 0 as c_int {
            return FAIL;
        }
        if n == 0 as c_int {
            n = (*curbuf.get()).handle as c_int;
        }
        *fnump = n;
    }
    let c2rust_fresh19 = i;
    i = i + 1;
    n = tv_list_find_nr(l, c2rust_fresh19, ::core::ptr::null_mut::<bool>()) as c_int;
    if n < 0 as c_int {
        return FAIL;
    }
    (*posp).lnum = n as linenr_T;
    let c2rust_fresh20 = i;
    i = i + 1;
    n = tv_list_find_nr(l, c2rust_fresh20, ::core::ptr::null_mut::<bool>()) as c_int;
    if n < 0 as c_int {
        return FAIL;
    }
    if charcol {
        let mut buf: *mut buf_T = buflist_findnr(if fnump.is_null() {
            (*curbuf.get()).handle as c_int
        } else {
            *fnump
        });
        if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
            return FAIL;
        }
        n = buf_charidx_to_byteidx(
            buf,
            if (*posp).lnum == 0 as linenr_T {
                (*curwin.get()).w_cursor.lnum
            } else {
                (*posp).lnum
            },
            n,
        ) + 1 as c_int;
    }
    (*posp).col = n as colnr_T;
    n = tv_list_find_nr(l, i, ::core::ptr::null_mut::<bool>()) as c_int;
    if n < 0 as c_int {
        (*posp).coladd = 0 as c_int as colnr_T;
    } else {
        (*posp).coladd = n as colnr_T;
    }
    if !curswantp.is_null() {
        *curswantp = tv_list_find_nr(l, i + 1 as c_int, ::core::ptr::null_mut::<bool>()) as colnr_T;
    }
    return OK;
}
pub unsafe extern "C" fn get_env_len(mut arg: *mut *const c_char) -> c_int {
    let mut p: *const c_char = ::core::ptr::null::<c_char>();
    p = *arg;
    while vim_isIDc(*p as uint8_t as c_int) {
        p = p.offset(1);
    }
    if p == *arg {
        return 0 as c_int;
    }
    let mut len: c_int = p.offset_from(*arg) as c_int;
    *arg = p;
    return len;
}
pub unsafe extern "C" fn get_id_len(arg: *mut *const c_char) -> c_int {
    let mut len: c_int = 0;
    let mut p: *const c_char = ::core::ptr::null::<c_char>();
    p = *arg;
    while eval_isnamec(*p as c_int) {
        if *p as c_int == ':' as c_int {
            len = p.offset_from(*arg) as c_int;
            if len > 1 as c_int
                || len == 1 as c_int
                    && vim_strchr(namespace_char.get(), **arg as uint8_t as c_int).is_null()
            {
                break;
            }
        }
        p = p.offset(1);
    }
    if p == *arg {
        return 0 as c_int;
    }
    len = p.offset_from(*arg) as c_int;
    *arg = skipwhite(p);
    return len;
}
pub unsafe extern "C" fn get_name_len(
    arg: *mut *const c_char,
    mut alias: *mut *mut c_char,
    mut evaluate: bool,
    mut verbose: bool,
) -> c_int {
    *alias = ::core::ptr::null_mut::<c_char>();
    if *(*arg).offset(0 as c_int as isize) as c_int == K_SPECIAL as c_char as c_int
        && *(*arg).offset(1 as c_int as isize) as c_int == KS_EXTRA as c_char as c_int
        && *(*arg).offset(2 as c_int as isize) as c_int == KE_SNR as c_int as c_char as c_int
    {
        *arg = (*arg).offset(3 as c_int as isize);
        return get_id_len(arg) + 3 as c_int;
    }
    let mut len: c_int = eval_fname_script(*arg);
    if len > 0 as c_int {
        *arg = (*arg).offset(len as isize);
    }
    let mut expr_start: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut expr_end: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut p: *const c_char = find_name_end(
        *arg,
        &raw mut expr_start as *mut *const c_char,
        &raw mut expr_end as *mut *const c_char,
        if len > 0 as c_int {
            0 as c_int
        } else {
            FNE_CHECK_START
        },
    );
    if !expr_start.is_null() {
        if !evaluate {
            len += p.offset_from(*arg) as c_int;
            *arg = skipwhite(p);
            return len;
        }
        let mut temp_string: *mut c_char = make_expanded_name(
            (*arg).offset(-(len as isize)),
            expr_start,
            expr_end,
            p as *mut c_char,
        );
        if temp_string.is_null() {
            return -1 as c_int;
        }
        *alias = temp_string;
        *arg = skipwhite(p);
        return strlen(temp_string) as c_int;
    }
    len += get_id_len(arg);
    if len == 0 as c_int && verbose as c_int != 0 && **arg as c_int != NUL {
        semsg(gettext(&raw const e_invexpr2 as *const c_char), *arg);
    }
    return len;
}
pub unsafe extern "C" fn find_name_end(
    mut arg: *const c_char,
    mut expr_start: *mut *const c_char,
    mut expr_end: *mut *const c_char,
    mut flags: c_int,
) -> *const c_char {
    if !expr_start.is_null() {
        *expr_start = ::core::ptr::null::<c_char>();
        *expr_end = ::core::ptr::null::<c_char>();
    }
    if flags & FNE_CHECK_START != 0
        && !eval_isnamec1(*arg as c_int)
        && *arg as c_int != '{' as c_int
    {
        return arg;
    }
    let mut mb_nest: c_int = 0 as c_int;
    let mut br_nest: c_int = 0 as c_int;
    let mut len: c_int = 0;
    let mut p: *const c_char = ::core::ptr::null::<c_char>();
    p = arg;
    while *p as c_int != NUL
        && (eval_isnamec(*p as c_int) as c_int != 0
            || *p as c_int == '{' as c_int
            || flags & FNE_INCL_BR != 0
                && (*p as c_int == '[' as c_int
                    || *p as c_int == '.' as c_int
                        && eval_isdictc(*p.offset(1 as c_int as isize) as c_int) as c_int != 0)
            || mb_nest != 0 as c_int
            || br_nest != 0 as c_int)
    {
        if *p as c_int == '\'' as c_int {
            p = p.offset(1 as c_int as isize);
            while *p as c_int != NUL && *p as c_int != '\'' as c_int {
                p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
            }
            if *p as c_int == NUL {
                break;
            }
        } else if *p as c_int == '"' as c_int {
            p = p.offset(1 as c_int as isize);
            while *p as c_int != NUL && *p as c_int != '"' as c_int {
                if *p as c_int == '\\' as c_int && *p.offset(1 as c_int as isize) as c_int != NUL {
                    p = p.offset(1);
                }
                p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
            }
            if *p as c_int == NUL {
                break;
            }
        } else if br_nest == 0 as c_int && mb_nest == 0 as c_int && *p as c_int == ':' as c_int {
            len = p.offset_from(arg) as c_int;
            if len > 1 as c_int && *p.offset(-1 as c_int as isize) as c_int != '}' as c_int
                || len == 1 as c_int
                    && vim_strchr(namespace_char.get(), *arg as uint8_t as c_int).is_null()
            {
                break;
            }
        }
        if mb_nest == 0 as c_int {
            if *p as c_int == '[' as c_int {
                br_nest += 1;
            } else if *p as c_int == ']' as c_int {
                br_nest -= 1;
            }
        }
        if br_nest == 0 as c_int {
            if *p as c_int == '{' as c_int {
                mb_nest += 1;
                if !expr_start.is_null() && (*expr_start).is_null() {
                    *expr_start = p;
                }
            } else if *p as c_int == '}' as c_int {
                mb_nest -= 1;
                if !expr_start.is_null() && mb_nest == 0 as c_int && (*expr_end).is_null() {
                    *expr_end = p;
                }
            }
        }
        p = p.offset(utfc_ptr2len(p as *mut c_char) as isize);
    }
    return p;
}
unsafe extern "C" fn make_expanded_name(
    mut in_start: *const c_char,
    mut expr_start: *mut c_char,
    mut expr_end: *mut c_char,
    mut in_end: *mut c_char,
) -> *mut c_char {
    if expr_end.is_null() || in_end.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut retval: *mut c_char = ::core::ptr::null_mut::<c_char>();
    *expr_start = NUL as c_char;
    *expr_end = NUL as c_char;
    let mut c1: c_char = *in_end;
    *in_end = NUL as c_char;
    let mut temp_result: *mut c_char = eval_to_string(
        expr_start.offset(1 as c_int as isize),
        false_0 != 0,
        false_0 != 0,
    );
    if !temp_result.is_null() {
        let mut retvalsize: size_t = (expr_start.offset_from(in_start) as size_t)
            .wrapping_add(strlen(temp_result))
            .wrapping_add(in_end.offset_from(expr_end) as size_t)
            .wrapping_add(1 as size_t);
        retval = xmalloc(retvalsize) as *mut c_char;
        vim_snprintf(
            retval,
            retvalsize,
            b"%s%s%s\0".as_ptr() as *const c_char,
            in_start,
            temp_result,
            expr_end.offset(1 as c_int as isize),
        );
    }
    xfree(temp_result as *mut c_void);
    *in_end = c1;
    *expr_start = '{' as c_char;
    *expr_end = '}' as c_char;
    if !retval.is_null() {
        temp_result = find_name_end(
            retval,
            &raw mut expr_start as *mut *const c_char,
            &raw mut expr_end as *mut *const c_char,
            0 as c_int,
        ) as *mut c_char;
        if !expr_start.is_null() {
            temp_result = make_expanded_name(retval, expr_start, expr_end, temp_result);
            xfree(retval as *mut c_void);
            retval = temp_result;
        }
    }
    return retval;
}
pub unsafe extern "C" fn eval_isnamec(mut c: c_int) -> bool {
    return c as c_uint >= 'A' as c_uint && c as c_uint <= 'Z' as c_uint
        || c as c_uint >= 'a' as c_uint && c as c_uint <= 'z' as c_uint
        || ascii_isdigit(c) as c_int != 0
        || c == '_' as c_int
        || c == ':' as c_int
        || c == AUTOLOAD_CHAR;
}
pub unsafe extern "C" fn eval_isnamec1(mut c: c_int) -> bool {
    return c as c_uint >= 'A' as c_uint && c as c_uint <= 'Z' as c_uint
        || c as c_uint >= 'a' as c_uint && c as c_uint <= 'z' as c_uint
        || c == '_' as c_int;
}
pub unsafe extern "C" fn eval_isdictc(mut c: c_int) -> bool {
    return c as c_uint >= 'A' as c_uint && c as c_uint <= 'Z' as c_uint
        || c as c_uint >= 'a' as c_uint && c as c_uint <= 'z' as c_uint
        || ascii_isdigit(c) as c_int != 0
        || c == '_' as c_int;
}
pub unsafe extern "C" fn set_argv_var(mut argv: *mut *mut c_char, mut argc: c_int) {
    let mut l: *mut list_T = tv_list_alloc(argc as ptrdiff_t);
    tv_list_set_lock(l, VAR_FIXED);
    let mut i: c_int = 0 as c_int;
    while i < argc {
        tv_list_append_string(l, *argv.offset(i as isize) as *const c_char, -1 as ssize_t);
        (*tv_list_last(l)).li_tv.v_lock = VAR_FIXED;
        i += 1;
    }
    set_vim_var_list(VV_ARGV, l);
}
pub unsafe extern "C" fn is_luafunc(mut partial: *mut partial_T) -> bool {
    return partial == get_vim_var_partial(VV_LUA);
}
unsafe extern "C" fn tv_is_luafunc(mut tv: *mut typval_T) -> bool {
    return (*tv).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
        && is_luafunc((*tv).vval.v_partial) as c_int != 0;
}
pub unsafe extern "C" fn skip_luafunc_name(mut p: *const c_char) -> *const c_char {
    while *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint
        || *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint
        || ascii_isdigit(*p as c_int) as c_int != 0
        || *p as c_int == '_' as c_int
        || *p as c_int == '-' as c_int
        || *p as c_int == '.' as c_int
        || *p as c_int == '\'' as c_int
    {
        p = p.offset(1);
    }
    return p;
}
pub unsafe extern "C" fn check_luafunc_name(str: *const c_char, paren: bool) -> c_int {
    let p: *const c_char = skip_luafunc_name(str);
    if *p as c_int
        != (if paren as c_int != 0 {
            '(' as c_int
        } else {
            NUL
        })
    {
        return 0 as c_int;
    }
    return p.offset_from(str) as c_int;
}
pub unsafe extern "C" fn char_from_string(
    mut str: *const c_char,
    mut index: varnumber_T,
) -> *mut c_char {
    let mut nchar: varnumber_T = index;
    if str.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut slen: size_t = strlen(str);
    if index < 0 as varnumber_T {
        let mut clen: c_int = 0 as c_int;
        let mut nbyte: size_t = 0 as size_t;
        while nbyte < slen {
            nbyte = nbyte.wrapping_add(utfc_ptr2len(str.offset(nbyte as isize)) as size_t);
            clen += 1;
        }
        nchar = clen as varnumber_T + index;
        if nchar < 0 as varnumber_T {
            return ::core::ptr::null_mut::<c_char>();
        }
    }
    let mut nbyte_0: size_t = 0 as size_t;
    while nchar > 0 as varnumber_T && nbyte_0 < slen {
        nbyte_0 = nbyte_0.wrapping_add(utfc_ptr2len(str.offset(nbyte_0 as isize)) as size_t);
        nchar -= 1;
    }
    if nbyte_0 >= slen {
        return ::core::ptr::null_mut::<c_char>();
    }
    return xmemdupz(
        str.offset(nbyte_0 as isize) as *const c_void,
        utfc_ptr2len(str.offset(nbyte_0 as isize)) as size_t,
    ) as *mut c_char;
}
unsafe extern "C" fn char_idx2byte(
    mut str: *const c_char,
    mut str_len: size_t,
    mut idx: varnumber_T,
) -> ssize_t {
    let mut nchar: varnumber_T = idx;
    let mut nbyte: size_t = 0 as size_t;
    if nchar >= 0 as varnumber_T {
        while nchar > 0 as varnumber_T && nbyte < str_len {
            nbyte = nbyte.wrapping_add(utfc_ptr2len(str.offset(nbyte as isize)) as size_t);
            nchar -= 1;
        }
    } else {
        nbyte = str_len;
        while nchar < 0 as varnumber_T && nbyte > 0 as size_t {
            nbyte = nbyte.wrapping_sub(1);
            nbyte = nbyte.wrapping_sub(utf_head_off(str, str.offset(nbyte as isize)) as size_t);
            nchar += 1;
        }
        if nchar < 0 as varnumber_T {
            return -1 as ssize_t;
        }
    }
    return nbyte as ssize_t;
}
pub unsafe extern "C" fn string_slice(
    mut str: *const c_char,
    mut first: varnumber_T,
    mut last: varnumber_T,
    mut exclusive: bool,
) -> *mut c_char {
    if str.is_null() {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut slen: size_t = strlen(str);
    let mut start_byte: ssize_t = char_idx2byte(str, slen, first);
    if start_byte < 0 as ssize_t {
        start_byte = 0 as ssize_t;
    }
    let mut end_byte: ssize_t = 0;
    if last == -1 as varnumber_T && !exclusive || last == VARNUMBER_MAX as varnumber_T {
        end_byte = slen as ssize_t;
    } else {
        end_byte = char_idx2byte(str, slen, last);
        if !exclusive && end_byte >= 0 as ssize_t && end_byte < slen as ssize_t {
            end_byte += utfc_ptr2len(str.offset(end_byte as isize)) as ssize_t;
        }
    }
    if start_byte >= slen as ssize_t || end_byte <= start_byte {
        return ::core::ptr::null_mut::<c_char>();
    }
    return xmemdupz(
        str.offset(start_byte as isize) as *const c_void,
        (end_byte - start_byte) as size_t,
    ) as *mut c_char;
}
pub unsafe extern "C" fn handle_subscript(
    arg: *mut *const c_char,
    mut rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    mut verbose: bool,
) -> c_int {
    let evaluate: bool = !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as c_int != 0;
    let mut ret: c_int = OK;
    let mut selfdict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut lua_funcname: *const c_char = ::core::ptr::null::<c_char>();
    if tv_is_luafunc(rettv) {
        if !evaluate {
            tv_clear(rettv);
        }
        if **arg as c_int != '.' as c_int {
            tv_clear(rettv);
            ret = FAIL;
        } else {
            *arg = (*arg).offset(1);
            lua_funcname = *arg;
            let len: c_int = check_luafunc_name(*arg, true_0 != 0);
            if len == 0 as c_int {
                tv_clear(rettv);
                ret = FAIL;
            }
            *arg = (*arg).offset(len as isize);
        }
    }
    while ret == OK
        && ((**arg as c_int == '[' as c_int
            || **arg as c_int == '.' as c_int
                && (*rettv).v_type as c_uint == VAR_DICT as c_int as c_uint
            || **arg as c_int == '(' as c_int && (!evaluate || tv_is_func(*rettv) as c_int != 0))
            && !ascii_iswhite(*(*arg).offset(-(1 as c_int as isize)) as c_int)
            || **arg as c_int == '-' as c_int
                && *(*arg).offset(1 as c_int as isize) as c_int == '>' as c_int)
    {
        if **arg as c_int == '(' as c_int {
            ret = call_func_rettv(
                arg as *mut *mut c_char,
                evalarg,
                rettv,
                evaluate,
                selfdict,
                ::core::ptr::null_mut::<typval_T>(),
                lua_funcname,
            );
            if aborting() {
                if ret == OK {
                    tv_clear(rettv);
                }
                ret = FAIL;
            }
            tv_dict_unref(selfdict);
            selfdict = ::core::ptr::null_mut::<dict_T>();
        } else if **arg as c_int == '-' as c_int {
            if *(*arg).offset(2 as c_int as isize) as c_int == '{' as c_int {
                ret = eval_lambda(arg as *mut *mut c_char, rettv, evalarg, verbose);
            } else {
                ret = eval_method(arg as *mut *mut c_char, rettv, evalarg, verbose);
            }
        } else {
            tv_dict_unref(selfdict);
            if (*rettv).v_type as c_uint == VAR_DICT as c_int as c_uint {
                selfdict = (*rettv).vval.v_dict;
                if !selfdict.is_null() {
                    (*selfdict).dv_refcount += 1;
                }
            } else {
                selfdict = ::core::ptr::null_mut::<dict_T>();
            }
            if eval_index(arg as *mut *mut c_char, rettv, evalarg, verbose) == FAIL {
                tv_clear(rettv);
                ret = FAIL;
            }
        }
    }
    if !selfdict.is_null() && tv_is_func(*rettv) as c_int != 0 {
        set_selfdict(rettv, selfdict);
    }
    tv_dict_unref(selfdict);
    return ret;
}
pub unsafe extern "C" fn set_selfdict(rettv: *mut typval_T, selfdict: *mut dict_T) {
    if (*rettv).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
        && !(*(*rettv).vval.v_partial).pt_auto
        && !(*(*rettv).vval.v_partial).pt_dict.is_null()
    {
        return;
    }
    make_partial(selfdict, rettv);
}
pub unsafe extern "C" fn var_item_copy(
    conv: *const vimconv_T,
    from: *mut typval_T,
    to: *mut typval_T,
    deep: bool,
    copyID: c_int,
) -> c_int {
    static recurse: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    let mut ret: c_int = OK;
    if recurse.get() >= DICT_MAXNEST {
        emsg(gettext(
            (e_variable_nested_too_deep_for_making_copy.ptr() as *const _) as *const c_char,
        ));
        return FAIL;
    }
    (*recurse.ptr()) += 1;
    match (*from).v_type as c_uint {
        1 | 6 | 3 | 9 | 7 | 8 => {
            tv_copy(from, to);
        }
        2 => {
            if conv.is_null()
                || (*conv).vc_type == CONV_NONE as c_int
                || (*from).vval.v_string.is_null()
            {
                tv_copy(from, to);
            } else {
                (*to).v_type = VAR_STRING;
                (*to).v_lock = VAR_UNLOCKED;
                (*to).vval.v_string = string_convert(
                    conv as *mut vimconv_T,
                    (*from).vval.v_string,
                    ::core::ptr::null_mut::<size_t>(),
                );
                if (*to).vval.v_string.is_null() {
                    (*to).vval.v_string = xstrdup((*from).vval.v_string);
                }
            }
        }
        4 => {
            (*to).v_type = VAR_LIST;
            (*to).v_lock = VAR_UNLOCKED;
            if (*from).vval.v_list.is_null() {
                (*to).vval.v_list = ::core::ptr::null_mut::<list_T>();
            } else if copyID != 0 as c_int && tv_list_copyid((*from).vval.v_list) == copyID {
                (*to).vval.v_list = tv_list_latest_copy((*from).vval.v_list);
                tv_list_ref((*to).vval.v_list);
            } else {
                (*to).vval.v_list = tv_list_copy(conv, (*from).vval.v_list, deep, copyID);
            }
            if (*to).vval.v_list.is_null() && !(*from).vval.v_list.is_null() {
                ret = FAIL;
            }
        }
        10 => {
            tv_blob_copy((*from).vval.v_blob, to);
        }
        5 => {
            (*to).v_type = VAR_DICT;
            (*to).v_lock = VAR_UNLOCKED;
            if (*from).vval.v_dict.is_null() {
                (*to).vval.v_dict = ::core::ptr::null_mut::<dict_T>();
            } else if copyID != 0 as c_int && (*(*from).vval.v_dict).dv_copyID == copyID {
                (*to).vval.v_dict = (*(*from).vval.v_dict).dv_copydict;
                (*(*to).vval.v_dict).dv_refcount += 1;
            } else {
                (*to).vval.v_dict = tv_dict_copy(conv, (*from).vval.v_dict, deep, copyID);
            }
            if (*to).vval.v_dict.is_null() && !(*from).vval.v_dict.is_null() {
                ret = FAIL;
            }
        }
        0 => {
            internal_error(b"var_item_copy(UNKNOWN)\0".as_ptr() as *const c_char);
            ret = FAIL;
        }
        _ => {}
    }
    (*recurse.ptr()) -= 1;
    return ret;
}
pub unsafe extern "C" fn ex_echo(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut atstart: bool = true_0 != 0;
    let mut need_clear: bool = true_0 != 0;
    let did_emsg_before: c_int = did_emsg.get();
    let called_emsg_before: c_int = called_emsg.get();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<c_void>(),
        eval_tofree: ::core::ptr::null_mut::<c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) += 1;
    }
    while *arg as c_int != NUL
        && *arg as c_int != '|' as c_int
        && *arg as c_int != '\n' as c_int
        && !got_int.get()
    {
        need_clr_eos.set(true_0 != 0);
        let mut p: *mut c_char = arg;
        if eval1(&raw mut arg, &raw mut rettv, &raw mut evalarg) == FAIL {
            if !aborting()
                && did_emsg.get() == did_emsg_before
                && called_emsg.get() == called_emsg_before
            {
                semsg(gettext(&raw const e_invexpr2 as *const c_char), p);
            }
            need_clr_eos.set(false_0 != 0);
            break;
        } else {
            need_clr_eos.set(false_0 != 0);
            if (*eap).skip == 0 {
                if atstart {
                    atstart = false_0 != 0;
                    msg_ext_set_append((*eap).cmdidx as c_int == CMD_echon as c_int);
                    msg_ext_set_kind(b"echo\0".as_ptr() as *const c_char);
                    if (*eap).cmdidx as c_int == CMD_echo as c_int {
                        if !msg_didout.get() {
                            msg_sb_eol();
                        }
                        msg_start();
                    }
                } else if (*eap).cmdidx as c_int == CMD_echo as c_int {
                    msg_puts_hl(
                        b" \0".as_ptr() as *const c_char,
                        echo_hl_id.get(),
                        false_0 != 0,
                    );
                }
                let mut tofree: *mut c_char =
                    encode_tv2echo(&raw mut rettv, ::core::ptr::null_mut::<size_t>());
                msg_multiline(
                    cstr_as_string(tofree),
                    echo_hl_id.get(),
                    true_0 != 0,
                    false_0 != 0,
                    &raw mut need_clear,
                );
                xfree(tofree as *mut c_void);
            }
            tv_clear(&raw mut rettv);
            arg = skipwhite(arg);
        }
    }
    (*eap).nextcmd = check_nextcmd(arg);
    clear_evalarg(&raw mut evalarg, eap);
    msg_ext_set_append(false_0 != 0);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) -= 1;
    } else {
        if ui_has(kUIMessages) as c_int != 0
            && (*(*eap).arg as c_int == NUL
                || *(*eap).arg as c_int == '|' as c_int
                || *(*eap).arg as c_int == '\n' as c_int)
        {
            msg_puts_len(
                b"\0".as_ptr() as *const c_char,
                0 as ptrdiff_t,
                0 as c_int,
                false_0 != 0,
            );
        } else if need_clear {
            msg_clr_eos();
        }
        if (*eap).cmdidx as c_int == CMD_echo as c_int {
            msg_end();
        }
    };
}
pub unsafe extern "C" fn ex_echohl(mut eap: *mut exarg_T) {
    echo_hl_id.set(syn_name2id((*eap).arg));
}
pub unsafe extern "C" fn get_echo_hl_id() -> c_int {
    return echo_hl_id.get();
}
pub unsafe extern "C" fn ex_execute(mut eap: *mut exarg_T) {
    let mut arg: *mut c_char = (*eap).arg;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut ret: c_int = OK;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    };
    ga_init(&raw mut ga, 1 as c_int, 80 as c_int);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) += 1;
    }
    while *arg as c_int != NUL && *arg as c_int != '|' as c_int && *arg as c_int != '\n' as c_int {
        ret = eval1_emsg(&raw mut arg, &raw mut rettv, eap);
        if ret == FAIL {
            break;
        }
        if (*eap).skip == 0 {
            let argstr: *const c_char = if (*eap).cmdidx as c_int == CMD_execute as c_int {
                tv_get_string(&raw mut rettv)
            } else {
                (if rettv.v_type as c_uint == VAR_STRING as c_int as c_uint {
                    encode_tv2echo(&raw mut rettv, ::core::ptr::null_mut::<size_t>())
                } else {
                    encode_tv2string(&raw mut rettv, ::core::ptr::null_mut::<size_t>())
                }) as *const c_char
            };
            let len: size_t = strlen(argstr);
            ga_grow(&raw mut ga, len as c_int + 2 as c_int);
            if !(ga.ga_len <= 0 as c_int) {
                let c2rust_fresh21 = ga.ga_len;
                ga.ga_len = ga.ga_len + 1;
                *(ga.ga_data as *mut c_char).offset(c2rust_fresh21 as isize) = ' ' as c_char;
            }
            memcpy(
                (ga.ga_data as *mut c_char).offset(ga.ga_len as isize) as *mut c_void,
                argstr as *const c_void,
                len.wrapping_add(1 as size_t),
            );
            if (*eap).cmdidx as c_int != CMD_execute as c_int {
                xfree(argstr as *mut c_void);
            }
            ga.ga_len += len as c_int;
        }
        tv_clear(&raw mut rettv);
        arg = skipwhite(arg);
    }
    if ret != FAIL && !ga.ga_data.is_null() {
        if (*eap).cmdidx as c_int == CMD_echomsg as c_int {
            msg_ext_set_kind(b"echomsg\0".as_ptr() as *const c_char);
            msg(ga.ga_data as *const c_char, echo_hl_id.get());
        } else if (*eap).cmdidx as c_int == CMD_echoerr as c_int {
            let mut save_did_emsg: c_int = did_emsg.get();
            emsg_multiline(
                ga.ga_data as *const c_char,
                b"echoerr\0".as_ptr() as *const c_char,
                HLF_E as c_int,
                true_0 != 0,
            );
            if !force_abort.get() {
                did_emsg.set(save_did_emsg);
            }
        } else if (*eap).cmdidx as c_int == CMD_execute as c_int {
            do_cmdline(
                ga.ga_data as *mut c_char,
                (*eap).ea_getline,
                (*eap).cookie,
                DOCMD_NOWAIT as c_int | DOCMD_VERBOSE as c_int,
            );
        }
    }
    ga_clear(&raw mut ga);
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) -= 1;
    }
    (*eap).nextcmd = check_nextcmd(arg);
}
pub unsafe extern "C" fn find_option_var_end(
    arg: *mut *const c_char,
    opt_idxp: *mut OptIndex,
    opt_flags: *mut c_int,
) -> *const c_char {
    let mut p: *const c_char = *arg;
    p = p.offset(1);
    if *p as c_int == 'g' as c_int && *p.offset(1 as c_int as isize) as c_int == ':' as c_int {
        *opt_flags = OPT_GLOBAL as c_int;
        p = p.offset(2 as c_int as isize);
    } else if *p as c_int == 'l' as c_int && *p.offset(1 as c_int as isize) as c_int == ':' as c_int
    {
        *opt_flags = OPT_LOCAL as c_int;
        p = p.offset(2 as c_int as isize);
    } else {
        *opt_flags = 0 as c_int;
    }
    let mut end: *const c_char = find_option_end(p, opt_idxp);
    *arg = if end.is_null() { *arg } else { p };
    return end;
}
pub unsafe extern "C" fn var_flavour(mut varname: *mut c_char) -> var_flavour_T {
    let mut p: *mut c_char = varname;
    if *p as c_uint >= 'A' as c_uint && *p as c_uint <= 'Z' as c_uint {
        loop {
            p = p.offset(1);
            if *p == 0 {
                break;
            }
            if *p as c_uint >= 'a' as c_uint && *p as c_uint <= 'z' as c_uint {
                return VAR_FLAVOUR_SESSION;
            }
        }
        return VAR_FLAVOUR_SHADA;
    }
    return VAR_FLAVOUR_DEFAULT;
}
pub unsafe extern "C" fn var_set_global(name: *const c_char, mut vartv: typval_T) {
    let mut funccall_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccall_entry);
    set_var(name, strlen(name), &raw mut vartv, false_0 != 0);
    restore_funccal();
}
pub unsafe extern "C" fn last_set_msg(mut script_ctx: sctx_T) {
    if script_ctx.sc_sid == 0 as c_int {
        return;
    }
    let mut should_free: bool = false;
    let mut p: *mut c_char = get_scriptname(script_ctx, &raw mut should_free);
    msg_ext_skip_verbose.set(true_0 != 0);
    verbose_enter();
    msg_puts(gettext(b"\n\tLast set from \0".as_ptr() as *const c_char));
    msg_puts(p);
    if script_ctx.sc_lnum > 0 as linenr_T {
        msg_puts(gettext(&raw const line_msg as *const c_char));
        msg_outnum(script_ctx.sc_lnum as c_int);
    } else if script_is_lua(script_ctx.sc_sid) {
        msg_puts(gettext(
            b" (run Nvim with -V1 for more details)\0".as_ptr() as *const c_char
        ));
    }
    if should_free {
        xfree(p as *mut c_void);
    }
    verbose_leave();
}
pub unsafe extern "C" fn do_string_sub(
    mut str: *mut c_char,
    mut len: size_t,
    mut pat: *mut c_char,
    mut sub: *mut c_char,
    mut expr: *mut typval_T,
    mut flags: *const c_char,
    mut ret_len: *mut size_t,
) -> *mut c_char {
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<c_char>(); 10],
        endp: [::core::ptr::null_mut::<c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    };
    let mut save_cpo: *mut c_char = p_cpo.get();
    p_cpo.set(empty_string_option.ptr() as *mut c_char);
    ga_init(&raw mut ga, 1 as c_int, 200 as c_int);
    regmatch.rm_ic = p_ic.get() != 0;
    regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
    if !regmatch.regprog.is_null() {
        let mut tail: *mut c_char = str;
        let mut end: *mut c_char = str.offset(len as isize);
        let mut do_all: bool = *flags.offset(0 as c_int as isize) as c_int == 'g' as c_int;
        let mut sublen: c_int = 0;
        let mut zero_width: *mut c_char = ::core::ptr::null_mut::<c_char>();
        while vim_regexec_nl(&raw mut regmatch, str, tail.offset_from(str) as colnr_T) {
            if regmatch.startp[0 as c_int as usize] == regmatch.endp[0 as c_int as usize] {
                if zero_width == regmatch.startp[0 as c_int as usize] {
                    let mut i: c_int = utfc_ptr2len(tail);
                    memmove(
                        (ga.ga_data as *mut c_char).offset(ga.ga_len as isize) as *mut c_void,
                        tail as *const c_void,
                        i as size_t,
                    );
                    ga.ga_len += i;
                    tail = tail.offset(i as isize);
                    continue;
                } else {
                    zero_width = regmatch.startp[0 as c_int as usize];
                }
            }
            sublen = vim_regsub(
                &raw mut regmatch,
                sub,
                expr,
                tail,
                0 as c_int,
                REGSUB_MAGIC as c_int,
            );
            if sublen <= 0 as c_int {
                ga_clear(&raw mut ga);
                break;
            } else {
                ga_grow(
                    &raw mut ga,
                    (end.offset_from(tail) + sublen as isize
                        - regmatch.endp[0 as c_int as usize]
                            .offset_from(regmatch.startp[0 as c_int as usize]))
                        as c_int,
                );
                let mut i_0: c_int =
                    regmatch.startp[0 as c_int as usize].offset_from(tail) as c_int;
                memmove(
                    (ga.ga_data as *mut c_char).offset(ga.ga_len as isize) as *mut c_void,
                    tail as *const c_void,
                    i_0 as size_t,
                );
                vim_regsub(
                    &raw mut regmatch,
                    sub,
                    expr,
                    (ga.ga_data as *mut c_char)
                        .offset(ga.ga_len as isize)
                        .offset(i_0 as isize),
                    sublen,
                    REGSUB_COPY as c_int | REGSUB_MAGIC as c_int,
                );
                ga.ga_len += i_0 + sublen - 1 as c_int;
                tail = regmatch.endp[0 as c_int as usize];
                if *tail as c_int == NUL {
                    break;
                }
                if !do_all {
                    break;
                }
            }
        }
        if !ga.ga_data.is_null() {
            strcpy((ga.ga_data as *mut c_char).offset(ga.ga_len as isize), tail);
            ga.ga_len += end.offset_from(tail) as c_int;
        }
        vim_regfree(regmatch.regprog);
    }
    if !ga.ga_data.is_null() {
        str = ga.ga_data as *mut c_char;
        len = ga.ga_len as size_t;
    }
    let mut ret: *mut c_char = xstrnsave(str, len);
    ga_clear(&raw mut ga);
    if p_cpo.get() == empty_string_option.ptr() as *mut c_char {
        p_cpo.set(save_cpo);
    } else {
        if *p_cpo.get() as c_int == NUL {
            set_option_value_give_err(
                kOptCpoptions,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_as_string(save_cpo),
                    },
                },
                0 as c_int,
            );
        }
        free_string_option(save_cpo);
    }
    if !ret_len.is_null() {
        *ret_len = len;
    }
    return ret;
}
pub unsafe extern "C" fn common_job_callbacks(
    mut vopts: *mut dict_T,
    mut on_stdout: *mut CallbackReader,
    mut on_stderr: *mut CallbackReader,
    mut on_exit: *mut Callback,
) -> bool {
    if tv_dict_get_callback(
        vopts,
        b"on_stdout\0".as_ptr() as *const c_char,
        ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as usize) as ptrdiff_t,
        &raw mut (*on_stdout).cb,
    ) as c_int
        != 0
        && tv_dict_get_callback(
            vopts,
            b"on_stderr\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 10]>().wrapping_sub(1 as usize) as ptrdiff_t,
            &raw mut (*on_stderr).cb,
        ) as c_int
            != 0
        && tv_dict_get_callback(
            vopts,
            b"on_exit\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
            on_exit,
        ) as c_int
            != 0
    {
        (*on_stdout).buffered =
            tv_dict_get_number(vopts, b"stdout_buffered\0".as_ptr() as *const c_char) != 0;
        (*on_stderr).buffered =
            tv_dict_get_number(vopts, b"stderr_buffered\0".as_ptr() as *const c_char) != 0;
        if (*on_stdout).buffered as c_int != 0
            && (*on_stdout).cb.type_0 as c_uint == kCallbackNone as c_int as c_uint
        {
            (*on_stdout).self_0 = vopts;
        }
        if (*on_stderr).buffered as c_int != 0
            && (*on_stderr).cb.type_0 as c_uint == kCallbackNone as c_int as c_uint
        {
            (*on_stderr).self_0 = vopts;
        }
        (*vopts).dv_refcount += 1;
        return true_0 != 0;
    }
    callback_reader_free(on_stdout);
    callback_reader_free(on_stderr);
    callback_free(on_exit);
    return false_0 != 0;
}
pub unsafe extern "C" fn find_job(mut id: uint64_t, mut show_error: bool) -> *mut Channel {
    let mut data: *mut Channel = find_channel(id);
    if data.is_null()
        || (*data).streamtype as c_uint != kChannelStreamProc as c_int as c_uint
        || proc_is_stopped(&*channel_proc(data)) as c_int != 0
    {
        if show_error {
            if !data.is_null()
                && (*data).streamtype as c_uint != kChannelStreamProc as c_int as c_uint
            {
                emsg(gettext(&raw const e_invchanjob as *const c_char));
            } else {
                emsg(gettext(&raw const e_invchan as *const c_char));
            }
        }
        return ::core::ptr::null_mut::<Channel>();
    }
    return data;
}
pub unsafe extern "C" fn script_host_eval(
    mut name: *mut c_char,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    if check_secure() {
        return;
    }
    if (*argvars.offset(0 as c_int as isize)).v_type as c_uint != VAR_STRING as c_int as c_uint {
        emsg(gettext(&raw const e_invarg as *const c_char));
        return;
    }
    let mut args: *mut list_T = tv_list_alloc(1 as ptrdiff_t);
    tv_list_append_string(
        args,
        (*argvars.offset(0 as c_int as isize)).vval.v_string,
        -1 as ssize_t,
    );
    *rettv = eval_call_provider(
        name,
        b"eval\0".as_ptr() as *const c_char as *mut c_char,
        args,
        false_0 != 0,
    );
}
pub unsafe extern "C" fn eval_call_provider(
    mut provider: *mut c_char,
    mut method: *mut c_char,
    mut arguments: *mut list_T,
    mut discard: bool,
) -> typval_T {
    if !eval_has_provider(provider, false_0 != 0) {
        semsg(
            b"E319: No \"%s\" provider found. Run \":checkhealth vim.provider\"\0".as_ptr()
                as *const c_char,
            provider,
        );
        return typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: 0 as varnumber_T,
            },
        };
    }
    let mut func: [c_char; 256] = [0; 256];
    let mut name_len: c_int = snprintf(
        &raw mut func as *mut c_char,
        ::core::mem::size_of::<[c_char; 256]>(),
        b"provider#%s#Call\0".as_ptr() as *const c_char,
        provider,
    );
    let mut saved_provider_caller_scope: caller_scope = provider_caller_scope.get() as caller_scope;
    provider_caller_scope.set(caller_scope {
        script_ctx: current_sctx.get(),
        es_entry: *((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize),
        autocmd_fname: autocmd_fname.get(),
        autocmd_match: autocmd_match.get(),
        autocmd_fname_full: autocmd_fname_full.get(),
        autocmd_bufnr: autocmd_bufnr.get(),
        funccalp: get_current_funccal() as *mut c_void,
    } as caller_scope);
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    save_funccal(&raw mut funccal_entry);
    (*provider_call_nesting.ptr()) += 1;
    let mut argvars: [typval_T; 3] = [
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_string: method },
        },
        typval_T {
            v_type: VAR_LIST,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_list: arguments },
        },
        typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        },
    ];
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    tv_list_ref(arguments);
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
    funcexe.fe_evaluate = true_0 != 0;
    call_func(
        &raw mut func as *mut c_char,
        name_len,
        &raw mut rettv,
        2 as c_int,
        &raw mut argvars as *mut typval_T,
        &raw mut funcexe,
    );
    tv_list_unref(arguments);
    restore_funccal();
    provider_caller_scope.set(saved_provider_caller_scope as caller_scope);
    (*provider_call_nesting.ptr()) -= 1;
    '_c2rust_label: {
        if provider_call_nesting.get() >= 0 as c_int {
        } else {
            __assert_fail(
                b"provider_call_nesting >= 0\0".as_ptr() as *const c_char,
                b"src/nvim/eval.rs\0".as_ptr() as *const c_char,
                6585 as c_uint,
                b"typval_T eval_call_provider(char *, char *, list_T *, _Bool)\0".as_ptr()
                    as *const c_char,
            );
        }
    };
    if discard {
        tv_clear(&raw mut rettv);
    }
    return rettv;
}
pub unsafe extern "C" fn eval_has_provider(
    mut feat: *const c_char,
    mut throw_if_fast: bool,
) -> bool {
    if !strequal(feat, b"clipboard\0".as_ptr() as *const c_char)
        && !strequal(feat, b"python3\0".as_ptr() as *const c_char)
        && !strequal(feat, b"python3_compiled\0".as_ptr() as *const c_char)
        && !strequal(feat, b"python3_dynamic\0".as_ptr() as *const c_char)
        && !strequal(feat, b"perl\0".as_ptr() as *const c_char)
        && !strequal(feat, b"ruby\0".as_ptr() as *const c_char)
        && !strequal(feat, b"node\0".as_ptr() as *const c_char)
    {
        return false_0 != 0;
    }
    if throw_if_fast as c_int != 0 && !nlua_is_deferred_safe() {
        semsg(
            &raw const e_fast_api_disabled as *const c_char,
            b"Vimscript function\0".as_ptr() as *const c_char,
        );
        return false_0 != 0;
    }
    let mut name: [c_char; 32] = [0; 32];
    snprintf(
        &raw mut name as *mut c_char,
        ::core::mem::size_of::<[c_char; 32]>(),
        b"%s\0".as_ptr() as *const c_char,
        feat,
    );
    strchrsub(&raw mut name as *mut c_char, '_' as c_char, NUL as c_char);
    let mut buf: [c_char; 256] = [0; 256];
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut len: c_int = snprintf(
        &raw mut buf as *mut c_char,
        ::core::mem::size_of::<[c_char; 256]>(),
        b"g:loaded_%s_provider\0".as_ptr() as *const c_char,
        &raw mut name as *mut c_char,
    );
    if eval_variable(
        &raw mut buf as *mut c_char,
        len,
        &raw mut tv,
        ::core::ptr::null_mut::<*mut dictitem_T>(),
        false_0 != 0,
        true_0 != 0,
    ) == FAIL
    {
        len = snprintf(
            &raw mut buf as *mut c_char,
            ::core::mem::size_of::<[c_char; 256]>(),
            b"provider#%s#bogus\0".as_ptr() as *const c_char,
            &raw mut name as *mut c_char,
        );
        script_autoload(&raw mut buf as *mut c_char, len as size_t, false_0 != 0);
        len = snprintf(
            &raw mut buf as *mut c_char,
            ::core::mem::size_of::<[c_char; 256]>(),
            b"g:loaded_%s_provider\0".as_ptr() as *const c_char,
            &raw mut name as *mut c_char,
        );
        if eval_variable(
            &raw mut buf as *mut c_char,
            len,
            &raw mut tv,
            ::core::ptr::null_mut::<*mut dictitem_T>(),
            false_0 != 0,
            true_0 != 0,
        ) == FAIL
        {
            snprintf(
                &raw mut buf as *mut c_char,
                ::core::mem::size_of::<[c_char; 256]>(),
                b"provider#%s#Call\0".as_ptr() as *const c_char,
                &raw mut name as *mut c_char,
            );
            if !find_func(&raw mut buf as *mut c_char).is_null() && p_lpl.get() != 0 {
                semsg(
                    b"provider: %s: missing required variable g:loaded_%s_provider\0".as_ptr()
                        as *const c_char,
                    &raw mut name as *mut c_char,
                    &raw mut name as *mut c_char,
                );
            }
            return false_0 != 0;
        }
    }
    let mut ok: bool = if tv.v_type as c_uint == VAR_NUMBER as c_int as c_uint {
        (2 as varnumber_T == tv.vval.v_number) as c_int
    } else {
        false_0
    } != 0;
    if ok {
        snprintf(
            &raw mut buf as *mut c_char,
            ::core::mem::size_of::<[c_char; 256]>(),
            b"provider#%s#Call\0".as_ptr() as *const c_char,
            &raw mut name as *mut c_char,
        );
        if find_func(&raw mut buf as *mut c_char).is_null() {
            semsg(
                b"provider: %s: g:loaded_%s_provider=2 but %s is not defined\0".as_ptr()
                    as *const c_char,
                &raw mut name as *mut c_char,
                &raw mut name as *mut c_char,
                &raw mut buf as *mut c_char,
            );
            ok = false_0 != 0;
        }
    }
    return ok;
}
pub unsafe extern "C" fn eval_fmt_source_name_line(mut buf: *mut c_char, mut bufsize: size_t) {
    if !(*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
    .es_name
    .is_null()
    {
        snprintf(
            buf,
            bufsize,
            b"%s:%d\0".as_ptr() as *const c_char,
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
            .es_name,
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as c_int) as isize))
            .es_lnum,
        );
    } else {
        snprintf(buf, bufsize, b"?\0".as_ptr() as *const c_char);
    };
}
pub unsafe extern "C" fn prompt_get_input(mut buf: *mut buf_T) -> *mut c_char {
    if !bt_prompt(buf) {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut lnum_start: linenr_T = (*buf).b_prompt_start.mark.lnum;
    let mut lnum_last: linenr_T = (*buf).b_ml.ml_line_count;
    let mut text: *mut c_char = ml_get_buf(buf, lnum_start);
    if strlen(text) as c_int >= (*buf).b_prompt_start.mark.col {
        text = text.offset((*buf).b_prompt_start.mark.col as isize);
    }
    let mut full_text: *mut c_char = xstrdup(text);
    let mut i: linenr_T = lnum_start + 1 as linenr_T;
    while i <= lnum_last {
        let mut half_text: *mut c_char = concat_str(full_text, b"\n\0".as_ptr() as *const c_char);
        xfree(full_text as *mut c_void);
        full_text = concat_str(half_text, ml_get_buf(buf, i));
        xfree(half_text as *mut c_void);
        i += 1;
    }
    return full_text;
}
pub unsafe extern "C" fn prompt_invoke_callback() {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut argv: [typval_T; 2] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 2];
    let mut lnum: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
    let mut user_input: *mut c_char = prompt_get_input(curbuf.get());
    if user_input.is_null() {
        return;
    }
    ml_append(
        lnum,
        b"\0".as_ptr() as *const c_char as *mut c_char,
        0 as colnr_T,
        false_0 != 0,
    );
    appended_lines_mark(lnum, 1 as c_int);
    (*curwin.get()).w_cursor.lnum = lnum + 1 as linenr_T;
    (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
    (*curbuf.get()).b_prompt_start.mark.lnum = lnum + 1 as linenr_T;
    if (*curbuf.get()).b_prompt_callback.type_0 as c_uint == kCallbackNone as c_int as c_uint {
        xfree(user_input as *mut c_void);
    } else {
        argv[0 as c_int as usize].v_type = VAR_STRING;
        argv[0 as c_int as usize].vval.v_string = user_input;
        argv[1 as c_int as usize].v_type = VAR_UNKNOWN;
        callback_call(
            &raw mut (*curbuf.get()).b_prompt_callback,
            1 as c_int,
            &raw mut argv as *mut typval_T,
            &raw mut rettv,
        );
        tv_clear((&raw mut argv as *mut typval_T).offset(0 as c_int as isize));
        tv_clear(&raw mut rettv);
    }
    u_clearallandblockfree(curbuf.get());
    (*curbuf.get()).b_prompt_start.mark.lnum = (*curbuf.get()).b_ml.ml_line_count;
    (*curbuf.get()).b_prompt_append_new_line = true_0 != 0;
}
pub unsafe extern "C" fn invoke_prompt_interrupt() -> bool {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut argv: [typval_T; 1] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 1];
    if (*curbuf.get()).b_prompt_interrupt.type_0 as c_uint == kCallbackNone as c_int as c_uint {
        return false_0 != 0;
    }
    argv[0 as c_int as usize].v_type = VAR_UNKNOWN;
    got_int.set(false_0 != 0);
    let mut ret: c_int = callback_call(
        &raw mut (*curbuf.get()).b_prompt_interrupt,
        0 as c_int,
        &raw mut argv as *mut typval_T,
        &raw mut rettv,
    ) as c_int;
    tv_clear(&raw mut rettv);
    return ret != FAIL;
}
pub unsafe extern "C" fn typval_compare(
    mut typ1: *mut typval_T,
    mut typ2: *mut typval_T,
    mut type_0: exprtype_T,
    mut ic: bool,
) -> c_int {
    let mut n1: varnumber_T = 0;
    let mut n2: varnumber_T = 0;
    let type_is: bool = type_0 as c_uint == EXPR_IS as c_int as c_uint
        || type_0 as c_uint == EXPR_ISNOT as c_int as c_uint;
    if type_is as c_int != 0 && (*typ1).v_type as c_uint != (*typ2).v_type as c_uint {
        n1 = (type_0 as c_uint == EXPR_ISNOT as c_int as c_uint) as c_int as varnumber_T;
    } else if (*typ1).v_type as c_uint == VAR_BLOB as c_int as c_uint
        || (*typ2).v_type as c_uint == VAR_BLOB as c_int as c_uint
    {
        if type_is {
            n1 = ((*typ1).v_type as c_uint == (*typ2).v_type as c_uint
                && (*typ1).vval.v_blob == (*typ2).vval.v_blob) as c_int
                as varnumber_T;
            if type_0 as c_uint == EXPR_ISNOT as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        } else if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint
            || type_0 as c_uint != EXPR_EQUAL as c_int as c_uint
                && type_0 as c_uint != EXPR_NEQUAL as c_int as c_uint
        {
            if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint {
                emsg(gettext(
                    b"E977: Can only compare Blob with Blob\0".as_ptr() as *const c_char
                ));
            } else {
                emsg(gettext(&raw const e_invalblob as *const c_char));
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            n1 = tv_blob_equal((*typ1).vval.v_blob, (*typ2).vval.v_blob) as varnumber_T;
            if type_0 as c_uint == EXPR_NEQUAL as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        }
    } else if (*typ1).v_type as c_uint == VAR_LIST as c_int as c_uint
        || (*typ2).v_type as c_uint == VAR_LIST as c_int as c_uint
    {
        if type_is {
            n1 = ((*typ1).v_type as c_uint == (*typ2).v_type as c_uint
                && (*typ1).vval.v_list == (*typ2).vval.v_list) as c_int
                as varnumber_T;
            if type_0 as c_uint == EXPR_ISNOT as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        } else if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint
            || type_0 as c_uint != EXPR_EQUAL as c_int as c_uint
                && type_0 as c_uint != EXPR_NEQUAL as c_int as c_uint
        {
            if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint {
                emsg(gettext(
                    b"E691: Can only compare List with List\0".as_ptr() as *const c_char
                ));
            } else {
                emsg(gettext(
                    b"E692: Invalid operation for List\0".as_ptr() as *const c_char
                ));
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            n1 = tv_list_equal((*typ1).vval.v_list, (*typ2).vval.v_list, ic) as varnumber_T;
            if type_0 as c_uint == EXPR_NEQUAL as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        }
    } else if (*typ1).v_type as c_uint == VAR_DICT as c_int as c_uint
        || (*typ2).v_type as c_uint == VAR_DICT as c_int as c_uint
    {
        if type_is {
            n1 = ((*typ1).v_type as c_uint == (*typ2).v_type as c_uint
                && (*typ1).vval.v_dict == (*typ2).vval.v_dict) as c_int
                as varnumber_T;
            if type_0 as c_uint == EXPR_ISNOT as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        } else if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint
            || type_0 as c_uint != EXPR_EQUAL as c_int as c_uint
                && type_0 as c_uint != EXPR_NEQUAL as c_int as c_uint
        {
            if (*typ1).v_type as c_uint != (*typ2).v_type as c_uint {
                emsg(gettext(
                    b"E735: Can only compare Dictionary with Dictionary\0".as_ptr()
                        as *const c_char,
                ));
            } else {
                emsg(gettext(
                    b"E736: Invalid operation for Dictionary\0".as_ptr() as *const c_char,
                ));
            }
            tv_clear(typ1);
            return FAIL;
        } else {
            n1 = tv_dict_equal((*typ1).vval.v_dict, (*typ2).vval.v_dict, ic) as varnumber_T;
            if type_0 as c_uint == EXPR_NEQUAL as c_int as c_uint {
                n1 = (n1 == 0) as c_int as varnumber_T;
            }
        }
    } else if tv_is_func(*typ1) as c_int != 0 || tv_is_func(*typ2) as c_int != 0 {
        if type_0 as c_uint != EXPR_EQUAL as c_int as c_uint
            && type_0 as c_uint != EXPR_NEQUAL as c_int as c_uint
            && type_0 as c_uint != EXPR_IS as c_int as c_uint
            && type_0 as c_uint != EXPR_ISNOT as c_int as c_uint
        {
            emsg(gettext(
                b"E694: Invalid operation for Funcrefs\0".as_ptr() as *const c_char
            ));
            tv_clear(typ1);
            return FAIL;
        }
        if (*typ1).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
            && (*typ1).vval.v_partial.is_null()
            || (*typ2).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
                && (*typ2).vval.v_partial.is_null()
        {
            n1 = ((*typ1).vval.v_partial == (*typ2).vval.v_partial) as c_int as varnumber_T;
        } else if type_is {
            if (*typ1).v_type as c_uint == VAR_FUNC as c_int as c_uint
                && (*typ2).v_type as c_uint == VAR_FUNC as c_int as c_uint
            {
                n1 = tv_equal(typ1, typ2, ic) as varnumber_T;
            } else if (*typ1).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
                && (*typ2).v_type as c_uint == VAR_PARTIAL as c_int as c_uint
            {
                n1 = ((*typ1).vval.v_partial == (*typ2).vval.v_partial) as c_int as varnumber_T;
            } else {
                n1 = false_0 as varnumber_T;
            }
        } else {
            n1 = tv_equal(typ1, typ2, ic) as varnumber_T;
        }
        if type_0 as c_uint == EXPR_NEQUAL as c_int as c_uint
            || type_0 as c_uint == EXPR_ISNOT as c_int as c_uint
        {
            n1 = (n1 == 0) as c_int as varnumber_T;
        }
    } else if ((*typ1).v_type as c_uint == VAR_FLOAT as c_int as c_uint
        || (*typ2).v_type as c_uint == VAR_FLOAT as c_int as c_uint)
        && type_0 as c_uint != EXPR_MATCH as c_int as c_uint
        && type_0 as c_uint != EXPR_NOMATCH as c_int as c_uint
    {
        let f1: float_T = tv_get_float(typ1);
        let f2: float_T = tv_get_float(typ2);
        n1 = false_0 as varnumber_T;
        match type_0 as c_uint {
            9 | 1 => {
                n1 = (f1 == f2) as c_int as varnumber_T;
            }
            10 | 2 => {
                n1 = (f1 != f2) as c_int as varnumber_T;
            }
            3 => {
                n1 = (f1 > f2) as c_int as varnumber_T;
            }
            4 => {
                n1 = (f1 >= f2) as c_int as varnumber_T;
            }
            5 => {
                n1 = (f1 < f2) as c_int as varnumber_T;
            }
            6 => {
                n1 = (f1 <= f2) as c_int as varnumber_T;
            }
            0 | 7 | 8 | _ => {}
        }
    } else if ((*typ1).v_type as c_uint == VAR_NUMBER as c_int as c_uint
        || (*typ2).v_type as c_uint == VAR_NUMBER as c_int as c_uint)
        && type_0 as c_uint != EXPR_MATCH as c_int as c_uint
        && type_0 as c_uint != EXPR_NOMATCH as c_int as c_uint
    {
        n1 = tv_get_number(typ1);
        n2 = tv_get_number(typ2);
        match type_0 as c_uint {
            9 | 1 => {
                n1 = (n1 == n2) as c_int as varnumber_T;
            }
            10 | 2 => {
                n1 = (n1 != n2) as c_int as varnumber_T;
            }
            3 => {
                n1 = (n1 > n2) as c_int as varnumber_T;
            }
            4 => {
                n1 = (n1 >= n2) as c_int as varnumber_T;
            }
            5 => {
                n1 = (n1 < n2) as c_int as varnumber_T;
            }
            6 => {
                n1 = (n1 <= n2) as c_int as varnumber_T;
            }
            0 | 7 | 8 | _ => {}
        }
    } else {
        let mut buf1: [c_char; 65] = [0; 65];
        let mut buf2: [c_char; 65] = [0; 65];
        let s1: *const c_char = tv_get_string_buf(typ1, &raw mut buf1 as *mut c_char);
        let s2: *const c_char = tv_get_string_buf(typ2, &raw mut buf2 as *mut c_char);
        let mut i: c_int = 0;
        if type_0 as c_uint != EXPR_MATCH as c_int as c_uint
            && type_0 as c_uint != EXPR_NOMATCH as c_int as c_uint
        {
            i = mb_strcmp_ic(ic, s1, s2);
        } else {
            i = 0 as c_int;
        }
        n1 = false_0 as varnumber_T;
        match type_0 as c_uint {
            9 | 1 => {
                n1 = (i == 0 as c_int) as c_int as varnumber_T;
            }
            10 | 2 => {
                n1 = (i != 0 as c_int) as c_int as varnumber_T;
            }
            3 => {
                n1 = (i > 0 as c_int) as c_int as varnumber_T;
            }
            4 => {
                n1 = (i >= 0 as c_int) as c_int as varnumber_T;
            }
            5 => {
                n1 = (i < 0 as c_int) as c_int as varnumber_T;
            }
            6 => {
                n1 = (i <= 0 as c_int) as c_int as varnumber_T;
            }
            7 | 8 => {
                n1 = pattern_match(s2, s1, ic) as varnumber_T;
                if type_0 as c_uint == EXPR_NOMATCH as c_int as c_uint {
                    n1 = (n1 == 0) as c_int as varnumber_T;
                }
            }
            0 | _ => {}
        }
    }
    tv_clear(typ1);
    (*typ1).v_type = VAR_NUMBER;
    (*typ1).vval.v_number = n1;
    return OK;
}
pub unsafe extern "C" fn typval_tostring(mut arg: *mut typval_T, mut quotes: bool) -> *mut c_char {
    if arg.is_null() {
        return xstrdup(b"(does not exist)\0".as_ptr() as *const c_char);
    }
    if !quotes && (*arg).v_type as c_uint == VAR_STRING as c_int as c_uint {
        return xstrdup(if (*arg).vval.v_string.is_null() {
            b"\0".as_ptr() as *const c_char
        } else {
            (*arg).vval.v_string as *const c_char
        });
    }
    return encode_tv2string(arg, ::core::ptr::null_mut::<size_t>());
}
#[inline]
unsafe extern "C" fn tv_list_latest_copy(l: *const list_T) -> *mut list_T {
    return (*l).lv_copylist;
}
#[inline]
unsafe extern "C" fn tv_list_has_watchers(l: *const list_T) -> bool {
    return !l.is_null() && !(*l).lv_watch.is_null();
}
#[inline]
unsafe extern "C" fn tv_init(tv: *mut typval_T) {
    if !tv.is_null() {
        memset(
            tv as *mut c_void,
            0 as c_int,
            ::core::mem::size_of::<typval_T>(),
        );
    }
}
pub const TV_CSTRING: c_ulong = SIZE_MAX.wrapping_sub(1 as c_ulong);
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
pub const PROF_YES: c_int = 1 as c_int;
pub const K_SPECIAL: c_int = 0x80 as c_int;
pub const KS_EXTRA: c_int = 253 as c_int;
pub const true_0: c_int = 1 as c_int;
pub const false_0: c_int = 0 as c_int;
pub const INT_MAX: c_int = __INT_MAX__;
pub const RE_MAGIC: c_int = 1 as c_int;
pub const RE_STRING: c_int = 2 as c_int;
pub const __INT_MAX__: c_int = 2147483647 as c_int;
