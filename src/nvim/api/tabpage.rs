use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, Array, BoolVarValue, Boolean, BufUpdateCallbacks, Buffer,
    Callback, CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, ExtmarkUndoObject, FileID,
    Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection, KeyDict_tabpage_config,
    KeyValuePair, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, Object, ObjectType, OptInt, OptionalKeys,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0,
    Tabpage, Terminal, Timestamp, TryState, VarLockStatus, VarType, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T,
    bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, colnr_T, dict_T,
    dictvar_S, diff_T, diffblock_S, disptick_T, except_T, except_type_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    msglist, msglist_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, vim_exception, virt_line, visualinfo_T, win_T, window_S, wininfo_S,
    winopt_T, wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn abort() -> !;
    fn try_enter(tstate: *mut TryState);
    fn try_leave(tstate: *const TryState, err: *mut Error);
    fn dict_get_value(
        dict: *mut dict_T,
        key: String_0,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn dict_set_var(
        dict: *mut dict_T,
        key: String_0,
        value: Object,
        del: bool,
        retval: bool,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn find_buffer_by_handle(buffer: Buffer, err: *mut Error) -> *mut buf_T;
    fn find_window_by_handle(window: Window, err: *mut Error) -> *mut win_T;
    fn find_tab_by_handle(tabpage: Tabpage, err: *mut Error) -> *mut tabpage_T;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn api_clear_error(value: *mut Error);
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn nvim_get_current_win() -> Window;
    static autocmd_no_enter: GlobalCell<::core::ffi::c_int>;
    static autocmd_no_leave: GlobalCell<::core::ffi::c_int>;
    static e_cmdwin: [::core::ffi::c_char; 0];
    static firstwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static cmdwin_type: GlobalCell<::core::ffi::c_int>;
    static cmdwin_buf: GlobalCell<*mut buf_T>;
    fn win_set_buf(win: *mut win_T, buf: *mut buf_T, err: *mut Error);
    fn tabpage_win_valid(tp: *const tabpage_T, win: *const win_T) -> bool;
    fn win_new_tabpage(
        after: ::core::ffi::c_int,
        filename: *mut ::core::ffi::c_char,
        enter: bool,
        first: *mut *mut win_T,
    ) -> *mut tabpage_T;
    fn valid_tabpage(tpc: *mut tabpage_T) -> bool;
    fn tabpage_index(ftp: *mut tabpage_T) -> ::core::ffi::c_int;
    fn win_goto(wp: *mut win_T);
}
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
pub const kStlClickFuncRun: C2Rust_Unnamed_12 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_12 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_12 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_12 = 0;
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
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const KEYSET_OPTIDX_tabpage_config__after: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
#[no_mangle]
pub unsafe extern "C" fn nvim_tabpage_list_wins(
    mut tabpage: Tabpage,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() || !valid_tabpage(tab) {
        return rv;
    }
    let mut n: size_t = 0 as size_t;
    let mut wp: *mut win_T = if tab == curtab.get() {
        firstwin.get()
    } else {
        (*tab).tp_firstwin
    };
    while !wp.is_null() {
        n = n.wrapping_add(1);
        wp = (*wp).w_next;
    }
    rv = arena_array(arena, n);
    let mut wp_0: *mut win_T = if tab == curtab.get() {
        firstwin.get()
    } else {
        (*tab).tp_firstwin
    };
    while !wp_0.is_null() {
        let c2rust_fresh0 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeWindow,
            data: C2Rust_Unnamed {
                integer: (*wp_0).handle as Integer,
            },
        };
        wp_0 = (*wp_0).w_next;
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_tabpage_get_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_get_value((*tab).tp_vars, name, arena, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_tabpage_set_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return;
    }
    dict_set_var(
        (*tab).tp_vars,
        name,
        value,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_tabpage_del_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut err: *mut Error,
) {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return;
    }
    dict_set_var(
        (*tab).tp_vars,
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_tabpage_get_win(mut tabpage: Tabpage, mut err: *mut Error) -> Window {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() || !valid_tabpage(tab) {
        return 0 as Window;
    }
    if tab == curtab.get() {
        return nvim_get_current_win();
    }
    let mut wp: *mut win_T = if tab == curtab.get() {
        firstwin.get()
    } else {
        (*tab).tp_firstwin
    };
    while !wp.is_null() {
        if wp == (*tab).tp_curwin {
            return (*wp).handle as Window;
        }
        wp = (*wp).w_next;
    }
    abort();
}
#[no_mangle]
pub unsafe extern "C" fn nvim_tabpage_set_win(
    mut tabpage: Tabpage,
    mut win: Window,
    mut err: *mut Error,
) {
    let mut tp: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tp.is_null() {
        return;
    }
    let mut wp: *mut win_T = find_window_by_handle(win, err);
    if wp.is_null() {
        return;
    }
    if !tabpage_win_valid(tp, wp) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Window does not belong to tabpage %d\0".as_ptr() as *const ::core::ffi::c_char,
            (*tp).handle,
        );
        return;
    }
    if tp == curtab.get() {
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        win_goto(wp);
        try_leave(&raw mut tstate, err);
    } else if (*tp).tp_curwin != wp {
        (*tp).tp_prevwin = (*tp).tp_curwin;
        (*tp).tp_curwin = wp;
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim_tabpage_get_number(
    mut tabpage: Tabpage,
    mut err: *mut Error,
) -> Integer {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return 0 as Integer;
    }
    return tabpage_index(tab) as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_tabpage_is_valid(mut tabpage: Tabpage) -> Boolean {
    let mut stub: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut ret: Boolean = !find_tab_by_handle(tabpage, &raw mut stub).is_null();
    api_clear_error(&raw mut stub);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_open_tabpage(
    mut buf: Buffer,
    mut enter: Boolean,
    mut config: *mut KeyDict_tabpage_config,
    mut err: *mut Error,
) -> Tabpage {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return 0 as Tabpage;
    }
    if cmdwin_type.get() != 0 as ::core::ffi::c_int && enter as ::core::ffi::c_int != 0
        || b == cmdwin_buf.get()
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return 0 as Tabpage;
    }
    let mut after: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if (*config).is_set__tabpage_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_tabpage_config__after
        != 0 as ::core::ffi::c_ulonglong
    {
        after = (*config).after as ::core::ffi::c_int;
    }
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    tp = win_new_tabpage(
        after + 1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        enter as bool,
        &raw mut wp,
    );
    try_leave(&raw mut tstate, err);
    if tp.is_null() {
        if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            api_set_error(
                err,
                kErrorTypeException,
                b"Failed to create new tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return 0 as Tabpage;
    }
    if !valid_tabpage(tp) {
        api_clear_error(err);
        api_set_error(
            err,
            kErrorTypeException,
            b"Tabpage was closed immediately\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 0 as Tabpage;
    }
    if tabpage_win_valid(tp, wp) as ::core::ffi::c_int != 0 && (*wp).w_buffer != b {
        let au_no_enter_leave: bool = curwin.get() != wp;
        if au_no_enter_leave {
            (*autocmd_no_enter.ptr()) += 1;
            (*autocmd_no_leave.ptr()) += 1;
        }
        win_set_buf(wp, b, err);
        if au_no_enter_leave {
            (*autocmd_no_enter.ptr()) -= 1;
            (*autocmd_no_leave.ptr()) -= 1;
        }
        if !valid_tabpage(tp) {
            api_clear_error(err);
            api_set_error(
                err,
                kErrorTypeException,
                b"Tabpage was closed immediately\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return 0 as Tabpage;
        }
    }
    return (*tp).handle as Tabpage;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
