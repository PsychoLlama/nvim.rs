use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_set_error, arena_array, arena_dict, cstr_as_string, dict_get_value,
    dict_set_var, find_buffer_by_handle, find_window_by_handle, normalize_index, try_enter,
    try_leave,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid};
use crate::src::nvim::autocmd::is_aucmd_win;
use crate::src::nvim::cursor::check_cursor_col;
use crate::src::nvim::drawscreen::redraw_later;
use crate::src::nvim::eval::window::{
    restore_win, switch_win, win_execute_after, win_execute_before,
};
use crate::src::nvim::ex_docmd::ex_win_close;

use crate::src::nvim::lua::executor::nlua_call_ref;
use crate::src::nvim::main::{
    cmdwin_buf, cmdwin_old_curwin, cmdwin_win, curtab, curwin, e_autocmd_close, e_cmdwin,
};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::plines::{win_get_fill, win_text_height};
use crate::src::nvim::r#move::{update_topline, validate_cursor};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, Array, BoolVarValue, Boolean, BufUpdateCallbacks, Buffer,
    Callback, CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, ExtmarkUndoObject, FileID,
    Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection, KeyDict_win_text_height,
    KeyValuePair, LuaRef, LuaRetMode, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, Object, ObjectType,
    OptInt, OptionalKeys, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t,
    Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0, Tabpage, Terminal, Timestamp,
    TryState, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig,
    WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T,
    buf_T, bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    except_T, except_type_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    msglist, msglist_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, switchwin_T,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T,
    tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, vim_exception, virt_line, visualinfo_T, win_T, win_execute_T,
    window_S, wininfo_S, winopt_T, wline_T, xfmark_T, NS, QUEUE,
};
use crate::src::nvim::window::{
    can_close_in_cmdwin, win_close, win_close_othertab, win_find_tabpage, win_get_tabwin,
    win_set_buf, win_setheight_win, win_setwidth_win,
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_13 = 2147483647;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const UPD_VALID: C2Rust_Unnamed_14 = 10;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const UPD_NOT_VALID: C2Rust_Unnamed_14 = 40;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_14 = 50;
pub const UPD_SOME_VALID: C2Rust_Unnamed_14 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_14 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_14 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_14 = 20;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const KEYSET_OPTIDX_win_text_height__end_row: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_text_height__end_vcol: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_text_height__start_row: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_text_height__max_height: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_text_height__start_vcol: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
#[no_mangle]
pub unsafe extern "C" fn nvim_win_get_buf(mut win: Window, mut err: *mut Error) -> Buffer {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return 0 as Buffer;
    }
    return (*(*w).w_buffer).handle as Buffer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_set_buf(mut win: Window, mut buf: Buffer, mut err: *mut Error) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if w.is_null() || b.is_null() {
        return;
    }
    if w == cmdwin_win.get() || w == cmdwin_old_curwin.get() || b == cmdwin_buf.get() {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return;
    }
    win_set_buf(w, b, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_get_cursor(
    mut win: Window,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if !w.is_null() {
        rv = arena_array(arena, 2 as size_t);
        let c2rust_fresh0 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*w).w_cursor.lnum as Integer,
            },
        };
        let c2rust_fresh1 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*w).w_cursor.col as Integer,
            },
        };
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_set_cursor(mut win: Window, mut pos: Array, mut err: *mut Error) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    if pos.size != 2 as size_t
        || (*pos.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*pos.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        api_err_exp(
            err,
            b"pos\0".as_ptr() as *const ::core::ffi::c_char,
            b"[row, col] array\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        return;
    }
    let mut row: int64_t = (*pos.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer as int64_t;
    let mut col: int64_t = (*pos.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer as int64_t;
    if row <= 0 as int64_t || row > (*(*w).w_buffer).b_ml.ml_line_count as int64_t {
        api_err_invalid(
            err,
            b"cursor line\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return;
    }
    if col > MAXCOL as ::core::ffi::c_int as int64_t || col < 0 as int64_t {
        api_err_invalid(
            err,
            b"cursor column\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return;
    }
    (*w).w_cursor.lnum = row as linenr_T;
    (*w).w_cursor.col = col as colnr_T;
    (*w).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    check_cursor_col(w);
    (*w).w_set_curswant = true_0;
    let mut switchwin: switchwin_T = switchwin_T {
        sw_curwin: ::core::ptr::null_mut::<win_T>(),
        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
        sw_same_win: false,
        sw_visual_active: false,
    };
    switch_win(
        &raw mut switchwin,
        w,
        ::core::ptr::null_mut::<tabpage_T>(),
        true_0 != 0,
    );
    update_topline(curwin.get());
    validate_cursor(curwin.get());
    restore_win(&raw mut switchwin, true_0 != 0);
    redraw_later(w, UPD_VALID as ::core::ffi::c_int);
    (*w).w_redr_status = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_get_height(mut win: Window, mut err: *mut Error) -> Integer {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return 0 as Integer;
    }
    return (*w).w_height as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_set_height(
    mut win: Window,
    mut height: Integer,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
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
    win_setheight_win(height as ::core::ffi::c_int, w);
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_get_width(mut win: Window, mut err: *mut Error) -> Integer {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return 0 as Integer;
    }
    return (*w).w_width as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_set_width(
    mut win: Window,
    mut width: Integer,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
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
    win_setwidth_win(width as ::core::ffi::c_int, w);
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_get_var(
    mut win: Window,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_get_value((*w).w_vars, name, arena, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_set_var(
    mut win: Window,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    dict_set_var(
        (*w).w_vars,
        name,
        value,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_del_var(
    mut win: Window,
    mut name: String_0,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    dict_set_var(
        (*w).w_vars,
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
pub unsafe extern "C" fn nvim_win_get_position(
    mut win: Window,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if !w.is_null() {
        rv = arena_array(arena, 2 as size_t);
        let c2rust_fresh2 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh2 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*w).w_winrow as Integer,
            },
        };
        let c2rust_fresh3 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh3 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*w).w_wincol as Integer,
            },
        };
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_get_tabpage(mut win: Window, mut err: *mut Error) -> Tabpage {
    let mut rv: Tabpage = 0 as Tabpage;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if !w.is_null() {
        rv = (*win_find_tabpage(w)).handle as Tabpage;
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_get_number(mut win: Window, mut err: *mut Error) -> Integer {
    let mut rv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return rv as Integer;
    }
    let mut tabnr: ::core::ffi::c_int = 0;
    win_get_tabwin((*w).handle, &raw mut tabnr, &raw mut rv);
    return rv as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_is_valid(mut win: Window) -> Boolean {
    let mut stub: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut ret: Boolean = !find_window_by_handle(win, &raw mut stub).is_null();
    api_clear_error(&raw mut stub);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_hide(mut win: Window, mut err: *mut Error) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() || !can_close_in_cmdwin(w, err) {
        return;
    }
    let mut tabpage: *mut tabpage_T = win_find_tabpage(w);
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
    if is_aucmd_win(w) {
        emsg(gettext(
            &raw const e_autocmd_close as *const ::core::ffi::c_char,
        ));
    } else if tabpage == curtab.get() {
        win_close(w, false, false);
    } else {
        win_close_othertab(w, 0 as ::core::ffi::c_int, tabpage, false);
    }
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_close(mut win: Window, mut force: Boolean, mut err: *mut Error) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() || !can_close_in_cmdwin(w, err) {
        return;
    }
    let mut tabpage: *mut tabpage_T = win_find_tabpage(w);
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
    ex_win_close(
        force as ::core::ffi::c_int,
        w,
        if tabpage == curtab.get() {
            ::core::ptr::null_mut::<tabpage_T>()
        } else {
            tabpage
        },
    );
    try_leave(&raw mut tstate, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_call(
    mut win: Window,
    mut fun: LuaRef,
    mut err: *mut Error,
) -> Object {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    let mut tabpage: *mut tabpage_T = win_find_tabpage(w);
    let mut res: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
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
    let mut win_execute_args: win_execute_T = win_execute_T {
        wp: ::core::ptr::null_mut::<win_T>(),
        curpos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cwd: [0; 4096],
        cwd_status: 0,
        apply_acd: false,
        save_sfname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        switchwin: switchwin_T {
            sw_curwin: ::core::ptr::null_mut::<win_T>(),
            sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
            sw_same_win: false,
            sw_visual_active: false,
        },
    };
    if win_execute_before(&raw mut win_execute_args, w, tabpage) {
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        res = nlua_call_ref(
            fun,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetLuaref,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
    }
    win_execute_after(&raw mut win_execute_args);
    try_leave(&raw mut tstate, err);
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_set_hl_ns(
    mut win: Window,
    mut ns_id: Integer,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    if !(ns_id >= -1 as Integer) {
        api_err_invalid(
            err,
            b"namespace\0".as_ptr() as *const ::core::ffi::c_char,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    (*w).w_ns_hl = ns_id as NS as ::core::ffi::c_int;
    (*w).w_hl_needs_update = true_0;
    redraw_later(w, UPD_NOT_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_text_height(
    mut win: Window,
    mut opts: *mut KeyDict_win_text_height,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut rv: Dict = arena_dict(arena, 2 as size_t);
    let w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return rv;
    }
    let buf: *mut buf_T = (*w).w_buffer;
    let line_count: linenr_T = (*buf).b_ml.ml_line_count;
    let mut start_lnum: linenr_T = 1 as linenr_T;
    let mut end_lnum: linenr_T = line_count;
    let mut start_vcol: int64_t = -1 as int64_t;
    let mut end_vcol: int64_t = -1 as int64_t;
    let mut oob: bool = false_0 != 0;
    if (*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__start_row
        != 0 as ::core::ffi::c_ulonglong
    {
        start_lnum = normalize_index(
            buf,
            (*opts).start_row as int64_t,
            false_0 != 0,
            &raw mut oob,
        ) as linenr_T;
    }
    if (*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__end_row
        != 0 as ::core::ffi::c_ulonglong
    {
        end_lnum = normalize_index(buf, (*opts).end_row as int64_t, false_0 != 0, &raw mut oob)
            as linenr_T;
    }
    if oob {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"Line index out of bounds\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    if !(start_lnum <= end_lnum) {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"'start_row' is higher than 'end_row'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return rv;
    }
    if (*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__start_vcol
        != 0 as ::core::ffi::c_ulonglong
    {
        if !((*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"'start_vcol' specified without 'start_row'\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            return rv;
        }
        start_vcol = (*opts).start_vcol as int64_t;
        if !(start_vcol >= 0 as int64_t && start_vcol <= MAXCOL as ::core::ffi::c_int as int64_t) {
            api_err_invalid(
                err,
                b"start_vcol\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
            return rv;
        }
    }
    if (*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__end_vcol
        != 0 as ::core::ffi::c_ulonglong
    {
        if !((*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"'end_vcol' specified without 'end_row'\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return rv;
        }
        end_vcol = (*opts).end_vcol as int64_t;
        if !(end_vcol >= 0 as int64_t && end_vcol <= MAXCOL as ::core::ffi::c_int as int64_t) {
            api_err_invalid(
                err,
                b"end_vcol\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
            return rv;
        }
    }
    let mut max: int64_t = INT64_MAX as int64_t;
    if (*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__max_height
        != 0 as ::core::ffi::c_ulonglong
    {
        if !((*opts).max_height > 0 as Integer) {
            api_err_invalid(
                err,
                b"max_height\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
            return rv;
        }
        max = (*opts).max_height as int64_t;
    }
    if start_lnum == end_lnum && start_vcol >= 0 as int64_t && end_vcol >= 0 as int64_t {
        if !(start_vcol <= end_vcol) {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"'start_vcol' is higher than 'end_vcol'\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return rv;
        }
    }
    let mut fill: int64_t = 0 as int64_t;
    let mut all: int64_t = win_text_height(
        w,
        start_lnum,
        start_vcol,
        &raw mut end_lnum,
        &raw mut end_vcol,
        &raw mut fill,
        max,
    );
    if !((*opts).is_set__win_text_height_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_text_height__end_row
        != 0 as ::core::ffi::c_ulonglong)
    {
        let end_fill: int64_t = win_get_fill(w, line_count + 1 as linenr_T) as int64_t;
        fill += end_fill;
        all += end_fill;
    }
    let c2rust_fresh4 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh4 as isize) = key_value_pair {
        key: cstr_as_string(b"all\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: all },
        },
    };
    let c2rust_fresh5 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh5 as isize) = key_value_pair {
        key: cstr_as_string(b"fill\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: fill },
        },
    };
    let c2rust_fresh6 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh6 as isize) = key_value_pair {
        key: cstr_as_string(b"end_row\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (end_lnum - 1 as linenr_T) as Integer,
            },
        },
    };
    let c2rust_fresh7 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh7 as isize) = key_value_pair {
        key: cstr_as_string(b"end_vcol\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: end_vcol },
        },
    };
    return rv;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
