use crate::src::nvim::autocmd::{block_autocmds, is_aucmd_win, unblock_autocmds};
use crate::src::nvim::buffer::{bt_quickfix, bt_terminal, do_autochdir};
use crate::src::nvim::cursor::{check_cursor, check_pos};
use crate::src::nvim::eval::funcs::execute_common;
use crate::src::nvim::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_dict_add_dict, tv_dict_add_list, tv_dict_add_nr,
    tv_dict_alloc, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_number, tv_get_number,
    tv_get_number_chk, tv_get_string_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict,
    tv_list_append_list, tv_list_append_number, tv_list_append_string,
};
use crate::src::nvim::ex_getln::text_or_buf_locked;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    cmdwin_type, cmdwin_win, curbuf, curtab, curwin, e_auabort, e_invalwindow, e_invexpr2,
    first_tabpage, firstwin, lastused_tabpage, lastwin, p_acd, prevwin, VIsual, VIsual_active,
};
use crate::src::nvim::memory::{strequal, xfree, xmallocz, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::normal::end_visual_mode;
use crate::src::nvim::os::fs::{os_chdir, os_dirname};
use crate::src::nvim::os::libc::{gettext, memset, strcmp, strtol};
use crate::src::nvim::r#move::{
    changed_window_setting, check_topfill, set_topline, update_curswant, validate_botline_win,
    validate_cursor, win_col_off,
};
use crate::src::nvim::strings::vim_snprintf_safelen;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1,
    EvalFuncData, ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection,
    ListLenSpecials, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MsgpackRpcRequestHandler, OptInt,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_11, Terminal,
    Timestamp, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig,
    WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T,
    buf_T, bufstate_T, chunksize_T, colnr_T, dict_T, dictitem_T, dictvar_S, diff_T, diffblock_S,
    disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_10,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, ssize_t, switchwin_T, syn_state, syn_state_sst_union as C2Rust_Unnamed_3,
    syn_time_T, synblock_T, synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T,
    typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_7, u_header_uh_alt_prev as C2Rust_Unnamed_6,
    u_header_uh_next as C2Rust_Unnamed_9, u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, win_execute_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
use crate::src::nvim::window::{
    check_split_disallowed, find_tabpage, goto_tabpage_tp, goto_tabpage_win, tabpage_index,
    unuse_tabpage, use_tabpage, valid_tabpage, win_drag_status_line, win_drag_vsep_line,
    win_get_tabwin, win_goto, win_horz_neighbor, win_new_height, win_new_width, win_splitmove,
    win_valid, win_vert_neighbor,
};
extern "C" {
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
}
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
pub const kStlClickFuncRun: C2Rust_Unnamed_11 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_11 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_11 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_11 = 0;
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
pub const LOWEST_WIN_ID: C2Rust_Unnamed_13 = 1000;
pub const WSP_ABOVE: C2Rust_Unnamed_12 = 128;
pub const WSP_BELOW: C2Rust_Unnamed_12 = 64;
pub const WSP_VERT: C2Rust_Unnamed_12 = 2;
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_12 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_12 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_12 = 256;
pub const WSP_HELP: C2Rust_Unnamed_12 = 32;
pub const WSP_BOT: C2Rust_Unnamed_12 = 16;
pub const WSP_TOP: C2Rust_Unnamed_12 = 8;
pub const WSP_HOR: C2Rust_Unnamed_12 = 4;
pub const WSP_ROOM: C2Rust_Unnamed_12 = 1;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
static e_cannot_resize_window_in_another_tab_page: GlobalCell<[::core::ffi::c_char; 50]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
            *b"E1308: Cannot resize a window in another tab page\0",
        )
    });
