use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    blob_T, blobvar_S, dict_T, dictvar_S, float_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed, funccall_T, garray_T, hash_T, hashitem_T, hashtab_T,
    int32_t, int64_t, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S,
    listwatch_T, partial_S, partial_T, proftime_T, queue, scid_T, sctx_T, size_t, typval_T,
    typval_vval_union, ufunc_S, ufunc_T, uint64_t, uint8_t, varnumber_T, BoolVarValue, LuaRef,
    ScopeDictDictItem, ScopeType, SpecialVarValue, VarLockStatus, VarType, QUEUE,
};
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub static gc_first_dict: GlobalCell<*mut dict_T> =
    GlobalCell::new(::core::ptr::null_mut::<dict_T>());
pub static gc_first_list: GlobalCell<*mut list_T> =
    GlobalCell::new(::core::ptr::null_mut::<list_T>());
