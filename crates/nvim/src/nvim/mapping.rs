use crate::src::nvim::api::private::converter::object_to_vim_take_luaref;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_object, api_set_error, api_set_sctx, arena_dict,
    arena_take_arraybuilder, cstr_as_string, find_buffer_by_handle, string_to_cstr,
};
use crate::src::nvim::ascii::{ascii_isspace, ascii_iswhite};
use crate::src::nvim::charset::{skipwhite, transchar, vim_iswordp};
use crate::src::nvim::cmdexpand::cmdline_fuzzy_complete;
use crate::src::nvim::eval::typval::{
    tv_check_for_dict_arg, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_bool, tv_dict_get_number,
    tv_dict_get_string, tv_get_bool, tv_get_number, tv_get_string, tv_get_string_buf,
    tv_get_string_buf_chk, tv_list_alloc_ret, tv_list_append_dict,
};
use crate::src::nvim::eval::userfunc::find_func;
use crate::src::nvim::eval::vars::set_vim_var_char;
use crate::src::nvim::eval::{eval_to_string, last_set_msg};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_session::put_eol;
use crate::src::nvim::fuzzy::{fuzzy_match_str, fuzzymatches_to_strmatches};
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat, ga_grow, ga_init};
use crate::src::nvim::getchar::{ins_typebuf, noremap_keys};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::HLF_8;
use crate::src::nvim::keycodes::{
    K_SPECIAL, K_ZERO, get_special_key_name, replace_termcodes, vim_strsave_escape_ks,
    vim_unescape_ks,
};
use crate::src::nvim::kvec::_memcpy_free;
use crate::src::nvim::lua::executor::{
    api_free_luaref, api_new_luaref, nlua_call_ref, nlua_funcref_str, nlua_set_sctx,
};
use crate::src::nvim::main::{
    State, curbuf, current_sctx, curwin, e_invarg, e_noabbr, e_nomap, expr_map_lock, got_int,
    langmap_mapchar, mapped_ctrl_c, msg_col, msg_row, msg_silent, no_abbr, p_cpo, p_langmap,
    p_verbose, secure, typebuf,
};
use crate::src::nvim::mbyte::{
    mb_prevptr, mb_unescape, utf_char2bytes, utf_ptr2char, utf_ptr2len, utfc_ptr2len,
};
use crate::src::nvim::memory::{
    ARENA_EMPTY, arena_alloc, arena_finish, arena_mem_free, xcalloc, xfree, xmalloc, xmemcpyz,
    xrealloc, xstrdup, xstrlcpy,
};
use crate::src::nvim::message::{
    emsg, iemsg, message_filtered, msg, msg_clr_eos, msg_ext_set_kind, msg_outtrans,
    msg_outtrans_special, msg_putchar, msg_puts, msg_puts_hl, msg_start, semsg, semsg_multiline,
    str2special_arena, str2special_save, swmsg,
};
use crate::src::nvim::os::libc::{
    __assert_fail, abort, fprintf, fputc, fputs, gettext, memcpy, memmove, memset, putc, snprintf,
    strcasecmp, strchr, strcmp, strlen, strncmp, strpbrk, strstr,
};
use crate::src::nvim::runtime::exestack;
use crate::src::nvim::state::{
    MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL, MODE_OP_PENDING, MODE_SELECT,
    MODE_TERMINAL, MODE_VISUAL,
};
use crate::src::nvim::strings::{sort_strings, vim_snprintf, vim_strchr};
use crate::src::nvim::types::api::{kErrorTypeException, kErrorTypeNone, kErrorTypeValidation};
use crate::src::nvim::types::{
    Arena, Array, ArrayBuilder, BoolVarValue, Buffer, CMD_index, Dict, Error, EvalFuncData, FILE,
    Integer, KeyDict_keymap, ListLenSpecials, LuaRef, LuaRetMode, Object, OptInt, RemapValues,
    ScopeType, SpecialVarValue, String_0, VarLockStatus, VarType, buf_T, cmdidx_T, colnr_T, dict_T,
    dictitem_T, estack_T, exarg_T, expand_T, fuzmatch_str_T, garray_T, kObjectTypeDict,
    kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString, key_extra,
    key_value_pair, linenr_T, mapblock_T, object, object_data as C2Rust_Unnamed, optset_T, pos_T,
    ptrdiff_t, regmatch_T, scid_T, sctx_T, size_t, typval_T, typval_vval_union, ufunc_T, uint8_t,
    uint64_t, varnumber_T,
};
unsafe extern "C" {
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
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
pub const kListLenUnknown: ListLenSpecials = -1;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXMAPLEN: C2Rust_Unnamed_13 = 50;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_int;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_15 = 16;
pub const EXPAND_NOTHING: C2Rust_Unnamed_15 = 0;
pub const CMD_unmap: CMD_index = 500;
pub const CMD_snext: CMD_index = 414;
pub const CMD_map: CMD_index = 275;
pub const CMD_drop: CMD_index = 130;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_args: CMD_index = 7;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const FUZZY_SCORE_NONE: C2Rust_Unnamed_17 = -2147483648;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub const KE_LUA: key_extra = 103;
pub const KE_SNR: key_extra = 82;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const REPTERM_NO_SIMPLIFY: C2Rust_Unnamed_20 = 8;
pub const REPTERM_DO_LT: C2Rust_Unnamed_20 = 2;
pub const REPTERM_FROM_PART: C2Rust_Unnamed_20 = 1;
pub const kRetObject: LuaRetMode = 0;
pub type MapArguments = map_arguments;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct map_arguments {
    pub buffer: bool,
    pub expr: bool,
    pub noremap: bool,
    pub nowait: bool,
    pub script: bool,
    pub silent: bool,
    pub unique: bool,
    pub replace_keycodes: bool,
    pub lhs: [::core::ffi::c_char; 51],
    pub lhs_len: size_t,
    pub alt_lhs: [::core::ffi::c_char; 51],
    pub alt_lhs_len: size_t,
    pub rhs: *mut ::core::ffi::c_char,
    pub rhs_len: size_t,
    pub rhs_lua: LuaRef,
    pub rhs_is_noop: bool,
    pub orig_rhs: *mut ::core::ffi::c_char,
    pub orig_rhs_len: size_t,
    pub desc: *mut ::core::ffi::c_char,
}
pub const MAPTYPE_UNMAP: C2Rust_Unnamed_21 = 1;
pub const MAPTYPE_NOREMAP: C2Rust_Unnamed_21 = 2;
pub const MAPTYPE_UNMAP_LHS: C2Rust_Unnamed_21 = 3;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct langmap_entry_T {
    pub from: ::core::ffi::c_int,
    pub to: ::core::ffi::c_int,
}
pub const MAPTYPE_MAP: C2Rust_Unnamed_21 = 0;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const KEYSET_OPTIDX_keymap__desc: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_keymap__callback: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_H: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const Ctrl_J: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_RSB: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const CPO_BSLASH: ::core::ffi::c_int = 'B' as ::core::ffi::c_int;
pub const MAX_MAPHASH: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const FC_LUAREF: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const ABBR_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
static first_abbr: GlobalCell<*mut mapblock_T> =
    GlobalCell::new(::core::ptr::null_mut::<mapblock_T>());
static maphash: GlobalCell<[*mut mapblock_T; 256]> = GlobalCell::new([
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
    ::core::ptr::null_mut::<mapblock_T>(),
]);
pub const MAP_ARGUMENTS_INIT: MapArguments = map_arguments {
    buffer: false_0 != 0,
    expr: false_0 != 0,
    noremap: false_0 != 0,
    nowait: false_0 != 0,
    script: false_0 != 0,
    silent: false_0 != 0,
    unique: false_0 != 0,
    replace_keycodes: false_0 != 0,
    lhs: [
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
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
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
    lhs_len: 0 as size_t,
    alt_lhs: [
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
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
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
    alt_lhs_len: 0 as size_t,
    rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    rhs_len: 0 as size_t,
    rhs_lua: LUA_NOREF,
    rhs_is_noop: false_0 != 0,
    orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    orig_rhs_len: 0 as size_t,
    desc: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
static e_global_abbreviation_already_exists_for_str: GlobalCell<[::core::ffi::c_char; 48]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
            *b"E224: Global abbreviation already exists for %s\0",
        )
    });
static e_global_mapping_already_exists_for_str: GlobalCell<[::core::ffi::c_char; 43]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
            *b"E225: Global mapping already exists for %s\0",
        )
    });
static e_abbreviation_already_exists_for_str: GlobalCell<[::core::ffi::c_char; 41]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
            *b"E226: Abbreviation already exists for %s\0",
        )
    });
static e_mapping_already_exists_for_str: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E227: Mapping already exists for %s\0",
        )
    });
static e_entries_missing_in_mapset_dict_argument: GlobalCell<[::core::ffi::c_char; 48]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
            *b"E460: Entries missing in mapset() dict argument\0",
        )
    });
static e_illegal_map_mode_string_str: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E1276: Illegal map mode string: '%s'\0",
        )
    });