#[no_mangle]
pub unsafe extern "C" fn win_has_winnr(mut wp: *mut win_T, mut tp: *mut tabpage_T) -> bool {
    return wp
        == (if tp == curtab.get() {
            curwin.get()
        } else {
            (*tp).tp_curwin
        })
        || !(*wp).w_config.hide && (*wp).w_config.focusable as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn win_getid(mut argvars: *mut typval_T) -> ::core::ffi::c_int {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (*curwin.get()).handle as ::core::ffi::c_int;
    }
    let mut winnr: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if winnr <= 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tp = curtab.get();
        wp = firstwin.get();
    } else {
        let mut tabnr: ::core::ffi::c_int =
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        let mut tp2: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp2.is_null() {
            tabnr -= 1;
            if tabnr == 0 as ::core::ffi::c_int {
                tp = tp2 as *mut tabpage_T;
                break;
            } else {
                tp2 = (*tp2).tp_next as *mut tabpage_T;
            }
        }
        if tp.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        if tp == curtab.get() {
            wp = firstwin.get();
        } else {
            wp = (*tp).tp_firstwin;
        }
    }
    while !wp.is_null() {
        winnr -= win_has_winnr(wp, tp) as ::core::ffi::c_int;
        if winnr == 0 as ::core::ffi::c_int {
            return (*wp).handle as ::core::ffi::c_int;
        }
        wp = (*wp).w_next;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn win_id2tabwin(argvars: *mut typval_T, rettv: *mut typval_T) {
    let mut id: handle_T =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as handle_T;
    let mut winnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut tabnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    win_get_tabwin(id, &raw mut tabnr, &raw mut winnr);
    let list: *mut list_T = tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    tv_list_append_number(list, tabnr as varnumber_T);
    tv_list_append_number(list, winnr as varnumber_T);
}
#[no_mangle]
pub unsafe extern "C" fn win_id2wp(mut id: ::core::ffi::c_int) -> *mut win_T {
    return win_id2wp_tp(id, ::core::ptr::null_mut::<*mut tabpage_T>());
}
#[no_mangle]
pub unsafe extern "C" fn win_id2wp_tp(
    mut id: ::core::ffi::c_int,
    mut tpp: *mut *mut tabpage_T,
) -> *mut win_T {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).handle == id {
                if !tpp.is_null() {
                    *tpp = tp as *mut tabpage_T;
                }
                return wp;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return ::core::ptr::null_mut::<win_T>();
}
unsafe extern "C" fn win_id2win(mut argvars: *mut typval_T) -> ::core::ffi::c_int {
    let mut nr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).handle == id {
            return if win_has_winnr(wp, curtab.get()) as ::core::ffi::c_int != 0 {
                nr
            } else {
                0 as ::core::ffi::c_int
            };
        }
        nr += win_has_winnr(wp, curtab.get()) as ::core::ffi::c_int;
        wp = (*wp).w_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn win_findbuf(mut argvars: *mut typval_T, mut list: *mut list_T) {
    let mut bufnr: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*(*wp).w_buffer).handle == bufnr {
                tv_list_append_number(list, (*wp).handle as varnumber_T);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn find_win_by_nr(
    mut vp: *mut typval_T,
    mut tp: *mut tabpage_T,
) -> *mut win_T {
    let mut nr: ::core::ffi::c_int =
        tv_get_number_chk(vp, ::core::ptr::null_mut::<bool>()) as ::core::ffi::c_int;
    if nr < 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<win_T>();
    }
    if nr == 0 as ::core::ffi::c_int {
        return curwin.get();
    }
    if tp.is_null() {
        tp = curtab.get();
    }
    let mut wp: *mut win_T = if tp == curtab.get() {
        firstwin.get()
    } else {
        (*tp).tp_firstwin
    };
    while !wp.is_null() {
        if nr >= LOWEST_WIN_ID as ::core::ffi::c_int {
            if (*wp).handle == nr {
                return wp;
            }
        } else {
            nr -= 1;
            if nr <= 0 as ::core::ffi::c_int {
                return wp;
            }
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
#[no_mangle]
pub unsafe extern "C" fn find_win_by_nr_or_id(mut vp: *mut typval_T) -> *mut win_T {
    let mut nr: ::core::ffi::c_int =
        tv_get_number_chk(vp, ::core::ptr::null_mut::<bool>()) as ::core::ffi::c_int;
    if nr >= LOWEST_WIN_ID as ::core::ffi::c_int {
        return win_id2wp(tv_get_number(vp) as ::core::ffi::c_int);
    }
    return find_win_by_nr(vp, ::core::ptr::null_mut::<tabpage_T>());
}
#[no_mangle]
pub unsafe extern "C" fn find_tabwin(mut wvp: *mut typval_T, mut tvp: *mut typval_T) -> *mut win_T {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if (*wvp).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*tvp).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut n: ::core::ffi::c_int = tv_get_number(tvp) as ::core::ffi::c_int;
            if n >= 0 as ::core::ffi::c_int {
                tp = find_tabpage(n);
            }
        } else {
            tp = curtab.get();
        }
        if !tp.is_null() {
            wp = find_win_by_nr(wvp, tp);
        }
    } else {
        wp = curwin.get();
    }
    return wp;
}
unsafe extern "C" fn get_framelayout(mut fr: *const frame_T, mut l: *mut list_T, mut outer: bool) {
    if fr.is_null() {
        return;
    }
    let mut fr_list: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if outer {
        fr_list = l;
    } else {
        fr_list = tv_list_alloc(2 as ptrdiff_t);
        tv_list_append_list(l, fr_list);
    }
    if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF {
        if !(*fr).fr_win.is_null() {
            tv_list_append_string(
                fr_list,
                b"leaf\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
            tv_list_append_number(fr_list, (*(*fr).fr_win).handle as varnumber_T);
        }
    } else {
        if (*fr).fr_layout as ::core::ffi::c_int == FR_ROW {
            tv_list_append_string(
                fr_list,
                b"row\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        } else {
            tv_list_append_string(
                fr_list,
                b"col\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        }
        let win_list: *mut list_T =
            tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
        tv_list_append_list(fr_list, win_list);
        let mut child: *const frame_T = (*fr).fr_child;
        while !child.is_null() {
            get_framelayout(child, win_list, false_0 != 0);
            child = (*child).fr_next;
        }
    };
}
unsafe extern "C" fn get_winnr(
    mut tp: *mut tabpage_T,
    mut argvar: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut nr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut twin: *mut win_T = if tp == curtab.get() {
        curwin.get()
    } else {
        (*tp).tp_curwin
    };
    if (*argvar).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut invalid_arg: bool = false_0 != 0;
        let arg: *const ::core::ffi::c_char = tv_get_string_chk(argvar);
        if arg.is_null() {
            nr = 0 as ::core::ffi::c_int;
        } else if strcmp(arg, b"$\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            twin = if tp == curtab.get() {
                lastwin.get()
            } else {
                (*tp).tp_lastwin
            };
        } else if strcmp(arg, b"#\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            twin = if tp == curtab.get() {
                prevwin.get()
            } else {
                (*tp).tp_prevwin
            };
            if twin.is_null() {
                nr = 0 as ::core::ffi::c_int;
            }
        } else {
            let mut endp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut count: ::core::ffi::c_int =
                strtol(arg, &raw mut endp, 10 as ::core::ffi::c_int) as ::core::ffi::c_int;
            if count <= 0 as ::core::ffi::c_int {
                count = 1 as ::core::ffi::c_int;
            }
            if !endp.is_null() && *endp as ::core::ffi::c_int != NUL {
                if strequal(endp, b"j\0".as_ptr() as *const ::core::ffi::c_char) {
                    twin = win_vert_neighbor(tp, twin, false_0 != 0, count);
                } else if strequal(endp, b"k\0".as_ptr() as *const ::core::ffi::c_char) {
                    twin = win_vert_neighbor(tp, twin, true_0 != 0, count);
                } else if strequal(endp, b"h\0".as_ptr() as *const ::core::ffi::c_char) {
                    twin = win_horz_neighbor(tp, twin, true_0 != 0, count);
                } else if strequal(endp, b"l\0".as_ptr() as *const ::core::ffi::c_char) {
                    twin = win_horz_neighbor(tp, twin, false_0 != 0, count);
                } else {
                    invalid_arg = true_0 != 0;
                }
            } else {
                invalid_arg = true_0 != 0;
            }
        }
        if invalid_arg {
            semsg(
                gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                arg,
            );
            nr = 0 as ::core::ffi::c_int;
        }
    } else if !win_has_winnr(twin, tp) {
        nr = 0 as ::core::ffi::c_int;
    }
    if nr <= 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    nr = 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = if tp == curtab.get() {
        firstwin.get()
    } else {
        (*tp).tp_firstwin
    };
    while !wp.is_null() {
        nr += win_has_winnr(wp, tp) as ::core::ffi::c_int;
        if wp == twin {
            break;
        }
        wp = (*wp).w_next;
    }
    if wp.is_null() {
        nr = 0 as ::core::ffi::c_int;
    }
    return nr;
}
unsafe extern "C" fn get_win_info(
    mut wp: *mut win_T,
    mut tpnr: int16_t,
    mut winnr: int16_t,
) -> *mut dict_T {
    let dict: *mut dict_T = tv_dict_alloc();
    validate_botline_win(wp);
    tv_dict_add_nr(
        dict,
        b"tabnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        tpnr as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"winnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        winnr as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"winid\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*wp).handle as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"height\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*wp).w_view_height as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"status_height\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
        (*wp).w_status_height as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"winrow\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        ((*wp).w_winrow + 1 as ::core::ffi::c_int) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"topline\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*wp).w_topline as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"botline\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        ((*wp).w_botline - 1 as linenr_T) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"leftcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*wp).w_leftcol as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"winbar\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*wp).w_winbar_height as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"width\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*wp).w_view_width as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*(*wp).w_buffer).handle as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"wincol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        ((*wp).w_wincol + 1 as ::core::ffi::c_int) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"textoff\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        win_col_off(wp) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"terminal\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        bt_terminal((*wp).w_buffer) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"quickfix\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        bt_quickfix((*wp).w_buffer) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"loclist\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && !(*wp).w_llist_ref.is_null())
            as ::core::ffi::c_int as varnumber_T,
    );
    tv_dict_add_dict(
        dict,
        b"variables\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (*wp).w_vars,
    );
    return dict;
}
unsafe extern "C" fn get_tabpage_info(
    mut tp: *mut tabpage_T,
    mut tp_idx: ::core::ffi::c_int,
) -> *mut dict_T {
    let dict: *mut dict_T = tv_dict_alloc();
    tv_dict_add_nr(
        dict,
        b"tabnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        tp_idx as varnumber_T,
    );
    let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut wp: *mut win_T = if tp == curtab.get() {
        firstwin.get()
    } else {
        (*tp).tp_firstwin
    };
    while !wp.is_null() {
        tv_list_append_number(l, (*wp).handle as varnumber_T);
        wp = (*wp).w_next;
    }
    tv_dict_add_list(
        dict,
        b"windows\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        l,
    );
    tv_dict_add_dict(
        dict,
        b"variables\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (*tp).tp_vars,
    );
    return dict;
}
#[no_mangle]
pub unsafe extern "C" fn f_gettabinfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut tparg: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    tv_list_alloc_ret(
        rettv,
        (if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            1 as ::core::ffi::c_int
        } else {
            kListLenMayKnow as ::core::ffi::c_int
        }) as ptrdiff_t,
    );
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tparg = find_tabpage(tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) as ::core::ffi::c_int);
        if tparg.is_null() {
            return;
        }
    }
    let mut tpnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        tpnr += 1;
        if !(!tparg.is_null() && tp != tparg) {
            let d: *mut dict_T = get_tabpage_info(tp as *mut tabpage_T, tpnr);
            tv_list_append_dict((*rettv).vval.v_list, d);
            if !tparg.is_null() {
                return;
            }
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_getwininfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wparg: *mut win_T = ::core::ptr::null_mut::<win_T>();
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        wparg = win_id2wp(
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
        );
        if wparg.is_null() {
            return;
        }
    }
    let mut tabnr: int16_t = 0 as int16_t;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        tabnr += 1;
        let mut winnr: int16_t = 0 as int16_t;
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            winnr = (winnr as ::core::ffi::c_int
                + win_has_winnr(wp, tp as *mut tabpage_T) as ::core::ffi::c_int)
                as int16_t;
            if !(!wparg.is_null() && wp != wparg) {
                let d: *mut dict_T = get_win_info(
                    wp,
                    tabnr,
                    (if win_has_winnr(wp, tp as *mut tabpage_T) as ::core::ffi::c_int != 0 {
                        winnr as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as int16_t,
                );
                tv_list_append_dict((*rettv).vval.v_list, d);
                if !wparg.is_null() {
                    return;
                }
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_getwinpos(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    tv_list_append_number((*rettv).vval.v_list, -1 as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, -1 as varnumber_T);
}
#[no_mangle]
pub unsafe extern "C" fn f_getwinposx(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_getwinposy(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_tabpagenr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut nr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let arg: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        nr = 0 as ::core::ffi::c_int;
        if !arg.is_null() {
            if strcmp(arg, b"$\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
            {
                nr = tabpage_index(::core::ptr::null_mut::<tabpage_T>()) - 1 as ::core::ffi::c_int;
            } else if strcmp(arg, b"#\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                nr = if valid_tabpage(lastused_tabpage.get()) as ::core::ffi::c_int != 0 {
                    tabpage_index(lastused_tabpage.get())
                } else {
                    0 as ::core::ffi::c_int
                };
            } else {
                semsg(
                    gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                    arg,
                );
            }
        }
    } else {
        nr = tabpage_index(curtab.get());
    }
    (*rettv).vval.v_number = nr as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_tabpagewinnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut nr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let tp: *mut tabpage_T = find_tabpage(tv_get_number(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ) as ::core::ffi::c_int);
    if tp.is_null() {
        nr = 0 as ::core::ffi::c_int;
    } else {
        nr = get_winnr(tp, argvars.offset(1 as ::core::ffi::c_int as isize));
    }
    (*rettv).vval.v_number = nr as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn win_execute_before(
    mut args: *mut win_execute_T,
    mut wp: *mut win_T,
    mut tp: *mut tabpage_T,
) -> bool {
    (*args).wp = wp;
    (*args).curpos = (*wp).w_cursor;
    (*args).cwd_status = FAIL;
    (*args).apply_acd = false_0 != 0;
    (*args).save_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if curwin.get() != wp
        && (!(*curwin.get()).w_localdir.is_null()
            || !(*wp).w_localdir.is_null()
            || curtab.get() != tp
                && (!(*curtab.get()).tp_localdir.is_null() || !(*tp).tp_localdir.is_null())
            || p_acd.get() != 0)
    {
        (*args).cwd_status = os_dirname(
            &raw mut (*args).cwd as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
        );
    }
    if (*args).cwd_status == OK && p_acd.get() != 0 {
        if !(*curbuf.get()).b_sfname.is_null()
            && (*curbuf.get()).b_fname == (*curbuf.get()).b_sfname
        {
            (*args).save_sfname = xstrdup((*curbuf.get()).b_sfname);
        }
        do_autochdir();
        let mut autocwd: [::core::ffi::c_char; 4096] = [0; 4096];
        if os_dirname(
            &raw mut autocwd as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
        ) == OK
        {
            (*args).apply_acd = strcmp(
                &raw mut (*args).cwd as *mut ::core::ffi::c_char,
                &raw mut autocwd as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int;
        }
    }
    if switch_win_noblock(&raw mut (*args).switchwin, wp, tp, true_0 != 0) == OK {
        check_cursor(curwin.get());
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn win_execute_after(mut args: *mut win_execute_T) {
    restore_win_noblock(&raw mut (*args).switchwin, true_0 != 0);
    if (*args).apply_acd {
        xfree((*args).save_sfname as *mut ::core::ffi::c_void);
        do_autochdir();
    } else if (*args).cwd_status == OK {
        os_chdir(&raw mut (*args).cwd as *mut ::core::ffi::c_char);
        if !(*args).save_sfname.is_null() {
            xfree((*curbuf.get()).b_sfname as *mut ::core::ffi::c_void);
            (*curbuf.get()).b_sfname = (*args).save_sfname;
            (*curbuf.get()).b_fname = (*curbuf.get()).b_sfname;
        }
    }
    if win_valid((*args).wp) as ::core::ffi::c_int != 0
        && !equalpos((*args).curpos, (*(*args).wp).w_cursor)
    {
        (*(*args).wp).w_redr_status = true_0 != 0;
    }
    check_cursor(curwin.get());
    if VIsual_active.get() {
        check_pos(curbuf.get(), VIsual.ptr());
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_win_execute(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = tv_get_number(argvars) as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut wp: *mut win_T = win_id2wp_tp(id, &raw mut tp);
    if wp.is_null() || tp.is_null() {
        return;
    }
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
    if win_execute_before(&raw mut win_execute_args, wp, tp) {
        execute_common(argvars, rettv, 1 as ::core::ffi::c_int);
    }
    win_execute_after(&raw mut win_execute_args);
}
#[no_mangle]
pub unsafe extern "C" fn f_win_findbuf(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    win_findbuf(argvars, (*rettv).vval.v_list);
}
#[no_mangle]
pub unsafe extern "C" fn f_win_getid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = win_getid(argvars) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_win_gotoid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    if (*curwin.get()).handle == id {
        (*rettv).vval.v_number = 1 as varnumber_T;
        return;
    }
    if text_or_buf_locked() {
        return;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).handle == id {
                if VIsual_active.get() as ::core::ffi::c_int != 0 && (*wp).w_buffer != curbuf.get()
                {
                    end_visual_mode();
                }
                goto_tabpage_win(tp as *mut tabpage_T, wp);
                (*rettv).vval.v_number = 1 as varnumber_T;
                return;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_win_id2tabwin(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    win_id2tabwin(argvars, rettv);
}
#[no_mangle]
pub unsafe extern "C" fn f_win_id2win(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = win_id2win(argvars) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_win_move_separator(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = false_0 as varnumber_T;
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() || (*wp).w_floating as ::core::ffi::c_int != 0 {
        return;
    }
    if !win_valid(wp) {
        emsg(gettext(
            (e_cannot_resize_window_in_another_tab_page.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut offset: ::core::ffi::c_int =
        tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    win_drag_vsep_line(wp, offset);
    (*rettv).vval.v_number = true_0 as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_win_move_statusline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut offset: ::core::ffi::c_int = 0;
    (*rettv).vval.v_number = false_0 as varnumber_T;
    wp = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() || (*wp).w_floating as ::core::ffi::c_int != 0 {
        return;
    }
    if !win_valid(wp) {
        emsg(gettext(
            (e_cannot_resize_window_in_another_tab_page.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    offset = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    win_drag_status_line(wp, offset);
    (*rettv).vval.v_number = true_0 as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_win_screenpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    let wp: *const win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    tv_list_append_number(
        (*rettv).vval.v_list,
        (if wp.is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*wp).w_winrow + 1 as ::core::ffi::c_int
        }) as varnumber_T,
    );
    tv_list_append_number(
        (*rettv).vval.v_list,
        (if wp.is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*wp).w_wincol + 1 as ::core::ffi::c_int
        }) as varnumber_T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn f_win_splitmove(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut targetwin: *mut win_T =
        find_win_by_nr_or_id(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut oldwin: *mut win_T = curwin.get();
    (*rettv).vval.v_number = -1 as varnumber_T;
    if wp.is_null()
        || targetwin.is_null()
        || wp == targetwin
        || !win_valid(wp)
        || !win_valid(targetwin)
        || (*targetwin).w_floating as ::core::ffi::c_int != 0
    {
        emsg(gettext(
            &raw const e_invalwindow as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        if tv_check_for_nonnull_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return;
        }
        d = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if tv_dict_get_number(d, b"vertical\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
            flags |= WSP_VERT as ::core::ffi::c_int;
        }
        di = tv_dict_find(
            d,
            b"rightbelow\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            flags |= if tv_get_number(&raw mut (*di).di_tv) != 0 {
                WSP_BELOW as ::core::ffi::c_int
            } else {
                WSP_ABOVE as ::core::ffi::c_int
            };
        }
        size = tv_dict_get_number(d, b"size\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int;
    }
    if is_aucmd_win(wp) as ::core::ffi::c_int != 0
        || text_or_buf_locked() as ::core::ffi::c_int != 0
        || check_split_disallowed(wp) == FAIL
    {
        return;
    }
    if curwin.get() != targetwin {
        win_goto(targetwin);
    }
    if curwin.get() == targetwin && win_valid(wp) as ::core::ffi::c_int != 0 {
        if win_splitmove(wp, size, flags) == OK {
            (*rettv).vval.v_number = 0 as varnumber_T;
        }
    } else {
        emsg(gettext(&raw const e_auabort as *const ::core::ffi::c_char));
    }
    if oldwin != curwin.get() && win_valid(oldwin) as ::core::ffi::c_int != 0 {
        win_goto(oldwin);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_win_gettype(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = curwin.get();
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        wp = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
        if wp.is_null() {
            (*rettv).vval.v_string = xstrdup(b"unknown\0".as_ptr() as *const ::core::ffi::c_char);
            return;
        }
    }
    if is_aucmd_win(wp) {
        (*rettv).vval.v_string = xstrdup(b"autocmd\0".as_ptr() as *const ::core::ffi::c_char);
    } else if (*wp).w_onebuf_opt.wo_pvw != 0 {
        (*rettv).vval.v_string = xstrdup(b"preview\0".as_ptr() as *const ::core::ffi::c_char);
    } else if (*wp).w_floating {
        (*rettv).vval.v_string = xstrdup(b"popup\0".as_ptr() as *const ::core::ffi::c_char);
    } else if wp == cmdwin_win.get() {
        (*rettv).vval.v_string = xstrdup(b"command\0".as_ptr() as *const ::core::ffi::c_char);
    } else if bt_quickfix((*wp).w_buffer) {
        (*rettv).vval.v_string = xstrdup(if !(*wp).w_llist_ref.is_null() {
            b"loclist\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"quickfix\0".as_ptr() as *const ::core::ffi::c_char
        });
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_getcmdwintype(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*rettv).vval.v_string = xmallocz(1 as size_t) as *mut ::core::ffi::c_char;
    *(*rettv)
        .vval
        .v_string
        .offset(0 as ::core::ffi::c_int as isize) = cmdwin_type.get() as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn f_winbufnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = (*(*wp).w_buffer).handle as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_wincol(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    validate_cursor(curwin.get());
    (*rettv).vval.v_number = ((*curwin.get()).w_wcol + 1 as ::core::ffi::c_int) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_winheight(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = (*wp).w_view_height as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_winlayout(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tp = curtab.get();
    } else {
        tp = find_tabpage(
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
        );
        if tp.is_null() {
            return;
        }
    }
    get_framelayout((*tp).tp_topframe, (*rettv).vval.v_list, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_winline(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    validate_cursor(curwin.get());
    (*rettv).vval.v_number = ((*curwin.get()).w_wrow + 1 as ::core::ffi::c_int) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_winnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = get_winnr(
        curtab.get(),
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_winrestcmd(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 50] = [0; 50];
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        70 as ::core::ffi::c_int,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 2 as ::core::ffi::c_int {
        let mut winnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if win_has_winnr(wp, curtab.get()) {
                let mut buflen: size_t = vim_snprintf_safelen(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 50]>(),
                    b"%dresize %d|\0".as_ptr() as *const ::core::ffi::c_char,
                    winnr,
                    (*wp).w_height,
                );
                ga_concat_len(
                    &raw mut ga,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    buflen,
                );
                buflen = vim_snprintf_safelen(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 50]>(),
                    b"vert %dresize %d|\0".as_ptr() as *const ::core::ffi::c_char,
                    winnr,
                    (*wp).w_width,
                );
                ga_concat_len(
                    &raw mut ga,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    buflen,
                );
                winnr += 1;
            }
            wp = (*wp).w_next;
        }
        i += 1;
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
    (*rettv).v_type = VAR_STRING;
}
#[no_mangle]
pub unsafe extern "C" fn f_winrestview(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_nonnull_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut dict: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    di = tv_dict_find(
        dict,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_cursor.lnum = tv_get_number(&raw mut (*di).di_tv) as linenr_T;
    }
    di = tv_dict_find(
        dict,
        b"col\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_cursor.col = tv_get_number(&raw mut (*di).di_tv) as colnr_T;
    }
    di = tv_dict_find(
        dict,
        b"coladd\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_cursor.coladd = tv_get_number(&raw mut (*di).di_tv) as colnr_T;
    }
    di = tv_dict_find(
        dict,
        b"curswant\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_curswant = tv_get_number(&raw mut (*di).di_tv) as colnr_T;
        (*curwin.get()).w_set_curswant = false_0;
    }
    di = tv_dict_find(
        dict,
        b"topline\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        set_topline(
            curwin.get(),
            tv_get_number(&raw mut (*di).di_tv) as linenr_T,
        );
    }
    di = tv_dict_find(
        dict,
        b"topfill\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_topfill = tv_get_number(&raw mut (*di).di_tv) as ::core::ffi::c_int;
    }
    di = tv_dict_find(
        dict,
        b"leftcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_leftcol = tv_get_number(&raw mut (*di).di_tv) as colnr_T;
    }
    di = tv_dict_find(
        dict,
        b"skipcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_skipcol = tv_get_number(&raw mut (*di).di_tv) as colnr_T;
    }
    check_cursor(curwin.get());
    win_new_height(curwin.get(), (*curwin.get()).w_height);
    win_new_width(curwin.get(), (*curwin.get()).w_width);
    changed_window_setting(curwin.get());
    if (*curwin.get()).w_topline <= 0 as linenr_T {
        (*curwin.get()).w_topline = 1 as ::core::ffi::c_int as linenr_T;
    }
    if (*curwin.get()).w_topline > (*curbuf.get()).b_ml.ml_line_count {
        (*curwin.get()).w_topline = (*curbuf.get()).b_ml.ml_line_count;
    }
    check_topfill(curwin.get(), true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_winsaveview(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let mut dict: *mut dict_T = (*rettv).vval.v_dict;
    tv_dict_add_nr(
        dict,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_cursor.lnum as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"col\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_cursor.col as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"coladd\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_cursor.coladd as varnumber_T,
    );
    update_curswant();
    tv_dict_add_nr(
        dict,
        b"curswant\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_curswant as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"topline\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_topline as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"topfill\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_topfill as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"leftcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_leftcol as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"skipcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_skipcol as varnumber_T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn f_winwidth(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = (*wp).w_view_width as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn switch_win(
    mut switchwin: *mut switchwin_T,
    mut win: *mut win_T,
    mut tp: *mut tabpage_T,
    mut no_display: bool,
) -> ::core::ffi::c_int {
    block_autocmds();
    return switch_win_noblock(switchwin, win, tp, no_display);
}
#[no_mangle]
pub unsafe extern "C" fn switch_win_noblock(
    mut switchwin: *mut switchwin_T,
    mut win: *mut win_T,
    mut tp: *mut tabpage_T,
    mut no_display: bool,
) -> ::core::ffi::c_int {
    memset(
        switchwin as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<switchwin_T>(),
    );
    (*switchwin).sw_curwin = curwin.get();
    if win == curwin.get() {
        (*switchwin).sw_same_win = true_0 != 0;
    } else {
        (*switchwin).sw_visual_active = VIsual_active.get();
        VIsual_active.set(false_0 != 0);
    }
    if !tp.is_null() {
        (*switchwin).sw_curtab = curtab.get();
        if no_display {
            unuse_tabpage(curtab.get());
            use_tabpage(tp);
        } else {
            goto_tabpage_tp(tp, false_0 != 0, false_0 != 0);
        }
    }
    if !win_valid(win) {
        return FAIL;
    }
    curwin.set(win);
    curbuf.set((*curwin.get()).w_buffer);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn restore_win(mut switchwin: *mut switchwin_T, mut no_display: bool) {
    restore_win_noblock(switchwin, no_display);
    unblock_autocmds();
}
#[no_mangle]
pub unsafe extern "C" fn restore_win_noblock(
    mut switchwin: *mut switchwin_T,
    mut no_display: bool,
) {
    if !(*switchwin).sw_curtab.is_null()
        && valid_tabpage((*switchwin).sw_curtab) as ::core::ffi::c_int != 0
    {
        if no_display {
            let old_tp_curwin: *mut win_T = (*curtab.get()).tp_curwin;
            unuse_tabpage(curtab.get());
            (*curtab.get()).tp_curwin = old_tp_curwin;
            use_tabpage((*switchwin).sw_curtab);
        } else {
            goto_tabpage_tp((*switchwin).sw_curtab, false_0 != 0, false_0 != 0);
        }
    }
    if !(*switchwin).sw_same_win {
        VIsual_active.set((*switchwin).sw_visual_active);
    }
    if win_valid((*switchwin).sw_curwin) {
        curwin.set((*switchwin).sw_curwin);
        curbuf.set((*curwin.get()).w_buffer);
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
