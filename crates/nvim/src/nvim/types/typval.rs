#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

pub type BoolVarValue = ::core::ffi::c_uint;
/// The two `VAR_BOOL` values: `v:false` and `v:true`.
pub const kBoolVarFalse: BoolVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Callback {
    pub data: Callback_data,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub union Callback_data {
    pub funcref: *mut ::core::ffi::c_char,
    pub partial: *mut partial_T,
    pub luaref: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DictWatcher {
    pub callback: Callback,
    pub key_pattern: *mut ::core::ffi::c_char,
    pub key_pattern_len: size_t,
    pub node: QUEUE,
    pub busy: bool,
    pub needs_free: bool,
}
pub type ListLenSpecials = ::core::ffi::c_int;
/// The negative lengths `tv_list_alloc` accepts in place of a real count.
pub const kListLenUnknown: ListLenSpecials = -1;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub type ScopeType = ::core::ffi::c_uint;
/// `dv_scope`: whether a dict is a scope dict, and whether it is the
/// function-local one `l:` refers to by default.
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub type SpecialVarValue = ::core::ffi::c_uint;
/// The only `VAR_SPECIAL` value: `v:null`.
pub const kSpecialVarNull: SpecialVarValue = 0;
pub type VarLockStatus = ::core::ffi::c_uint;
/// `v_lock`: `:lockvar` sets `VAR_LOCKED`; `VAR_FIXED` is a slot that
/// cannot be unlocked at all (`v:` variables, `a:` arguments).
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_FIXED: VarLockStatus = 2;
pub type VarType = ::core::ffi::c_uint;
/// `typval_T::v_type` — which arm of `typval_T::vval` is live.
pub const VAR_UNKNOWN: VarType = 0;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_STRING: VarType = 2;
pub const VAR_FUNC: VarType = 3;
pub const VAR_LIST: VarType = 4;
pub const VAR_DICT: VarType = 5;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_BOOL: VarType = 7;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_BLOB: VarType = 10;
/// The numbers `type()` answers with.  A separate enum from [`VarType`]:
/// funcref and partial share one code, and the codes are the documented
/// `v:t_*` values, so they are not free to follow `v_type`.
pub type VarTypeCode = ::core::ffi::c_uint;
pub const VAR_TYPE_NUMBER: VarTypeCode = 0;
pub const VAR_TYPE_STRING: VarTypeCode = 1;
pub const VAR_TYPE_FUNC: VarTypeCode = 2;
pub const VAR_TYPE_LIST: VarTypeCode = 3;
pub const VAR_TYPE_DICT: VarTypeCode = 4;
pub const VAR_TYPE_FLOAT: VarTypeCode = 5;
pub const VAR_TYPE_BOOL: VarTypeCode = 6;
pub const VAR_TYPE_SPECIAL: VarTypeCode = 7;
pub const VAR_TYPE_BLOB: VarTypeCode = 10;
/// Longest variable name `eval_variable` will look up without allocating.
pub const VAR_SHORT_LEN: ::core::ffi::c_uint = 20;
pub type blob_T = blobvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct blobvar_S {
    pub bv_ga: garray_T,
    pub bv_refcount: ::core::ffi::c_int,
    pub bv_lock: VarLockStatus,
}
pub type dict_T = dictvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictvar_S {
    pub dv_lock: VarLockStatus,
    pub dv_scope: ScopeType,
    pub dv_refcount: ::core::ffi::c_int,
    pub dv_copyID: ::core::ffi::c_int,
    pub dv_hashtab: hashtab_T,
    pub dv_copydict: *mut dict_T,
    pub dv_used_next: *mut dict_T,
    pub dv_used_prev: *mut dict_T,
    pub watchers: QUEUE,
    pub lua_table_ref: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccall_S {
    pub fc_func: *mut ufunc_T,
    pub fc_linenr: ::core::ffi::c_int,
    pub fc_returned: ::core::ffi::c_int,
    pub fc_fixvar: [funccall_S_fc_fixvar; 12],
    pub fc_l_vars: dict_T,
    pub fc_l_vars_var: ScopeDictDictItem,
    pub fc_l_avars: dict_T,
    pub fc_l_avars_var: ScopeDictDictItem,
    pub fc_l_varlist: list_T,
    pub fc_l_listitems: [listitem_T; 20],
    pub fc_rettv: *mut typval_T,
    pub fc_breakpoint: linenr_T,
    pub fc_dbg_tick: ::core::ffi::c_int,
    pub fc_level: ::core::ffi::c_int,
    pub fc_defer: garray_T,
    pub fc_prof_child: proftime_T,
    pub fc_caller: *mut funccall_T,
    pub fc_refcount: ::core::ffi::c_int,
    pub fc_copyID: ::core::ffi::c_int,
    pub fc_ufuncs: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccall_S_fc_fixvar {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 21],
}
pub type funccall_T = funccall_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ht_stack_S {
    pub ht: *mut hashtab_T,
    pub prev: *mut ht_stack_S,
}
pub type ht_stack_T = ht_stack_S;
pub type list_T = listvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct list_stack_S {
    pub list: *mut list_T,
    pub prev: *mut list_stack_S,
}
pub type list_stack_T = list_stack_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listitem_S {
    pub li_next: *mut listitem_T,
    pub li_prev: *mut listitem_T,
    pub li_tv: typval_T,
}
pub type listitem_T = listitem_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listvar_S {
    pub lv_first: *mut listitem_T,
    pub lv_last: *mut listitem_T,
    pub lv_watch: *mut listwatch_T,
    pub lv_idx_item: *mut listitem_T,
    pub lv_copylist: *mut list_T,
    pub lv_used_next: *mut list_T,
    pub lv_used_prev: *mut list_T,
    pub lv_refcount: ::core::ffi::c_int,
    pub lv_len: ::core::ffi::c_int,
    pub lv_idx: ::core::ffi::c_int,
    pub lv_copyID: ::core::ffi::c_int,
    pub lv_lock: VarLockStatus,
    pub lua_table_ref: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listwatch_S {
    pub lw_item: *mut listitem_T,
    pub lw_next: *mut listwatch_T,
}
pub type listwatch_T = listwatch_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct partial_S {
    pub pt_refcount: ::core::ffi::c_int,
    pub pt_copyID: ::core::ffi::c_int,
    pub pt_name: *mut ::core::ffi::c_char,
    pub pt_func: *mut ufunc_T,
    pub pt_auto: bool,
    pub pt_argc: ::core::ffi::c_int,
    pub pt_argv: *mut typval_T,
    pub pt_dict: *mut dict_T,
}
pub type partial_T = partial_S;
pub type scid_T = ::core::ffi::c_int;
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct sctx_T {
    pub sc_sid: scid_T,
    pub sc_seq: ::core::ffi::c_int,
    pub sc_lnum: linenr_T,
    pub sc_chan: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct staticList10_T {
    pub sl_list: list_T,
    pub sl_items: [listitem_T; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct typval_T {
    pub v_type: VarType,
    pub v_lock: VarLockStatus,
    pub vval: typval_vval_union,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union typval_vval_union {
    pub v_number: varnumber_T,
    pub v_bool: BoolVarValue,
    pub v_special: SpecialVarValue,
    pub v_float: float_T,
    pub v_string: *mut ::core::ffi::c_char,
    pub v_list: *mut list_T,
    pub v_dict: *mut dict_T,
    pub v_partial: *mut partial_T,
    pub v_blob: *mut blob_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ufunc_S {
    pub uf_varargs: ::core::ffi::c_int,
    pub uf_flags: ::core::ffi::c_int,
    pub uf_calls: ::core::ffi::c_int,
    pub uf_cleared: bool,
    pub uf_args: garray_T,
    pub uf_def_args: garray_T,
    pub uf_lines: garray_T,
    pub uf_profiling: ::core::ffi::c_int,
    pub uf_prof_initialized: ::core::ffi::c_int,
    pub uf_luaref: LuaRef,
    pub uf_tm_count: ::core::ffi::c_int,
    pub uf_tm_total: proftime_T,
    pub uf_tm_self: proftime_T,
    pub uf_tm_children: proftime_T,
    pub uf_tml_count: *mut ::core::ffi::c_int,
    pub uf_tml_total: *mut proftime_T,
    pub uf_tml_self: *mut proftime_T,
    pub uf_tml_start: proftime_T,
    pub uf_tml_children: proftime_T,
    pub uf_tml_wait: proftime_T,
    pub uf_tml_idx: ::core::ffi::c_int,
    pub uf_tml_execed: ::core::ffi::c_int,
    pub uf_script_ctx: sctx_T,
    pub uf_refcount: ::core::ffi::c_int,
    pub uf_scoped: *mut funccall_T,
    pub uf_name_exp: *mut ::core::ffi::c_char,
    pub uf_namelen: size_t,
    pub uf_name: [::core::ffi::c_char; 0],
}
pub type ufunc_T = ufunc_S;
pub type uvarnumber_T = uint64_t;
pub type varnumber_T = int64_t;
