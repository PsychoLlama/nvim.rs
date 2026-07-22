use crate::src::nvim::autocmd::{aucmd_prepbuf, aucmd_restbuf, block_autocmds, unblock_autocmds};
use crate::src::nvim::buffer::{
    bt_nofilename, bt_prompt, buf_ensure_loaded, buflist_add, buflist_findlnum,
    buflist_findname_exp, buflist_findnr, buflist_new, bufref_valid, set_bufref,
};
use crate::src::nvim::change::{
    appended_lines_mark, changed_lines, deleted_lines_mark, inserted_bytes,
};
use crate::src::nvim::cursor::check_cursor_col;
use crate::src::nvim::edit::buf_prompt_text;
use crate::src::nvim::eval::funcs::{get_buf_arg, tv_get_buf, tv_get_buf_from_arg};
use crate::src::nvim::eval::typval::{
    callback_free, tv_check_str_or_nr, tv_clear, tv_dict_add_dict, tv_dict_add_list,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find, tv_get_lnum, tv_get_lnum_buf,
    tv_get_number, tv_get_number_chk, tv_get_string, tv_get_string_chk, tv_list_alloc,
    tv_list_alloc_ret, tv_list_append_dict, tv_list_append_number, tv_list_append_string,
    tv_list_item_remove,
};
use crate::src::nvim::eval::window::win_has_winnr;
use crate::src::nvim::eval_1::{callback_from_typval, typval_tostring};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::extmark::extmark_splice_cols;

use crate::src::nvim::main::{
    cmdwin_buf, curbuf, curtab, curwin, did_emsg, emsg_off, first_tabpage, firstbuf, firstwin,
    swap_exists_action, u_sync_once, VIsual_active,
};
use crate::src::nvim::memline::{
    ml_append, ml_delete_flags, ml_get, ml_get_buf, ml_get_buf_len, ml_replace, ml_replace_buf,
};
use crate::src::nvim::memory::{strnequal, xfree, xstrdup};
use crate::src::nvim::os::libc::{memset, strcmp, strlen};
use crate::src::nvim::path::path_with_url;
use crate::src::nvim::r#move::update_topline;
use crate::src::nvim::sign::{buf_has_signs, get_buffer_signs};
use crate::src::nvim::strings::{concat_str, xstrnsave};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, Array, BoolVarValue, Boolean,
    BufUpdateCallbacks, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, EvalFuncData,
    ExtmarkMove, ExtmarkOp, ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject, FileID, Float,
    FloatAnchor, FloatRelative, GridView, Integer, Intersection, KeyValuePair, ListLenSpecials,
    LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MsgpackRpcRequestHandler, Object,
    ObjectType, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t,
    Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_13, String_0, Terminal, Timestamp, UndoObjectType,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, __time_t, aco_save_T, alist_T, bcount_t, bhdr_T, blob_T, blobvar_S,
    blocknr_T, buf_T, bufref_T, bufstate_T, chunksize_T, colnr_T, dict_T, dictitem_T, dictvar_S,
    diff_T, diffblock_S, disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_12,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S, partial_T, pos_T,
    pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, ssize_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_7, varnumber_T, virt_line, visualinfo_T, win_T,
    window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