pub unsafe extern "C" fn get_maphash_list(
    mut state: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> *mut mapblock_T {
    return (*maphash.ptr())[(if state
        & (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING | MODE_TERMINAL)
        != 0
    {
        c
    } else {
        c ^ 0x80 as ::core::ffi::c_int
    }) as usize] as *mut mapblock_T;
}
pub unsafe extern "C" fn get_buf_maphash_list(
    mut state: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> *mut mapblock_T {
    return (*curbuf.get()).b_maphash[(if state
        & (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING | MODE_TERMINAL)
        != 0
    {
        c
    } else {
        c ^ 0x80 as ::core::ffi::c_int
    }) as usize] as *mut mapblock_T;
}
unsafe extern "C" fn mapblock_free(mut mpp: *mut *mut mapblock_T) {
    let mut mp: *mut mapblock_T = *mpp;
    xfree((*mp).m_keys as *mut ::core::ffi::c_void);
    if !(*mp).m_alt.is_null() {
        (*(*mp).m_alt).m_alt = ::core::ptr::null_mut::<mapblock_T>();
    } else {
        if (*mp).m_luaref != LUA_NOREF {
            api_free_luaref((*mp).m_luaref);
            (*mp).m_luaref = LUA_NOREF as LuaRef;
        }
        xfree((*mp).m_str as *mut ::core::ffi::c_void);
        xfree((*mp).m_orig_str as *mut ::core::ffi::c_void);
        xfree((*mp).m_desc as *mut ::core::ffi::c_void);
    }
    *mpp = (*mp).m_next;
    xfree(mp as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn map_mode_to_chars(
    mut mode: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_char,
) {
    let mut p: *mut ::core::ffi::c_char = buf;
    if mode & (MODE_INSERT | MODE_CMDLINE) == MODE_INSERT | MODE_CMDLINE {
        let c2rust_fresh0 = p;
        p = p.offset(1);
        *c2rust_fresh0 = '!' as ::core::ffi::c_char;
    } else if mode & MODE_INSERT != 0 {
        let c2rust_fresh1 = p;
        p = p.offset(1);
        *c2rust_fresh1 = 'i' as ::core::ffi::c_char;
    } else if mode & MODE_LANGMAP != 0 {
        let c2rust_fresh2 = p;
        p = p.offset(1);
        *c2rust_fresh2 = 'l' as ::core::ffi::c_char;
    } else if mode & MODE_CMDLINE != 0 {
        let c2rust_fresh3 = p;
        p = p.offset(1);
        *c2rust_fresh3 = 'c' as ::core::ffi::c_char;
    } else if mode & (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING)
        == MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING
    {
        let c2rust_fresh4 = p;
        p = p.offset(1);
        *c2rust_fresh4 = ' ' as ::core::ffi::c_char;
    } else {
        if mode & MODE_NORMAL != 0 {
            let c2rust_fresh5 = p;
            p = p.offset(1);
            *c2rust_fresh5 = 'n' as ::core::ffi::c_char;
        }
        if mode & MODE_OP_PENDING != 0 {
            let c2rust_fresh6 = p;
            p = p.offset(1);
            *c2rust_fresh6 = 'o' as ::core::ffi::c_char;
        }
        if mode & MODE_TERMINAL != 0 {
            let c2rust_fresh7 = p;
            p = p.offset(1);
            *c2rust_fresh7 = 't' as ::core::ffi::c_char;
        }
        if mode & (MODE_VISUAL | MODE_SELECT) == MODE_VISUAL | MODE_SELECT {
            let c2rust_fresh8 = p;
            p = p.offset(1);
            *c2rust_fresh8 = 'v' as ::core::ffi::c_char;
        } else {
            if mode & MODE_VISUAL != 0 {
                let c2rust_fresh9 = p;
                p = p.offset(1);
                *c2rust_fresh9 = 'x' as ::core::ffi::c_char;
            }
            if mode & MODE_SELECT != 0 {
                let c2rust_fresh10 = p;
                p = p.offset(1);
                *c2rust_fresh10 = 's' as ::core::ffi::c_char;
            }
        }
    }
    *p = NUL as ::core::ffi::c_char;
}
unsafe extern "C" fn showmap(mut mp: *mut mapblock_T, mut local: bool) {
    if message_filtered((*mp).m_keys) as ::core::ffi::c_int != 0
        && message_filtered((*mp).m_str) as ::core::ffi::c_int != 0
        && ((*mp).m_desc.is_null() || message_filtered((*mp).m_desc) as ::core::ffi::c_int != 0)
    {
        return;
    }
    if msg_col.get() > 0 as ::core::ffi::c_int || msg_silent.get() != 0 as ::core::ffi::c_int {
        msg_putchar('\n' as ::core::ffi::c_int);
        if got_int.get() {
            return;
        }
    }
    let mut mapchars: [::core::ffi::c_char; 7] = [0; 7];
    map_mode_to_chars((*mp).m_mode, &raw mut mapchars as *mut ::core::ffi::c_char);
    msg_puts(&raw mut mapchars as *mut ::core::ffi::c_char);
    let mut len: size_t = strlen(&raw mut mapchars as *mut ::core::ffi::c_char);
    loop {
        len = len.wrapping_add(1);
        if len > 3 as size_t {
            break;
        }
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    len = msg_outtrans_special((*mp).m_keys, true_0 != 0, 0 as ::core::ffi::c_int) as size_t;
    loop {
        msg_putchar(' ' as ::core::ffi::c_int);
        len = len.wrapping_add(1);
        if len >= 12 as size_t {
            break;
        }
    }
    if (*mp).m_noremap == REMAP_NONE as ::core::ffi::c_int {
        msg_puts_hl(
            b"*\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_8,
            false_0 != 0,
        );
    } else if (*mp).m_noremap == REMAP_SCRIPT as ::core::ffi::c_int {
        msg_puts_hl(
            b"&\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_8,
            false_0 != 0,
        );
    } else {
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    if local {
        msg_putchar('@' as ::core::ffi::c_int);
    } else {
        msg_putchar(' ' as ::core::ffi::c_int);
    }
    if (*mp).m_luaref != LUA_NOREF {
        let mut str: *mut ::core::ffi::c_char =
            nlua_funcref_str((*mp).m_luaref, ::core::ptr::null_mut::<Arena>());
        msg_puts_hl(str, HLF_8, false_0 != 0);
        xfree(str as *mut ::core::ffi::c_void);
    } else if *(*mp).m_str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        msg_puts_hl(
            b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_8,
            false_0 != 0,
        );
    } else {
        msg_outtrans_special((*mp).m_str, false_0 != 0, 0 as ::core::ffi::c_int);
    }
    if !(*mp).m_desc.is_null() {
        msg_puts(b"\n                 \0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts((*mp).m_desc);
    }
    if p_verbose.get() > 0 as OptInt {
        last_set_msg((*mp).m_script_ctx);
    }
    msg_clr_eos();
}
unsafe extern "C" fn set_maparg_lhs_rhs(
    orig_lhs: *const ::core::ffi::c_char,
    orig_lhs_len: size_t,
    orig_rhs: *const ::core::ffi::c_char,
    orig_rhs_len: size_t,
    rhs_lua: LuaRef,
    cpo_val: *const ::core::ffi::c_char,
    mapargs: *mut MapArguments,
) -> bool {
    (*mapargs).rhs_lua = rhs_lua;
    let mut lhs_buf: [::core::ffi::c_char; 128] = [0; 128];
    let mut did_simplify: bool = false_0 != 0;
    let flags: ::core::ffi::c_int =
        REPTERM_FROM_PART as ::core::ffi::c_int | REPTERM_DO_LT as ::core::ffi::c_int;
    let mut bufarg: *mut ::core::ffi::c_char = &raw mut lhs_buf as *mut ::core::ffi::c_char;
    let mut replaced: *mut ::core::ffi::c_char = replace_termcodes(
        orig_lhs,
        orig_lhs_len,
        &raw mut bufarg,
        0 as scid_T,
        flags,
        &raw mut did_simplify,
        cpo_val,
    );
    if replaced.is_null() {
        return false_0 != 0;
    }
    (*mapargs).lhs_len = strlen(replaced);
    xstrlcpy(
        &raw mut (*mapargs).lhs as *mut ::core::ffi::c_char,
        replaced,
        ::core::mem::size_of::<[::core::ffi::c_char; 51]>(),
    );
    if did_simplify {
        replaced = replace_termcodes(
            orig_lhs,
            orig_lhs_len,
            &raw mut bufarg,
            0 as scid_T,
            flags | REPTERM_NO_SIMPLIFY as ::core::ffi::c_int,
            ::core::ptr::null_mut::<bool>(),
            cpo_val,
        );
        if replaced.is_null() {
            return false_0 != 0;
        }
        (*mapargs).alt_lhs_len = strlen(replaced);
        xstrlcpy(
            &raw mut (*mapargs).alt_lhs as *mut ::core::ffi::c_char,
            replaced,
            ::core::mem::size_of::<[::core::ffi::c_char; 51]>(),
        );
    } else {
        (*mapargs).alt_lhs_len = 0 as size_t;
    }
    set_maparg_rhs(
        orig_rhs,
        orig_rhs_len,
        rhs_lua,
        0 as scid_T,
        cpo_val,
        mapargs,
    );
    return true_0 != 0;
}
unsafe extern "C" fn set_maparg_rhs(
    orig_rhs: *const ::core::ffi::c_char,
    orig_rhs_len: size_t,
    rhs_lua: LuaRef,
    sid: scid_T,
    cpo_val: *const ::core::ffi::c_char,
    mapargs: *mut MapArguments,
) {
    (*mapargs).rhs_lua = rhs_lua;
    if rhs_lua == LUA_NOREF {
        (*mapargs).orig_rhs_len = orig_rhs_len;
        (*mapargs).orig_rhs = xcalloc(
            (*mapargs).orig_rhs_len.wrapping_add(1 as size_t),
            ::core::mem::size_of::<::core::ffi::c_char>(),
        ) as *mut ::core::ffi::c_char;
        xmemcpyz(
            (*mapargs).orig_rhs as *mut ::core::ffi::c_void,
            orig_rhs as *const ::core::ffi::c_void,
            (*mapargs).orig_rhs_len,
        );
        if strcasecmp(
            orig_rhs as *mut ::core::ffi::c_char,
            b"<nop>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*mapargs).rhs = xcalloc(1 as size_t, ::core::mem::size_of::<::core::ffi::c_char>())
                as *mut ::core::ffi::c_char;
            (*mapargs).rhs_len = 0 as size_t;
            (*mapargs).rhs_is_noop = true_0 != 0;
        } else {
            let mut rhs_buf: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut replaced: *mut ::core::ffi::c_char = replace_termcodes(
                orig_rhs,
                orig_rhs_len,
                &raw mut rhs_buf,
                sid,
                REPTERM_DO_LT as ::core::ffi::c_int,
                ::core::ptr::null_mut::<bool>(),
                cpo_val,
            );
            (*mapargs).rhs_len = strlen(replaced);
            (*mapargs).rhs_is_noop =
                orig_rhs_len != 0 as size_t && (*mapargs).rhs_len == 0 as size_t;
            (*mapargs).rhs = replaced;
        }
    } else {
        let mut tmp_buf: [::core::ffi::c_char; 64] = [0; 64];
        (*mapargs).orig_rhs = xcalloc(1 as size_t, ::core::mem::size_of::<::core::ffi::c_char>())
            as *mut ::core::ffi::c_char;
        (*mapargs).orig_rhs_len = 0 as size_t;
        (*mapargs).rhs_len = vim_snprintf(
            &raw mut tmp_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(1 as size_t),
            b"%c%c%c%d\r\0".as_ptr() as *const ::core::ffi::c_char,
            K_SPECIAL,
            KS_EXTRA,
            KE_LUA as ::core::ffi::c_int,
            rhs_lua,
        ) as size_t;
        (*mapargs).rhs = xstrdup(&raw mut tmp_buf as *mut ::core::ffi::c_char);
    };
}
unsafe extern "C" fn str_to_mapargs(
    mut strargs: *const ::core::ffi::c_char,
    mut is_unmap: bool,
    mut mapargs: *mut MapArguments,
) -> ::core::ffi::c_int {
    let mut to_parse: *const ::core::ffi::c_char = strargs;
    to_parse = skipwhite(to_parse);
    memset(
        mapargs as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<MapArguments>(),
    );
    loop {
        if strncmp(
            to_parse,
            b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
            (*mapargs).buffer = true_0 != 0;
        } else if strncmp(
            to_parse,
            b"<nowait>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
            (*mapargs).nowait = true_0 != 0;
        } else if strncmp(
            to_parse,
            b"<silent>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
            (*mapargs).silent = true_0 != 0;
        } else if strncmp(
            to_parse,
            b"<special>\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(9 as ::core::ffi::c_int as isize));
        } else if strncmp(
            to_parse,
            b"<script>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
            (*mapargs).script = true_0 != 0;
        } else if strncmp(
            to_parse,
            b"<expr>\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            to_parse = skipwhite(to_parse.offset(6 as ::core::ffi::c_int as isize));
            (*mapargs).expr = true_0 != 0;
        } else {
            if strncmp(
                to_parse,
                b"<unique>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                break;
            }
            to_parse = skipwhite(to_parse.offset(8 as ::core::ffi::c_int as isize));
            (*mapargs).unique = true_0 != 0;
        }
    }
    let mut lhs_end: *const ::core::ffi::c_char = to_parse;
    let mut do_backslash: bool = vim_strchr(p_cpo.get(), CPO_BSLASH).is_null();
    while *lhs_end as ::core::ffi::c_int != 0
        && (is_unmap as ::core::ffi::c_int != 0 || !ascii_iswhite(*lhs_end as ::core::ffi::c_int))
    {
        if (*lhs_end.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == Ctrl_V
            || do_backslash as ::core::ffi::c_int != 0
                && *lhs_end.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int)
            && *lhs_end.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            lhs_end = lhs_end.offset(1);
        }
        lhs_end = lhs_end.offset(1);
    }
    let mut rhs_start: *const ::core::ffi::c_char = skipwhite(lhs_end);
    let mut orig_lhs_len: size_t = lhs_end.offset_from(to_parse) as size_t;
    if orig_lhs_len >= 256 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    let mut lhs_to_replace: [::core::ffi::c_char; 256] = [0; 256];
    xmemcpyz(
        &raw mut lhs_to_replace as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        to_parse as *const ::core::ffi::c_void,
        orig_lhs_len,
    );
    let mut orig_rhs_len: size_t = strlen(rhs_start);
    if !set_maparg_lhs_rhs(
        &raw mut lhs_to_replace as *mut ::core::ffi::c_char,
        orig_lhs_len,
        rhs_start,
        orig_rhs_len,
        LUA_NOREF,
        p_cpo.get(),
        mapargs,
    ) {
        return 1 as ::core::ffi::c_int;
    }
    if (*mapargs).lhs_len > MAXMAPLEN as ::core::ffi::c_int as size_t {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn map_add(
    mut buf: *mut buf_T,
    mut map_table: *mut *mut mapblock_T,
    mut abbr_table: *mut *mut mapblock_T,
    mut keys: *const ::core::ffi::c_char,
    mut args: *mut MapArguments,
    mut noremap: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
    mut is_abbr: bool,
    mut sid: scid_T,
    mut lnum: linenr_T,
    mut simplified: bool,
) -> *mut mapblock_T {
    let mut mp: *mut mapblock_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<mapblock_T>()) as *mut mapblock_T;
    if *keys as ::core::ffi::c_int == Ctrl_C {
        if map_table == &raw mut (*buf).b_maphash as *mut *mut mapblock_T {
            (*buf).b_mapped_ctrl_c |= mode;
        } else {
            (*mapped_ctrl_c.ptr()) |= mode;
        }
    }
    (*mp).m_keys = xstrdup(keys);
    (*mp).m_str = (*args).rhs;
    (*mp).m_orig_str = (*args).orig_rhs;
    (*mp).m_luaref = (*args).rhs_lua;
    (*mp).m_keylen = strlen((*mp).m_keys) as ::core::ffi::c_int;
    (*mp).m_noremap = noremap;
    (*mp).m_nowait = (*args).nowait as ::core::ffi::c_char;
    (*mp).m_silent = (*args).silent as ::core::ffi::c_char;
    (*mp).m_mode = mode;
    (*mp).m_simplified = simplified as ::core::ffi::c_int;
    (*mp).m_expr = (*args).expr as ::core::ffi::c_char;
    (*mp).m_replace_keycodes = (*args).replace_keycodes;
    if sid != 0 as ::core::ffi::c_int {
        (*mp).m_script_ctx.sc_sid = sid;
        (*mp).m_script_ctx.sc_lnum = lnum;
    } else {
        (*mp).m_script_ctx = current_sctx.get();
        (*mp).m_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
        nlua_set_sctx(&raw mut (*mp).m_script_ctx);
    }
    (*mp).m_desc = (*args).desc;
    if is_abbr {
        (*mp).m_next = *abbr_table;
        *abbr_table = mp;
    } else {
        let n: ::core::ffi::c_int = if (*mp).m_mode
            & (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING | MODE_TERMINAL)
            != 0
        {
            *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
        } else {
            *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                ^ 0x80 as ::core::ffi::c_int
        };
        (*mp).m_next = *map_table.offset(n as isize);
        *map_table.offset(n as isize) = mp;
    }
    return mp;
}
unsafe extern "C" fn buf_do_map(
    mut maptype: ::core::ffi::c_int,
    mut args: *mut MapArguments,
    mut mode: ::core::ffi::c_int,
    mut is_abbrev: bool,
    mut buf: *mut buf_T,
) -> ::core::ffi::c_int {
    let mut lhs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut did_simplify: bool = false;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut map_table: *mut *mut mapblock_T = if (*args).buffer as ::core::ffi::c_int != 0 {
        &raw mut (*buf).b_maphash as *mut *mut mapblock_T
    } else {
        maphash.ptr() as *mut *mut mapblock_T
    };
    let mut abbr_table: *mut *mut mapblock_T = if (*args).buffer as ::core::ffi::c_int != 0 {
        &raw mut (*buf).b_first_abbr
    } else {
        first_abbr.ptr()
    };
    let mut mp_result: [*mut mapblock_T; 2] = [
        ::core::ptr::null_mut::<mapblock_T>(),
        ::core::ptr::null_mut::<mapblock_T>(),
    ];
    let mut unmap_lhs_only: bool = false_0 != 0;
    if maptype == MAPTYPE_UNMAP_LHS as ::core::ffi::c_int {
        unmap_lhs_only = true_0 != 0;
        maptype = MAPTYPE_UNMAP as ::core::ffi::c_int;
    }
    let mut noremap: ::core::ffi::c_int = if (*args).script as ::core::ffi::c_int != 0 {
        REMAP_SCRIPT as ::core::ffi::c_int
    } else if maptype == MAPTYPE_NOREMAP as ::core::ffi::c_int {
        REMAP_NONE as ::core::ffi::c_int
    } else {
        REMAP_YES as ::core::ffi::c_int
    };
    let has_lhs: bool = (*args).lhs[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL;
    let has_rhs: bool = (*args).rhs_lua != LUA_NOREF
        || *(*args).rhs.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        || (*args).rhs_is_noop as ::core::ffi::c_int != 0;
    let do_print: bool = !has_lhs || maptype != MAPTYPE_UNMAP as ::core::ffi::c_int && !has_rhs;
    if do_print {
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    }
    '_theend: {
        if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int && !has_lhs {
            retval = 1 as ::core::ffi::c_int;
        } else {
            lhs = &raw mut (*args).lhs as *mut ::core::ffi::c_char;
            did_simplify = (*args).alt_lhs_len != 0 as size_t;
            let mut keyround: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while keyround <= 2 as ::core::ffi::c_int {
                let mut did_it: bool = false_0 != 0;
                let mut did_local: bool = false_0 != 0;
                let mut keyround1_simplified: bool =
                    keyround == 1 as ::core::ffi::c_int && did_simplify as ::core::ffi::c_int != 0;
                let mut len: ::core::ffi::c_int = (*args).lhs_len as ::core::ffi::c_int;
                if keyround == 2 as ::core::ffi::c_int {
                    if !did_simplify {
                        break;
                    }
                    lhs = &raw mut (*args).alt_lhs as *mut ::core::ffi::c_char;
                    len = (*args).alt_lhs_len as ::core::ffi::c_int;
                } else if did_simplify as ::core::ffi::c_int != 0
                    && do_print as ::core::ffi::c_int != 0
                {
                    lhs = &raw mut (*args).alt_lhs as *mut ::core::ffi::c_char;
                    len = (*args).alt_lhs_len as ::core::ffi::c_int;
                }
                's_209: {
                    if has_lhs {
                        if len > MAXMAPLEN as ::core::ffi::c_int {
                            retval = 1 as ::core::ffi::c_int;
                            break '_theend;
                        } else if is_abbrev as ::core::ffi::c_int != 0
                            && maptype != MAPTYPE_UNMAP as ::core::ffi::c_int
                        {
                            let mut same: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                            let first: ::core::ffi::c_int = vim_iswordp(lhs) as ::core::ffi::c_int;
                            let mut last: ::core::ffi::c_int = first;
                            let mut p: *const ::core::ffi::c_char =
                                lhs.offset(utfc_ptr2len(lhs) as isize);
                            let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            while p < lhs.offset(len as isize) {
                                n += 1;
                                last = vim_iswordp(p) as ::core::ffi::c_int;
                                if same == -1 as ::core::ffi::c_int && last != first {
                                    same = n - 1 as ::core::ffi::c_int;
                                }
                                p = p.offset(utfc_ptr2len(p) as isize);
                            }
                            if last != 0
                                && n > 2 as ::core::ffi::c_int
                                && same >= 0 as ::core::ffi::c_int
                                && same < n - 1 as ::core::ffi::c_int
                            {
                                retval = 1 as ::core::ffi::c_int;
                                break '_theend;
                            } else {
                                n = 0 as ::core::ffi::c_int;
                                loop {
                                    if n >= len {
                                        break 's_209;
                                    }
                                    if ascii_iswhite(*lhs.offset(n as isize) as ::core::ffi::c_int)
                                    {
                                        retval = 1 as ::core::ffi::c_int;
                                        break '_theend;
                                    } else {
                                        n += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                if has_lhs as ::core::ffi::c_int != 0
                    && has_rhs as ::core::ffi::c_int != 0
                    && is_abbrev as ::core::ffi::c_int != 0
                {
                    no_abbr.set(false_0 != 0);
                }
                if do_print {
                    msg_start();
                }
                's_299: {
                    if (*args).unique as ::core::ffi::c_int != 0
                        && map_table == &raw mut (*buf).b_maphash as *mut *mut mapblock_T
                        && has_lhs as ::core::ffi::c_int != 0
                        && has_rhs as ::core::ffi::c_int != 0
                        && maptype != MAPTYPE_UNMAP as ::core::ffi::c_int
                    {
                        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        loop {
                            if !(hash < 256 as ::core::ffi::c_int && !got_int.get()) {
                                break 's_299;
                            }
                            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
                            if is_abbrev {
                                if hash != 0 as ::core::ffi::c_int {
                                    break 's_299;
                                }
                                mp = first_abbr.get();
                            } else {
                                mp = (*maphash.ptr())[hash as usize] as *mut mapblock_T;
                            }
                            while !mp.is_null() && !got_int.get() {
                                if (*mp).m_mode & mode != 0 as ::core::ffi::c_int
                                    && (*mp).m_keylen == len
                                    && strncmp((*mp).m_keys, lhs, len as size_t)
                                        == 0 as ::core::ffi::c_int
                                {
                                    retval = 6 as ::core::ffi::c_int;
                                    break '_theend;
                                } else {
                                    mp = (*mp).m_next;
                                }
                            }
                            hash += 1;
                        }
                    }
                }
                if map_table != &raw mut (*buf).b_maphash as *mut *mut mapblock_T
                    && !has_rhs
                    && maptype != MAPTYPE_UNMAP as ::core::ffi::c_int
                {
                    let mut hash_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while hash_0 < 256 as ::core::ffi::c_int && !got_int.get() {
                        let mut mp_0: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
                        if is_abbrev {
                            if hash_0 != 0 as ::core::ffi::c_int {
                                break;
                            }
                            mp_0 = (*buf).b_first_abbr;
                        } else {
                            mp_0 = (*buf).b_maphash[hash_0 as usize] as *mut mapblock_T;
                        }
                        while !mp_0.is_null() && !got_int.get() {
                            if (*mp_0).m_simplified == 0
                                && (*mp_0).m_mode & mode != 0 as ::core::ffi::c_int
                            {
                                if !has_lhs {
                                    showmap(mp_0, true_0 != 0);
                                    did_local = true_0 != 0;
                                } else {
                                    let mut n_0: ::core::ffi::c_int = (*mp_0).m_keylen;
                                    if strncmp(
                                        (*mp_0).m_keys,
                                        lhs,
                                        (if n_0 < len { n_0 } else { len }) as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        showmap(mp_0, true_0 != 0);
                                        did_local = true_0 != 0;
                                    }
                                }
                            }
                            mp_0 = (*mp_0).m_next;
                        }
                        hash_0 += 1;
                    }
                }
                let num_rounds: ::core::ffi::c_int =
                    if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int && !unmap_lhs_only {
                        2 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    };
                let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while round < num_rounds && !did_it && !got_int.get() {
                    let mut hash_start: ::core::ffi::c_int = 0;
                    let mut hash_end: ::core::ffi::c_int = 0;
                    if round == 0 as ::core::ffi::c_int && has_lhs as ::core::ffi::c_int != 0
                        || is_abbrev as ::core::ffi::c_int != 0
                    {
                        hash_start = if is_abbrev as ::core::ffi::c_int != 0 {
                            0 as ::core::ffi::c_int
                        } else if mode
                            & (MODE_NORMAL
                                | MODE_VISUAL
                                | MODE_SELECT
                                | MODE_OP_PENDING
                                | MODE_TERMINAL)
                            != 0
                        {
                            *lhs.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                        } else {
                            *lhs.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                                ^ 0x80 as ::core::ffi::c_int
                        };
                        hash_end = hash_start + 1 as ::core::ffi::c_int;
                    } else {
                        hash_start = 0 as ::core::ffi::c_int;
                        hash_end = 256 as ::core::ffi::c_int;
                    }
                    let mut hash_1: ::core::ffi::c_int = hash_start;
                    while hash_1 < hash_end && !got_int.get() {
                        let mut mpp: *mut *mut mapblock_T = if is_abbrev as ::core::ffi::c_int != 0
                        {
                            abbr_table
                        } else {
                            map_table.offset(hash_1 as isize)
                        };
                        let mut mp_1: *mut mapblock_T = *mpp;
                        's_448: while !mp_1.is_null() && !got_int.get() {
                            's_458: {
                                if (*mp_1).m_mode & mode == 0 as ::core::ffi::c_int {
                                    mpp = &raw mut (*mp_1).m_next;
                                } else {
                                    if !has_lhs {
                                        if (*mp_1).m_simplified == 0 {
                                            showmap(
                                                mp_1,
                                                map_table != maphash.ptr() as *mut *mut mapblock_T,
                                            );
                                            did_it = true_0 != 0;
                                        }
                                    } else {
                                        let mut n_1: ::core::ffi::c_int = 0;
                                        let mut p_0: *const ::core::ffi::c_char =
                                            ::core::ptr::null::<::core::ffi::c_char>();
                                        if round != 0 {
                                            n_1 = strlen((*mp_1).m_str) as ::core::ffi::c_int;
                                            p_0 = (*mp_1).m_str;
                                        } else {
                                            n_1 = (*mp_1).m_keylen;
                                            p_0 = (*mp_1).m_keys;
                                        }
                                        if strncmp(
                                            p_0,
                                            lhs,
                                            (if n_1 < len { n_1 } else { len }) as size_t,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int {
                                                if n_1 != len
                                                    && (!is_abbrev
                                                        || round != 0
                                                        || n_1 > len
                                                        || *skipwhite(lhs.offset(n_1 as isize))
                                                            as ::core::ffi::c_int
                                                            != NUL)
                                                {
                                                    mpp = &raw mut (*mp_1).m_next;
                                                    break 's_458;
                                                } else {
                                                    if keyround1_simplified as ::core::ffi::c_int
                                                        != 0
                                                        && (*mp_1).m_simplified == 0
                                                    {
                                                        break 's_448;
                                                    }
                                                    (*mp_1).m_mode &= !mode;
                                                    did_it = true_0 != 0;
                                                }
                                            } else if !has_rhs {
                                                if (*mp_1).m_simplified == 0 {
                                                    showmap(
                                                        mp_1,
                                                        map_table
                                                            != maphash.ptr()
                                                                as *mut *mut mapblock_T,
                                                    );
                                                    did_it = true_0 != 0;
                                                }
                                            } else if n_1 != len {
                                                mpp = &raw mut (*mp_1).m_next;
                                                break 's_458;
                                            } else if keyround1_simplified as ::core::ffi::c_int
                                                != 0
                                                && (*mp_1).m_simplified == 0
                                            {
                                                did_it = true_0 != 0;
                                                break 's_448;
                                            } else if (*args).unique {
                                                retval = 5 as ::core::ffi::c_int;
                                                break '_theend;
                                            } else {
                                                (*mp_1).m_mode &= !mode;
                                                if (*mp_1).m_mode == 0 as ::core::ffi::c_int
                                                    && !did_it
                                                {
                                                    if !(*mp_1).m_alt.is_null() {
                                                        (*(*mp_1).m_alt).m_alt =
                                                            ::core::ptr::null_mut::<mapblock_T>();
                                                        (*mp_1).m_alt = (*(*mp_1).m_alt).m_alt;
                                                    } else {
                                                        if (*mp_1).m_luaref != LUA_NOREF {
                                                            api_free_luaref((*mp_1).m_luaref);
                                                            (*mp_1).m_luaref = LUA_NOREF as LuaRef;
                                                        }
                                                        xfree(
                                                            (*mp_1).m_str
                                                                as *mut ::core::ffi::c_void,
                                                        );
                                                        xfree(
                                                            (*mp_1).m_orig_str
                                                                as *mut ::core::ffi::c_void,
                                                        );
                                                        xfree(
                                                            (*mp_1).m_desc
                                                                as *mut ::core::ffi::c_void,
                                                        );
                                                    }
                                                    (*mp_1).m_str = (*args).rhs;
                                                    (*mp_1).m_orig_str = (*args).orig_rhs;
                                                    (*mp_1).m_luaref = (*args).rhs_lua;
                                                    (*mp_1).m_noremap = noremap;
                                                    (*mp_1).m_nowait =
                                                        (*args).nowait as ::core::ffi::c_char;
                                                    (*mp_1).m_silent =
                                                        (*args).silent as ::core::ffi::c_char;
                                                    (*mp_1).m_mode = mode;
                                                    (*mp_1).m_simplified =
                                                        keyround1_simplified as ::core::ffi::c_int;
                                                    (*mp_1).m_expr =
                                                        (*args).expr as ::core::ffi::c_char;
                                                    (*mp_1).m_replace_keycodes =
                                                        (*args).replace_keycodes;
                                                    (*mp_1).m_script_ctx = current_sctx.get();
                                                    (*mp_1).m_script_ctx.sc_lnum +=
                                                        (*((*exestack.ptr()).ga_data
                                                            as *mut estack_T)
                                                            .offset(
                                                                ((*exestack.ptr()).ga_len
                                                                    - 1 as ::core::ffi::c_int)
                                                                    as isize,
                                                            ))
                                                        .es_lnum;
                                                    nlua_set_sctx(&raw mut (*mp_1).m_script_ctx);
                                                    (*mp_1).m_desc = (*args).desc;
                                                    mp_result[(keyround - 1 as ::core::ffi::c_int)
                                                        as usize] = mp_1;
                                                    did_it = true_0 != 0;
                                                }
                                            }
                                            if (*mp_1).m_mode == 0 as ::core::ffi::c_int {
                                                mapblock_free(mpp);
                                                break 's_458;
                                            } else {
                                                let mut new_hash: ::core::ffi::c_int = if (*mp_1)
                                                    .m_mode
                                                    & (MODE_NORMAL
                                                        | MODE_VISUAL
                                                        | MODE_SELECT
                                                        | MODE_OP_PENDING
                                                        | MODE_TERMINAL)
                                                    != 0
                                                {
                                                    *(*mp_1)
                                                        .m_keys
                                                        .offset(0 as ::core::ffi::c_int as isize)
                                                        as uint8_t
                                                        as ::core::ffi::c_int
                                                } else {
                                                    *(*mp_1)
                                                        .m_keys
                                                        .offset(0 as ::core::ffi::c_int as isize)
                                                        as uint8_t
                                                        as ::core::ffi::c_int
                                                        ^ 0x80 as ::core::ffi::c_int
                                                };
                                                if !is_abbrev && new_hash != hash_1 {
                                                    *mpp = (*mp_1).m_next;
                                                    (*mp_1).m_next =
                                                        *map_table.offset(new_hash as isize);
                                                    *map_table.offset(new_hash as isize) = mp_1;
                                                    break 's_458;
                                                }
                                            }
                                        }
                                    }
                                    mpp = &raw mut (*mp_1).m_next;
                                }
                            }
                            mp_1 = *mpp;
                        }
                        hash_1 += 1;
                    }
                    round += 1;
                }
                if maptype == MAPTYPE_UNMAP as ::core::ffi::c_int {
                    if !did_it {
                        if !keyround1_simplified {
                            retval = 2 as ::core::ffi::c_int;
                        }
                    } else if *lhs as ::core::ffi::c_int == Ctrl_C {
                        if map_table == &raw mut (*buf).b_maphash as *mut *mut mapblock_T {
                            (*buf).b_mapped_ctrl_c &= !mode;
                        } else {
                            (*mapped_ctrl_c.ptr()) &= !mode;
                        }
                    }
                } else if !has_lhs || !has_rhs {
                    if !did_it && !did_local {
                        if is_abbrev {
                            msg(
                                gettext(b"No abbreviation found\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                0 as ::core::ffi::c_int,
                            );
                        } else {
                            msg(
                                gettext(
                                    b"No mapping found\0".as_ptr() as *const ::core::ffi::c_char
                                ),
                                0 as ::core::ffi::c_int,
                            );
                        }
                    }
                    break '_theend;
                } else if !did_it {
                    mp_result[(keyround - 1 as ::core::ffi::c_int) as usize] = map_add(
                        buf,
                        map_table,
                        abbr_table,
                        lhs,
                        args,
                        noremap,
                        mode,
                        is_abbrev,
                        0 as scid_T,
                        0 as linenr_T,
                        keyround1_simplified,
                    );
                }
                keyround += 1;
            }
            if !mp_result[0 as ::core::ffi::c_int as usize].is_null()
                && !mp_result[1 as ::core::ffi::c_int as usize].is_null()
            {
                (*mp_result[0 as ::core::ffi::c_int as usize]).m_alt =
                    mp_result[1 as ::core::ffi::c_int as usize];
                (*mp_result[1 as ::core::ffi::c_int as usize]).m_alt =
                    mp_result[0 as ::core::ffi::c_int as usize];
            }
        }
    }
    if !mp_result[0 as ::core::ffi::c_int as usize].is_null()
        || !mp_result[1 as ::core::ffi::c_int as usize].is_null()
    {
        (*args).rhs = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*args).orig_rhs = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*args).rhs_lua = LUA_NOREF as LuaRef;
        (*args).desc = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return retval;
}
pub unsafe extern "C" fn do_map(
    mut maptype: ::core::ffi::c_int,
    mut arg: *mut ::core::ffi::c_char,
    mut mode: ::core::ffi::c_int,
    mut is_abbrev: bool,
) -> ::core::ffi::c_int {
    let mut parsed_args: MapArguments = MapArguments {
        buffer: false,
        expr: false,
        noremap: false,
        nowait: false,
        script: false,
        silent: false,
        unique: false,
        replace_keycodes: false,
        lhs: [0; 51],
        lhs_len: 0,
        alt_lhs: [0; 51],
        alt_lhs_len: 0,
        rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        rhs_len: 0,
        rhs_lua: 0,
        rhs_is_noop: false,
        orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        orig_rhs_len: 0,
        desc: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut result: ::core::ffi::c_int = str_to_mapargs(
        arg,
        maptype == MAPTYPE_UNMAP as ::core::ffi::c_int,
        &raw mut parsed_args,
    );
    match result {
        0 => {
            result = buf_do_map(maptype, &raw mut parsed_args, mode, is_abbrev, curbuf.get());
        }
        1 => {}
        _ => {
            '_c2rust_label: {
                if false {
                } else {
                    __assert_fail(
                        b"false && \"Unknown return code from str_to_mapargs!\"\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/mapping.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        968 as ::core::ffi::c_uint,
                        b"int do_map(int, char *, int, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            result = -1 as ::core::ffi::c_int;
        }
    }
    xfree(parsed_args.rhs as *mut ::core::ffi::c_void);
    xfree(parsed_args.orig_rhs as *mut ::core::ffi::c_void);
    return result;
}
unsafe extern "C" fn get_map_mode(
    mut cmdp: *mut *mut ::core::ffi::c_char,
    mut forceit: bool,
) -> ::core::ffi::c_int {
    let mut mode: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = *cmdp;
    let c2rust_fresh11 = p;
    p = p.offset(1);
    let mut modec: ::core::ffi::c_int = *c2rust_fresh11 as uint8_t as ::core::ffi::c_int;
    if modec == 'i' as ::core::ffi::c_int {
        mode = MODE_INSERT;
    } else if modec == 'l' as ::core::ffi::c_int {
        mode = MODE_LANGMAP;
    } else if modec == 'c' as ::core::ffi::c_int {
        mode = MODE_CMDLINE;
    } else if modec == 'n' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != 'o' as ::core::ffi::c_int
    {
        mode = MODE_NORMAL;
    } else if modec == 'v' as ::core::ffi::c_int {
        mode = MODE_VISUAL | MODE_SELECT;
    } else if modec == 'x' as ::core::ffi::c_int {
        mode = MODE_VISUAL;
    } else if modec == 's' as ::core::ffi::c_int {
        mode = MODE_SELECT;
    } else if modec == 'o' as ::core::ffi::c_int {
        mode = MODE_OP_PENDING;
    } else if modec == 't' as ::core::ffi::c_int {
        mode = MODE_TERMINAL;
    } else {
        p = p.offset(-1);
        if forceit {
            mode = MODE_INSERT | MODE_CMDLINE;
        } else {
            mode = MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
        }
    }
    *cmdp = p;
    return mode;
}
unsafe extern "C" fn do_mapclear(
    mut cmdp: *mut ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_char,
    mut forceit: ::core::ffi::c_int,
    mut abbr: ::core::ffi::c_int,
) {
    let mut local: bool = strcmp(arg, b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int;
    if !local && *arg as ::core::ffi::c_int != NUL {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut mode: ::core::ffi::c_int = get_map_mode(&raw mut cmdp, forceit != 0);
    map_clear_mode(curbuf.get(), mode, local, abbr != 0);
}
pub unsafe extern "C" fn map_clear_mode(
    mut buf: *mut buf_T,
    mut mode: ::core::ffi::c_int,
    mut local: bool,
    mut abbr: bool,
) {
    let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while hash < 256 as ::core::ffi::c_int {
        let mut mpp: *mut *mut mapblock_T = ::core::ptr::null_mut::<*mut mapblock_T>();
        if abbr {
            if hash > 0 as ::core::ffi::c_int {
                break;
            }
            if local {
                mpp = &raw mut (*buf).b_first_abbr;
            } else {
                mpp = first_abbr.ptr();
            }
        } else if local {
            mpp = (&raw mut (*buf).b_maphash as *mut *mut mapblock_T).offset(hash as isize)
                as *mut *mut mapblock_T;
        } else {
            mpp = (maphash.ptr() as *mut *mut mapblock_T).offset(hash as isize)
                as *mut *mut mapblock_T;
        }
        while !(*mpp).is_null() {
            let mut mp: *mut mapblock_T = *mpp;
            if (*mp).m_mode & mode != 0 {
                (*mp).m_mode &= !mode;
                if (*mp).m_mode == 0 as ::core::ffi::c_int {
                    mapblock_free(mpp);
                    continue;
                } else {
                    let mut new_hash: ::core::ffi::c_int = if (*mp).m_mode
                        & (MODE_NORMAL
                            | MODE_VISUAL
                            | MODE_SELECT
                            | MODE_OP_PENDING
                            | MODE_TERMINAL)
                        != 0
                    {
                        *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                    } else {
                        *(*mp).m_keys.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                            ^ 0x80 as ::core::ffi::c_int
                    };
                    if !abbr && new_hash != hash {
                        *mpp = (*mp).m_next;
                        if local {
                            (*mp).m_next = (*buf).b_maphash[new_hash as usize] as *mut mapblock_T;
                            (*buf).b_maphash[new_hash as usize] = mp as *mut mapblock_T;
                        } else {
                            (*mp).m_next = (*maphash.ptr())[new_hash as usize] as *mut mapblock_T;
                            (*maphash.ptr())[new_hash as usize] = mp as *mut mapblock_T;
                        }
                        continue;
                    }
                }
            }
            mpp = &raw mut (*mp).m_next;
        }
        hash += 1;
    }
}
pub unsafe extern "C" fn map_to_exists(
    str: *const ::core::ffi::c_char,
    modechars: *const ::core::ffi::c_char,
    abbr: bool,
) -> bool {
    let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let rhs: *const ::core::ffi::c_char = replace_termcodes(
        str,
        strlen(str),
        &raw mut buf,
        0 as scid_T,
        REPTERM_DO_LT as ::core::ffi::c_int,
        ::core::ptr::null_mut::<bool>(),
        p_cpo.get(),
    );
    if !strchr(modechars, 'n' as ::core::ffi::c_int).is_null() {
        mode |= MODE_NORMAL;
    }
    if !strchr(modechars, 'v' as ::core::ffi::c_int).is_null() {
        mode |= MODE_VISUAL | MODE_SELECT;
    }
    if !strchr(modechars, 'x' as ::core::ffi::c_int).is_null() {
        mode |= MODE_VISUAL;
    }
    if !strchr(modechars, 's' as ::core::ffi::c_int).is_null() {
        mode |= MODE_SELECT;
    }
    if !strchr(modechars, 'o' as ::core::ffi::c_int).is_null() {
        mode |= MODE_OP_PENDING;
    }
    if !strchr(modechars, 'i' as ::core::ffi::c_int).is_null() {
        mode |= MODE_INSERT;
    }
    if !strchr(modechars, 'l' as ::core::ffi::c_int).is_null() {
        mode |= MODE_LANGMAP;
    }
    if !strchr(modechars, 'c' as ::core::ffi::c_int).is_null() {
        mode |= MODE_CMDLINE;
    }
    let mut retval: bool = map_to_exists_mode(rhs, mode, abbr);
    xfree(buf as *mut ::core::ffi::c_void);
    return retval;
}
pub unsafe extern "C" fn map_to_exists_mode(
    rhs: *const ::core::ffi::c_char,
    mode: ::core::ffi::c_int,
    abbr: bool,
) -> bool {
    let mut exp_buffer: bool = false_0 != 0;
    loop {
        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while hash < 256 as ::core::ffi::c_int {
            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
            if abbr {
                if hash > 0 as ::core::ffi::c_int {
                    break;
                }
                if exp_buffer {
                    mp = (*curbuf.get()).b_first_abbr;
                } else {
                    mp = first_abbr.get();
                }
            } else if exp_buffer {
                mp = (*curbuf.get()).b_maphash[hash as usize] as *mut mapblock_T;
            } else {
                mp = (*maphash.ptr())[hash as usize] as *mut mapblock_T;
            }
            while !mp.is_null() {
                if (*mp).m_mode & mode != 0 && !strstr((*mp).m_str, rhs).is_null() {
                    return true_0 != 0;
                }
                mp = (*mp).m_next;
            }
            hash += 1;
        }
        if exp_buffer {
            break;
        }
        exp_buffer = true_0 != 0;
    }
    return false_0 != 0;
}
static expand_mapmodes: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static expand_isabbrev: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static expand_buffer: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
unsafe extern "C" fn translate_mapping(
    str_in: *const ::core::ffi::c_char,
    cpo_val: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut str: *const uint8_t = str_in as *const uint8_t;
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
        40 as ::core::ffi::c_int,
    );
    let cpo_bslash: bool = !vim_strchr(cpo_val, CPO_BSLASH).is_null();
    while *str != 0 {
        let mut c: ::core::ffi::c_int = *str as ::core::ffi::c_int;
        's_13: {
            if c == K_SPECIAL
                && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == KS_MODIFIER
                {
                    str = str.offset(1);
                    str = str.offset(1);
                    modifiers = *str as ::core::ffi::c_int;
                    str = str.offset(1);
                    c = *str as ::core::ffi::c_int;
                }
                if c == K_SPECIAL
                    && *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                    && *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    c = if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == KS_SPECIAL
                    {
                        K_SPECIAL
                    } else if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == KS_ZERO
                    {
                        K_ZERO
                    } else {
                        -(*str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            + ((*str.offset(2 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int)
                                << 8 as ::core::ffi::c_int))
                    };
                    if c == K_ZERO {
                        c = NUL;
                    }
                    str = str.offset(2 as ::core::ffi::c_int as isize);
                }
                if c < 0 as ::core::ffi::c_int || modifiers != 0 {
                    ga_concat(&raw mut ga, get_special_key_name(c, modifiers));
                    break 's_13;
                }
            }
            if c == ' ' as ::core::ffi::c_int
                || c == '\t' as ::core::ffi::c_int
                || c == Ctrl_J
                || c == Ctrl_V
                || c == '<' as ::core::ffi::c_int
                || c == '\\' as ::core::ffi::c_int && !cpo_bslash
            {
                ga_append(
                    &raw mut ga,
                    (if cpo_bslash as ::core::ffi::c_int != 0 {
                        Ctrl_V
                    } else {
                        '\\' as ::core::ffi::c_int
                    }) as uint8_t,
                );
            }
            if c != 0 {
                ga_append(&raw mut ga, c as uint8_t);
            }
        }
        str = str.offset(1);
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    return ga.ga_data as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn set_context_in_map_cmd(
    mut xp: *mut expand_T,
    mut cmd: *mut ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_char,
    mut forceit: bool,
    mut isabbrev: bool,
    mut isunmap: bool,
    mut cmdidx: cmdidx_T,
) -> *mut ::core::ffi::c_char {
    if forceit as ::core::ffi::c_int != 0
        && cmdidx as ::core::ffi::c_int != CMD_map as ::core::ffi::c_int
        && cmdidx as ::core::ffi::c_int != CMD_unmap as ::core::ffi::c_int
    {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    } else {
        if isunmap {
            expand_mapmodes.set(get_map_mode(
                &raw mut cmd,
                forceit as ::core::ffi::c_int != 0 || isabbrev as ::core::ffi::c_int != 0,
            ));
        } else {
            expand_mapmodes.set(MODE_INSERT | MODE_CMDLINE);
            if !isabbrev {
                (*expand_mapmodes.ptr()) |=
                    MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
            }
        }
        expand_isabbrev.set(isabbrev);
        (*xp).xp_context = EXPAND_MAPPINGS as ::core::ffi::c_int;
        expand_buffer.set(false_0 != 0);
        loop {
            if strncmp(
                arg,
                b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                expand_buffer.set(true_0 != 0);
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"<unique>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"<nowait>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"<silent>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"<special>\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = skipwhite(arg.offset(9 as ::core::ffi::c_int as isize));
            } else if strncmp(
                arg,
                b"<script>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else {
                if strncmp(
                    arg,
                    b"<expr>\0".as_ptr() as *const ::core::ffi::c_char,
                    6 as size_t,
                ) != 0 as ::core::ffi::c_int
                {
                    break;
                }
                arg = skipwhite(arg.offset(6 as ::core::ffi::c_int as isize));
            }
        }
        (*xp).xp_pattern = arg;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn ExpandMappings(
    mut pat: *mut ::core::ffi::c_char,
    mut regmatch: *mut regmatch_T,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let fuzzy: bool = cmdline_fuzzy_complete(pat);
    *numMatches = 0 as ::core::ffi::c_int;
    *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if !fuzzy {
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            3 as ::core::ffi::c_int,
        );
    } else {
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<fuzmatch_str_T>() as ::core::ffi::c_int,
            3 as ::core::ffi::c_int,
        );
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 7 as ::core::ffi::c_int {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        's_34: {
            if i == 0 as ::core::ffi::c_int {
                p = b"<silent>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if i == 1 as ::core::ffi::c_int {
                p = b"<unique>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if i == 2 as ::core::ffi::c_int {
                p = b"<script>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if i == 3 as ::core::ffi::c_int {
                p = b"<expr>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else if i == 4 as ::core::ffi::c_int && !expand_buffer.get() {
                p = b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if i == 5 as ::core::ffi::c_int {
                p = b"<nowait>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else if i == 6 as ::core::ffi::c_int {
                p = b"<special>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            } else {
                break 's_34;
            }
            let mut match_0: bool = false;
            let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if !fuzzy {
                match_0 = vim_regexec(regmatch, p, 0 as colnr_T);
            } else {
                score = fuzzy_match_str(p, pat);
                match_0 = score != FUZZY_SCORE_NONE as ::core::ffi::c_int;
            }
            if match_0 {
                if fuzzy {
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) =
                        fuzmatch_str_T {
                            idx: ga.ga_len,
                            str: xstrdup(p),
                            score: score,
                        };
                    ga.ga_len += 1;
                } else {
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) =
                        xstrdup(p);
                    ga.ga_len += 1;
                }
            }
        }
        i += 1;
    }
    let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while hash < 256 as ::core::ffi::c_int {
        let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
        if expand_isabbrev.get() {
            if hash > 0 as ::core::ffi::c_int {
                break;
            } else {
                mp = first_abbr.get();
            }
        } else if expand_buffer.get() {
            mp = (*curbuf.get()).b_maphash[hash as usize] as *mut mapblock_T;
        } else {
            mp = (*maphash.ptr())[hash as usize] as *mut mapblock_T;
        }
        while !mp.is_null() {
            if !((*mp).m_simplified != 0 || (*mp).m_mode & expand_mapmodes.get() == 0) {
                let mut p_0: *mut ::core::ffi::c_char =
                    translate_mapping((*mp).m_keys, p_cpo.get());
                if !p_0.is_null() {
                    let mut match_1: bool = false;
                    let mut score_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if !fuzzy {
                        match_1 = vim_regexec(regmatch, p_0, 0 as colnr_T);
                    } else {
                        score_0 = fuzzy_match_str(p_0, pat);
                        match_1 = score_0 != FUZZY_SCORE_NONE as ::core::ffi::c_int;
                    }
                    if !match_1 {
                        xfree(p_0 as *mut ::core::ffi::c_void);
                    } else if fuzzy {
                        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                        *(ga.ga_data as *mut fuzmatch_str_T).offset(ga.ga_len as isize) =
                            fuzmatch_str_T {
                                idx: ga.ga_len,
                                str: p_0,
                                score: score_0,
                            };
                        ga.ga_len += 1;
                    } else {
                        ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                        *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) =
                            p_0;
                        ga.ga_len += 1;
                    }
                }
            }
            mp = (*mp).m_next;
        }
        hash += 1;
    }
    if ga.ga_len == 0 as ::core::ffi::c_int {
        return FAIL;
    }
    if !fuzzy {
        *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
        *numMatches = ga.ga_len;
    } else {
        fuzzymatches_to_strmatches(
            ga.ga_data as *mut fuzmatch_str_T,
            matches,
            ga.ga_len,
            false_0 != 0,
        );
        *numMatches = ga.ga_len;
    }
    let mut count: ::core::ffi::c_int = *numMatches;
    if count > 1 as ::core::ffi::c_int {
        if !fuzzy {
            sort_strings(*matches, count);
        }
        let mut ptr1: *mut *mut ::core::ffi::c_char = *matches;
        let mut ptr2: *mut *mut ::core::ffi::c_char = ptr1.offset(1 as ::core::ffi::c_int as isize);
        let mut ptr3: *mut *mut ::core::ffi::c_char = ptr1.offset(count as isize);
        while ptr2 < ptr3 {
            if strcmp(*ptr1, *ptr2) != 0 as ::core::ffi::c_int {
                let c2rust_fresh12 = ptr2;
                ptr2 = ptr2.offset(1);
                ptr1 = ptr1.offset(1);
                let c2rust_lvalue_ptr = &raw mut *ptr1;
                *c2rust_lvalue_ptr = *c2rust_fresh12;
            } else {
                let c2rust_fresh13 = ptr2;
                ptr2 = ptr2.offset(1);
                xfree(*c2rust_fresh13 as *mut ::core::ffi::c_void);
                count -= 1;
            }
        }
    }
    *numMatches = count;
    return if count == 0 as ::core::ffi::c_int {
        FAIL
    } else {
        OK
    };
}
pub unsafe extern "C" fn check_abbr(
    mut c: ::core::ffi::c_int,
    mut ptr: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut mincol: ::core::ffi::c_int,
) -> bool {
    let mut tb: [uint8_t; 25] = [0; 25];
    let mut clen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*typebuf.ptr()).tb_no_abbr_cnt != 0 {
        return false_0 != 0;
    }
    if noremap_keys() as ::core::ffi::c_int != 0 && c != Ctrl_RSB {
        return false_0 != 0;
    }
    if col == 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut scol: ::core::ffi::c_int = 0;
    let mut is_id: bool = true_0 != 0;
    let mut vim_abbr: bool = false;
    let mut p: *mut ::core::ffi::c_char = mb_prevptr(ptr, ptr.offset(col as isize));
    if !vim_iswordp(p) {
        vim_abbr = true_0 != 0;
    } else {
        vim_abbr = false_0 != 0;
        if p > ptr {
            is_id = vim_iswordp(mb_prevptr(ptr, p));
        }
    }
    clen = 1 as ::core::ffi::c_int;
    while p > ptr.offset(mincol as isize) {
        p = mb_prevptr(ptr, p);
        if ascii_isspace(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || !vim_abbr && is_id as ::core::ffi::c_int != vim_iswordp(p) as ::core::ffi::c_int
        {
            p = p.offset(utfc_ptr2len(p) as isize);
            break;
        } else {
            clen += 1;
        }
    }
    scol = p.offset_from(ptr) as ::core::ffi::c_int;
    if scol < mincol {
        scol = mincol;
    }
    if scol < col {
        ptr = ptr.offset(scol as isize);
        let mut len: ::core::ffi::c_int = col - scol;
        let mut mp: *mut mapblock_T = (*curbuf.get()).b_first_abbr;
        let mut mp2: *mut mapblock_T = first_abbr.get();
        if mp.is_null() {
            mp = mp2;
            mp2 = ::core::ptr::null_mut::<mapblock_T>();
        }
        while !mp.is_null() {
            let mut qlen: ::core::ffi::c_int = (*mp).m_keylen;
            let mut q: *mut ::core::ffi::c_char = (*mp).m_keys;
            if !strchr((*mp).m_keys, K_SPECIAL).is_null() {
                q = xstrdup((*mp).m_keys);
                vim_unescape_ks(q);
                qlen = strlen(q) as ::core::ffi::c_int;
            }
            let mut match_0: ::core::ffi::c_int = ((*mp).m_mode & State.get() != 0
                && qlen == len
                && strncmp(q, ptr, len as size_t) == 0)
                as ::core::ffi::c_int;
            if q != (*mp).m_keys {
                xfree(q as *mut ::core::ffi::c_void);
            }
            if match_0 != 0 {
                break;
            }
            if (*mp).m_next.is_null() {
                mp = mp2;
                mp2 = ::core::ptr::null_mut::<mapblock_T>();
            } else {
                mp = (*mp).m_next;
            };
        }
        if !mp.is_null() {
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if c != Ctrl_RSB {
                if c < 0 as ::core::ffi::c_int || c == K_SPECIAL {
                    let c2rust_fresh14 = j;
                    j = j + 1;
                    tb[c2rust_fresh14 as usize] = K_SPECIAL as uint8_t;
                    let c2rust_fresh15 = j;
                    j = j + 1;
                    tb[c2rust_fresh15 as usize] = (if c == K_SPECIAL {
                        KS_SPECIAL
                    } else if c == NUL {
                        KS_ZERO
                    } else {
                        -c & 0xff as ::core::ffi::c_int
                    }) as uint8_t;
                    let c2rust_fresh16 = j;
                    j = j + 1;
                    tb[c2rust_fresh16 as usize] = (if c == K_SPECIAL || c == NUL {
                        KE_FILLER as ::core::ffi::c_uint
                    } else {
                        -c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                            & 0xff as ::core::ffi::c_uint
                    }) as uint8_t;
                } else {
                    if c < ABBR_OFF
                        && (c < ' ' as ::core::ffi::c_int || c > '~' as ::core::ffi::c_int)
                    {
                        let c2rust_fresh17 = j;
                        j = j + 1;
                        tb[c2rust_fresh17 as usize] = Ctrl_V as uint8_t;
                    }
                    if c >= ABBR_OFF {
                        c -= ABBR_OFF;
                    }
                    let mut newlen: ::core::ffi::c_int = utf_char2bytes(
                        c,
                        (&raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char)
                            .offset(j as isize),
                    );
                    tb[(j + newlen) as usize] = NUL as uint8_t;
                    let mut escaped: *mut ::core::ffi::c_char = vim_strsave_escape_ks(
                        (&raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char)
                            .offset(j as isize),
                    );
                    if !escaped.is_null() {
                        newlen = strlen(escaped) as ::core::ffi::c_int;
                        memmove(
                            (&raw mut tb as *mut uint8_t).offset(j as isize)
                                as *mut ::core::ffi::c_void,
                            escaped as *const ::core::ffi::c_void,
                            newlen as size_t,
                        );
                        j += newlen;
                        xfree(escaped as *mut ::core::ffi::c_void);
                    }
                }
                tb[j as usize] = NUL as uint8_t;
                ins_typebuf(
                    &raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    (*mp).m_silent != 0,
                );
            }
            let noremap: ::core::ffi::c_int = (*mp).m_noremap;
            let silent: bool = (*mp).m_silent != 0;
            let expr: bool = (*mp).m_expr != 0;
            let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if expr {
                s = eval_map_expr(mp, c);
            } else {
                s = (*mp).m_str;
            }
            if !s.is_null() {
                ins_typebuf(s, noremap, 0 as ::core::ffi::c_int, true_0 != 0, silent);
                (*typebuf.ptr()).tb_no_abbr_cnt +=
                    strlen(s) as ::core::ffi::c_int + j + 1 as ::core::ffi::c_int;
                if expr {
                    xfree(s as *mut ::core::ffi::c_void);
                }
            }
            tb[0 as ::core::ffi::c_int as usize] = Ctrl_H as uint8_t;
            tb[1 as ::core::ffi::c_int as usize] = NUL as uint8_t;
            len = clen;
            loop {
                let c2rust_fresh18 = len;
                len = len - 1;
                if c2rust_fresh18 <= 0 as ::core::ffi::c_int {
                    break;
                }
                ins_typebuf(
                    &raw mut tb as *mut uint8_t as *mut ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    silent,
                );
            }
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn eval_map_expr(
    mut mp: *mut mapblock_T,
    mut c: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*mp).m_luaref == LUA_NOREF {
        expr = xstrdup((*mp).m_str);
        vim_unescape_ks(expr);
    }
    let replace_keycodes: bool = (*mp).m_replace_keycodes;
    (*expr_map_lock.ptr()) += 1;
    set_vim_var_char(c);
    let save_cursor: pos_T = (*curwin.get()).w_cursor;
    let save_msg_col: ::core::ffi::c_int = msg_col.get();
    let save_msg_row: ::core::ffi::c_int = msg_row.get();
    if (*mp).m_luaref != LUA_NOREF {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut args: Array = ARRAY_DICT_INIT;
        let mut ret: Object = nlua_call_ref(
            (*mp).m_luaref,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetObject,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        if ret.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            p = string_to_cstr(ret.data.string);
        }
        api_free_object(ret);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            semsg_multiline(
                b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
                b"E5108: %s\0".as_ptr() as *const ::core::ffi::c_char,
                err.msg,
            );
            api_clear_error(&raw mut err);
        }
    } else {
        p = eval_to_string(expr, false_0 != 0, false_0 != 0);
        xfree(expr as *mut ::core::ffi::c_void);
    }
    (*expr_map_lock.ptr()) -= 1;
    (*curwin.get()).w_cursor = save_cursor;
    msg_col.set(save_msg_col);
    msg_row.set(save_msg_row);
    if p.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut res: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if replace_keycodes {
        replace_termcodes(
            p,
            strlen(p),
            &raw mut res,
            0 as scid_T,
            REPTERM_DO_LT as ::core::ffi::c_int,
            ::core::ptr::null_mut::<bool>(),
            p_cpo.get(),
        );
    } else {
        res = vim_strsave_escape_ks(p);
    }
    xfree(p as *mut ::core::ffi::c_void);
    return res;
}
pub unsafe extern "C" fn makemap(mut fd: *mut FILE, mut buf: *mut buf_T) -> ::core::ffi::c_int {
    let mut did_cpo: bool = false_0 != 0;
    let mut abbr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while abbr < 2 as ::core::ffi::c_int {
        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while hash < 256 as ::core::ffi::c_int {
            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
            if abbr != 0 {
                if hash > 0 as ::core::ffi::c_int {
                    break;
                }
                if !buf.is_null() {
                    mp = (*buf).b_first_abbr;
                } else {
                    mp = first_abbr.get();
                }
            } else if !buf.is_null() {
                mp = (*buf).b_maphash[hash as usize] as *mut mapblock_T;
            } else {
                mp = (*maphash.ptr())[hash as usize] as *mut mapblock_T;
            }
            while !mp.is_null() {
                if (*mp).m_noremap != REMAP_SCRIPT as ::core::ffi::c_int {
                    if (*mp).m_luaref == LUA_NOREF {
                        let mut p: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        p = (*mp).m_str;
                        while *p as ::core::ffi::c_int != NUL {
                            if *p.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                                as ::core::ffi::c_int
                                == K_SPECIAL
                                && *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                    as ::core::ffi::c_int
                                    == KS_EXTRA
                                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                    == KE_SNR as ::core::ffi::c_int
                            {
                                break;
                            }
                            p = p.offset(1);
                        }
                        if *p as ::core::ffi::c_int == NUL {
                            let mut c1: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                            let mut c2: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                            let mut c3: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                            let mut cmd: *mut ::core::ffi::c_char = (if abbr != 0 {
                                b"abbr\0".as_ptr() as *const ::core::ffi::c_char
                            } else {
                                b"map\0".as_ptr() as *const ::core::ffi::c_char
                            })
                                as *mut ::core::ffi::c_char;
                            match (*mp).m_mode {
                                71 => {}
                                1 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                }
                                2 => {
                                    c1 = 'x' as ::core::ffi::c_char;
                                }
                                64 => {
                                    c1 = 's' as ::core::ffi::c_char;
                                }
                                4 => {
                                    c1 = 'o' as ::core::ffi::c_char;
                                }
                                3 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 'x' as ::core::ffi::c_char;
                                }
                                65 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 's' as ::core::ffi::c_char;
                                }
                                5 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 'o' as ::core::ffi::c_char;
                                }
                                66 => {
                                    c1 = 'v' as ::core::ffi::c_char;
                                }
                                6 => {
                                    c1 = 'x' as ::core::ffi::c_char;
                                    c2 = 'o' as ::core::ffi::c_char;
                                }
                                68 => {
                                    c1 = 's' as ::core::ffi::c_char;
                                    c2 = 'o' as ::core::ffi::c_char;
                                }
                                67 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 'v' as ::core::ffi::c_char;
                                }
                                7 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 'x' as ::core::ffi::c_char;
                                    c3 = 'o' as ::core::ffi::c_char;
                                }
                                69 => {
                                    c1 = 'n' as ::core::ffi::c_char;
                                    c2 = 's' as ::core::ffi::c_char;
                                    c3 = 'o' as ::core::ffi::c_char;
                                }
                                70 => {
                                    c1 = 'v' as ::core::ffi::c_char;
                                    c2 = 'o' as ::core::ffi::c_char;
                                }
                                24 => {
                                    if abbr == 0 {
                                        cmd = b"map!\0".as_ptr() as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char;
                                    }
                                }
                                8 => {
                                    c1 = 'c' as ::core::ffi::c_char;
                                }
                                16 => {
                                    c1 = 'i' as ::core::ffi::c_char;
                                }
                                32 => {
                                    c1 = 'l' as ::core::ffi::c_char;
                                }
                                128 => {
                                    c1 = 't' as ::core::ffi::c_char;
                                }
                                _ => {
                                    iemsg(gettext(b"E228: makemap: Illegal mode\0".as_ptr()
                                        as *const ::core::ffi::c_char));
                                    return FAIL;
                                }
                            }
                            loop {
                                if !did_cpo {
                                    if *(*mp).m_str as ::core::ffi::c_int == NUL {
                                        did_cpo = true_0 != 0;
                                    } else {
                                        let specials: [::core::ffi::c_char; 3] = [
                                            K_SPECIAL as uint8_t as ::core::ffi::c_char,
                                            NL as ::core::ffi::c_char,
                                            NUL as ::core::ffi::c_char,
                                        ];
                                        if !strpbrk(
                                            (*mp).m_str,
                                            &raw const specials as *const ::core::ffi::c_char,
                                        )
                                        .is_null()
                                            || !strpbrk(
                                                (*mp).m_keys,
                                                &raw const specials as *const ::core::ffi::c_char,
                                            )
                                            .is_null()
                                        {
                                            did_cpo = true_0 != 0;
                                        }
                                    }
                                    if did_cpo {
                                        if fprintf(
                                            fd,
                                            b"let s:cpo_save=&cpo\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ) < 0 as ::core::ffi::c_int
                                            || put_eol(fd) < 0 as ::core::ffi::c_int
                                            || fprintf(
                                                fd,
                                                b"set cpo&vim\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ) < 0 as ::core::ffi::c_int
                                            || put_eol(fd) < 0 as ::core::ffi::c_int
                                        {
                                            return FAIL;
                                        }
                                    }
                                }
                                if c1 as ::core::ffi::c_int != 0
                                    && putc(c1 as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if (*mp).m_noremap != REMAP_YES as ::core::ffi::c_int
                                    && fprintf(fd, b"nore\0".as_ptr() as *const ::core::ffi::c_char)
                                        < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if fputs(cmd, fd) < 0 as ::core::ffi::c_int {
                                    return FAIL;
                                }
                                if !buf.is_null()
                                    && fputs(
                                        b" <buffer>\0".as_ptr() as *const ::core::ffi::c_char,
                                        fd,
                                    ) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if (*mp).m_nowait as ::core::ffi::c_int != 0
                                    && fputs(
                                        b" <nowait>\0".as_ptr() as *const ::core::ffi::c_char,
                                        fd,
                                    ) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if (*mp).m_silent as ::core::ffi::c_int != 0
                                    && fputs(
                                        b" <silent>\0".as_ptr() as *const ::core::ffi::c_char,
                                        fd,
                                    ) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if (*mp).m_expr as ::core::ffi::c_int != 0
                                    && fputs(
                                        b" <expr>\0".as_ptr() as *const ::core::ffi::c_char,
                                        fd,
                                    ) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                if putc(' ' as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int
                                    || put_escstr(fd, (*mp).m_keys, 0 as ::core::ffi::c_int) == FAIL
                                    || putc(' ' as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int
                                    || put_escstr(fd, (*mp).m_str, 1 as ::core::ffi::c_int) == FAIL
                                    || put_eol(fd) < 0 as ::core::ffi::c_int
                                {
                                    return FAIL;
                                }
                                c1 = c2;
                                c2 = c3;
                                c3 = NUL as ::core::ffi::c_char;
                                if c1 as ::core::ffi::c_int == NUL {
                                    break;
                                }
                            }
                        }
                    }
                }
                mp = (*mp).m_next;
            }
            hash += 1;
        }
        abbr += 1;
    }
    if did_cpo {
        if fprintf(
            fd,
            b"let &cpo=s:cpo_save\0".as_ptr() as *const ::core::ffi::c_char,
        ) < 0 as ::core::ffi::c_int
            || put_eol(fd) < 0 as ::core::ffi::c_int
            || fprintf(
                fd,
                b"unlet s:cpo_save\0".as_ptr() as *const ::core::ffi::c_char,
            ) < 0 as ::core::ffi::c_int
            || put_eol(fd) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
    }
    return OK;
}
pub unsafe extern "C" fn put_escstr(
    mut fd: *mut FILE,
    mut strstart: *const ::core::ffi::c_char,
    mut what: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut str: *mut uint8_t = strstart as *mut uint8_t;
    if *str as ::core::ffi::c_int == NUL && what == 1 as ::core::ffi::c_int {
        if fprintf(fd, b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char) < 0 as ::core::ffi::c_int
        {
            return FAIL;
        }
        return OK;
    }
    while *str as ::core::ffi::c_int != NUL {
        let mut p: *const ::core::ffi::c_char =
            mb_unescape(&raw mut str as *mut *const ::core::ffi::c_char);
        's_26: {
            if !p.is_null() {
                while *p as ::core::ffi::c_int != NUL {
                    let c2rust_fresh19 = p;
                    p = p.offset(1);
                    if fputc(*c2rust_fresh19 as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int {
                        return FAIL;
                    }
                }
                str = str.offset(-1);
            } else {
                let mut c: ::core::ffi::c_int = *str as ::core::ffi::c_int;
                if c == K_SPECIAL && what != 2 as ::core::ffi::c_int {
                    let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == KS_MODIFIER
                    {
                        modifiers =
                            *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
                        str = str.offset(3 as ::core::ffi::c_int as isize);
                        p = mb_unescape(&raw mut str as *mut *const ::core::ffi::c_char);
                        if p.is_null() {
                            c = *str as ::core::ffi::c_int;
                        } else {
                            c = utf_ptr2char(p);
                            str = str.offset(-1);
                        }
                    }
                    if c == K_SPECIAL {
                        c = if *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == KS_SPECIAL
                        {
                            K_SPECIAL
                        } else if *str.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == KS_ZERO
                        {
                            K_ZERO
                        } else {
                            -(*str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                + ((*str.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    << 8 as ::core::ffi::c_int))
                        };
                        str = str.offset(2 as ::core::ffi::c_int as isize);
                    }
                    if c < 0 as ::core::ffi::c_int || modifiers != 0 {
                        if fputs(get_special_key_name(c, modifiers), fd) < 0 as ::core::ffi::c_int {
                            return FAIL;
                        }
                        break 's_26;
                    }
                }
                if c == NL {
                    if what == 2 as ::core::ffi::c_int {
                        if fprintf(fd, b"\\\x16\n\0".as_ptr() as *const ::core::ffi::c_char)
                            < 0 as ::core::ffi::c_int
                        {
                            return FAIL;
                        }
                    } else if fprintf(fd, b"<NL>\0".as_ptr() as *const ::core::ffi::c_char)
                        < 0 as ::core::ffi::c_int
                    {
                        return FAIL;
                    }
                } else {
                    if what == 2 as ::core::ffi::c_int
                        && (ascii_iswhite(c) as ::core::ffi::c_int != 0
                            || c == '"' as ::core::ffi::c_int
                            || c == '\\' as ::core::ffi::c_int)
                    {
                        if putc('\\' as ::core::ffi::c_int, fd) < 0 as ::core::ffi::c_int {
                            return FAIL;
                        }
                    } else if c < ' ' as ::core::ffi::c_int
                        || c > '~' as ::core::ffi::c_int
                        || c == '|' as ::core::ffi::c_int
                        || what == 0 as ::core::ffi::c_int && c == ' ' as ::core::ffi::c_int
                        || what == 1 as ::core::ffi::c_int
                            && str == strstart as *mut uint8_t
                            && c == ' ' as ::core::ffi::c_int
                        || what != 2 as ::core::ffi::c_int && c == '<' as ::core::ffi::c_int
                    {
                        if putc(Ctrl_V, fd) < 0 as ::core::ffi::c_int {
                            return FAIL;
                        }
                    }
                    if putc(c, fd) < 0 as ::core::ffi::c_int {
                        return FAIL;
                    }
                }
            }
        }
        str = str.offset(1);
    }
    return OK;
}
pub unsafe extern "C" fn check_map(
    mut keys: *mut ::core::ffi::c_char,
    mut mode: ::core::ffi::c_int,
    mut exact: ::core::ffi::c_int,
    mut ign_mod: ::core::ffi::c_int,
    mut abbr: ::core::ffi::c_int,
    mut mp_ptr: *mut *mut mapblock_T,
    mut local_ptr: *mut ::core::ffi::c_int,
    mut rhs_lua: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    *rhs_lua = LUA_NOREF;
    let mut len: ::core::ffi::c_int = strlen(keys) as ::core::ffi::c_int;
    let mut local: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while local >= 0 as ::core::ffi::c_int {
        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while hash < 256 as ::core::ffi::c_int {
            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
            if abbr != 0 {
                if hash > 0 as ::core::ffi::c_int {
                    break;
                }
                if local != 0 {
                    mp = (*curbuf.get()).b_first_abbr;
                } else {
                    mp = first_abbr.get();
                }
            } else if local != 0 {
                mp = (*curbuf.get()).b_maphash[hash as usize] as *mut mapblock_T;
            } else {
                mp = (*maphash.ptr())[hash as usize] as *mut mapblock_T;
            }
            while !mp.is_null() {
                if (*mp).m_mode & mode != 0 && (exact == 0 || (*mp).m_keylen == len) {
                    let mut s: *mut ::core::ffi::c_char = (*mp).m_keys;
                    let mut keylen: ::core::ffi::c_int = (*mp).m_keylen;
                    if ign_mod != 0
                        && keylen >= 3 as ::core::ffi::c_int
                        && *s.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                            == K_SPECIAL
                        && *s.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int
                            == KS_MODIFIER
                    {
                        s = s.offset(3 as ::core::ffi::c_int as isize);
                        keylen -= 3 as ::core::ffi::c_int;
                    }
                    let mut minlen: ::core::ffi::c_int = if keylen < len { keylen } else { len };
                    if strncmp(s, keys, minlen as size_t) == 0 as ::core::ffi::c_int {
                        if !mp_ptr.is_null() {
                            *mp_ptr = mp;
                        }
                        if !local_ptr.is_null() {
                            *local_ptr = local;
                        }
                        *rhs_lua = (*mp).m_luaref as ::core::ffi::c_int;
                        return if (*mp).m_luaref == LUA_NOREF {
                            (*mp).m_str
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                        };
                    }
                }
                mp = (*mp).m_next;
            }
            hash += 1;
        }
        local -= 1;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn f_hasmapto(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut mode: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let name: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut abbr: bool = false_0 != 0;
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        mode = b"nvo\0".as_ptr() as *const ::core::ffi::c_char;
    } else {
        mode = tv_get_string_buf(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            abbr = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) != 0;
        }
    }
    (*rettv).vval.v_number = map_to_exists(name, mode, abbr) as varnumber_T;
}
unsafe extern "C" fn mapblock_fill_dict(
    mp: *const mapblock_T,
    mut lhsrawalt: *const ::core::ffi::c_char,
    buffer_value: ::core::ffi::c_int,
    abbr: bool,
    compatible: bool,
    mut arena: *mut Arena,
) -> Dict {
    let mut dict: Dict = arena_dict(arena, 20 as size_t);
    let lhs: *mut ::core::ffi::c_char =
        str2special_arena((*mp).m_keys, compatible, !compatible, arena);
    let mut mapmode: *mut ::core::ffi::c_char =
        arena_alloc(arena, 7 as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
    map_mode_to_chars((*mp).m_mode, mapmode);
    let mut noremap_value: ::core::ffi::c_int = 0;
    if compatible {
        noremap_value = ((*mp).m_noremap != 0) as ::core::ffi::c_int;
    } else {
        noremap_value = if (*mp).m_noremap == REMAP_SCRIPT as ::core::ffi::c_int {
            2 as ::core::ffi::c_int
        } else {
            ((*mp).m_noremap != 0) as ::core::ffi::c_int
        };
    }
    if (*mp).m_luaref != LUA_NOREF {
        let c2rust_fresh21 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"callback\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeLuaRef,
                data: C2Rust_Unnamed {
                    luaref: api_new_luaref((*mp).m_luaref),
                },
            },
        };
    } else {
        let mut rhs: String_0 = cstr_as_string(if compatible as ::core::ffi::c_int != 0 {
            (*mp).m_orig_str
        } else {
            str2special_arena((*mp).m_str, false_0 != 0, true_0 != 0, arena)
        });
        let c2rust_fresh22 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh22 as isize) = key_value_pair {
            key: cstr_as_string(b"rhs\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed { string: rhs },
            },
        };
    }
    if !(*mp).m_desc.is_null() {
        let c2rust_fresh23 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"desc\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*mp).m_desc),
                },
            },
        };
    }
    let c2rust_fresh24 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh24 as isize) = key_value_pair {
        key: cstr_as_string(b"lhs\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(lhs),
            },
        },
    };
    let c2rust_fresh25 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh25 as isize) = key_value_pair {
        key: cstr_as_string(b"lhsraw\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string((*mp).m_keys),
            },
        },
    };
    if !lhsrawalt.is_null() {
        let c2rust_fresh26 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh26 as isize) = key_value_pair {
            key: cstr_as_string(b"lhsrawalt\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(lhsrawalt),
                },
            },
        };
    }
    let c2rust_fresh27 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh27 as isize) = key_value_pair {
        key: cstr_as_string(b"noremap\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: noremap_value as Integer,
            },
        },
    };
    let c2rust_fresh28 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh28 as isize) = key_value_pair {
        key: cstr_as_string(b"script\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if (*mp).m_noremap == REMAP_SCRIPT as ::core::ffi::c_int {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh29 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh29 as isize) = key_value_pair {
        key: cstr_as_string(b"expr\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if (*mp).m_expr as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh30 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh30 as isize) = key_value_pair {
        key: cstr_as_string(b"silent\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if (*mp).m_silent as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh31 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh31 as isize) = key_value_pair {
        key: cstr_as_string(b"sid\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*mp).m_script_ctx.sc_sid as Integer,
            },
        },
    };
    let c2rust_fresh32 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh32 as isize) = key_value_pair {
        key: cstr_as_string(b"scriptversion\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 1 as Integer,
            },
        },
    };
    let c2rust_fresh33 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh33 as isize) = key_value_pair {
        key: cstr_as_string(b"lnum\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*mp).m_script_ctx.sc_lnum as Integer,
            },
        },
    };
    let c2rust_fresh34 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh34 as isize) = key_value_pair {
        key: cstr_as_string(b"buffer\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: buffer_value as Integer,
            },
        },
    };
    if !compatible {
        let c2rust_fresh35 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh35 as isize) = key_value_pair {
            key: cstr_as_string(b"buf\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: buffer_value as Integer,
                },
            },
        };
    }
    let c2rust_fresh36 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh36 as isize) = key_value_pair {
        key: cstr_as_string(b"nowait\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if (*mp).m_nowait as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh37 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh37 as isize) = key_value_pair {
        key: cstr_as_string(b"replace_keycodes\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if (*mp).m_replace_keycodes as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh38 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh38 as isize) = key_value_pair {
        key: cstr_as_string(b"mode\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(mapmode),
            },
        },
    };
    let c2rust_fresh39 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh39 as isize) = key_value_pair {
        key: cstr_as_string(b"abbr\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (if abbr as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as Integer,
            },
        },
    };
    let c2rust_fresh40 = dict.size;
    dict.size = dict.size.wrapping_add(1);
    *dict.items.offset(c2rust_fresh40 as isize) = key_value_pair {
        key: cstr_as_string(b"mode_bits\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*mp).m_mode as Integer,
            },
        },
    };
    return dict;
}
unsafe extern "C" fn get_maparg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut exact: ::core::ffi::c_int,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut keys: *mut ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char;
    if *keys as ::core::ffi::c_int == NUL {
        return;
    }
    let mut which: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut abbr: bool = false_0 != 0;
    let mut get_dict: bool = false_0 != 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        which = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            abbr = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) != 0;
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                get_dict = tv_get_number(argvars.offset(3 as ::core::ffi::c_int as isize)) != 0;
            }
        }
    } else {
        which = b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if which.is_null() {
        return;
    }
    let mut keys_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut alt_keys_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut did_simplify: bool = false_0 != 0;
    let flags: ::core::ffi::c_int =
        REPTERM_FROM_PART as ::core::ffi::c_int | REPTERM_DO_LT as ::core::ffi::c_int;
    let mode: ::core::ffi::c_int =
        get_map_mode(&raw mut which as *mut *mut ::core::ffi::c_char, false);
    let mut keys_simplified: *mut ::core::ffi::c_char = replace_termcodes(
        keys,
        strlen(keys),
        &raw mut keys_buf,
        0 as scid_T,
        flags,
        &raw mut did_simplify,
        p_cpo.get(),
    );
    let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
    let mut buffer_local: ::core::ffi::c_int = 0;
    let mut rhs_lua: LuaRef = 0;
    let mut rhs: *mut ::core::ffi::c_char = check_map(
        keys_simplified,
        mode,
        exact,
        false_0,
        abbr as ::core::ffi::c_int,
        &raw mut mp,
        &raw mut buffer_local,
        &raw mut rhs_lua,
    );
    if did_simplify {
        replace_termcodes(
            keys,
            strlen(keys),
            &raw mut alt_keys_buf,
            0 as scid_T,
            flags | REPTERM_NO_SIMPLIFY as ::core::ffi::c_int,
            ::core::ptr::null_mut::<bool>(),
            p_cpo.get(),
        );
        rhs = check_map(
            alt_keys_buf,
            mode,
            exact,
            false_0,
            abbr as ::core::ffi::c_int,
            &raw mut mp,
            &raw mut buffer_local,
            &raw mut rhs_lua,
        );
    }
    if !get_dict {
        if !rhs.is_null() {
            if *rhs as ::core::ffi::c_int == NUL {
                (*rettv).vval.v_string = xstrdup(b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                (*rettv).vval.v_string = str2special_save(rhs, false_0 != 0, false_0 != 0);
            }
        } else if rhs_lua != LUA_NOREF {
            (*rettv).vval.v_string =
                nlua_funcref_str((*mp).m_luaref, ::core::ptr::null_mut::<Arena>());
        }
    } else if !mp.is_null() && (!rhs.is_null() || rhs_lua != LUA_NOREF) {
        let mut arena: Arena = ARENA_EMPTY;
        let mut dict: Dict = mapblock_fill_dict(
            mp,
            if did_simplify as ::core::ffi::c_int != 0 {
                keys_simplified
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            },
            buffer_local,
            abbr,
            true_0 != 0,
            &raw mut arena,
        );
        let mut c2rust_lvalue: Object = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: dict },
        };
        object_to_vim_take_luaref(
            &raw mut c2rust_lvalue,
            rettv,
            true_0 != 0,
            ::core::ptr::null_mut::<Error>(),
        );
        arena_mem_free(arena_finish(&raw mut arena));
    } else {
        tv_dict_alloc_ret(rettv);
    }
    xfree(keys_buf as *mut ::core::ffi::c_void);
    xfree(alt_keys_buf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn get_map_mode_string(
    mode_string: *const ::core::ffi::c_char,
    abbr: bool,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = mode_string;
    let MASK_V: ::core::ffi::c_int = MODE_VISUAL | MODE_SELECT;
    let MASK_MAP: ::core::ffi::c_int = MODE_VISUAL | MODE_SELECT | MODE_NORMAL | MODE_OP_PENDING;
    let MASK_BANG: ::core::ffi::c_int = MODE_INSERT | MODE_CMDLINE;
    if *p as ::core::ffi::c_int == NUL {
        p = b" \0".as_ptr() as *const ::core::ffi::c_char;
    }
    let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut modec: ::core::ffi::c_int = 0;
    loop {
        let c2rust_fresh20 = p;
        p = p.offset(1);
        modec = *c2rust_fresh20 as uint8_t as ::core::ffi::c_int;
        if modec == 0 {
            break;
        }
        let mut tmode: ::core::ffi::c_int = 0;
        match modec {
            105 => {
                tmode = MODE_INSERT;
            }
            108 => {
                tmode = MODE_LANGMAP;
            }
            99 => {
                tmode = MODE_CMDLINE;
            }
            110 => {
                tmode = MODE_NORMAL;
            }
            120 => {
                tmode = MODE_VISUAL;
            }
            115 => {
                tmode = MODE_SELECT;
            }
            111 => {
                tmode = MODE_OP_PENDING;
            }
            116 => {
                tmode = MODE_TERMINAL;
            }
            118 => {
                tmode = MASK_V;
            }
            33 => {
                tmode = MASK_BANG;
            }
            32 => {
                tmode = MASK_MAP;
            }
            _ => return 0 as ::core::ffi::c_int,
        }
        mode |= tmode;
    }
    if abbr as ::core::ffi::c_int != 0 && mode & !MASK_BANG != 0 as ::core::ffi::c_int
        || !abbr
            && mode & mode - 1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            && !(mode & MASK_BANG != 0 as ::core::ffi::c_int
                && mode & !MASK_BANG == 0 as ::core::ffi::c_int
                || mode & MASK_MAP != 0 as ::core::ffi::c_int
                    && mode & !MASK_MAP == 0 as ::core::ffi::c_int)
    {
        return 0 as ::core::ffi::c_int;
    }
    return mode;
}
pub unsafe extern "C" fn f_mapset(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    let mut which: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let mut is_abbr: ::core::ffi::c_int = 0;
    let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let dict_only: bool = (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type
        as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint;
    if dict_only {
        d = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        which = tv_dict_get_string(
            d,
            b"mode\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        is_abbr = tv_dict_get_bool(
            d,
            b"abbr\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int;
        if which.is_null() || is_abbr < 0 as ::core::ffi::c_int {
            emsg(gettext(
                (e_entries_missing_in_mapset_dict_argument.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ));
            return;
        }
    } else {
        which = tv_get_string_buf_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if which.is_null() {
            return;
        }
        is_abbr =
            tv_get_bool(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        if tv_check_for_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return;
        }
        d = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
    }
    let mode: ::core::ffi::c_int = get_map_mode_string(which, is_abbr != 0);
    if mode == 0 as ::core::ffi::c_int {
        semsg(
            gettext(
                (e_illegal_map_mode_string_str.ptr() as *const _) as *const ::core::ffi::c_char,
            ),
            which,
        );
        return;
    }
    let mut lhs: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"lhs\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    let mut lhsraw: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"lhsraw\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    let mut lhsrawalt: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"lhsrawalt\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    let mut orig_rhs: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"rhs\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    let mut rhs_lua: LuaRef = LUA_NOREF;
    let mut callback_di: *mut dictitem_T = tv_dict_find(
        d,
        b"callback\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !callback_di.is_null() {
        if (*callback_di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut fp: *mut ufunc_T = find_func((*callback_di).di_tv.vval.v_string);
            if !fp.is_null() && (*fp).uf_flags & FC_LUAREF != 0 {
                rhs_lua = api_new_luaref((*fp).uf_luaref);
                orig_rhs = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
        }
    }
    if lhs.is_null() || lhsraw.is_null() || orig_rhs.is_null() {
        emsg(gettext(
            (e_entries_missing_in_mapset_dict_argument.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
        api_free_luaref(rhs_lua);
        return;
    }
    let mut noremap: ::core::ffi::c_int =
        if tv_dict_get_number(d, b"noremap\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T
        {
            REMAP_NONE as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    if tv_dict_get_number(d, b"script\0".as_ptr() as *const ::core::ffi::c_char) != 0 as varnumber_T
    {
        noremap = REMAP_SCRIPT as ::core::ffi::c_int;
    }
    let mut args: MapArguments = map_arguments {
        buffer: false,
        expr: tv_dict_get_number(d, b"expr\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T,
        noremap: false,
        nowait: tv_dict_get_number(d, b"nowait\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T,
        script: false,
        silent: tv_dict_get_number(d, b"silent\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T,
        unique: false,
        replace_keycodes: tv_dict_get_number(
            d,
            b"replace_keycodes\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0 as varnumber_T,
        lhs: [0; 51],
        lhs_len: 0,
        alt_lhs: [0; 51],
        alt_lhs_len: 0,
        rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        rhs_len: 0,
        rhs_lua: 0,
        rhs_is_noop: false,
        orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        orig_rhs_len: 0,
        desc: tv_dict_get_string(
            d,
            b"desc\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        ),
    };
    let mut sid: scid_T =
        tv_dict_get_number(d, b"sid\0".as_ptr() as *const ::core::ffi::c_char) as scid_T;
    let mut lnum: linenr_T =
        tv_dict_get_number(d, b"lnum\0".as_ptr() as *const ::core::ffi::c_char) as linenr_T;
    let mut buffer: bool =
        tv_dict_get_number(d, b"buffer\0".as_ptr() as *const ::core::ffi::c_char)
            != 0 as varnumber_T;
    set_maparg_rhs(
        orig_rhs,
        strlen(orig_rhs),
        rhs_lua,
        sid,
        p_cpo.get(),
        &raw mut args,
    );
    let mut map_table: *mut *mut mapblock_T = if buffer as ::core::ffi::c_int != 0 {
        &raw mut (*curbuf.get()).b_maphash as *mut *mut mapblock_T
    } else {
        maphash.ptr() as *mut *mut mapblock_T
    };
    let mut abbr_table: *mut *mut mapblock_T = if buffer as ::core::ffi::c_int != 0 {
        &raw mut (*curbuf.get()).b_first_abbr
    } else {
        first_abbr.ptr()
    };
    let mut unmap_args: MapArguments = MAP_ARGUMENTS_INIT;
    set_maparg_lhs_rhs(
        lhs,
        strlen(lhs),
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        0 as size_t,
        LUA_NOREF,
        p_cpo.get(),
        &raw mut unmap_args,
    );
    unmap_args.buffer = buffer;
    buf_do_map(
        MAPTYPE_UNMAP_LHS as ::core::ffi::c_int,
        &raw mut unmap_args,
        mode,
        is_abbr != 0,
        curbuf.get(),
    );
    xfree(unmap_args.rhs as *mut ::core::ffi::c_void);
    xfree(unmap_args.orig_rhs as *mut ::core::ffi::c_void);
    let mut mp_result: [*mut mapblock_T; 2] = [
        ::core::ptr::null_mut::<mapblock_T>(),
        ::core::ptr::null_mut::<mapblock_T>(),
    ];
    mp_result[0 as ::core::ffi::c_int as usize] = map_add(
        curbuf.get(),
        map_table,
        abbr_table,
        lhsraw,
        &raw mut args,
        noremap,
        mode,
        is_abbr != 0,
        sid,
        lnum,
        false_0 != 0,
    );
    if !lhsrawalt.is_null() {
        mp_result[1 as ::core::ffi::c_int as usize] = map_add(
            curbuf.get(),
            map_table,
            abbr_table,
            lhsrawalt,
            &raw mut args,
            noremap,
            mode,
            is_abbr != 0,
            sid,
            lnum,
            true_0 != 0,
        );
    }
    if !mp_result[0 as ::core::ffi::c_int as usize].is_null()
        && !mp_result[1 as ::core::ffi::c_int as usize].is_null()
    {
        (*mp_result[0 as ::core::ffi::c_int as usize]).m_alt =
            mp_result[1 as ::core::ffi::c_int as usize];
        (*mp_result[1 as ::core::ffi::c_int as usize]).m_alt =
            mp_result[0 as ::core::ffi::c_int as usize];
    }
}
pub unsafe extern "C" fn f_maplist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let flags: ::core::ffi::c_int =
        REPTERM_FROM_PART as ::core::ffi::c_int | REPTERM_DO_LT as ::core::ffi::c_int;
    let abbr: bool = (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type
        as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_bool(argvars.offset(0 as ::core::ffi::c_int as isize)) != 0;
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    let mut buffer_local: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while buffer_local <= 1 as ::core::ffi::c_int {
        let mut hash: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while hash < 256 as ::core::ffi::c_int {
            let mut mp: *mut mapblock_T = ::core::ptr::null_mut::<mapblock_T>();
            if abbr {
                if hash > 0 as ::core::ffi::c_int {
                    break;
                }
                if buffer_local != 0 {
                    mp = (*curbuf.get()).b_first_abbr;
                } else {
                    mp = first_abbr.get();
                }
            } else if buffer_local != 0 {
                mp = (*curbuf.get()).b_maphash[hash as usize] as *mut mapblock_T;
            } else {
                mp = (*maphash.ptr())[hash as usize] as *mut mapblock_T;
            }
            while !mp.is_null() {
                if (*mp).m_simplified == 0 {
                    let mut keys_buf: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut did_simplify: bool = false_0 != 0;
                    let mut arena: Arena = ARENA_EMPTY;
                    let mut lhs: *mut ::core::ffi::c_char =
                        str2special_arena((*mp).m_keys, true_0 != 0, false_0 != 0, &raw mut arena);
                    replace_termcodes(
                        lhs,
                        strlen(lhs),
                        &raw mut keys_buf,
                        0 as scid_T,
                        flags,
                        &raw mut did_simplify,
                        p_cpo.get(),
                    );
                    let mut dict: Dict = mapblock_fill_dict(
                        mp,
                        if did_simplify as ::core::ffi::c_int != 0 {
                            keys_buf
                        } else {
                            ::core::ptr::null_mut::<::core::ffi::c_char>()
                        },
                        buffer_local,
                        abbr,
                        true_0 != 0,
                        &raw mut arena,
                    );
                    let mut d: typval_T = typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    let mut c2rust_lvalue: Object = object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed { dict: dict },
                    };
                    object_to_vim_take_luaref(
                        &raw mut c2rust_lvalue,
                        &raw mut d,
                        true_0 != 0,
                        ::core::ptr::null_mut::<Error>(),
                    );
                    '_c2rust_label: {
                        if d.v_type as ::core::ffi::c_uint
                            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                        } else {
                            __assert_fail(
                                b"d.v_type == VAR_DICT\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/mapping.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                2431 as ::core::ffi::c_uint,
                                b"void f_maplist(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    tv_list_append_dict((*rettv).vval.v_list, d.vval.v_dict);
                    arena_mem_free(arena_finish(&raw mut arena));
                    xfree(keys_buf as *mut ::core::ffi::c_void);
                }
                mp = (*mp).m_next;
            }
            hash += 1;
        }
        buffer_local += 1;
    }
}
pub unsafe extern "C" fn f_maparg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_maparg(argvars, rettv, true_0);
}
pub unsafe extern "C" fn f_mapcheck(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_maparg(argvars, rettv, false_0);
}
pub unsafe extern "C" fn add_map(
    mut lhs: *mut ::core::ffi::c_char,
    mut rhs: *mut ::core::ffi::c_char,
    mut mode: ::core::ffi::c_int,
    mut buffer: bool,
) {
    let mut args: MapArguments = MAP_ARGUMENTS_INIT;
    set_maparg_lhs_rhs(
        lhs,
        strlen(lhs),
        rhs,
        strlen(rhs),
        LUA_NOREF,
        p_cpo.get(),
        &raw mut args,
    );
    args.buffer = buffer;
    buf_do_map(
        MAPTYPE_NOREMAP as ::core::ffi::c_int,
        &raw mut args,
        mode,
        false_0 != 0,
        curbuf.get(),
    );
    xfree(args.rhs as *mut ::core::ffi::c_void);
    xfree(args.orig_rhs as *mut ::core::ffi::c_void);
}
static langmap_mapga: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
unsafe extern "C" fn langmap_set_entry(mut from: ::core::ffi::c_int, mut to: ::core::ffi::c_int) {
    let mut entries: *mut langmap_entry_T = (*langmap_mapga.ptr()).ga_data as *mut langmap_entry_T;
    let mut a: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    '_c2rust_label: {
        if (*langmap_mapga.ptr()).ga_len >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"langmap_mapga.ga_len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/mapping.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2496 as ::core::ffi::c_uint,
                b"void langmap_set_entry(int, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut b: ::core::ffi::c_uint = (*langmap_mapga.ptr()).ga_len as ::core::ffi::c_uint;
    while a != b {
        let mut i: ::core::ffi::c_uint = a.wrapping_add(b).wrapping_div(2 as ::core::ffi::c_uint);
        let mut d: ::core::ffi::c_int = (*entries.offset(i as isize)).from - from;
        if d == 0 as ::core::ffi::c_int {
            (*entries.offset(i as isize)).to = to;
            return;
        }
        if d < 0 as ::core::ffi::c_int {
            a = i.wrapping_add(1 as ::core::ffi::c_uint);
        } else {
            b = i;
        }
    }
    ga_grow(langmap_mapga.ptr(), 1 as ::core::ffi::c_int);
    entries = ((*langmap_mapga.ptr()).ga_data as *mut langmap_entry_T).offset(a as isize);
    memmove(
        entries.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
        entries as *const ::core::ffi::c_void,
        (((*langmap_mapga.ptr()).ga_len as ::core::ffi::c_uint).wrapping_sub(a) as size_t)
            .wrapping_mul(::core::mem::size_of::<langmap_entry_T>()),
    );
    (*langmap_mapga.ptr()).ga_len += 1;
    (*entries.offset(0 as ::core::ffi::c_int as isize)).from = from;
    (*entries.offset(0 as ::core::ffi::c_int as isize)).to = to;
}
pub unsafe extern "C" fn langmap_adjust_mb(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut entries: *mut langmap_entry_T = (*langmap_mapga.ptr()).ga_data as *mut langmap_entry_T;
    let mut a: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut b: ::core::ffi::c_int = (*langmap_mapga.ptr()).ga_len;
    while a != b {
        let mut i: ::core::ffi::c_int = (a + b) / 2 as ::core::ffi::c_int;
        let mut d: ::core::ffi::c_int = (*entries.offset(i as isize)).from - c;
        if d == 0 as ::core::ffi::c_int {
            return (*entries.offset(i as isize)).to;
        }
        if d < 0 as ::core::ffi::c_int {
            a = i + 1 as ::core::ffi::c_int;
        } else {
            b = i;
        }
    }
    return c;
}
pub unsafe extern "C" fn langmap_init() {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        (*langmap_mapchar.ptr())[i as usize] = i as uint8_t;
        i += 1;
    }
    ga_init(
        langmap_mapga.ptr(),
        ::core::mem::size_of::<langmap_entry_T>() as ::core::ffi::c_int,
        8 as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn did_set_langmap(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    ga_clear(langmap_mapga.ptr());
    langmap_init();
    let mut p: *mut ::core::ffi::c_char = p_langmap.get();
    while *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
        let mut p2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p2 = p;
        while *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ',' as ::core::ffi::c_int
            && *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ';' as ::core::ffi::c_int
        {
            if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p2 = p2.offset(1);
            }
            p2 = p2.offset(utfc_ptr2len(p2) as isize);
        }
        if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ';' as ::core::ffi::c_int
        {
            p2 = p2.offset(1);
        } else {
            p2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        while *p.offset(0 as ::core::ffi::c_int as isize) != 0 {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ',' as ::core::ffi::c_int
            {
                p = p.offset(1);
                break;
            } else {
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    p = p.offset(1);
                }
                let mut from: ::core::ffi::c_int = utf_ptr2char(p);
                let from_ptr: *const ::core::ffi::c_char = p;
                let mut to: ::core::ffi::c_int = NUL;
                let mut to_ptr: *const ::core::ffi::c_char =
                    b"\0".as_ptr() as *const ::core::ffi::c_char;
                if p2.is_null() {
                    p = p.offset(utfc_ptr2len(p) as isize);
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ',' as ::core::ffi::c_int
                    {
                        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        {
                            p = p.offset(1);
                        }
                        to_ptr = p;
                        to = utf_ptr2char(to_ptr);
                    }
                } else if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ',' as ::core::ffi::c_int
                {
                    if *p2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                    {
                        p2 = p2.offset(1);
                    }
                    to_ptr = p2;
                    to = utf_ptr2char(to_ptr);
                }
                if to == NUL {
                    snprintf(
                        (*args).os_errbuf,
                        (*args).os_errbuflen,
                        gettext(
                            b"E357: 'langmap': Matching character missing for %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        transchar(from),
                    );
                    return (*args).os_errbuf;
                }
                if from >= 256 as ::core::ffi::c_int {
                    langmap_set_entry(from, to);
                } else {
                    if to > UCHAR_MAX {
                        swmsg(
                            true_0 != 0,
                            b"'langmap': Mapping from %.*s to %.*s will not work properly\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            utf_ptr2len(from_ptr),
                            from_ptr,
                            utf_ptr2len(to_ptr),
                            to_ptr,
                        );
                    }
                    (*langmap_mapchar.ptr())[(from & 255 as ::core::ffi::c_int) as usize] =
                        to as uint8_t;
                }
                p = p.offset(utfc_ptr2len(p) as isize);
                if p2.is_null() {
                    continue;
                }
                p2 = p2.offset(utfc_ptr2len(p2) as isize);
                if *p as ::core::ffi::c_int != ';' as ::core::ffi::c_int {
                    continue;
                }
                p = p2;
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ',' as ::core::ffi::c_int
                    {
                        snprintf(
                            (*args).os_errbuf,
                            (*args).os_errbuflen,
                            gettext(
                                b"E358: 'langmap': Extra characters after semicolon: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ),
                            p,
                        );
                        return (*args).os_errbuf;
                    }
                    p = p.offset(1);
                }
                break;
            }
        }
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
unsafe extern "C" fn do_exmap(mut eap: *mut exarg_T, mut isabbrev: ::core::ffi::c_int) {
    let mut cmdp: *mut ::core::ffi::c_char = (*eap).cmd;
    let mut mode: ::core::ffi::c_int =
        get_map_mode(&raw mut cmdp, (*eap).forceit != 0 || isabbrev != 0);
    let mut maptype: ::core::ffi::c_int = 0;
    if *cmdp as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
        maptype = MAPTYPE_NOREMAP as ::core::ffi::c_int;
    } else if *cmdp as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
        maptype = MAPTYPE_UNMAP as ::core::ffi::c_int;
    } else {
        maptype = MAPTYPE_MAP as ::core::ffi::c_int;
    }
    let mut parsed_args: MapArguments = MapArguments {
        buffer: false,
        expr: false,
        noremap: false,
        nowait: false,
        script: false,
        silent: false,
        unique: false,
        replace_keycodes: false,
        lhs: [0; 51],
        lhs_len: 0,
        alt_lhs: [0; 51],
        alt_lhs_len: 0,
        rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        rhs_len: 0,
        rhs_lua: 0,
        rhs_is_noop: false,
        orig_rhs: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        orig_rhs_len: 0,
        desc: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut result: ::core::ffi::c_int = str_to_mapargs(
        (*eap).arg,
        maptype == MAPTYPE_UNMAP as ::core::ffi::c_int,
        &raw mut parsed_args,
    );
    match result {
        0 => match buf_do_map(
            maptype,
            &raw mut parsed_args,
            mode,
            isabbrev != 0,
            curbuf.get(),
        ) {
            1 => {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            }
            2 => {
                emsg(if isabbrev != 0 {
                    gettext(&raw const e_noabbr as *const ::core::ffi::c_char)
                } else {
                    gettext(&raw const e_nomap as *const ::core::ffi::c_char)
                });
            }
            5 => {
                semsg(
                    if isabbrev != 0 {
                        gettext(
                            (e_abbreviation_already_exists_for_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        )
                    } else {
                        gettext(
                            (e_mapping_already_exists_for_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        )
                    },
                    &raw mut parsed_args.lhs as *mut ::core::ffi::c_char,
                );
            }
            6 => {
                semsg(
                    if isabbrev != 0 {
                        gettext(
                            (e_global_abbreviation_already_exists_for_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        )
                    } else {
                        gettext(
                            (e_global_mapping_already_exists_for_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        )
                    },
                    &raw mut parsed_args.lhs as *mut ::core::ffi::c_char,
                );
            }
            _ => {}
        },
        1 => {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        }
        _ => {
            '_c2rust_label: {
                if false {
                } else {
                    __assert_fail(
                        b"false && \"Unknown return code from str_to_mapargs!\"\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/mapping.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2669 as ::core::ffi::c_uint,
                        b"void do_exmap(exarg_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
        }
    }
    xfree(parsed_args.rhs as *mut ::core::ffi::c_void);
    xfree(parsed_args.orig_rhs as *mut ::core::ffi::c_void);
}
pub unsafe fn ex_abbreviate(mut eap: *mut exarg_T) {
    do_exmap(eap, true_0);
}
pub unsafe fn ex_map(mut eap: *mut exarg_T) {
    if secure.get() != 0 {
        secure.set(2 as ::core::ffi::c_int);
        msg_outtrans((*eap).cmd, 0 as ::core::ffi::c_int, false_0 != 0);
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    do_exmap(eap, false_0);
}
pub unsafe fn ex_unmap(mut eap: *mut exarg_T) {
    do_exmap(eap, false_0);
}
pub unsafe fn ex_mapclear(mut eap: *mut exarg_T) {
    do_mapclear((*eap).cmd, (*eap).arg, (*eap).forceit, false_0);
}
pub unsafe fn ex_abclear(mut eap: *mut exarg_T) {
    do_mapclear((*eap).cmd, (*eap).arg, true_0, true_0);
}
pub unsafe extern "C" fn modify_keymap(
    mut channel_id: uint64_t,
    mut buffer: Buffer,
    mut is_unmap: bool,
    mut mode: String_0,
    mut lhs: String_0,
    mut rhs: String_0,
    mut opts: *mut KeyDict_keymap,
    mut err: *mut Error,
) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut forceit: bool = false;
    let mut mode_val: ::core::ffi::c_int = 0;
    let mut is_abbrev: bool = false;
    let mut is_noremap: bool = false;
    let mut maptype_val: ::core::ffi::c_int = 0;
    let mut lua_funcref: LuaRef = LUA_NOREF;
    let mut global: bool = buffer == -1 as ::core::ffi::c_int;
    if global {
        buffer = 0 as ::core::ffi::c_int as Buffer;
    }
    let mut target_buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if target_buf.is_null() {
        return;
    }
    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
    let mut parsed_args: MapArguments = MAP_ARGUMENTS_INIT;
    if !opts.is_null() {
        parsed_args.nowait = (*opts).nowait as bool;
        parsed_args.noremap = (*opts).noremap as bool;
        parsed_args.silent = (*opts).silent as bool;
        parsed_args.script = (*opts).script as bool;
        parsed_args.expr = (*opts).expr as bool;
        parsed_args.unique = (*opts).unique as bool;
        parsed_args.replace_keycodes = (*opts).replace_keycodes as bool;
        if (*opts).is_set__keymap_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_keymap__callback
            != 0 as ::core::ffi::c_ulonglong
        {
            lua_funcref = (*opts).callback;
            (*opts).callback = LUA_NOREF as LuaRef;
        }
        if (*opts).is_set__keymap_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_keymap__desc
            != 0 as ::core::ffi::c_ulonglong
        {
            parsed_args.desc = string_to_cstr((*opts).desc);
        }
    }
    parsed_args.buffer = !global;
    '_fail_and_free: {
        if parsed_args.replace_keycodes as ::core::ffi::c_int != 0 && !parsed_args.expr {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"\"replace_keycodes\" requires \"expr\"\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if !set_maparg_lhs_rhs(
            lhs.data,
            lhs.size,
            rhs.data,
            rhs.size,
            lua_funcref,
            p_cpo.get(),
            &raw mut parsed_args,
        ) {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"LHS exceeds maximum map length: %s\0".as_ptr() as *const ::core::ffi::c_char,
                lhs.data,
            );
        } else if parsed_args.lhs_len > MAXMAPLEN as ::core::ffi::c_int as size_t
            || parsed_args.alt_lhs_len > MAXMAPLEN as ::core::ffi::c_int as size_t
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"LHS exceeds maximum map length: %s\0".as_ptr() as *const ::core::ffi::c_char,
                lhs.data,
            );
        } else {
            p = (if mode.size > 0 as size_t {
                mode.data as *const ::core::ffi::c_char
            } else {
                b"m\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            forceit = *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
            mode_val = get_map_mode(&raw mut p, forceit);
            if forceit {
                '_c2rust_label: {
                    if p == mode.data {
                    } else {
                        __assert_fail(
                            b"p == mode.data\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/mapping.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            2794 as ::core::ffi::c_uint,
                            b"void modify_keymap(uint64_t, Buffer, _Bool, String, String, String, KeyDict_keymap *, Error *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                p = p.offset(1);
            }
            is_abbrev = mode_val & (MODE_INSERT | MODE_CMDLINE) != 0 as ::core::ffi::c_int
                && *p as ::core::ffi::c_int == 'a' as ::core::ffi::c_int;
            if is_abbrev {
                p = p.offset(1);
            }
            if mode.size > 0 as size_t && p.offset_from(mode.data) as size_t != mode.size {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid mode shortname: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                    mode.data,
                );
            } else if parsed_args.lhs_len == 0 as size_t {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid (empty) LHS\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                is_noremap = parsed_args.noremap;
                '_c2rust_label_0: {
                    if !(is_unmap as ::core::ffi::c_int != 0
                        && is_noremap as ::core::ffi::c_int != 0)
                    {
                    } else {
                        __assert_fail(
                            b"!(is_unmap && is_noremap)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/mapping.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            2812 as ::core::ffi::c_uint,
                            b"void modify_keymap(uint64_t, Buffer, _Bool, String, String, String, KeyDict_keymap *, Error *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if !is_unmap
                    && lua_funcref == LUA_NOREF
                    && (parsed_args.rhs_len == 0 as size_t && !parsed_args.rhs_is_noop)
                {
                    if rhs.size == 0 as size_t {
                        parsed_args.rhs_is_noop = true_0 != 0;
                    } else {
                        abort();
                    }
                } else if is_unmap as ::core::ffi::c_int != 0
                    && (parsed_args.rhs_len != 0 || parsed_args.rhs_lua != LUA_NOREF)
                {
                    if parsed_args.rhs_len != 0 {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Gave nonempty RHS in unmap command: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            parsed_args.rhs,
                        );
                    } else {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Gave nonempty RHS for unmap\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                    break '_fail_and_free;
                }
                maptype_val = MAPTYPE_MAP as ::core::ffi::c_int;
                if is_unmap {
                    maptype_val = MAPTYPE_UNMAP as ::core::ffi::c_int;
                } else if is_noremap {
                    maptype_val = MAPTYPE_NOREMAP as ::core::ffi::c_int;
                }
                match buf_do_map(
                    maptype_val,
                    &raw mut parsed_args,
                    mode_val,
                    is_abbrev,
                    target_buf,
                ) {
                    0 => {}
                    1 => {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            &raw const e_invarg as *const ::core::ffi::c_char,
                            0 as ::core::ffi::c_int,
                        );
                    }
                    2 => {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            &raw const e_nomap as *const ::core::ffi::c_char,
                            0 as ::core::ffi::c_int,
                        );
                    }
                    5 => {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            if is_abbrev as ::core::ffi::c_int != 0 {
                                (e_abbreviation_already_exists_for_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char
                            } else {
                                (e_mapping_already_exists_for_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char
                            },
                            lhs.data,
                        );
                    }
                    6 => {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            if is_abbrev as ::core::ffi::c_int != 0 {
                                (e_global_abbreviation_already_exists_for_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char
                            } else {
                                (e_global_mapping_already_exists_for_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char
                            },
                            lhs.data,
                        );
                    }
                    _ => {
                        '_c2rust_label_1: {
                            if false {
                            } else {
                                __assert_fail(
                                    b"false && \"Unrecognized return code!\"\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/mapping.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2860 as ::core::ffi::c_uint,
                                    b"void modify_keymap(uint64_t, Buffer, _Bool, String, String, String, KeyDict_keymap *, Error *)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                    }
                }
            }
        }
    }
    current_sctx.set(save_current_sctx);
    if parsed_args.rhs_lua != LUA_NOREF {
        api_free_luaref(parsed_args.rhs_lua);
        parsed_args.rhs_lua = LUA_NOREF as LuaRef;
    }
    xfree(parsed_args.rhs as *mut ::core::ffi::c_void);
    xfree(parsed_args.orig_rhs as *mut ::core::ffi::c_void);
    xfree(parsed_args.desc as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn keymap_array(
    mut mode: String_0,
    mut buf: *mut buf_T,
    mut arena: *mut Arena,
) -> Array {
    let mut mappings: ArrayBuilder = ArrayBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
        init_array: [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 16],
    };
    mappings.capacity = ::core::mem::size_of::<[Object; 16]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 16]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    mappings.size = 0 as size_t;
    mappings.items = &raw mut mappings.init_array as *mut Object;
    let mut p: *mut ::core::ffi::c_char = (if mode.size > 0 as size_t {
        mode.data as *const ::core::ffi::c_char
    } else {
        b"m\0".as_ptr() as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    let mut forceit: bool = *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int;
    let mut int_mode: ::core::ffi::c_int = get_map_mode(&raw mut p, forceit);
    if forceit {
        '_c2rust_label: {
            if p == mode.data {
            } else {
                __assert_fail(
                    b"p == mode.data\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/mapping.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2888 as ::core::ffi::c_uint,
                    b"Array keymap_array(String, buf_T *, Arena *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        p = p.offset(1);
    }
    let mut is_abbrev: bool = int_mode & (MODE_INSERT | MODE_CMDLINE) != 0 as ::core::ffi::c_int
        && *p as ::core::ffi::c_int == 'a' as ::core::ffi::c_int;
    let mut buffer_value: ::core::ffi::c_int = if buf.is_null() {
        0 as ::core::ffi::c_int
    } else {
        (*buf).handle as ::core::ffi::c_int
    };
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i
        < (if is_abbrev as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            MAX_MAPHASH
        })
    {
        let mut current_maphash: *const mapblock_T = if is_abbrev as ::core::ffi::c_int != 0 {
            if !buf.is_null() {
                (*buf).b_first_abbr
            } else {
                first_abbr.get()
            }
        } else if !buf.is_null() {
            (*buf).b_maphash[i as usize] as *mut mapblock_T
        } else {
            (*maphash.ptr())[i as usize] as *mut mapblock_T
        };
        while !current_maphash.is_null() {
            if (*current_maphash).m_simplified == 0 {
                if int_mode & (*current_maphash).m_mode != 0 {
                    if mappings.size == mappings.capacity {
                        mappings.capacity = if mappings.capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            mappings.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        mappings.items = (if mappings.capacity
                            == ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if mappings.items == &raw mut mappings.init_array as *mut Object {
                                mappings.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut mappings.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    mappings.items as *mut ::core::ffi::c_void,
                                    mappings.size.wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if mappings.items == &raw mut mappings.init_array as *mut Object {
                                memcpy(
                                    xmalloc(
                                        mappings
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    mappings.items as *const ::core::ffi::c_void,
                                    mappings.size.wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    mappings.items as *mut ::core::ffi::c_void,
                                    mappings
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh41 = mappings.size;
                    mappings.size = mappings.size.wrapping_add(1);
                    *mappings.items.offset(c2rust_fresh41 as isize) = object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed {
                            dict: mapblock_fill_dict(
                                current_maphash,
                                if !(*current_maphash).m_alt.is_null() {
                                    (*(*current_maphash).m_alt).m_keys
                                } else {
                                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                                },
                                buffer_value,
                                is_abbrev,
                                false,
                                arena,
                            ),
                        },
                    };
                }
            }
            current_maphash = (*current_maphash).m_next;
        }
        i += 1;
    }
    return arena_take_arraybuilder(arena, &raw mut mappings);
}
pub const SCHAR_MAX: ::core::ffi::c_int = __SCHAR_MAX__;
pub const UCHAR_MAX: ::core::ffi::c_int =
    SCHAR_MAX * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __SCHAR_MAX__: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
