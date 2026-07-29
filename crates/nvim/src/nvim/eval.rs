pub mod buffer;
pub mod decode;
pub mod deprecated;
pub mod encode;
pub mod executor;
pub mod fs;
pub mod funcs;
pub mod gc;
pub mod list;
pub mod typval;
pub mod userfunc;
pub mod vars;
pub mod window;
use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::buffer::{bt_prompt, buflist_findnr};
use crate::src::nvim::change::appended_lines_mark;
use crate::src::nvim::channel::callback_reader_free;
use crate::src::nvim::channel::channel_proc;
use crate::src::nvim::channel::find_channel;
use crate::src::nvim::charset::{skipwhite, vim_isIDc};
use crate::src::nvim::eval::encode::{encode_list_write, encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::executor::eexe_mod_op;
use crate::src::nvim::eval::gc::{gc_first_dict, gc_first_list};
use crate::src::nvim::eval::typval::{
    callback_free, callback_put, tv_blob_alloc_ret, tv_blob_check_index, tv_blob_check_range,
    tv_blob_copy, tv_blob_set_append, tv_blob_set_range, tv_blob_unref, tv_check_lock,
    tv_check_str, tv_clear, tv_copy, tv_dict_add, tv_dict_add_nr, tv_dict_alloc, tv_dict_copy,
    tv_dict_find, tv_dict_free_contents, tv_dict_free_dict, tv_dict_get_callback,
    tv_dict_get_number, tv_dict_item_alloc, tv_dict_watcher_notify, tv_dict_wrong_func_name,
    tv_get_number, tv_get_number_chk, tv_get_string, tv_get_string_chk, tv_in_free_unref_items,
    tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict, tv_list_append_string,
    tv_list_assign_range, tv_list_check_range_index_one, tv_list_check_range_index_two,
    tv_list_copy, tv_list_find, tv_list_find_nr, tv_list_free_contents, tv_list_free_list,
    tv_list_unref, tv_list_watch_add, tv_list_watch_remove, value_check_lock,
};
use crate::src::nvim::eval::typval::{
    tv_blob_get, tv_blob_len, tv_dict_is_watched, tv_dict_watcher_node_data, tv_is_func,
    tv_list_copyid, tv_list_first, tv_list_len, tv_list_ref,
};
use crate::src::nvim::eval::userfunc::{
    call_func, eval_fname_script, find_func, free_unref_funccal, func_ref, get_current_funccal,
    get_funccal_args_ht, get_scriptlocal_funcname, restore_funccal, save_funccal,
    set_ref_in_call_stack, set_ref_in_func, set_ref_in_func_args, set_ref_in_functions,
    set_ref_in_previous_funccal,
};
use crate::src::nvim::eval::vars::{
    eval_variable, ex_let_vars, find_var, garbage_collect_globvars, garbage_collect_scriptvars,
    garbage_collect_vimvars, get_vim_var_partial, get_vimvar_dict, set_var, set_var_const,
    set_vim_var_nr, skip_var_list, valid_varname, var_check_lock, var_check_ro,
    var_wrong_func_name,
};
use crate::src::nvim::event::multiqueue::{multiqueue_free, multiqueue_new_child};
use crate::src::nvim::event::proc::proc_is_stopped;
use crate::src::nvim::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_docmd::{
    check_nextcmd, do_cmdline, ends_excmd, get_pressedreturn, set_pressedreturn,
    set_ref_in_findfunc,
};
use crate::src::nvim::ex_eval::{aborting, discard_current_exception};
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::highlight_group::syn_name2id;
use crate::src::nvim::insexpand::{set_ref_in_cpt_callbacks, set_ref_in_insexpand_funcs};
use crate::src::nvim::lua::executor::{
    nlua_call_ref, nlua_is_deferred_safe, nlua_is_table_from_lua, nlua_register_table_as_callable,
};
use crate::src::nvim::main::aucmd_win_vec;
use crate::src::nvim::main::{
    EVALARG_EVALUATE, VIsual, VIsual_active, autocmd_bufnr, autocmd_fname, autocmd_fname_full,
    autocmd_match, called_emsg, channels, curbuf, current_sctx, curtab, curwin, did_emsg,
    did_throw, do_profiling, e_cannot_mod, e_command_too_recursive, e_dictkey, e_fast_api_disabled,
    e_illvar, e_invalblob, e_invalid_value_for_blob_nr, e_invarg, e_invarg2, e_invargNval,
    e_invchan, e_invchanjob, e_invexpr2, e_letwrong, e_nobufnr, e_trailing_arg, emsg_severe,
    emsg_skip, first_tabpage, firstbuf, firstwin, force_abort, garbage_collect_at_exit, got_int,
    line_msg, main_loop, may_garbage_collect, msg_didout, msg_ext_skip_verbose, need_clr_eos,
    p_lpl, p_mfd, p_verbose, provider_call_nesting, provider_caller_scope, want_garbage_collect,
};
use crate::src::nvim::map::{map_del_uint64_t_ptr_t, map_put_ref_uint64_t_ptr_t, mh_get_uint64_t};
use crate::src::nvim::mark::{mark_get, mark_global_iter};
use crate::src::nvim::mbyte::{mb_charlen, string_convert, utfc_ptr2len};
use crate::src::nvim::memline::{ml_append, ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{
    memchrsub, strchrsub, strequal, xcalloc, xfree, xmalloc, xmemdupz, xrealloc, xstrdup,
};
use crate::src::nvim::message::{
    emsg, emsg_multiline, internal_error, msg, msg_clr_eos, msg_end, msg_ext_set_append,
    msg_ext_set_kind, msg_multiline, msg_outnum, msg_puts, msg_puts_hl, msg_puts_len, msg_sb_eol,
    msg_start, semsg, smsg, verb_msg, verbose_enter, verbose_enter_scroll, verbose_leave,
    verbose_leave_scroll,
};
use crate::src::nvim::r#move::{check_cursor_moved, update_topline, validate_botline_win};
use crate::src::nvim::ops::set_ref_in_opfunc;
use crate::src::nvim::option::find_option_end;
use crate::src::nvim::os::fs::os_can_exe;
use crate::src::nvim::os::libc::{
    __assert_fail, abort, gettext, memcmp, memcpy, memset, snprintf, strcmp, strlen,
};
use crate::src::nvim::os::shell::{
    os_system, shell_argv_to_str, shell_build_argv, shell_free_argv,
};
use crate::src::nvim::profile::{prof_child_enter, prof_child_exit};
use crate::src::nvim::quickfix::set_ref_in_quickfix;
use crate::src::nvim::register::op_global_reg_iter;
use crate::src::nvim::runtime::{exestack, get_scriptname, script_autoload, script_is_lua};
use crate::src::nvim::strings::{concat_str, vim_snprintf, vim_strchr};
use crate::src::nvim::tag::set_ref_in_tagfunc;
use crate::src::nvim::types::{
    AdditionalData, Arena, Array, BoolVarValue, CMD_index, Callback, CallbackReader, CallbackType,
    Channel, ChannelStreamType, DictWatcher, Error, EvalFuncData, GRegFlags, ListLenSpecials,
    LuaRetMode, Map_uint64_t_ptr_t, MapHash, MarkGet, MotionType, Object, ObjectType, OptIndex,
    OptInt, OptValType, QUEUE, ScopeType, Set_uint64_t, String_0, TimeWatcher, UIExtension,
    VarLockStatus, VarType, VimVarIndex, blob_T, buf_T, caller_scope, colnr_T, dict_T, dictitem_T,
    estack_T, evalarg_T, exarg_T, exprtype_T, fmark_T, fmarkv_T, funccal_entry_T, funcexe_T,
    garray_T, hashitem_T, hashtab_T, ht_stack_S, ht_stack_T, int64_t, key_extra, linenr_T, list_T,
    list_stack_S, list_stack_T, listitem_T, listwatch_T, lval_T, object_data, partial_T, pos_T,
    proftime_T, ptr_t, ptrdiff_t, sctx_T, size_t, ssize_t, tabpage_T, timer_T, typval_T,
    typval_vval_union, ufunc_T, uint8_t, uint32_t, uint64_t, var_flavour_T, varnumber_T, vimconv_T,
    win_T, xfmark_T, yankreg_T,
};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::undo::u_clearallandblockfree;
use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

mod entry;
pub use self::entry::*;
mod lval;
pub use self::lval::*;
mod forloop;
pub use self::forloop::*;
mod collect;
pub use self::collect::*;
mod callback;
pub use self::callback::*;
mod timer;
pub use self::timer::*;
mod name;
pub use self::name::*;
mod system;
pub use self::system::*;
mod pos;
pub use self::pos::*;
mod echo;
pub use self::echo::*;
mod provider;
pub use self::provider::*;
mod pattern;
pub use self::pattern::*;
mod expr;
pub(crate) use self::expr::*;
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
static callback_depth: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
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