use crate::src::nvim::undo::{bufIsChanged, u_clearallandblockfree, u_save, u_savesub, u_sync};
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
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cob_T {
    pub cob_curwin_save: *mut win_T,
    pub cob_aco: aco_save_T,
    pub cob_using_aco: ::core::ffi::c_int,
    pub cob_save_VIsual_active: ::core::ffi::c_int,
}
pub const ML_DEL_MESSAGE: C2Rust_Unnamed_14 = 1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
#[no_mangle]
pub unsafe extern "C" fn find_buffer(mut avar: *mut typval_T) -> *mut buf_T {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if (*avar).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = buflist_findnr((*avar).vval.v_number as ::core::ffi::c_int);
    } else if (*avar).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*avar).vval.v_string.is_null()
    {
        buf = buflist_findname_exp((*avar).vval.v_string);
        if buf.is_null() {
            let mut bp: *mut buf_T = firstbuf.get();
            while !bp.is_null() {
                if !(*bp).b_fname.is_null()
                    && (path_with_url((*bp).b_fname) != 0
                        || bt_nofilename(bp) as ::core::ffi::c_int != 0)
                    && strcmp((*bp).b_fname, (*avar).vval.v_string) == 0 as ::core::ffi::c_int
                {
                    buf = bp;
                    break;
                } else {
                    bp = (*bp).b_next;
                }
            }
        }
    }
    return buf;
}
unsafe extern "C" fn find_win_for_curbuf() {
    let mut i: size_t = 0 as size_t;
    while i < (*curbuf.get()).b_wininfo.size {
        let mut wip: *mut WinInfo = *(*curbuf.get()).b_wininfo.items.offset(i as isize);
        if !(*wip).wi_win.is_null() && (*(*wip).wi_win).w_buffer == curbuf.get() {
            curwin.set((*wip).wi_win);
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
}
unsafe extern "C" fn change_other_buffer_prepare(mut cob: *mut cob_T, mut buf: *mut buf_T) {
    memset(
        cob as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cob_T>(),
    );
    (*cob).cob_save_VIsual_active = VIsual_active.get() as ::core::ffi::c_int;
    VIsual_active.set(false_0 != 0);
    (*cob).cob_curwin_save = curwin.get();
    curbuf.set(buf);
    find_win_for_curbuf();
    if (*curwin.get()).w_buffer != buf {
        curbuf.set((*curwin.get()).w_buffer);
        aucmd_prepbuf(&raw mut (*cob).cob_aco, buf);
        (*cob).cob_using_aco = true_0;
    }
}
unsafe extern "C" fn change_other_buffer_restore(mut cob: *mut cob_T) {
    if (*cob).cob_using_aco != 0 {
        aucmd_restbuf(&raw mut (*cob).cob_aco);
    } else {
        curwin.set((*cob).cob_curwin_save);
        curbuf.set((*curwin.get()).w_buffer);
    }
    VIsual_active.set((*cob).cob_save_VIsual_active != 0);
}
unsafe extern "C" fn set_buffer_lines(
    mut buf: *mut buf_T,
    mut lnum_arg: linenr_T,
    mut append: bool,
    mut lines: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    let mut lnum: linenr_T = lnum_arg
        + (if append as ::core::ffi::c_int != 0 {
            1 as linenr_T
        } else {
            0 as linenr_T
        });
    let mut added: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let is_curbuf: bool = buf == curbuf.get();
    if buf.is_null() || !is_curbuf && (*buf).b_ml.ml_mfp.is_null() || lnum < 1 as linenr_T {
        (*rettv).vval.v_number = 1 as varnumber_T;
        return;
    }
    let mut cob: cob_T = cob_T {
        cob_curwin_save: ::core::ptr::null_mut::<win_T>(),
        cob_aco: aco_save_T {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            },
            tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            save_VIsual_active: false,
            save_prompt_insert: 0,
        },
        cob_using_aco: 0,
        cob_save_VIsual_active: 0,
    };
    if !is_curbuf {
        change_other_buffer_prepare(&raw mut cob, buf);
    }
    let mut append_lnum: linenr_T = 0;
    if append {
        append_lnum = lnum - 1 as linenr_T;
    } else {
        append_lnum = (*curbuf.get()).b_ml.ml_line_count;
    }
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut li: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_cleanup: {
        if (*lines).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            l = (*lines).vval.v_list;
            if l.is_null() || tv_list_len(l) == 0 as ::core::ffi::c_int {
                break '_cleanup;
            } else {
                li = tv_list_first(l);
            }
        } else {
            line = typval_tostring(lines, false_0 != 0);
        }
        loop {
            if (*lines).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if li.is_null() {
                    break;
                }
                xfree(line as *mut ::core::ffi::c_void);
                line = typval_tostring(&raw mut (*li).li_tv, false_0 != 0);
                li = (*li).li_next;
            }
            (*rettv).vval.v_number = 1 as varnumber_T;
            if line.is_null() || lnum > (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T {
                break;
            }
            if u_sync_once.get() == 2 as ::core::ffi::c_int {
                u_sync_once.set(1 as ::core::ffi::c_int);
                u_sync(true_0 != 0);
            }
            if !append && lnum <= (*curbuf.get()).b_ml.ml_line_count {
                let mut old_len: ::core::ffi::c_int = strlen(ml_get(lnum)) as ::core::ffi::c_int;
                if u_savesub(lnum) == OK && ml_replace(lnum, line, true_0 != 0) == OK {
                    inserted_bytes(
                        lnum,
                        0 as colnr_T,
                        old_len,
                        strlen(line) as ::core::ffi::c_int,
                    );
                    if is_curbuf as ::core::ffi::c_int != 0 && lnum == (*curwin.get()).w_cursor.lnum
                    {
                        check_cursor_col(curwin.get());
                    }
                    (*rettv).vval.v_number = 0 as varnumber_T;
                }
            } else if added > 0 as ::core::ffi::c_int || u_save(lnum - 1 as linenr_T, lnum) == OK {
                added += 1;
                if ml_append(lnum - 1 as linenr_T, line, 0 as colnr_T, false_0 != 0) == OK {
                    (*rettv).vval.v_number = 0 as varnumber_T;
                }
            }
            if l.is_null() {
                break;
            }
            lnum += 1;
        }
        xfree(line as *mut ::core::ffi::c_void);
        if added > 0 as ::core::ffi::c_int {
            appended_lines_mark(append_lnum, added);
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                let mut wp: *mut win_T = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                while !wp.is_null() {
                    if (*wp).w_buffer == buf
                        && ((*wp).w_buffer != curbuf.get() || wp == curwin.get())
                        && (*wp).w_cursor.lnum > append_lnum
                    {
                        (*wp).w_cursor.lnum += added as linenr_T;
                    }
                    wp = (*wp).w_next;
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
            check_cursor_col(curwin.get());
            update_topline(curwin.get());
        }
    }
    if !is_curbuf {
        change_other_buffer_restore(&raw mut cob);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_append(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
    let lnum: linenr_T = tv_get_lnum(argvars.offset(0 as ::core::ffi::c_int as isize));
    if did_emsg.get() == did_emsg_before {
        set_buffer_lines(
            curbuf.get(),
            lnum,
            true_0 != 0,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    }
}
unsafe extern "C" fn buf_set_append_line(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut append: bool,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
    let buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        (*rettv).vval.v_number = 1 as varnumber_T;
    } else {
        let lnum: linenr_T = tv_get_lnum_buf(argvars.offset(1 as ::core::ffi::c_int as isize), buf);
        if did_emsg.get() == did_emsg_before {
            set_buffer_lines(
                buf,
                lnum,
                append,
                argvars.offset(2 as ::core::ffi::c_int as isize),
                rettv,
            );
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_appendbufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    buf_set_append_line(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_prompt_appendbuf(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 1 as varnumber_T;
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    if buf.is_null() || !bt_prompt(buf) {
        return;
    }
    let mut lnum: linenr_T = if 0 as linenr_T > (*buf).b_prompt_start.mark.lnum - 1 as linenr_T {
        0 as linenr_T
    } else {
        (*buf).b_prompt_start.mark.lnum - 1 as linenr_T
    };
    let mut lines: *mut typval_T = argvars.offset(1 as ::core::ffi::c_int as isize);
    let mut did_concat: bool = false_0 != 0;
    if !(*buf).b_prompt_append_new_line {
        let mut text: *const ::core::ffi::c_char = if lnum > 0 as linenr_T {
            ml_get_buf(buf, lnum) as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        };
        if (*lines).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut l: *mut list_T = (*lines).vval.v_list;
            if !l.is_null() && tv_list_len(l) > 0 as ::core::ffi::c_int {
                let mut li: *mut listitem_T = tv_list_first(l);
                let mut str: *const ::core::ffi::c_char = tv_get_string(&raw mut (*li).li_tv);
                let mut new_str: *mut ::core::ffi::c_char = concat_str(text, str);
                tv_clear(&raw mut (*li).li_tv);
                (*li).li_tv.v_type = VAR_STRING;
                (*li).li_tv.vval.v_string = new_str;
                did_concat = true_0 != 0;
            }
        } else if (*lines).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut str_0: *const ::core::ffi::c_char = tv_get_string(lines);
            let mut new_str_0: *mut ::core::ffi::c_char = concat_str(text, str_0);
            tv_clear(lines);
            (*lines).v_type = VAR_STRING;
            (*lines).vval.v_string = new_str_0;
        }
    }
    if did_emsg.get() == did_emsg_before {
        if did_concat as ::core::ffi::c_int != 0
            && tv_list_len((*lines).vval.v_list) > 1 as ::core::ffi::c_int
        {
            let mut l_0: *mut list_T = (*lines).vval.v_list;
            let mut li_0: *mut listitem_T = tv_list_first(l_0);
            set_buffer_lines(buf, lnum, false_0 != 0, &raw mut (*li_0).li_tv, rettv);
            if (*rettv).vval.v_number == 0 as varnumber_T {
                tv_list_item_remove(l_0, li_0);
                set_buffer_lines(buf, lnum, true_0 != 0, lines, rettv);
            }
        } else {
            set_buffer_lines(buf, lnum, (*buf).b_prompt_append_new_line, lines, rettv);
        }
    }
    if (*rettv).vval.v_number == 0 as varnumber_T {
        (*buf).b_prompt_append_new_line = false_0 != 0;
        if (*lines).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut l_1: *mut list_T = (*lines).vval.v_list;
            if !l_1.is_null() && tv_list_len(l_1) > 0 as ::core::ffi::c_int {
                let mut li_1: *mut listitem_T = tv_list_last(l_1);
                let mut str_1: *const ::core::ffi::c_char = tv_get_string(&raw mut (*li_1).li_tv);
                let mut len: size_t = strlen(str_1);
                if len > 0 as size_t
                    && *str_1.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int
                {
                    (*buf).b_prompt_append_new_line = true_0 != 0;
                }
            }
        } else if (*lines).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut str_2: *const ::core::ffi::c_char = tv_get_string(lines);
            let mut len_0: size_t = strlen(str_2);
            if len_0 > 0 as size_t
                && *str_2.offset(len_0.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    == '\n' as ::core::ffi::c_int
            {
                (*buf).b_prompt_append_new_line = true_0 != 0;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_bufadd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut name: *mut ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char;
    (*rettv).vval.v_number = buflist_add(
        if *name as ::core::ffi::c_int == NUL {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            name
        },
        0 as ::core::ffi::c_int,
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_bufexists(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = !find_buffer(argvars.offset(0 as ::core::ffi::c_int as isize))
        .is_null() as ::core::ffi::c_int as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_buflisted(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    buf = find_buffer(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number =
        (!buf.is_null() && (*buf).b_p_bl != 0) as ::core::ffi::c_int as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_bufload(
    mut argvars: *mut typval_T,
    mut _unused: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: *mut buf_T = get_buf_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    if !buf.is_null() {
        if swap_exists_action.get() != SEA_READONLY {
            swap_exists_action.set(SEA_NONE);
        }
        buf_ensure_loaded(buf);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_bufloaded(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    buf = find_buffer(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number =
        (!buf.is_null() && !(*buf).b_ml.ml_mfp.is_null()) as ::core::ffi::c_int as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_bufname(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: *const buf_T = ::core::ptr::null::<buf_T>();
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = curbuf.get();
    } else {
        buf = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    }
    if !buf.is_null() && !(*buf).b_fname.is_null() {
        (*rettv).vval.v_string = xstrdup((*buf).b_fname);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_bufnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: *const buf_T = ::core::ptr::null::<buf_T>();
    let mut error: bool = false_0 != 0;
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = curbuf.get();
    } else {
        if !tv_check_str_or_nr(argvars.offset(0 as ::core::ffi::c_int as isize)) {
            return;
        }
        (*emsg_off.ptr()) += 1;
        buf = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
        (*emsg_off.ptr()) -= 1;
    }
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if buf.is_null()
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) != 0 as varnumber_T
        && !error
        && {
            name = tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
            !name.is_null()
        }
    {
        buf = buflist_new(
            name as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            1 as linenr_T,
            0 as ::core::ffi::c_int,
        );
    }
    if !buf.is_null() {
        (*rettv).vval.v_number = (*buf).handle as varnumber_T;
    }
}
unsafe extern "C" fn buf_win_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut get_nr: bool,
) {
    let buf: *const buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    if buf.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
        return;
    }
    let mut winnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut winid: ::core::ffi::c_int = 0;
    let mut found_buf: bool = false_0 != 0;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        winnr += win_has_winnr(wp, curtab.get()) as ::core::ffi::c_int;
        if (*wp).w_buffer == buf as *mut buf_T
            && (!get_nr || win_has_winnr(wp, curtab.get()) as ::core::ffi::c_int != 0)
        {
            found_buf = true_0 != 0;
            winid = (*wp).handle as ::core::ffi::c_int;
            break;
        } else {
            wp = (*wp).w_next;
        }
    }
    (*rettv).vval.v_number = (if found_buf as ::core::ffi::c_int != 0 {
        if get_nr as ::core::ffi::c_int != 0 {
            winnr
        } else {
            winid
        }
    } else {
        -1 as ::core::ffi::c_int
    }) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_bufwinid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    buf_win_common(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_bufwinnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    buf_win_common(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_deletebufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
    (*rettv).vval.v_number = 1 as varnumber_T;
    let buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        return;
    }
    let mut last: linenr_T = 0;
    let first: linenr_T = tv_get_lnum_buf(argvars.offset(1 as ::core::ffi::c_int as isize), buf);
    if did_emsg.get() > did_emsg_before {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        last = tv_get_lnum_buf(argvars.offset(2 as ::core::ffi::c_int as isize), buf);
    } else {
        last = first;
    }
    if (*buf).b_ml.ml_mfp.is_null()
        || first < 1 as linenr_T
        || first > (*buf).b_ml.ml_line_count
        || last < first
    {
        return;
    }
    let is_curbuf: bool = buf == curbuf.get();
    let mut cob: cob_T = cob_T {
        cob_curwin_save: ::core::ptr::null_mut::<win_T>(),
        cob_aco: aco_save_T {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            },
            tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            save_VIsual_active: false,
            save_prompt_insert: 0,
        },
        cob_using_aco: 0,
        cob_save_VIsual_active: 0,
    };
    if !is_curbuf {
        change_other_buffer_prepare(&raw mut cob, buf);
    }
    if last > (*curbuf.get()).b_ml.ml_line_count {
        last = (*curbuf.get()).b_ml.ml_line_count;
    }
    let count: ::core::ffi::c_int =
        last as ::core::ffi::c_int - first as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if u_sync_once.get() == 2 as ::core::ffi::c_int {
        u_sync_once.set(1 as ::core::ffi::c_int);
        u_sync(true_0 != 0);
    }
    if u_save(first - 1 as linenr_T, last + 1 as linenr_T) != FAIL {
        let mut lnum: linenr_T = first;
        while lnum <= last {
            ml_delete_flags(first, ML_DEL_MESSAGE as ::core::ffi::c_int);
            lnum += 1;
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == buf {
                    if (*wp).w_cursor.lnum > last {
                        (*wp).w_cursor.lnum -= count as linenr_T;
                    } else if (*wp).w_cursor.lnum > first {
                        (*wp).w_cursor.lnum = first;
                    }
                    if (*wp).w_cursor.lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
                        (*wp).w_cursor.lnum = (*(*wp).w_buffer).b_ml.ml_line_count;
                    }
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        check_cursor_col(curwin.get());
        deleted_lines_mark(first, count);
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
    if !is_curbuf {
        change_other_buffer_restore(&raw mut cob);
    }
}
unsafe extern "C" fn get_buffer_info(mut buf: *mut buf_T) -> *mut dict_T {
    let dict: *mut dict_T = tv_dict_alloc();
    tv_dict_add_nr(
        dict,
        b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*buf).handle as varnumber_T,
    );
    tv_dict_add_str(
        dict,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        if !(*buf).b_ffname.is_null() {
            (*buf).b_ffname as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    tv_dict_add_nr(
        dict,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (if buf == curbuf.get() {
            (*curwin.get()).w_cursor.lnum
        } else {
            buflist_findlnum(buf)
        }) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"linecount\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (*buf).b_ml.ml_line_count as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        !(*buf).b_ml.ml_mfp.is_null() as ::core::ffi::c_int as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"listed\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*buf).b_p_bl as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"changed\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        bufIsChanged(buf) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        buf_get_changedtick(buf),
    );
    tv_dict_add_nr(
        dict,
        b"hidden\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (!(*buf).b_ml.ml_mfp.is_null() && (*buf).b_nwindows == 0 as ::core::ffi::c_int)
            as ::core::ffi::c_int as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"command\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (buf == cmdwin_buf.get()) as ::core::ffi::c_int as varnumber_T,
    );
    tv_dict_add_dict(
        dict,
        b"variables\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (*buf).b_vars,
    );
    let windows: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                tv_list_append_number(windows, (*wp).handle as varnumber_T);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    tv_dict_add_list(
        dict,
        b"windows\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        windows,
    );
    if buf_has_signs(buf) {
        tv_dict_add_list(
            dict,
            b"signs\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            get_buffer_signs(buf),
        );
    }
    tv_dict_add_nr(
        dict,
        b"lastused\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*buf).b_last_used as varnumber_T,
    );
    return dict;
}
#[no_mangle]
pub unsafe extern "C" fn f_getbufinfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut argbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut filtered: bool = false_0 != 0;
    let mut sel_buflisted: bool = false_0 != 0;
    let mut sel_bufloaded: bool = false_0 != 0;
    let mut sel_bufmodified: bool = false_0 != 0;
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut sel_d: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if !sel_d.is_null() {
            let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
            filtered = true_0 != 0;
            di = tv_dict_find(
                sel_d,
                b"buflisted\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if !di.is_null() && tv_get_number(&raw mut (*di).di_tv) != 0 {
                sel_buflisted = true_0 != 0;
            }
            di = tv_dict_find(
                sel_d,
                b"bufloaded\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if !di.is_null() && tv_get_number(&raw mut (*di).di_tv) != 0 {
                sel_bufloaded = true_0 != 0;
            }
            di = tv_dict_find(
                sel_d,
                b"bufmodified\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if !di.is_null() && tv_get_number(&raw mut (*di).di_tv) != 0 {
                sel_bufmodified = true_0 != 0;
            }
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        argbuf = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
        if argbuf.is_null() {
            return;
        }
    }
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !(!argbuf.is_null() && argbuf != buf) {
            if !(filtered as ::core::ffi::c_int != 0
                && (sel_bufloaded as ::core::ffi::c_int != 0 && (*buf).b_ml.ml_mfp.is_null()
                    || sel_buflisted as ::core::ffi::c_int != 0 && (*buf).b_p_bl == 0
                    || sel_bufmodified as ::core::ffi::c_int != 0 && (*buf).b_changed == 0))
            {
                let d: *mut dict_T = get_buffer_info(buf);
                tv_list_append_dict((*rettv).vval.v_list, d);
                if !argbuf.is_null() {
                    return;
                }
            }
        }
        buf = (*buf).b_next;
    }
}
unsafe extern "C" fn get_buffer_lines(
    mut buf: *mut buf_T,
    mut start: linenr_T,
    mut end: linenr_T,
    mut retlist: bool,
    mut rettv: *mut typval_T,
) {
    (*rettv).v_type = (if retlist as ::core::ffi::c_int != 0 {
        VAR_LIST as ::core::ffi::c_int
    } else {
        VAR_STRING as ::core::ffi::c_int
    }) as VarType;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() || start < 0 as linenr_T || end < start {
        if retlist {
            tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
        }
        return;
    }
    if retlist {
        if start < 1 as linenr_T {
            start = 1 as ::core::ffi::c_int as linenr_T;
        }
        if end > (*buf).b_ml.ml_line_count {
            end = (*buf).b_ml.ml_line_count;
        }
        tv_list_alloc_ret(rettv, (end - start + 1 as linenr_T) as ptrdiff_t);
        while start <= end {
            tv_list_append_string(
                (*rettv).vval.v_list,
                ml_get_buf(buf, start),
                ml_get_buf_len(buf, start) as ssize_t,
            );
            start += 1;
        }
    } else {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = if start >= 1 as linenr_T && start <= (*buf).b_ml.ml_line_count {
            xstrnsave(ml_get_buf(buf, start), ml_get_buf_len(buf, start) as size_t)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
    };
}
unsafe extern "C" fn getbufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut retlist: bool,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    let lnum: linenr_T = tv_get_lnum_buf(argvars.offset(1 as ::core::ffi::c_int as isize), buf);
    if did_emsg.get() > did_emsg_before {
        return;
    }
    let end: linenr_T = if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type
        as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lnum
    } else {
        tv_get_lnum_buf(argvars.offset(2 as ::core::ffi::c_int as isize), buf)
    };
    get_buffer_lines(buf, lnum, end, retlist, rettv);
}
#[no_mangle]
pub unsafe extern "C" fn f_getbufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getbufline(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_getbufoneline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getbufline(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_getline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut end: linenr_T = 0;
    let mut retlist: bool = false;
    let lnum: linenr_T = tv_get_lnum(argvars);
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        end = lnum;
        retlist = false_0 != 0;
    } else {
        end = tv_get_lnum(argvars.offset(1 as ::core::ffi::c_int as isize));
        retlist = true_0 != 0;
    }
    get_buffer_lines(curbuf.get(), lnum, end, retlist, rettv);
}
#[no_mangle]
pub unsafe extern "C" fn f_setbufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    buf_set_append_line(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_setline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
    let mut lnum: linenr_T = tv_get_lnum(argvars.offset(0 as ::core::ffi::c_int as isize));
    if did_emsg.get() == did_emsg_before {
        set_buffer_lines(
            curbuf.get(),
            lnum,
            false_0 != 0,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn switch_buffer(mut save_curbuf: *mut bufref_T, mut buf: *mut buf_T) {
    block_autocmds();
    set_bufref(save_curbuf, curbuf.get());
    (*curbuf.get()).b_nwindows -= 1;
    curbuf.set(buf);
    (*curwin.get()).w_buffer = buf;
    (*curbuf.get()).b_nwindows += 1;
}
#[no_mangle]
pub unsafe extern "C" fn restore_buffer(mut save_curbuf: *mut bufref_T) {
    unblock_autocmds();
    if bufref_valid(save_curbuf) {
        (*curbuf.get()).b_nwindows -= 1;
        (*curwin.get()).w_buffer = (*save_curbuf).br_buf;
        curbuf.set((*save_curbuf).br_buf);
        (*curbuf.get()).b_nwindows += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_prompt_setcallback(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut prompt_callback: Callback = Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if check_secure() {
        return;
    }
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        return;
    }
    if !callback_from_typval(
        &raw mut prompt_callback,
        argvars.offset(1 as ::core::ffi::c_int as isize),
    ) {
        return;
    }
    callback_free(&raw mut (*buf).b_prompt_callback);
    (*buf).b_prompt_callback = prompt_callback;
}
#[no_mangle]
pub unsafe extern "C" fn f_prompt_setinterrupt(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut interrupt_callback: Callback = Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if check_secure() {
        return;
    }
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        return;
    }
    if !callback_from_typval(
        &raw mut interrupt_callback,
        argvars.offset(1 as ::core::ffi::c_int as isize),
    ) {
        return;
    }
    callback_free(&raw mut (*buf).b_prompt_interrupt);
    (*buf).b_prompt_interrupt = interrupt_callback;
}
#[no_mangle]
pub unsafe extern "C" fn f_prompt_setprompt(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        return;
    }
    let mut new_prompt: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut new_prompt_len: ::core::ffi::c_int = strlen(new_prompt) as ::core::ffi::c_int;
    if bt_prompt(buf) as ::core::ffi::c_int != 0 && !(*buf).b_ml.ml_mfp.is_null() {
        if (*buf).b_prompt_start.mark.lnum < 1 as linenr_T
            || (*buf).b_prompt_start.mark.lnum > (*curbuf.get()).b_ml.ml_line_count
        {
            (*buf).b_prompt_start.mark.lnum = if 1 as linenr_T
                > (if (*buf).b_prompt_start.mark.lnum < (*buf).b_ml.ml_line_count {
                    (*buf).b_prompt_start.mark.lnum
                } else {
                    (*buf).b_ml.ml_line_count
                }) {
                1 as linenr_T
            } else if (*buf).b_prompt_start.mark.lnum < (*buf).b_ml.ml_line_count {
                (*buf).b_prompt_start.mark.lnum
            } else {
                (*buf).b_ml.ml_line_count
            };
            (*curbuf.get()).b_prompt_append_new_line = true_0 != 0;
        }
        let mut prompt_lno: linenr_T = (*buf).b_prompt_start.mark.lnum;
        let mut old_prompt: *mut ::core::ffi::c_char = buf_prompt_text(buf);
        let mut old_line: *mut ::core::ffi::c_char = ml_get_buf(buf, prompt_lno);
        let mut old_line_len: colnr_T = ml_get_buf_len(buf, prompt_lno);
        let mut old_prompt_len: ::core::ffi::c_int = strlen(old_prompt) as ::core::ffi::c_int;
        let mut cursor_col: colnr_T = (*curwin.get()).w_cursor.col;
        if (*buf).b_prompt_start.mark.col < old_prompt_len
            || (*buf).b_prompt_start.mark.col > old_line_len
            || !strnequal(
                old_prompt,
                old_line
                    .offset((*buf).b_prompt_start.mark.col as isize)
                    .offset(-(old_prompt_len as isize)),
                old_prompt_len as size_t,
            )
        {
            ml_replace_buf(
                buf,
                prompt_lno,
                new_prompt as *mut ::core::ffi::c_char,
                true_0 != 0,
                false_0 != 0,
            );
            extmark_splice_cols(
                buf,
                prompt_lno as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                0 as colnr_T,
                old_line_len,
                new_prompt_len as colnr_T,
                kExtmarkNoUndo,
            );
            cursor_col = new_prompt_len as colnr_T;
        } else {
            let mut new_line: *mut ::core::ffi::c_char = concat_str(
                new_prompt,
                old_line.offset((*buf).b_prompt_start.mark.col as isize),
            );
            if ml_replace_buf(buf, prompt_lno, new_line, false_0 != 0, false_0 != 0) != OK {
                xfree(new_line as *mut ::core::ffi::c_void);
            }
            extmark_splice_cols(
                buf,
                prompt_lno as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                0 as colnr_T,
                (*buf).b_prompt_start.mark.col,
                new_prompt_len as colnr_T,
                kExtmarkNoUndo,
            );
            cursor_col +=
                (new_prompt_len as colnr_T - (*buf).b_prompt_start.mark.col) as ::core::ffi::c_int;
        }
        if (*curwin.get()).w_buffer == buf && (*curwin.get()).w_cursor.lnum == prompt_lno {
            (*curwin.get()).w_cursor.col = cursor_col;
            check_cursor_col(curwin.get());
        }
        changed_lines(
            buf,
            prompt_lno,
            0 as colnr_T,
            prompt_lno + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
        u_clearallandblockfree(buf);
    }
    xfree((*buf).b_prompt_text as *mut ::core::ffi::c_void);
    (*buf).b_prompt_text = xstrdup(new_prompt);
    (*buf).b_prompt_start.mark.col = new_prompt_len;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
#[inline]
unsafe extern "C" fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_last;
}
pub const SEA_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEA_READONLY: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
