use crate::src::nvim::api::private::helpers::{
    api_set_error, api_typename, arena_array, arena_dict, copy_string, cstr_as_string,
    find_buffer_by_handle, find_window_by_handle, object_to_hl_id, string_to_cstr,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid};
use crate::src::nvim::charset::{transstr, vim_isprintc};
use crate::src::nvim::decoration::{
    clear_virtlines, clear_virttext, decor_free, decor_put_sh, decor_put_vt, decor_range_add_sh,
    decor_range_add_virt, decor_sh_from_inline, decor_to_dict_legacy, hl_group_name,
};
use crate::src::nvim::decoration_provider::{decor_provider_clear, get_decor_provider};
use crate::src::nvim::drawscreen::redraw_all_later;
use crate::src::nvim::extmark::{
    extmark_clear, extmark_del_id, extmark_from_id, extmark_get, extmark_set,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::schar_high;
use crate::src::nvim::main::{
    curtab, decor_state, first_tabpage, firstwin, namespace_ids, namespace_localscope,
    next_namespace_id,
};
use crate::src::nvim::map::{
    map_put_ref_String_int, mh_delete_uint32_t, mh_get_String, mh_get_ptr_t, mh_get_uint32_t,
    mh_put_ptr_t, mh_put_uint32_t,
};
use crate::src::nvim::marktree::mt_inspect;
use crate::src::nvim::mbyte::{mb_string2cells, utfc_ptr2schar};
use crate::src::nvim::memline::ml_get_buf_len;
use crate::src::nvim::memory::{strequal, xfree, xrealloc};
use crate::src::nvim::os::libc::__assert_fail;
use crate::src::nvim::r#move::changed_window_setting;
use crate::src::nvim::sign::init_sign_text;
pub use crate::src::nvim::types::{
    __time_t, alist_T, bcount_t, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T,
    chunksize_T, colnr_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_12, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S, partial_T, pos_T,
    pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, sign_T, size_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_7, varnumber_T, virt_line, visualinfo_T, win_T,
    window_S, wininfo_S, winopt_T, wline_T, xfmark_T, AdditionalData, AlignTextPos, Arena, Array,
    BoolVarValue, Boolean, BufUpdateCallbacks, Buffer, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInline, DecorInlineData, DecorPriority, DecorPriorityInternal, DecorProvider,
    DecorProvider_state as C2Rust_Unnamed_20, DecorRange, DecorRangeKind, DecorRangeSlot,
    DecorRange_data as C2Rust_Unnamed_24, DecorRange_data_ui as C2Rust_Unnamed_25,
    DecorSignHighlight, DecorState, DecorState_ranges_i as C2Rust_Unnamed_22,
    DecorState_slots as C2Rust_Unnamed_23, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2,
    Dict, Error, ErrorType, ExtmarkInfoArray, ExtmarkMove, ExtmarkSavePos, ExtmarkSplice,
    ExtmarkType, ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridView, HLGroupID,
    Integer, Intersection, KeyDict_get_extmark, KeyDict_get_extmarks, KeyDict_ns_opts,
    KeyDict_set_decoration_provider, KeyDict_set_extmark, KeySetLink, KeyValuePair, LuaRef,
    MHPutStatus, MTKey, MTNode, MTPair, MTPos, MapHash, Map_String_int, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_21, Object, ObjectType, OptInt, OptionalKeys,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_String, Set_int64_t, Set_ptr_t, Set_uint32_t,
    Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_13, String_0, Terminal, Timestamp, TriState,
    UndoObjectType, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinInfo, WinSplit, WinStyle, Window, NS, QUEUE,
};
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
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
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kVLScroll: C2Rust_Unnamed_16 = 2;
pub const kVLLeftcol: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kHlModeBlend: C2Rust_Unnamed_17 = 3;
pub const kHlModeCombine: C2Rust_Unnamed_17 = 2;
pub const kHlModeReplace: C2Rust_Unnamed_17 = 1;
pub const kHlModeUnknown: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kSHConcealLines: C2Rust_Unnamed_18 = 128;
pub const kSHConceal: C2Rust_Unnamed_18 = 64;
pub const kSHSpellOff: C2Rust_Unnamed_18 = 32;
pub const kSHSpellOn: C2Rust_Unnamed_18 = 16;
pub const kSHUIWatchedOverlay: C2Rust_Unnamed_18 = 8;
pub const kSHUIWatched: C2Rust_Unnamed_18 = 4;
pub const kSHHlEol: C2Rust_Unnamed_18 = 2;
pub const kSHIsSign: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kVTRepeatLinebreak: C2Rust_Unnamed_19 = 8;
pub const kVTLinesAbove: C2Rust_Unnamed_19 = 4;
pub const kVTHide: C2Rust_Unnamed_19 = 2;
pub const kVTIsLines: C2Rust_Unnamed_19 = 1;
pub const kDecorProviderDisabled: C2Rust_Unnamed_20 = 4;
pub const kDecorProviderRedrawDisabled: C2Rust_Unnamed_20 = 3;
pub const kDecorProviderWinDisabled: C2Rust_Unnamed_20 = 2;
pub const kDecorProviderActive: C2Rust_Unnamed_20 = 1;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
pub const kExtmarkHighlight: ExtmarkType = 32;
pub const kExtmarkVirtLines: ExtmarkType = 16;
pub const kExtmarkVirtText: ExtmarkType = 8;
pub const kExtmarkSignHL: ExtmarkType = 4;
pub const kExtmarkSign: ExtmarkType = 2;
pub const kExtmarkNone: ExtmarkType = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
    pub name: *const ::core::ffi::c_char,
    pub source: *mut LuaRef,
    pub dest: *mut LuaRef,
}
pub const UPD_NOT_VALID: C2Rust_Unnamed_27 = 40;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_27 = 50;
pub const UPD_SOME_VALID: C2Rust_Unnamed_27 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_27 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_27 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_27 = 20;
pub const UPD_VALID: C2Rust_Unnamed_27 = 10;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 87] = unsafe {
    ::core::mem::transmute::<
        [u8; 87],
        [::core::ffi::c_char; 87],
    >(
        *b"void nvim_set_decoration_provider(Integer, KeyDict_set_decoration_provider *, Error *)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    conceal_char: 0 as schar_T,
};
pub const DECOR_SIGN_HIGHLIGHT_INIT: DecorSignHighlight = DecorSignHighlight {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    text: [0 as schar_T, 0 as schar_T],
    sign_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    sign_add_id: 0 as ::core::ffi::c_int,
    number_hl_id: 0 as ::core::ffi::c_int,
    line_hl_id: 0 as ::core::ffi::c_int,
    cursorline_hl_id: 0 as ::core::ffi::c_int,
    next: DECOR_ID_INVALID as uint32_t,
    url: ::core::ptr::null::<::core::ffi::c_char>(),
};
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false_0 != 0,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> bool {
    return mh_get_ptr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn set_put_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut key_alloc: *mut *mut ptr_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_ptr_t(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn set_has_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> bool {
    return mh_get_uint32_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn set_del_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> uint32_t {
    mh_delete_uint32_t(set, &raw mut key);
    return key;
}
#[inline]
unsafe extern "C" fn set_put_uint32_t(
    mut set: *mut Set_uint32_t,
    mut key: uint32_t,
    mut key_alloc: *mut *mut uint32_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_uint32_t(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn map_put_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut value: ::core::ffi::c_int,
) {
    let mut val: *mut ::core::ffi::c_int = map_put_ref_String_int(
        map,
        key,
        ::core::ptr::null_mut::<*mut String_0>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_String(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_int.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub unsafe extern "C" fn nvim_create_namespace(mut name: String_0) -> Integer {
    let mut id: handle_T = map_get_String_int(namespace_ids.ptr(), name);
    if id > 0 as ::core::ffi::c_int {
        return id as Integer;
    }
    let c2rust_fresh0 = next_namespace_id.get();
    next_namespace_id.set(next_namespace_id.get() + 1);
    id = c2rust_fresh0;
    if name.size > 0 as size_t {
        let mut name_alloc: String_0 = copy_string(name, ::core::ptr::null_mut::<Arena>());
        map_put_String_int(namespace_ids.ptr(), name_alloc, id as ::core::ffi::c_int);
    }
    return id as Integer;
}
pub unsafe extern "C" fn nvim_get_namespaces(mut arena: *mut Arena) -> Dict {
    let mut retval: Dict = arena_dict(arena, (*namespace_ids.ptr()).set.h.size as size_t);
    let mut name: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut id: handle_T = 0;
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*namespace_ids.ptr()).set.h.n_keys {
        name = *(*namespace_ids.ptr()).set.keys.offset(__i as isize);
        id = *(*namespace_ids.ptr()).values.offset(__i as isize) as handle_T;
        let c2rust_fresh1 = retval.size;
        retval.size = retval.size.wrapping_add(1);
        *retval.items.offset(c2rust_fresh1 as isize) = key_value_pair {
            key: cstr_as_string(name.data),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: id as Integer,
                },
            },
        };
        __i = __i.wrapping_add(1);
    }
    return retval;
}
pub unsafe extern "C" fn describe_ns(
    mut ns_id: NS,
    mut unknown: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut name: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut id: handle_T = 0;
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*namespace_ids.ptr()).set.h.n_keys {
        name = *(*namespace_ids.ptr()).set.keys.offset(__i as isize);
        id = *(*namespace_ids.ptr()).values.offset(__i as isize) as handle_T;
        if id == ns_id && name.size != 0 {
            return name.data;
        }
        __i = __i.wrapping_add(1);
    }
    return unknown;
}
pub unsafe extern "C" fn ns_initialized(mut ns: uint32_t) -> bool {
    if ns < 1 as uint32_t {
        return false_0 != 0;
    }
    return ns < next_namespace_id.get() as uint32_t;
}
pub unsafe extern "C" fn virt_text_to_array(
    mut vt: VirtText,
    mut hl_name: bool,
    mut arena: *mut Arena,
) -> Array {
    let mut chunks: Array = arena_array(arena, vt.size);
    let mut i: size_t = 0 as size_t;
    while i < vt.size {
        let mut j: size_t = i;
        while j < vt.size {
            if !(*vt.items.offset(j as isize)).text.is_null() {
                break;
            }
            j = j.wrapping_add(1);
        }
        let mut hl_array: Array = arena_array(
            arena,
            if i < j {
                j.wrapping_sub(i).wrapping_add(1 as size_t)
            } else {
                0 as size_t
            },
        );
        while i < j {
            let mut hl_id: ::core::ffi::c_int = (*vt.items.offset(i as isize)).hl_id;
            if hl_id >= 0 as ::core::ffi::c_int {
                let c2rust_fresh2 = hl_array.size;
                hl_array.size = hl_array.size.wrapping_add(1);
                *hl_array.items.offset(c2rust_fresh2 as isize) = hl_group_name(hl_id, hl_name);
            }
            i = i.wrapping_add(1);
        }
        let mut text: *mut ::core::ffi::c_char = (*vt.items.offset(i as isize)).text;
        let mut hl_id_0: ::core::ffi::c_int = (*vt.items.offset(i as isize)).hl_id;
        let mut chunk: Array = arena_array(arena, 2 as size_t);
        let c2rust_fresh3 = chunk.size;
        chunk.size = chunk.size.wrapping_add(1);
        *chunk.items.offset(c2rust_fresh3 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(text),
            },
        };
        if hl_array.size > 0 as size_t {
            if hl_id_0 >= 0 as ::core::ffi::c_int {
                let c2rust_fresh4 = hl_array.size;
                hl_array.size = hl_array.size.wrapping_add(1);
                *hl_array.items.offset(c2rust_fresh4 as isize) = hl_group_name(hl_id_0, hl_name);
            }
            let c2rust_fresh5 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh5 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: hl_array },
            };
        } else if hl_id_0 >= 0 as ::core::ffi::c_int {
            let c2rust_fresh6 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh6 as isize) = hl_group_name(hl_id_0, hl_name);
        }
        let c2rust_fresh7 = chunks.size;
        chunks.size = chunks.size.wrapping_add(1);
        *chunks.items.offset(c2rust_fresh7 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: chunk },
        };
        i = i.wrapping_add(1);
    }
    return chunks;
}
unsafe extern "C" fn extmark_to_array(
    mut extmark: MTPair,
    mut id: bool,
    mut add_dict: bool,
    mut hl_name: bool,
    mut arena: *mut Arena,
) -> Array {
    let mut start: MTKey = extmark.start;
    let mut rv: Array = arena_array(arena, 4 as size_t);
    if id {
        let c2rust_fresh8 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh8 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: start.id as Integer,
            },
        };
    }
    let c2rust_fresh9 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh9 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: start.pos.row as Integer,
        },
    };
    let c2rust_fresh10 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh10 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: start.pos.col as Integer,
        },
    };
    if add_dict {
        let mut dict: Dict = arena_dict(
            arena,
            ::core::mem::size_of::<[KeySetLink; 36]>()
                .wrapping_div(::core::mem::size_of::<KeySetLink>())
                .wrapping_div(
                    (::core::mem::size_of::<[KeySetLink; 36]>()
                        .wrapping_rem(::core::mem::size_of::<KeySetLink>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
        );
        let c2rust_fresh11 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh11 as isize) = key_value_pair {
            key: cstr_as_string(b"ns_id\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: start.ns as Integer,
                },
            },
        };
        let c2rust_fresh12 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh12 as isize) = key_value_pair {
            key: cstr_as_string(b"right_gravity\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: mt_right(start),
                },
            },
        };
        if mt_paired(start) {
            let c2rust_fresh13 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh13 as isize) = key_value_pair {
                key: cstr_as_string(b"end_row\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: extmark.end_pos.row as Integer,
                    },
                },
            };
            let c2rust_fresh14 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh14 as isize) = key_value_pair {
                key: cstr_as_string(b"end_col\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: extmark.end_pos.col as Integer,
                    },
                },
            };
            let c2rust_fresh15 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh15 as isize) = key_value_pair {
                key: cstr_as_string(b"end_right_gravity\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed {
                        boolean: extmark.end_right_gravity,
                    },
                },
            };
        }
        if mt_no_undo(start) {
            let c2rust_fresh16 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh16 as isize) = key_value_pair {
                key: cstr_as_string(b"undo_restore\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: false },
                },
            };
        }
        if mt_invalidate(start) {
            let c2rust_fresh17 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh17 as isize) = key_value_pair {
                key: cstr_as_string(b"invalidate\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        if mt_invalid(start) {
            let c2rust_fresh18 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh18 as isize) = key_value_pair {
                key: cstr_as_string(b"invalid\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        decor_to_dict_legacy(&raw mut dict, mt_decor(start), hl_name, arena);
        let c2rust_fresh19 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh19 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: dict },
        };
    }
    return rv;
}
pub unsafe extern "C" fn nvim_buf_get_extmark_by_id(
    mut buf: Buffer,
    mut ns_id: Integer,
    mut id: Integer,
    mut opts: *mut KeyDict_get_extmark,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return rv;
    }
    if !ns_initialized(ns_id as uint32_t) {
        api_err_invalid(
            err,
            b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return rv;
    }
    let mut details: bool = (*opts).details as bool;
    let mut hl_name: bool = if (*opts).is_set__get_extmark_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_extmark__hl_name
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).hl_name as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut extmark: MTPair = extmark_from_id(b, ns_id as uint32_t, id as uint32_t);
    if extmark.start.pos.row < 0 as int32_t {
        return rv;
    }
    return extmark_to_array(extmark, false_0 != 0, details, hl_name, arena);
}
pub unsafe extern "C" fn nvim_buf_get_extmarks(
    mut buf: Buffer,
    mut ns_id: Integer,
    mut start: Object,
    mut end: Object,
    mut opts: *mut KeyDict_get_extmarks,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return rv;
    }
    if !(ns_id == -1 as Integer || ns_initialized(ns_id as uint32_t) as ::core::ffi::c_int != 0) {
        api_err_invalid(
            err,
            b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return rv;
    }
    let mut details: bool = (*opts).details as bool;
    let mut hl_name: bool = if (*opts).is_set__get_extmarks_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_extmarks__hl_name
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).hl_name as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut type_0: ExtmarkType = kExtmarkNone;
    if (*opts).is_set__get_extmarks_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_extmarks__type
        != 0 as ::core::ffi::c_ulonglong
    {
        if strequal(
            (*opts).type_0.data,
            b"sign\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            type_0 = kExtmarkSign;
        } else if strequal(
            (*opts).type_0.data,
            b"virt_text\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            type_0 = kExtmarkVirtText;
        } else if strequal(
            (*opts).type_0.data,
            b"virt_lines\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            type_0 = kExtmarkVirtLines;
        } else if strequal(
            (*opts).type_0.data,
            b"highlight\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            type_0 = kExtmarkHighlight;
        } else if true {
            api_err_exp(
                err,
                b"type\0".as_ptr() as *const ::core::ffi::c_char,
                b"sign, virt_text, virt_lines or highlight\0".as_ptr()
                    as *const ::core::ffi::c_char,
                (*opts).type_0.data,
            );
            return rv;
        }
    }
    let mut limit: Integer = if (*opts).is_set__get_extmarks_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_extmarks__limit
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).limit
    } else {
        -1 as Integer
    };
    if limit == 0 as Integer {
        return rv;
    } else if limit < 0 as Integer {
        limit = INT64_MAX as Integer;
    }
    let mut l_row: ::core::ffi::c_int = 0;
    let mut l_col: colnr_T = 0;
    if !extmark_get_index_from_obj(b, ns_id, start, &raw mut l_row, &raw mut l_col, err) {
        return rv;
    }
    let mut u_row: ::core::ffi::c_int = 0;
    let mut u_col: colnr_T = 0;
    if !extmark_get_index_from_obj(b, ns_id, end, &raw mut u_row, &raw mut u_col, err) {
        return rv;
    }
    let mut rv_limit: size_t = limit as size_t;
    let mut reverse: bool = l_row > u_row || l_row == u_row && l_col > u_col;
    if reverse {
        limit = INT64_MAX as Integer;
        let mut row: ::core::ffi::c_int = l_row;
        l_row = u_row;
        u_row = row;
        let mut col: colnr_T = l_col;
        l_col = u_col;
        u_col = col;
    }
    let mut marks: ExtmarkInfoArray = extmark_get(
        b,
        ns_id as uint32_t,
        l_row,
        l_col,
        u_row,
        u_col,
        limit,
        type_0,
        (*opts).overlap as bool,
    );
    rv = arena_array(
        arena,
        if marks.size < rv_limit {
            marks.size
        } else {
            rv_limit
        },
    );
    if reverse {
        let mut i: ::core::ffi::c_int = marks.size as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int && rv.size < rv_limit {
            let c2rust_fresh20 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh20 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed {
                    array: extmark_to_array(
                        *marks.items.offset(i as isize),
                        true,
                        details,
                        hl_name,
                        arena,
                    ),
                },
            };
            i -= 1;
        }
    } else {
        let mut i_0: size_t = 0 as size_t;
        while i_0 < marks.size {
            let c2rust_fresh21 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh21 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed {
                    array: extmark_to_array(
                        *marks.items.offset(i_0 as isize),
                        true,
                        details,
                        hl_name,
                        arena,
                    ),
                },
            };
            i_0 = i_0.wrapping_add(1);
        }
    }
    xfree(marks.items as *mut ::core::ffi::c_void);
    marks.capacity = 0 as size_t;
    marks.size = marks.capacity;
    marks.items = ::core::ptr::null_mut::<MTPair>();
    return rv;
}
pub unsafe extern "C" fn nvim_buf_set_extmark(
    mut buf: Buffer,
    mut ns_id: Integer,
    mut line: Integer,
    mut col: Integer,
    mut opts: *mut KeyDict_set_extmark,
    mut err: *mut Error,
) -> Integer {
    let mut id: uint32_t = 0;
    let mut line2: ::core::ffi::c_int = 0;
    let mut did_end_line: bool = false;
    let mut strict: bool = false;
    let mut col2: colnr_T = 0;
    let mut virt_lines_flags: ::core::ffi::c_int = 0;
    let mut right_gravity: bool = false;
    let mut len: colnr_T = 0;
    let mut hl: DecorHighlightInline = DECOR_HIGHLIGHT_INLINE_INIT;
    let mut sign: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    let mut virt_text: DecorVirtText = DecorVirtText {
        flags: 0 as uint8_t,
        hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
        priority: DECOR_PRIORITY_BASE as DecorPriority,
        width: 0 as ::core::ffi::c_int,
        col: 0 as ::core::ffi::c_int,
        pos: kVPosEndOfLine,
        data: C2Rust_Unnamed_2 {
            virt_text: VirtText {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
        },
        next: ::core::ptr::null_mut::<DecorVirtText>(),
    };
    let mut virt_lines: DecorVirtText = DecorVirtText {
        flags: kVTIsLines as ::core::ffi::c_int as uint8_t,
        hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
        priority: DECOR_PRIORITY_BASE as DecorPriority,
        width: 0 as ::core::ffi::c_int,
        col: 0 as ::core::ffi::c_int,
        pos: kVPosEndOfLine,
        data: C2Rust_Unnamed_2 {
            virt_lines: VirtLines {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<virt_line>(),
            },
        },
        next: ::core::ptr::null_mut::<DecorVirtText>(),
    };
    let mut url: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut has_hl: bool = false_0 != 0;
    let mut has_hl_multiple: bool = false_0 != 0;
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    '_error: {
        if !b.is_null() {
            if !ns_initialized(ns_id as uint32_t) {
                api_err_invalid(
                    err,
                    b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    ns_id as int64_t,
                    false_0 != 0,
                );
            } else {
                id = 0 as uint32_t;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__id
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if !((*opts).id > 0 as Integer) {
                        api_err_exp(
                            err,
                            b"id\0".as_ptr() as *const ::core::ffi::c_char,
                            b"positive Integer\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        );
                        break '_error;
                    } else {
                        id = (*opts).id as uint32_t;
                    }
                }
                line2 = -1 as ::core::ffi::c_int;
                did_end_line = false_0 != 0;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__end_line
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 10 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                            b"cannot use both 'end_row' and 'end_line'\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_error;
                    } else {
                        (*opts).end_row = (*opts).end_line;
                        did_end_line = true_0 != 0;
                    }
                }
                strict = if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__strict
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*opts).strict as ::core::ffi::c_int
                } else {
                    true_0
                } != 0;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__end_row
                    != 0 as ::core::ffi::c_ulonglong
                    || did_end_line as ::core::ffi::c_int != 0
                {
                    let mut val: Integer = (*opts).end_row;
                    if !(val >= 0 as Integer
                        && !(val > (*b).b_ml.ml_line_count as Integer
                            && strict as ::core::ffi::c_int != 0))
                    {
                        api_err_invalid(
                            err,
                            b"end_row\0".as_ptr() as *const ::core::ffi::c_char,
                            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            false_0 != 0,
                        );
                        break '_error;
                    } else {
                        line2 = val as ::core::ffi::c_int;
                    }
                }
                col2 = -1 as colnr_T;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__end_col
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut val_0: Integer = (*opts).end_col;
                    if !(val_0 >= -1 as Integer && val_0 <= MAXCOL as ::core::ffi::c_int as Integer)
                    {
                        api_err_invalid(
                            err,
                            b"end_col\0".as_ptr() as *const ::core::ffi::c_char,
                            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            false_0 != 0,
                        );
                        break '_error;
                    } else {
                        if val_0 == -1 as Integer {
                            val_0 = MAXCOL as ::core::ffi::c_int as Integer;
                        }
                        col2 = val_0 as ::core::ffi::c_int as colnr_T;
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__hl_group
                    != 0 as ::core::ffi::c_ulonglong
                {
                    's_293: {
                        if (*opts).hl_group.type_0 as ::core::ffi::c_uint
                            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            let mut arr: Array = (*opts).hl_group.data.array;
                            if arr.size >= 1 as size_t {
                                hl.hl_id = object_to_hl_id(
                                    *arr.items.offset(0 as ::core::ffi::c_int as isize),
                                    b"hl_group item\0".as_ptr() as *const ::core::ffi::c_char,
                                    err,
                                );
                                if (*err).type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    break '_error;
                                }
                            }
                            let mut i: size_t = 1 as size_t;
                            loop {
                                if i >= arr.size {
                                    break 's_293;
                                }
                                let mut hl_id: ::core::ffi::c_int = object_to_hl_id(
                                    *arr.items.offset(i as isize),
                                    b"hl_group item\0".as_ptr() as *const ::core::ffi::c_char,
                                    err,
                                );
                                if (*err).type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    break '_error;
                                }
                                if hl_id != 0 {
                                    has_hl_multiple = true_0 != 0;
                                }
                                i = i.wrapping_add(1);
                            }
                        } else {
                            hl.hl_id = object_to_hl_id(
                                (*opts).hl_group,
                                b"hl_group\0".as_ptr() as *const ::core::ffi::c_char,
                                err,
                            );
                            if (*err).type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                break '_error;
                            }
                        }
                    }
                    has_hl = hl.hl_id > 0 as ::core::ffi::c_int;
                }
                sign.hl_id = (*opts).sign_hl_group as ::core::ffi::c_int;
                sign.cursorline_hl_id = (*opts).cursorline_hl_group as ::core::ffi::c_int;
                sign.number_hl_id = (*opts).number_hl_group as ::core::ffi::c_int;
                sign.line_hl_id = (*opts).line_hl_group as ::core::ffi::c_int;
                if sign.hl_id != 0
                    || sign.cursorline_hl_id != 0
                    || sign.number_hl_id != 0
                    || sign.line_hl_id != 0
                {
                    sign.flags = (sign.flags as ::core::ffi::c_int
                        | kSHIsSign as ::core::ffi::c_int)
                        as uint16_t;
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__conceal
                    != 0 as ::core::ffi::c_ulonglong
                {
                    hl.flags = (hl.flags as ::core::ffi::c_int | kSHConceal as ::core::ffi::c_int)
                        as uint16_t;
                    has_hl = true_0 != 0;
                    if (*opts).conceal.size > 0 as size_t {
                        let mut ch: ::core::ffi::c_int = 0;
                        hl.conceal_char = utfc_ptr2schar((*opts).conceal.data, &raw mut ch);
                        if !(hl.conceal_char != 0 && vim_isprintc(ch) as ::core::ffi::c_int != 0) {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                b"conceal char has to be printable\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_error;
                        }
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__conceal_lines
                    != 0 as ::core::ffi::c_ulonglong
                {
                    hl.flags = (hl.flags as ::core::ffi::c_int
                        | kSHConcealLines as ::core::ffi::c_int)
                        as uint16_t;
                    has_hl = true_0 != 0;
                    if (*opts).conceal_lines.size > 0 as size_t {
                        if !(*(*opts).conceal_lines.data as ::core::ffi::c_int
                            == '\0' as ::core::ffi::c_int)
                        {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                b"conceal_lines has to be an empty string\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_error;
                        }
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__virt_text
                    != 0 as ::core::ffi::c_ulonglong
                {
                    virt_text.data.virt_text =
                        parse_virt_text((*opts).virt_text, err, &raw mut virt_text.width);
                    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        break '_error;
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__virt_text_pos
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut str: String_0 = (*opts).virt_text_pos;
                    if strequal(b"eol\0".as_ptr() as *const ::core::ffi::c_char, str.data) {
                        virt_text.pos = kVPosEndOfLine;
                    } else if strequal(
                        b"overlay\0".as_ptr() as *const ::core::ffi::c_char,
                        str.data,
                    ) {
                        virt_text.pos = kVPosOverlay;
                    } else if strequal(
                        b"right_align\0".as_ptr() as *const ::core::ffi::c_char,
                        str.data,
                    ) {
                        virt_text.pos = kVPosRightAlign;
                    } else if strequal(
                        b"eol_right_align\0".as_ptr() as *const ::core::ffi::c_char,
                        str.data,
                    ) {
                        virt_text.pos = kVPosEndOfLineRightAlign;
                    } else if strequal(b"inline\0".as_ptr() as *const ::core::ffi::c_char, str.data)
                    {
                        virt_text.pos = kVPosInline;
                    } else if true {
                        api_err_invalid(
                            err,
                            b"virt_text_pos\0".as_ptr() as *const ::core::ffi::c_char,
                            str.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_error;
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong)
                        << KEYSET_OPTIDX_set_extmark__virt_text_win_col
                    != 0 as ::core::ffi::c_ulonglong
                {
                    virt_text.col = (*opts).virt_text_win_col as ::core::ffi::c_int;
                    virt_text.pos = kVPosWinCol;
                }
                hl.flags = (hl.flags as ::core::ffi::c_int
                    | if (*opts).hl_eol as ::core::ffi::c_int != 0 {
                        kSHHlEol as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as uint16_t;
                virt_text.flags = (virt_text.flags as ::core::ffi::c_int
                    | ((if (*opts).virt_text_hide as ::core::ffi::c_int != 0 {
                        kVTHide as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) | (if (*opts).virt_text_repeat_linebreak as ::core::ffi::c_int != 0 {
                        kVTRepeatLinebreak as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }))) as uint8_t;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__hl_mode
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut str_0: String_0 = (*opts).hl_mode;
                    if strequal(
                        b"replace\0".as_ptr() as *const ::core::ffi::c_char,
                        str_0.data,
                    ) {
                        virt_text.hl_mode = kHlModeReplace as ::core::ffi::c_int as uint8_t;
                    } else if strequal(
                        b"combine\0".as_ptr() as *const ::core::ffi::c_char,
                        str_0.data,
                    ) {
                        virt_text.hl_mode = kHlModeCombine as ::core::ffi::c_int as uint8_t;
                    } else if strequal(
                        b"blend\0".as_ptr() as *const ::core::ffi::c_char,
                        str_0.data,
                    ) {
                        if virt_text.pos as ::core::ffi::c_uint
                            == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if true {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"cannot use 'blend' hl_mode with inline virtual text\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                                break '_error;
                            }
                        }
                        virt_text.hl_mode = kHlModeBlend as ::core::ffi::c_int as uint8_t;
                    } else if true {
                        api_err_invalid(
                            err,
                            b"hl_mode\0".as_ptr() as *const ::core::ffi::c_char,
                            str_0.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_error;
                    }
                }
                virt_lines_flags = if (*opts).virt_lines_leftcol as ::core::ffi::c_int != 0 {
                    kVLLeftcol as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong)
                        << KEYSET_OPTIDX_set_extmark__virt_lines_overflow
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut str_1: String_0 = (*opts).virt_lines_overflow;
                    if strequal(
                        b"scroll\0".as_ptr() as *const ::core::ffi::c_char,
                        str_1.data,
                    ) {
                        virt_lines_flags |= kVLScroll as ::core::ffi::c_int;
                    } else if !strequal(
                        b"trunc\0".as_ptr() as *const ::core::ffi::c_char,
                        str_1.data,
                    ) {
                        if true {
                            api_err_invalid(
                                err,
                                b"virt_lines_overflow\0".as_ptr() as *const ::core::ffi::c_char,
                                str_1.data,
                                0 as int64_t,
                                true_0 != 0,
                            );
                            break '_error;
                        }
                    }
                }
                's_785: {
                    if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__virt_lines
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        let mut a: Array = (*opts).virt_lines;
                        let mut j: size_t = 0 as size_t;
                        loop {
                            if j >= a.size {
                                break 's_785;
                            }
                            if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                                != (*a.items.offset(j as isize)).type_0 as ::core::ffi::c_uint
                            {
                                api_err_exp(
                                    err,
                                    b"virt_text_line\0".as_ptr() as *const ::core::ffi::c_char,
                                    api_typename(kObjectTypeArray),
                                    api_typename((*a.items.offset(j as isize)).type_0),
                                );
                                break '_error;
                            } else {
                                let mut dummig: ::core::ffi::c_int = 0;
                                let mut jtem: VirtText = parse_virt_text(
                                    (*a.items.offset(j as isize)).data.array,
                                    err,
                                    &raw mut dummig,
                                );
                                if virt_lines.data.virt_lines.size
                                    == virt_lines.data.virt_lines.capacity
                                {
                                    virt_lines.data.virt_lines.capacity =
                                        if virt_lines.data.virt_lines.capacity != 0 {
                                            virt_lines.data.virt_lines.capacity
                                                << 1 as ::core::ffi::c_int
                                        } else {
                                            8 as size_t
                                        };
                                    virt_lines.data.virt_lines.items = xrealloc(
                                        virt_lines.data.virt_lines.items
                                            as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<virt_line>()
                                            .wrapping_mul(virt_lines.data.virt_lines.capacity),
                                    )
                                        as *mut virt_line;
                                } else {
                                };
                                let c2rust_fresh22 = virt_lines.data.virt_lines.size;
                                virt_lines.data.virt_lines.size =
                                    virt_lines.data.virt_lines.size.wrapping_add(1);
                                *virt_lines
                                    .data
                                    .virt_lines
                                    .items
                                    .offset(c2rust_fresh22 as isize) = virt_line {
                                    line: jtem,
                                    flags: virt_lines_flags,
                                }
                                    as virt_line;
                                if (*err).type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    break '_error;
                                }
                                j = j.wrapping_add(1);
                            }
                        }
                    }
                }
                virt_lines.flags = (virt_lines.flags as ::core::ffi::c_int
                    | if (*opts).virt_lines_above as ::core::ffi::c_int != 0 {
                        kVTLinesAbove as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as uint8_t;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__priority
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if !((*opts).priority >= 0 as Integer && (*opts).priority <= 65535 as Integer) {
                        api_err_invalid(
                            err,
                            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
                            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            false_0 != 0,
                        );
                        break '_error;
                    } else {
                        hl.priority = (*opts).priority as DecorPriority;
                        sign.priority = (*opts).priority as DecorPriority;
                        virt_text.priority = (*opts).priority as DecorPriority;
                        virt_lines.priority = (*opts).priority as DecorPriority;
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__sign_text
                    != 0 as ::core::ffi::c_ulonglong
                {
                    sign.text[0 as ::core::ffi::c_int as usize] = 0 as schar_T;
                    if init_sign_text(
                        ::core::ptr::null_mut::<sign_T>(),
                        &raw mut sign.text as *mut schar_T,
                        (*opts).sign_text.data,
                    ) == 0
                    {
                        api_err_invalid(
                            err,
                            b"sign_text\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_error;
                    } else {
                        sign.flags = (sign.flags as ::core::ffi::c_int
                            | kSHIsSign as ::core::ffi::c_int)
                            as uint16_t;
                    }
                }
                right_gravity = if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__right_gravity
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*opts).right_gravity as ::core::ffi::c_int
                } else {
                    true_0
                } != 0;
                if line2 == -1 as ::core::ffi::c_int
                    && col2 == -1 as ::core::ffi::c_int
                    && (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 30 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"cannot set end_right_gravity without end_row or end_col\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                } else {
                    len = 0 as colnr_T;
                    if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__spell
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        hl.flags = (hl.flags as ::core::ffi::c_int
                            | if (*opts).spell as ::core::ffi::c_int != 0 {
                                kSHSpellOn as ::core::ffi::c_int
                            } else {
                                kSHSpellOff as ::core::ffi::c_int
                            }) as uint16_t;
                        has_hl = true_0 != 0;
                    }
                    if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__url
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        url = string_to_cstr((*opts).url);
                        has_hl = true_0 != 0;
                    }
                    if (*opts).ui_watched {
                        hl.flags = (hl.flags as ::core::ffi::c_int
                            | kSHUIWatched as ::core::ffi::c_int)
                            as uint16_t;
                        if virt_text.pos as ::core::ffi::c_uint
                            == kVPosOverlay as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            hl.flags = (hl.flags as ::core::ffi::c_int
                                | kSHUIWatchedOverlay as ::core::ffi::c_int)
                                as uint16_t;
                        }
                        has_hl = true_0 != 0;
                    }
                    if !(line >= 0 as Integer) {
                        api_err_invalid(
                            err,
                            b"line\0".as_ptr() as *const ::core::ffi::c_char,
                            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            false_0 != 0,
                        );
                    } else {
                        if line > (*b).b_ml.ml_line_count as Integer {
                            if strict {
                                api_err_invalid(
                                    err,
                                    b"line\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    false_0 != 0,
                                );
                                break '_error;
                            } else {
                                line = (*b).b_ml.ml_line_count as Integer;
                            }
                        } else if line < (*b).b_ml.ml_line_count as Integer {
                            len = (if (*opts).ephemeral as ::core::ffi::c_int != 0 {
                                MAXCOL as ::core::ffi::c_int
                            } else {
                                ml_get_buf_len(b, line as linenr_T + 1 as linenr_T)
                            }) as colnr_T;
                        }
                        if col == -1 as Integer {
                            col = len as Integer;
                        } else if col > len as Integer {
                            if strict {
                                api_err_invalid(
                                    err,
                                    b"col\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    false_0 != 0,
                                );
                                break '_error;
                            } else {
                                col = len as Integer;
                            }
                        } else if col < -1 as Integer {
                            if true {
                                api_err_invalid(
                                    err,
                                    b"col\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    false_0 != 0,
                                );
                                break '_error;
                            }
                        }
                        if col2 >= 0 as ::core::ffi::c_int {
                            if line2 >= 0 as ::core::ffi::c_int
                                && (line2 as linenr_T) < (*b).b_ml.ml_line_count
                            {
                                len = (if (*opts).ephemeral as ::core::ffi::c_int != 0 {
                                    MAXCOL as ::core::ffi::c_int
                                } else {
                                    ml_get_buf_len(b, line2 as linenr_T + 1 as linenr_T)
                                }) as colnr_T;
                            } else if line2 as linenr_T == (*b).b_ml.ml_line_count {
                                len = 0 as ::core::ffi::c_int as colnr_T;
                            } else {
                                line2 = line as ::core::ffi::c_int;
                            }
                            if col2 > len {
                                if strict {
                                    api_err_invalid(
                                        err,
                                        b"end_col\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                                        0 as int64_t,
                                        false_0 != 0,
                                    );
                                    break '_error;
                                } else {
                                    col2 = len;
                                }
                            }
                        } else if line2 >= 0 as ::core::ffi::c_int {
                            col2 = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        if (*opts).ephemeral as ::core::ffi::c_int != 0
                            && !(*decor_state.ptr()).win.is_null()
                            && (*(*decor_state.ptr()).win).w_buffer == b
                        {
                            let mut r: ::core::ffi::c_int = line as ::core::ffi::c_int;
                            let mut c: ::core::ffi::c_int = col as ::core::ffi::c_int;
                            if line2 == -1 as ::core::ffi::c_int {
                                line2 = r;
                                col2 = c as colnr_T;
                            }
                            let mut subpriority: DecorPriority = 0 as DecorPriority;
                            if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                                & (1 as ::core::ffi::c_ulonglong)
                                    << KEYSET_OPTIDX_set_extmark___subpriority
                                != 0 as ::core::ffi::c_ulonglong
                            {
                                if !((*opts)._subpriority >= 0 as Integer
                                    && (*opts)._subpriority <= 65535 as Integer)
                                {
                                    api_err_invalid(
                                        err,
                                        b"_subpriority\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                                        0 as int64_t,
                                        false_0 != 0,
                                    );
                                    break '_error;
                                } else {
                                    subpriority = (*opts)._subpriority as DecorPriority;
                                }
                            }
                            if virt_text.data.virt_text.size != 0 {
                                decor_range_add_virt(
                                    decor_state.ptr(),
                                    r,
                                    c,
                                    line2,
                                    col2 as ::core::ffi::c_int,
                                    decor_put_vt(
                                        virt_text,
                                        ::core::ptr::null_mut::<DecorVirtText>(),
                                    ),
                                    true_0 != 0,
                                );
                            }
                            if virt_lines.data.virt_lines.size != 0 {
                                decor_range_add_virt(
                                    decor_state.ptr(),
                                    r,
                                    c,
                                    line2,
                                    col2 as ::core::ffi::c_int,
                                    decor_put_vt(
                                        virt_lines,
                                        ::core::ptr::null_mut::<DecorVirtText>(),
                                    ),
                                    true_0 != 0,
                                );
                            }
                            if has_hl {
                                let mut sh: DecorSignHighlight = decor_sh_from_inline(hl);
                                sh.url = url;
                                decor_range_add_sh(
                                    decor_state.ptr(),
                                    r,
                                    c,
                                    line2,
                                    col2 as ::core::ffi::c_int,
                                    &raw mut sh,
                                    true_0 != 0,
                                    ns_id as uint32_t,
                                    id,
                                    subpriority,
                                );
                            }
                        } else if (*opts).ephemeral {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                b"cannot set emphemeral mark outside of a decoration provider\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_error;
                        } else {
                            let mut decor_flags: uint16_t = 0 as uint16_t;
                            let mut decor_alloc: *mut DecorVirtText =
                                ::core::ptr::null_mut::<DecorVirtText>();
                            if virt_text.data.virt_text.size != 0 {
                                decor_alloc = decor_put_vt(virt_text, decor_alloc);
                                if virt_text.pos as ::core::ffi::c_uint
                                    == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    decor_flags = (decor_flags as ::core::ffi::c_int
                                        | MT_FLAG_DECOR_VIRT_TEXT_INLINE)
                                        as uint16_t;
                                }
                            }
                            if virt_lines.data.virt_lines.size != 0 {
                                decor_alloc = decor_put_vt(virt_lines, decor_alloc);
                                decor_flags = (decor_flags as ::core::ffi::c_int
                                    | MT_FLAG_DECOR_VIRT_LINES)
                                    as uint16_t;
                            }
                            let mut decor_indexed: uint32_t = DECOR_ID_INVALID as uint32_t;
                            if sign.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int
                                != 0
                            {
                                sign.next = decor_indexed;
                                decor_indexed = decor_put_sh(sign);
                                if sign.text[0 as ::core::ffi::c_int as usize] != 0 {
                                    decor_flags = (decor_flags as ::core::ffi::c_int
                                        | MT_FLAG_DECOR_SIGNTEXT)
                                        as uint16_t;
                                }
                                if sign.number_hl_id != 0
                                    || sign.line_hl_id != 0
                                    || sign.cursorline_hl_id != 0
                                {
                                    decor_flags = (decor_flags as ::core::ffi::c_int
                                        | MT_FLAG_DECOR_SIGNHL)
                                        as uint16_t;
                                }
                            }
                            if has_hl_multiple {
                                let mut arr_0: Array = (*opts).hl_group.data.array;
                                let mut i_0: size_t = arr_0.size.wrapping_sub(1 as size_t);
                                while i_0 > 0 as size_t {
                                    let mut hl_id_0: ::core::ffi::c_int = object_to_hl_id(
                                        *arr_0.items.offset(i_0 as isize),
                                        b"hl_group item\0".as_ptr() as *const ::core::ffi::c_char,
                                        err,
                                    );
                                    if hl_id_0 > 0 as ::core::ffi::c_int {
                                        let mut sh_0: DecorSignHighlight =
                                            DECOR_SIGN_HIGHLIGHT_INIT;
                                        sh_0.hl_id = hl_id_0;
                                        sh_0.flags = (if (*opts).hl_eol as ::core::ffi::c_int != 0 {
                                            kSHHlEol as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        })
                                            as uint16_t;
                                        sh_0.next = decor_indexed;
                                        decor_indexed = decor_put_sh(sh_0);
                                        decor_flags = (decor_flags as ::core::ffi::c_int
                                            | MT_FLAG_DECOR_HL)
                                            as uint16_t;
                                    }
                                    i_0 = i_0.wrapping_sub(1);
                                }
                            }
                            if hl.flags as ::core::ffi::c_int
                                & kSHConcealLines as ::core::ffi::c_int
                                != 0
                            {
                                decor_flags = (decor_flags as ::core::ffi::c_int
                                    | MT_FLAG_DECOR_CONCEAL_LINES)
                                    as uint16_t;
                            }
                            let mut decor: DecorInline = DECOR_INLINE_INIT;
                            if !decor_alloc.is_null()
                                || decor_indexed != DECOR_ID_INVALID as uint32_t
                                || !url.is_null()
                                || schar_high(hl.conceal_char) as ::core::ffi::c_int != 0
                            {
                                if has_hl {
                                    let mut sh_1: DecorSignHighlight = decor_sh_from_inline(hl);
                                    sh_1.url = url;
                                    sh_1.next = decor_indexed;
                                    decor_indexed = decor_put_sh(sh_1);
                                }
                                decor.ext = true_0 != 0;
                                decor.data.ext = DecorExt {
                                    sh_idx: decor_indexed,
                                    vt: decor_alloc,
                                };
                            } else {
                                decor.data.hl = hl;
                            }
                            if has_hl {
                                decor_flags = (decor_flags as ::core::ffi::c_int | MT_FLAG_DECOR_HL)
                                    as uint16_t;
                            }
                            extmark_set(
                                b,
                                ns_id as uint32_t,
                                &raw mut id,
                                line as ::core::ffi::c_int,
                                col as colnr_T,
                                line2,
                                col2,
                                decor,
                                decor_flags,
                                right_gravity,
                                (*opts).end_right_gravity as bool,
                                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_set_extmark__undo_restore
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*opts).undo_restore as ::core::ffi::c_int
                                } else {
                                    true_0
                                } == 0,
                                (*opts).invalidate as bool,
                                err,
                            );
                            if (*err).type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                decor_free(decor);
                                return 0 as Integer;
                            }
                        }
                        return id as Integer;
                    }
                }
            }
        }
    }
    clear_virttext(&raw mut virt_text.data.virt_text);
    clear_virtlines(&raw mut virt_lines.data.virt_lines);
    if !url.is_null() {
        xfree(url as *mut ::core::ffi::c_void);
    }
    return 0 as Integer;
}
pub unsafe extern "C" fn nvim_buf_del_extmark(
    mut buf: Buffer,
    mut ns_id: Integer,
    mut id: Integer,
    mut err: *mut Error,
) -> Boolean {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return false_0 != 0;
    }
    if !ns_initialized(ns_id as uint32_t) {
        api_err_invalid(
            err,
            b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return false;
    }
    return extmark_del_id(b, ns_id as uint32_t, id as uint32_t);
}
pub unsafe extern "C" fn nvim_buf_clear_namespace(
    mut buf: Buffer,
    mut ns_id: Integer,
    mut line_start: Integer,
    mut line_end: Integer,
    mut err: *mut Error,
) {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return;
    }
    if !(line_start >= 0 as Integer && line_start < MAXLNUM as ::core::ffi::c_int as Integer) {
        api_err_invalid(
            err,
            b"line number\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return;
    }
    if line_end < 0 as Integer || line_end > MAXLNUM as ::core::ffi::c_int as Integer {
        line_end = MAXLNUM as ::core::ffi::c_int as Integer;
    }
    extmark_clear(
        b,
        if ns_id < 0 as Integer {
            0 as uint32_t
        } else {
            ns_id as uint32_t
        },
        line_start as ::core::ffi::c_int,
        0 as colnr_T,
        line_end as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        MAXCOL as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn nvim_set_decoration_provider(
    mut ns_id: Integer,
    mut opts: *mut KeyDict_set_decoration_provider,
    mut _err: *mut Error,
) {
    let mut p: *mut DecorProvider = get_decor_provider(ns_id as NS, true_0 != 0);
    '_c2rust_label: {
        if !p.is_null() {
        } else {
            __assert_fail(
                b"p != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/extmark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1083 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    decor_provider_clear(p);
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    let mut cbs: [C2Rust_Unnamed_26; 10] = [
        C2Rust_Unnamed_26 {
            name: b"on_start\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_start,
            dest: &raw mut (*p).redraw_start,
        },
        C2Rust_Unnamed_26 {
            name: b"on_buf\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_buf,
            dest: &raw mut (*p).redraw_buf,
        },
        C2Rust_Unnamed_26 {
            name: b"on_win\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_win,
            dest: &raw mut (*p).redraw_win,
        },
        C2Rust_Unnamed_26 {
            name: b"on_line\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_line,
            dest: &raw mut (*p).redraw_line,
        },
        C2Rust_Unnamed_26 {
            name: b"on_range\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_range,
            dest: &raw mut (*p).redraw_range,
        },
        C2Rust_Unnamed_26 {
            name: b"on_end\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_end,
            dest: &raw mut (*p).redraw_end,
        },
        C2Rust_Unnamed_26 {
            name: b"_on_hl_def\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts)._on_hl_def,
            dest: &raw mut (*p).hl_def,
        },
        C2Rust_Unnamed_26 {
            name: b"_on_spell_nav\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts)._on_spell_nav,
            dest: &raw mut (*p).spell_nav,
        },
        C2Rust_Unnamed_26 {
            name: b"_on_conceal_line\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts)._on_conceal_line,
            dest: &raw mut (*p).conceal_line,
        },
        C2Rust_Unnamed_26 {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            source: ::core::ptr::null_mut::<LuaRef>(),
            dest: ::core::ptr::null_mut::<LuaRef>(),
        },
    ];
    let mut i: size_t = 0 as size_t;
    while !cbs[i as usize].source.is_null()
        && !cbs[i as usize].dest.is_null()
        && !cbs[i as usize].name.is_null()
    {
        let mut v: *mut LuaRef = cbs[i as usize].source;
        if *v > 0 as ::core::ffi::c_int {
            *cbs[i as usize].dest = *v;
            *v = LUA_NOREF as LuaRef;
        }
        i = i.wrapping_add(1);
    }
    (*p).state = kDecorProviderActive;
    (*p).hl_valid += 1;
    (*p).hl_cached = false_0 != 0;
}
unsafe extern "C" fn extmark_get_index_from_obj(
    mut buf: *mut buf_T,
    mut ns_id: Integer,
    mut obj: Object,
    mut row: *mut ::core::ffi::c_int,
    mut col: *mut colnr_T,
    mut err: *mut Error,
) -> bool {
    if obj.type_0 as ::core::ffi::c_uint
        == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut id: Integer = obj.data.integer;
        if id == 0 as Integer {
            *row = 0 as ::core::ffi::c_int;
            *col = 0 as ::core::ffi::c_int as colnr_T;
            return true_0 != 0;
        } else if id == -1 as Integer {
            *row = MAXLNUM as ::core::ffi::c_int;
            *col = MAXCOL as ::core::ffi::c_int as colnr_T;
            return true_0 != 0;
        } else if id < 0 as Integer {
            if true {
                api_err_invalid(
                    err,
                    b"mark id\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    id as int64_t,
                    false_0 != 0,
                );
                return false;
            }
        }
        let mut extmark: MTPair = extmark_from_id(buf, ns_id as uint32_t, id as uint32_t);
        if !(extmark.start.pos.row >= 0 as int32_t) {
            api_err_invalid(
                err,
                b"mark id (not found)\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
                id as int64_t,
                false_0 != 0,
            );
            return false;
        }
        *row = extmark.start.pos.row as ::core::ffi::c_int;
        *col = extmark.start.pos.col as colnr_T;
        return true_0 != 0;
    } else if obj.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut pos: Array = obj.data.array;
        if !(pos.size == 2 as size_t
            && (*pos.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*pos.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            api_err_exp(
                err,
                b"mark position\0".as_ptr() as *const ::core::ffi::c_char,
                b"2 Integer items\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return false;
        }
        let mut pos_row: Integer = (*pos.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .integer;
        let mut pos_col: Integer = (*pos.items.offset(1 as ::core::ffi::c_int as isize))
            .data
            .integer;
        *row = (if pos_row >= 0 as Integer {
            pos_row
        } else {
            MAXLNUM as ::core::ffi::c_int as Integer
        }) as ::core::ffi::c_int;
        *col = (if pos_col >= 0 as Integer {
            pos_col
        } else {
            MAXCOL as ::core::ffi::c_int as Integer
        }) as colnr_T;
        return true_0 != 0;
    } else if true {
        api_err_exp(
            err,
            b"mark position\0".as_ptr() as *const ::core::ffi::c_char,
            b"mark id Integer or 2-item Array\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        return false;
    }
    panic!("Reached end of non-void function without returning");
}
pub unsafe extern "C" fn parse_virt_text(
    mut chunks: Array,
    mut err: *mut Error,
    mut width: *mut ::core::ffi::c_int,
) -> VirtText {
    let mut virt_text: VirtText = VirtText {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    };
    let mut w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    '_free_exit: {
        while i < chunks.size {
            if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                != (*chunks.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
            {
                api_err_exp(
                    err,
                    b"chunk\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(kObjectTypeArray),
                    api_typename((*chunks.items.offset(i as isize)).type_0),
                );
                break '_free_exit;
            } else {
                let mut chunk: Array = (*chunks.items.offset(i as isize)).data.array;
                if !(chunk.size > 0 as size_t
                    && chunk.size <= 2 as size_t
                    && (*chunk.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                        as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"Invalid chunk: expected Array with 1 or 2 Strings\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                    break '_free_exit;
                } else {
                    let mut str: String_0 = (*chunk.items.offset(0 as ::core::ffi::c_int as isize))
                        .data
                        .string;
                    let mut hl_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                    's_146: {
                        if chunk.size == 2 as size_t {
                            let mut hl: Object =
                                *chunk.items.offset(1 as ::core::ffi::c_int as isize);
                            if hl.type_0 as ::core::ffi::c_uint
                                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                let mut arr: Array = hl.data.array;
                                let mut j: size_t = 0 as size_t;
                                loop {
                                    if j >= arr.size {
                                        break 's_146;
                                    }
                                    hl_id = object_to_hl_id(
                                        *arr.items.offset(j as isize),
                                        b"virt_text highlight\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        err,
                                    );
                                    if (*err).type_0 as ::core::ffi::c_int
                                        != kErrorTypeNone as ::core::ffi::c_int
                                    {
                                        break '_free_exit;
                                    }
                                    if j < arr.size.wrapping_sub(1 as size_t) {
                                        if virt_text.size == virt_text.capacity {
                                            virt_text.capacity = if virt_text.capacity != 0 {
                                                virt_text.capacity << 1 as ::core::ffi::c_int
                                            } else {
                                                8 as size_t
                                            };
                                            virt_text.items = xrealloc(
                                                virt_text.items as *mut ::core::ffi::c_void,
                                                ::core::mem::size_of::<VirtTextChunk>()
                                                    .wrapping_mul(virt_text.capacity),
                                            )
                                                as *mut VirtTextChunk;
                                        } else {
                                        };
                                        let c2rust_fresh23 = virt_text.size;
                                        virt_text.size = virt_text.size.wrapping_add(1);
                                        *virt_text.items.offset(c2rust_fresh23 as isize) =
                                            VirtTextChunk {
                                                text: ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                ),
                                                hl_id: hl_id,
                                            };
                                    }
                                    j = j.wrapping_add(1);
                                }
                            } else {
                                hl_id = object_to_hl_id(
                                    hl,
                                    b"virt_text highlight\0".as_ptr() as *const ::core::ffi::c_char,
                                    err,
                                );
                                if (*err).type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    break '_free_exit;
                                }
                            }
                        }
                    }
                    let mut text: *mut ::core::ffi::c_char = transstr(
                        if str.size > 0 as size_t {
                            str.data as *const ::core::ffi::c_char
                        } else {
                            b"\0".as_ptr() as *const ::core::ffi::c_char
                        },
                        false_0 != 0,
                    );
                    w += mb_string2cells(text) as ::core::ffi::c_int;
                    if virt_text.size == virt_text.capacity {
                        virt_text.capacity = if virt_text.capacity != 0 {
                            virt_text.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        virt_text.items = xrealloc(
                            virt_text.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<VirtTextChunk>()
                                .wrapping_mul(virt_text.capacity),
                        ) as *mut VirtTextChunk;
                    } else {
                    };
                    let c2rust_fresh24 = virt_text.size;
                    virt_text.size = virt_text.size.wrapping_add(1);
                    *virt_text.items.offset(c2rust_fresh24 as isize) = VirtTextChunk {
                        text: text,
                        hl_id: hl_id,
                    };
                    i = i.wrapping_add(1);
                }
            }
        }
        if !width.is_null() {
            *width = w;
        }
        return virt_text;
    }
    clear_virttext(&raw mut virt_text);
    return virt_text;
}
pub unsafe extern "C" fn nvim__buf_debug_extmarks(
    mut buf: Buffer,
    mut keys: Boolean,
    mut dot: Boolean,
    mut err: *mut Error,
) -> String_0 {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return NULL_STRING;
    }
    return mt_inspect(
        &raw mut (*b).b_marktree as *mut MarkTree,
        keys as bool,
        dot as bool,
    );
}
pub unsafe extern "C" fn nvim__ns_set(
    mut ns_id: Integer,
    mut opts: *mut KeyDict_ns_opts,
    mut err: *mut Error,
) {
    if !ns_initialized(ns_id as uint32_t) {
        api_err_invalid(
            err,
            b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return;
    }
    let mut set_scoped: bool = true_0 != 0;
    if (*opts).is_set__ns_opts_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_ns_opts__wins
        != 0 as ::core::ffi::c_ulonglong
    {
        if (*opts).wins.size == 0 as size_t {
            set_scoped = false_0 != 0;
        }
        let mut windows: Set_ptr_t = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        };
        let mut i: size_t = 0 as size_t;
        while i < (*opts).wins.size {
            let mut win: Integer = (*(*opts).wins.items.offset(i as isize)).data.integer;
            let mut wp: *mut win_T = find_window_by_handle(win as Window, err);
            if wp.is_null() {
                return;
            }
            set_put_ptr_t(
                &raw mut windows,
                wp as ptr_t,
                ::core::ptr::null_mut::<*mut ptr_t>(),
            );
            i = i.wrapping_add(1);
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp_0: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp_0.is_null() {
                if set_has_ptr_t(&raw mut windows, wp_0 as ptr_t) as ::core::ffi::c_int != 0
                    && !set_has_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t)
                {
                    set_put_uint32_t(
                        &raw mut (*wp_0).w_ns_set,
                        ns_id as uint32_t,
                        ::core::ptr::null_mut::<*mut uint32_t>(),
                    );
                    if set_has_uint32_t(
                        &raw mut (*(&raw mut (*(*wp_0).w_buffer).b_extmark_ns
                            as *mut Map_uint32_t_uint32_t))
                            .set,
                        ns_id as uint32_t,
                    ) {
                        changed_window_setting(wp_0);
                    }
                }
                if set_has_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t)
                    as ::core::ffi::c_int
                    != 0
                    && !set_has_ptr_t(&raw mut windows, wp_0 as ptr_t)
                {
                    set_del_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t);
                    if set_has_uint32_t(
                        &raw mut (*(&raw mut (*(*wp_0).w_buffer).b_extmark_ns
                            as *mut Map_uint32_t_uint32_t))
                            .set,
                        ns_id as uint32_t,
                    ) {
                        changed_window_setting(wp_0);
                    }
                }
                wp_0 = (*wp_0).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        xfree(windows.keys as *mut ::core::ffi::c_void);
        xfree(windows.h.hash as *mut ::core::ffi::c_void);
        windows = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        };
    }
    if set_scoped as ::core::ffi::c_int != 0
        && !set_has_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t)
    {
        set_put_uint32_t(
            namespace_localscope.ptr(),
            ns_id as uint32_t,
            ::core::ptr::null_mut::<*mut uint32_t>(),
        );
        let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp_0.is_null() {
            let mut wp_1: *mut win_T = if tp_0 == curtab.get() {
                firstwin.get()
            } else {
                (*tp_0).tp_firstwin
            };
            while !wp_1.is_null() {
                if set_has_uint32_t(
                    &raw mut (*(&raw mut (*(*wp_1).w_buffer).b_extmark_ns
                        as *mut Map_uint32_t_uint32_t))
                        .set,
                    ns_id as uint32_t,
                ) {
                    changed_window_setting(wp_1);
                }
                wp_1 = (*wp_1).w_next;
            }
            tp_0 = (*tp_0).tp_next as *mut tabpage_T;
        }
    } else if !set_scoped
        && set_has_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t) as ::core::ffi::c_int
            != 0
    {
        set_del_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t);
        let mut tp_1: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp_1.is_null() {
            let mut wp_2: *mut win_T = if tp_1 == curtab.get() {
                firstwin.get()
            } else {
                (*tp_1).tp_firstwin
            };
            while !wp_2.is_null() {
                if set_has_uint32_t(
                    &raw mut (*(&raw mut (*(*wp_2).w_buffer).b_extmark_ns
                        as *mut Map_uint32_t_uint32_t))
                        .set,
                    ns_id as uint32_t,
                ) {
                    changed_window_setting(wp_2);
                }
                wp_2 = (*wp_2).w_next;
            }
            tp_1 = (*tp_1).tp_next as *mut tabpage_T;
        }
    }
}
pub unsafe extern "C" fn nvim__ns_get(
    mut ns_id: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> KeyDict_ns_opts {
    let mut opts: KeyDict_ns_opts = KEYDICT_INIT;
    let mut windows: Array = ARRAY_DICT_INIT;
    opts.is_set__ns_opts_ = (opts.is_set__ns_opts_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_ns_opts__wins)
        as OptionalKeys;
    opts.wins = windows;
    if !ns_initialized(ns_id as uint32_t) {
        api_err_invalid(
            err,
            b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return opts;
    }
    if !set_has_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t) {
        return opts;
    }
    let mut count: size_t = 0 as size_t;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if set_has_uint32_t(&raw mut (*wp).w_ns_set, ns_id as uint32_t) {
                count = count.wrapping_add(1);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    windows = arena_array(arena, count);
    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp_0.is_null() {
        let mut wp_0: *mut win_T = if tp_0 == curtab.get() {
            firstwin.get()
        } else {
            (*tp_0).tp_firstwin
        };
        while !wp_0.is_null() {
            if set_has_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t) {
                if windows.size == windows.capacity {
                    windows.capacity = if windows.capacity != 0 {
                        windows.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    windows.items = xrealloc(
                        windows.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(windows.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh25 = windows.size;
                windows.size = windows.size.wrapping_add(1);
                *windows.items.offset(c2rust_fresh25 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*wp_0).handle as Integer,
                    },
                };
            }
            wp_0 = (*wp_0).w_next;
        }
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    opts.is_set__ns_opts_ = (opts.is_set__ns_opts_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_ns_opts__wins)
        as OptionalKeys;
    opts.wins = windows;
    return opts;
}
pub const KEYSET_OPTIDX_set_extmark__id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__url: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__spell: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__strict: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__end_col: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__conceal: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__hl_mode: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__end_row: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__end_line: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__hl_group: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__priority: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__sign_text: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_text: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_lines: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark___subpriority: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__undo_restore: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__conceal_lines: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__right_gravity: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_text_pos: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_text_win_col: ::core::ffi::c_int =
    31 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_lines_overflow: ::core::ffi::c_int =
    34 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmark__hl_name: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmarks__type: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmarks__limit: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmarks__hl_name: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_ns_opts__wins: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NULL_STRING: String_0 = STRING_INIT;
pub const KEYDICT_INIT: KeyDict_ns_opts = KeyDict_ns_opts {
    is_set__ns_opts_: 0 as OptionalKeys,
    wins: Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    },
};
pub const MT_FLAG_PAIRED: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
pub const MT_FLAG_NO_UNDO: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
pub const MT_FLAG_INVALIDATE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 5 as ::core::ffi::c_int;
pub const MT_FLAG_INVALID: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 6 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_EXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_HL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 8 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNTEXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 9 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNHL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 10 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_VIRT_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 11 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_VIRT_TEXT_INLINE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 12 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_CONCEAL_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 13 as ::core::ffi::c_int;
pub const MT_FLAG_RIGHT_GRAVITY: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mt_paired(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_PAIRED != 0;
}
#[inline]
unsafe extern "C" fn mt_right(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_RIGHT_GRAVITY != 0;
}
#[inline]
unsafe extern "C" fn mt_no_undo(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_NO_UNDO != 0;
}
#[inline]
unsafe extern "C" fn mt_invalidate(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_INVALIDATE != 0;
}
#[inline]
unsafe extern "C" fn mt_invalid(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_INVALID != 0;
}
#[inline]
unsafe extern "C" fn mt_decor(mut key: MTKey) -> DecorInline {
    return DecorInline {
        ext: key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0,
        data: key.decor_data,
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
