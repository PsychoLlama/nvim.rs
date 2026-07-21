use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_6, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_3,
    Direction, EvalFuncData, ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView,
    Intersection, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MotionType, MsgpackRpcRequestHandler,
    OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_13, Terminal,
    Timestamp, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig,
    WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T,
    buf_T, bufstate_T, chunksize_T, cmdarg_T, colnr_T, dict_T, dictvar_S, diff_T, diffblock_S,
    disptick_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_4, file_buffer_b_wininfo as C2Rust_Unnamed_12,
    file_buffer_update_callbacks as C2Rust_Unnamed_1,
    file_buffer_update_channels as C2Rust_Unnamed_2, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_7, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, oparg_T, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_5, syn_time_T, synblock_T,
    synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union,
    u_entry, u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn labs(__x: ::core::ffi::c_long) -> ::core::ffi::c_long;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn buf_is_empty(buf: *mut buf_T) -> bool;
    fn coladvance(wp: *mut win_T, wcol: colnr_T) -> ::core::ffi::c_int;
    fn check_cursor_lnum(win: *mut win_T);
    fn check_cursor(wp: *mut win_T);
    fn decor_conceal_line(wp: *mut win_T, row: ::core::ffi::c_int, check_cursor_0: bool) -> bool;
    fn win_lines_concealed(wp: *mut win_T) -> bool;
    fn diff_get_corresponding_line(buf1: *mut buf_T, lnum1: linenr_T) -> linenr_T;
    fn redrawing() -> bool;
    fn win_scroll_lines(wp: *mut win_T, row: ::core::ffi::c_int, line_count: ::core::ffi::c_int);
    fn number_width(wp: *mut win_T) -> ::core::ffi::c_int;
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_buf_later(buf: *mut buf_T, type_0: ::core::ffi::c_int);
    fn redrawWinline(wp: *mut win_T, lnum: linenr_T);
    fn conceal_cursor_line(wp: *const win_T) -> bool;
    fn win_cursorline_standout(wp: *const win_T) -> bool;
    fn beginline(flags: ::core::ffi::c_int);
    fn cursor_up_inner(wp: *mut win_T, n: linenr_T, skip_conceal: bool);
    fn cursor_up(n: linenr_T, upd_topline: bool) -> ::core::ffi::c_int;
    fn cursor_down_inner(wp: *mut win_T, n: ::core::ffi::c_int, skip_conceal: bool);
    fn cursor_down(n: ::core::ffi::c_int, upd_topline: bool) -> ::core::ffi::c_int;
    static e_invalid_line_number_nr: [::core::ffi::c_char; 0];
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_dict_add_nr(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: varnumber_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_alloc_ret(ret_tv: *mut typval_T);
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_number_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn find_win_by_nr_or_id(vp: *mut typval_T) -> *mut win_T;
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    fn foldAdjustCursor(wp: *mut win_T);
    fn beep_flush();
    static Rows: GlobalCell<::core::ffi::c_int>;
    static dollar_vcol: GlobalCell<colnr_T>;
    static mouse_dragging: GlobalCell<::core::ffi::c_int>;
    static firstwin: GlobalCell<*mut win_T>;
    static lastwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static VIsual_active: GlobalCell<bool>;
    static VIsual_select: GlobalCell<bool>;
    static restart_edit: GlobalCell<::core::ffi::c_int>;
    static cmdwin_win: GlobalCell<*mut win_T>;
    static skip_update_topline: GlobalCell<bool>;
    static default_grid: GlobalCell<ScreenGrid>;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn mb_adjust_cursor();
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn vcol2col(wp: *mut win_T, lnum: linenr_T, vcol: colnr_T, coladdp: *mut colnr_T) -> colnr_T;
    fn nv_screengo(
        oap: *mut oparg_T,
        dir: ::core::ffi::c_int,
        dist: ::core::ffi::c_int,
        skip_conceal: bool,
    ) -> bool;
    fn nv_g_home_m_cmd(cap: *mut cmdarg_T);
    fn get_showbreak_value(win: *mut win_T) -> *mut ::core::ffi::c_char;
    fn get_scrolloff_value(wp: *mut win_T) -> int64_t;
    fn get_sidescrolloff_value(wp: *mut win_T) -> int64_t;
    static p_cpo: GlobalCell<*mut ::core::ffi::c_char>;
    static p_sj: GlobalCell<OptInt>;
    static p_so: GlobalCell<OptInt>;
    static p_ss: GlobalCell<OptInt>;
    static p_sol: GlobalCell<::core::ffi::c_int>;
    static p_window: GlobalCell<OptInt>;
    fn linetabsize_eol(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn getvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn getvvcol(
        wp: *mut win_T,
        pos: *mut pos_T,
        start: *mut colnr_T,
        cursor: *mut colnr_T,
        end: *mut colnr_T,
    );
    fn win_may_fill(wp: *mut win_T) -> bool;
    fn win_get_fill(wp: *mut win_T, lnum: linenr_T) -> ::core::ffi::c_int;
    fn plines_win(wp: *mut win_T, lnum: linenr_T, limit_winheight: bool) -> ::core::ffi::c_int;
    fn plines_win_nofill(
        wp: *mut win_T,
        lnum: linenr_T,
        limit_winheight: bool,
    ) -> ::core::ffi::c_int;
    fn plines_win_full(
        wp: *mut win_T,
        lnum: linenr_T,
        nextp: *mut linenr_T,
        foldedp: *mut bool,
        cache: bool,
        limit_winheight: bool,
    ) -> ::core::ffi::c_int;
    fn plines_m_win(
        wp: *mut win_T,
        first: linenr_T,
        last: linenr_T,
        max: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn win_fdccol_count(wp: *mut win_T) -> ::core::ffi::c_int;
    fn win_check_anchored_floats(win: *mut win_T);
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed = 2147483647;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const SIGN_WIDTH: C2Rust_Unnamed_0 = 2;
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
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_14 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_14 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_14 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_14 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_14 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_14 = 20;
pub const UPD_VALID: C2Rust_Unnamed_14 = 10;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_15 = 4;
pub const BL_SOL: C2Rust_Unnamed_15 = 2;
pub const BL_WHITE: C2Rust_Unnamed_15 = 1;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const kOptCuloptFlagScreenline: C2Rust_Unnamed_16 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lineoff_T {
    pub lnum: linenr_T,
    pub fill: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kOptCuloptFlagNumber: C2Rust_Unnamed_16 = 4;
pub const kOptCuloptFlagLine: C2Rust_Unnamed_16 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const VALID_CHEIGHT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VALID_BOTLINE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VALID_BOTLINE_AP: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
unsafe extern "C" fn adjust_plines_for_skipcol(mut wp: *mut win_T) -> ::core::ffi::c_int {
    if (*wp).w_skipcol == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut width: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    let mut w2: ::core::ffi::c_int = width + win_col_off2(wp);
    if (*wp).w_skipcol >= width && w2 > 0 as ::core::ffi::c_int {
        return ((*wp).w_skipcol as ::core::ffi::c_int - width) / w2 + 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn plines_correct_topline(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut nextp: *mut linenr_T,
    mut limit_winheight: bool,
    mut foldedp: *mut bool,
) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int =
        plines_win_full(wp, lnum, nextp, foldedp, true_0 != 0, false_0 != 0);
    if lnum == (*wp).w_topline {
        n -= adjust_plines_for_skipcol(wp);
    }
    if limit_winheight as ::core::ffi::c_int != 0 && n > (*wp).w_view_height {
        return (*wp).w_view_height;
    }
    return n;
}
unsafe extern "C" fn comp_botline(mut wp: *mut win_T) {
    let mut lnum: linenr_T = 0;
    let mut done: ::core::ffi::c_int = 0;
    check_cursor_moved(wp);
    if (*wp).w_valid & VALID_CROW != 0 {
        lnum = (*wp).w_cursor.lnum;
        done = (*wp).w_cline_row;
    } else {
        lnum = (*wp).w_topline;
        done = 0 as ::core::ffi::c_int;
    }
    while lnum <= (*(*wp).w_buffer).b_ml.ml_line_count {
        let mut last: linenr_T = lnum;
        let mut folded: bool = false;
        let mut n: ::core::ffi::c_int =
            plines_correct_topline(wp, lnum, &raw mut last, true_0 != 0, &raw mut folded);
        if lnum <= (*wp).w_cursor.lnum && last >= (*wp).w_cursor.lnum {
            (*wp).w_cline_row = done;
            (*wp).w_cline_height = n;
            (*wp).w_cline_folded = folded;
            redraw_for_cursorline(wp);
            (*wp).w_valid |= VALID_CROW | VALID_CHEIGHT;
        }
        if done + n > (*wp).w_view_height {
            break;
        }
        done += n;
        lnum = last;
        lnum += 1;
    }
    (*wp).w_botline = lnum;
    (*wp).w_valid |= VALID_BOTLINE | VALID_BOTLINE_AP;
    (*wp).w_viewport_invalid = true_0 != 0;
    set_empty_rows(wp, done);
    win_check_anchored_floats(wp);
}
unsafe extern "C" fn redraw_for_cursorline(mut wp: *mut win_T) {
    if (*wp).w_valid & VALID_CROW != 0 {
        return;
    }
    if (*wp).w_onebuf_opt.wo_rnu != 0 || win_cursorline_standout(wp) as ::core::ffi::c_int != 0 {
        redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn redraw_for_cursorcolumn(mut wp: *mut win_T) {
    if wp == curwin.get()
        && (*wp).w_onebuf_opt.wo_cole > 0 as OptInt
        && conceal_cursor_line(wp) as ::core::ffi::c_int != 0
    {
        redrawWinline(wp, (*wp).w_cursor.lnum);
    }
    if (*wp).w_valid & VALID_VIRTCOL != 0 {
        return;
    }
    if (*wp).w_onebuf_opt.wo_cuc != 0 {
        redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
    } else if (*wp).w_onebuf_opt.wo_cul != 0
        && (*wp).w_p_culopt_flags as ::core::ffi::c_int
            & kOptCuloptFlagScreenline as ::core::ffi::c_int
            != 0
    {
        redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0 && (*wp).w_buffer == curbuf.get() {
        redraw_buf_later(curbuf.get(), UPD_INVERTED as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn set_valid_virtcol(mut wp: *mut win_T, mut vcol: colnr_T) {
    (*wp).w_virtcol = vcol;
    redraw_for_cursorcolumn(wp);
    (*wp).w_valid |= VALID_VIRTCOL;
}
#[no_mangle]
pub unsafe extern "C" fn sms_marker_overlap(
    mut wp: *mut win_T,
    mut extra2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if extra2 == -1 as ::core::ffi::c_int {
        extra2 = win_col_off(wp) - win_col_off2(wp);
    }
    if *get_showbreak_value(wp) as ::core::ffi::c_int != NUL {
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).w_onebuf_opt.wo_list != 0 && (*wp).w_p_lcs_chars.prec != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return if extra2 > 3 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        3 as ::core::ffi::c_int - extra2
    };
}
unsafe extern "C" fn skipcol_from_plines(
    mut wp: *mut win_T,
    mut plines_off: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    let mut skipcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if plines_off > 0 as ::core::ffi::c_int {
        skipcol += width1;
    }
    if plines_off > 1 as ::core::ffi::c_int {
        skipcol += (width1 + win_col_off2(wp)) * (plines_off - 1 as ::core::ffi::c_int);
    }
    return skipcol;
}
unsafe extern "C" fn reset_skipcol(mut wp: *mut win_T) {
    if (*wp).w_skipcol == 0 as ::core::ffi::c_int {
        return;
    }
    (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
    redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn update_topline(mut wp: *mut win_T) {
    let mut check_botline: bool = false_0 != 0;
    let mut so_ptr: *mut OptInt = if (*wp).w_onebuf_opt.wo_so >= 0 as OptInt {
        &raw mut (*wp).w_onebuf_opt.wo_so
    } else {
        p_so.ptr()
    };
    let mut save_so: OptInt = *so_ptr;
    if skip_update_topline.get() {
        return;
    }
    if (*default_grid.ptr()).chars.is_null() || (*wp).w_view_height == 0 as ::core::ffi::c_int {
        check_cursor_lnum(wp);
        (*wp).w_topline = (*wp).w_cursor.lnum;
        (*wp).w_botline = (*wp).w_topline;
        (*wp).w_viewport_invalid = true_0 != 0;
        (*wp).w_scbind_pos = 1 as ::core::ffi::c_int;
        return;
    }
    check_cursor_moved(wp);
    if (*wp).w_valid & VALID_TOPLINE != 0 {
        return;
    }
    if mouse_dragging.get() > 0 as ::core::ffi::c_int {
        *so_ptr = (mouse_dragging.get() - 1 as ::core::ffi::c_int) as OptInt;
    }
    let mut old_topline: linenr_T = (*wp).w_topline;
    let mut old_topfill: ::core::ffi::c_int = (*wp).w_topfill;
    if buf_is_empty((*wp).w_buffer) {
        if (*wp).w_topline != 1 as linenr_T {
            redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
        }
        (*wp).w_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*wp).w_botline = 2 as ::core::ffi::c_int as linenr_T;
        (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
        (*wp).w_valid |= VALID_BOTLINE | VALID_BOTLINE_AP;
        (*wp).w_viewport_invalid = true_0 != 0;
        (*wp).w_scbind_pos = 1 as ::core::ffi::c_int;
    } else {
        let mut check_topline: bool = false_0 != 0;
        if (*wp).w_topline > 1 as linenr_T || (*wp).w_skipcol > 0 as ::core::ffi::c_int {
            if (*wp).w_cursor.lnum < (*wp).w_topline {
                check_topline = true_0 != 0;
            } else if check_top_offset(wp) {
                check_topline = true_0 != 0;
            } else if (*wp).w_skipcol > 0 as ::core::ffi::c_int
                && (*wp).w_cursor.lnum == (*wp).w_topline
            {
                let mut vcol: colnr_T = 0;
                getvvcol(
                    wp,
                    &raw mut (*wp).w_cursor,
                    &raw mut vcol,
                    ::core::ptr::null_mut::<colnr_T>(),
                    ::core::ptr::null_mut::<colnr_T>(),
                );
                let mut overlap: ::core::ffi::c_int =
                    sms_marker_overlap(wp, -1 as ::core::ffi::c_int);
                if (*wp).w_skipcol as ::core::ffi::c_int + overlap > vcol {
                    check_topline = true_0 != 0;
                }
            }
        }
        if !check_topline && (*wp).w_topfill > win_get_fill(wp, (*wp).w_topline) {
            check_topline = true_0 != 0;
        }
        if check_topline {
            let mut halfheight: ::core::ffi::c_int =
                (*wp).w_view_height / 2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            if halfheight < 2 as ::core::ffi::c_int {
                halfheight = 2 as ::core::ffi::c_int;
            }
            let mut n: int64_t = 0;
            if win_lines_concealed(wp) {
                n = 0 as int64_t;
                let mut lnum: linenr_T = (*wp).w_cursor.lnum;
                while (lnum as OptInt) < (*wp).w_topline as OptInt + *so_ptr {
                    '_c2rust_label: {
                        if !(*wp).w_buffer.is_null() {
                        } else {
                            __assert_fail(
                                b"wp->w_buffer != 0\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/move.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                338 as ::core::ffi::c_uint,
                                b"void update_topline(win_T *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if lnum >= (*(*wp).w_buffer).b_ml.ml_line_count || {
                        n += !decor_conceal_line(wp, lnum as ::core::ffi::c_int, false_0 != 0)
                            as ::core::ffi::c_int as int64_t;
                        n >= halfheight as int64_t
                    } {
                        break;
                    }
                    hasFolding(wp, lnum, ::core::ptr::null_mut::<linenr_T>(), &raw mut lnum);
                    lnum += 1;
                }
            } else {
                n = ((*wp).w_topline as OptInt + *so_ptr - (*wp).w_cursor.lnum as OptInt)
                    as int64_t;
            }
            if n >= halfheight as int64_t {
                scroll_cursor_halfway(wp, false_0 != 0, false_0 != 0);
            } else {
                scroll_cursor_top(wp, scrolljump_value(wp), false_0);
                check_botline = true_0 != 0;
            }
        } else {
            hasFolding(
                wp,
                (*wp).w_topline,
                &raw mut (*wp).w_topline,
                ::core::ptr::null_mut::<linenr_T>(),
            );
            check_botline = true_0 != 0;
        }
    }
    if check_botline {
        if (*wp).w_valid & VALID_BOTLINE_AP == 0 {
            validate_botline_win(wp);
        }
        '_c2rust_label_0: {
            if !(*wp).w_buffer.is_null() {
            } else {
                __assert_fail(
                    b"wp->w_buffer != 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/move.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    376 as ::core::ffi::c_uint,
                    b"void update_topline(win_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if (*wp).w_botline <= (*(*wp).w_buffer).b_ml.ml_line_count {
            if (*wp).w_cursor.lnum < (*wp).w_botline {
                if (*wp).w_cursor.lnum as OptInt >= (*wp).w_botline as OptInt - *so_ptr
                    || win_lines_concealed(wp) as ::core::ffi::c_int != 0
                {
                    let mut loff: lineoff_T = lineoff_T {
                        lnum: 0,
                        fill: 0,
                        height: 0,
                    };
                    let mut n_0: ::core::ffi::c_int = (*wp).w_empty_rows;
                    loff.lnum = (*wp).w_cursor.lnum;
                    hasFolding(
                        wp,
                        loff.lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut loff.lnum,
                    );
                    loff.fill = 0 as ::core::ffi::c_int;
                    n_0 += (*wp).w_filler_rows;
                    loff.height = 0 as ::core::ffi::c_int;
                    while loff.lnum < (*wp).w_botline
                        && ((loff.lnum + 1 as linenr_T) < (*wp).w_botline
                            || loff.fill == 0 as ::core::ffi::c_int)
                    {
                        n_0 += loff.height;
                        if n_0 as OptInt >= *so_ptr {
                            break;
                        }
                        botline_forw(wp, &raw mut loff);
                    }
                    if n_0 as OptInt >= *so_ptr {
                        check_botline = false_0 != 0;
                    }
                } else {
                    check_botline = false_0 != 0;
                }
            }
            if check_botline {
                let mut n_1: int64_t = 0 as int64_t;
                if win_lines_concealed(wp) {
                    let mut lnum_0: linenr_T = (*wp).w_cursor.lnum;
                    while (lnum_0 as OptInt) >= (*wp).w_botline as OptInt - *so_ptr {
                        if lnum_0 <= 0 as linenr_T || {
                            n_1 += !decor_conceal_line(
                                wp,
                                lnum_0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                false_0 != 0,
                            ) as ::core::ffi::c_int as int64_t;
                            n_1 > ((*wp).w_view_height + 1 as ::core::ffi::c_int) as int64_t
                        } {
                            break;
                        }
                        hasFolding(
                            wp,
                            lnum_0,
                            &raw mut lnum_0,
                            ::core::ptr::null_mut::<linenr_T>(),
                        );
                        lnum_0 -= 1;
                    }
                } else {
                    n_1 = (((*wp).w_cursor.lnum - (*wp).w_botline + 1 as linenr_T) as OptInt
                        + *so_ptr) as int64_t;
                }
                if n_1 <= ((*wp).w_view_height + 1 as ::core::ffi::c_int) as int64_t {
                    scroll_cursor_bot(wp, scrolljump_value(wp), false_0 != 0);
                } else {
                    scroll_cursor_halfway(wp, false_0 != 0, false_0 != 0);
                }
            }
        }
    }
    (*wp).w_valid |= VALID_TOPLINE;
    (*wp).w_viewport_invalid = true_0 != 0;
    win_check_anchored_floats(wp);
    if (*wp).w_topline != old_topline || (*wp).w_topfill != old_topfill {
        dollar_vcol.set(-1 as ::core::ffi::c_int as colnr_T);
        redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
        if (*wp).w_onebuf_opt.wo_sms == 0 {
            reset_skipcol(wp);
        } else if (*wp).w_skipcol != 0 as ::core::ffi::c_int {
            redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
        }
        if (*wp).w_cursor.lnum == (*wp).w_topline {
            validate_cursor(wp);
        }
    }
    *so_ptr = save_so;
}
unsafe extern "C" fn scrolljump_value(mut wp: *mut win_T) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = if p_sj.get() >= 0 as OptInt {
        p_sj.get() as ::core::ffi::c_int
    } else {
        (*wp).w_view_height * -p_sj.get() as ::core::ffi::c_int / 100 as ::core::ffi::c_int
    };
    return result;
}
unsafe extern "C" fn check_top_offset(mut wp: *mut win_T) -> bool {
    let mut so: int64_t = get_scrolloff_value(wp);
    if ((*wp).w_cursor.lnum as int64_t) < (*wp).w_topline as int64_t + so
        || win_lines_concealed(wp) as ::core::ffi::c_int != 0
    {
        let mut loff: lineoff_T = lineoff_T {
            lnum: 0,
            fill: 0,
            height: 0,
        };
        loff.lnum = (*wp).w_cursor.lnum;
        loff.fill = 0 as ::core::ffi::c_int;
        let mut n: ::core::ffi::c_int = (*wp).w_topfill;
        while (n as int64_t) < so {
            topline_back(wp, &raw mut loff);
            if loff.lnum < (*wp).w_topline
                || loff.lnum == (*wp).w_topline && loff.fill > 0 as ::core::ffi::c_int
            {
                break;
            }
            n += loff.height;
        }
        if (n as int64_t) < so {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn update_curswant_force() {
    validate_virtcol(curwin.get());
    (*curwin.get()).w_curswant = (*curwin.get()).w_virtcol;
    (*curwin.get()).w_set_curswant = false_0;
}
#[no_mangle]
pub unsafe extern "C" fn update_curswant() {
    if (*curwin.get()).w_set_curswant != 0 {
        update_curswant_force();
    }
}
#[no_mangle]
pub unsafe extern "C" fn check_cursor_moved(mut wp: *mut win_T) {
    if (*wp).w_cursor.lnum != (*wp).w_valid_cursor.lnum {
        (*wp).w_valid &=
            !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CHEIGHT | VALID_CROW | VALID_TOPLINE);
        if wp == curwin.get()
            && (*wp).w_valid_cursor.lnum > 0 as linenr_T
            && (*wp).w_onebuf_opt.wo_cole >= 2 as OptInt
            && !conceal_cursor_line(wp)
            && (decor_conceal_line(
                wp,
                (*wp).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                true_0 != 0,
            ) as ::core::ffi::c_int
                != 0
                || decor_conceal_line(
                    wp,
                    (*wp).w_valid_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    true_0 != 0,
                ) as ::core::ffi::c_int
                    != 0)
        {
            changed_window_setting(wp);
        }
        (*wp).w_valid_cursor = (*wp).w_cursor;
        (*wp).w_valid_leftcol = (*wp).w_leftcol;
        (*wp).w_valid_skipcol = (*wp).w_skipcol;
        (*wp).w_viewport_invalid = true_0 != 0;
    } else if (*wp).w_skipcol != (*wp).w_valid_skipcol {
        (*wp).w_valid &= !(VALID_WROW
            | VALID_WCOL
            | VALID_VIRTCOL
            | VALID_CHEIGHT
            | VALID_CROW
            | VALID_BOTLINE
            | VALID_BOTLINE_AP);
        (*wp).w_valid_cursor = (*wp).w_cursor;
        (*wp).w_valid_leftcol = (*wp).w_leftcol;
        (*wp).w_valid_skipcol = (*wp).w_skipcol;
    } else if (*wp).w_cursor.col != (*wp).w_valid_cursor.col
        || (*wp).w_leftcol != (*wp).w_valid_leftcol
        || (*wp).w_cursor.coladd != (*wp).w_valid_cursor.coladd
    {
        (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
        (*wp).w_valid_cursor.col = (*wp).w_cursor.col;
        (*wp).w_valid_leftcol = (*wp).w_leftcol;
        (*wp).w_valid_cursor.coladd = (*wp).w_cursor.coladd;
        (*wp).w_viewport_invalid = true_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn changed_window_setting(mut wp: *mut win_T) {
    (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
    changed_line_abv_curs_win(wp);
    (*wp).w_valid &= !(VALID_BOTLINE | VALID_BOTLINE_AP | VALID_TOPLINE);
    redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn changed_window_setting_all() {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            changed_window_setting(wp);
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn set_topline(mut wp: *mut win_T, mut lnum: linenr_T) {
    let mut prev_topline: linenr_T = (*wp).w_topline;
    hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
    (*wp).w_botline += lnum - (*wp).w_topline;
    if (*wp).w_botline > (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T {
        (*wp).w_botline = (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T;
    }
    (*wp).w_topline = lnum;
    (*wp).w_topline_was_set = true_0 as ::core::ffi::c_char;
    if lnum != prev_topline {
        (*wp).w_topfill = 0 as ::core::ffi::c_int;
    }
    (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_TOPLINE);
    redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn changed_cline_bef_curs(mut wp: *mut win_T) {
    (*wp).w_valid &=
        !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE);
}
#[no_mangle]
pub unsafe extern "C" fn changed_line_abv_curs() {
    (*curwin.get()).w_valid &=
        !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE);
}
#[no_mangle]
pub unsafe extern "C" fn changed_line_abv_curs_win(mut wp: *mut win_T) {
    (*wp).w_valid &=
        !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL | VALID_CROW | VALID_CHEIGHT | VALID_TOPLINE);
}
#[no_mangle]
pub unsafe extern "C" fn validate_botline_win(mut wp: *mut win_T) {
    if (*wp).w_valid & VALID_BOTLINE == 0 {
        comp_botline(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn invalidate_botline_win(mut wp: *mut win_T) {
    (*wp).w_valid &= !(VALID_BOTLINE | VALID_BOTLINE_AP);
}
#[no_mangle]
pub unsafe extern "C" fn approximate_botline_win(mut wp: *mut win_T) {
    (*wp).w_valid &= !VALID_BOTLINE;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_valid(mut wp: *mut win_T) -> ::core::ffi::c_int {
    check_cursor_moved(wp);
    return ((*wp).w_valid & (VALID_WROW | VALID_WCOL) == VALID_WROW | VALID_WCOL)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn validate_cursor(mut wp: *mut win_T) {
    check_cursor_lnum(wp);
    check_cursor_moved(wp);
    if (*wp).w_valid & (VALID_WCOL | VALID_WROW) != VALID_WCOL | VALID_WROW {
        curs_columns(wp, true_0);
    }
}
unsafe extern "C" fn curs_rows(mut wp: *mut win_T) {
    let mut all_invalid: bool = !redrawing()
        || (*wp).w_lines_valid == 0 as ::core::ffi::c_int
        || (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum > (*wp).w_topline;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*wp).w_cline_row = 0 as ::core::ffi::c_int;
    let mut lnum: linenr_T = (*wp).w_topline;
    's_111: while lnum < (*wp).w_cursor.lnum {
        let mut valid: bool = false_0 != 0;
        's_11: {
            if !all_invalid && i < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(i as isize)).wl_lnum < lnum
                    || !(*(*wp).w_lines.offset(i as isize)).wl_valid
                {
                    break 's_11;
                } else if (*(*wp).w_lines.offset(i as isize)).wl_lnum == lnum {
                    if !(*(*wp).w_buffer).b_mod_set
                        || (*(*wp).w_lines.offset(i as isize)).wl_lastlnum < (*wp).w_cursor.lnum
                        || (*(*wp).w_buffer).b_mod_top
                            > (*(*wp).w_lines.offset(i as isize)).wl_lastlnum + 1 as linenr_T
                    {
                        valid = true_0 != 0;
                    }
                } else if (*(*wp).w_lines.offset(i as isize)).wl_lnum > lnum {
                    i -= 1;
                }
            }
            if valid as ::core::ffi::c_int != 0
                && (lnum != (*wp).w_topline
                    || (*wp).w_skipcol == 0 as ::core::ffi::c_int && !win_may_fill(wp))
            {
                lnum = (*(*wp).w_lines.offset(i as isize)).wl_lastlnum + 1 as linenr_T;
                if lnum > (*wp).w_cursor.lnum {
                    break 's_111;
                }
                (*wp).w_cline_row +=
                    (*(*wp).w_lines.offset(i as isize)).wl_size as ::core::ffi::c_int;
            } else {
                let mut last: linenr_T = lnum;
                let mut folded: bool = false;
                let mut n: ::core::ffi::c_int =
                    plines_correct_topline(wp, lnum, &raw mut last, true_0 != 0, &raw mut folded);
                lnum = last + 1 as linenr_T;
                if lnum
                    + decor_conceal_line(
                        wp,
                        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        false_0 != 0,
                    ) as linenr_T
                    > (*wp).w_cursor.lnum
                {
                    break 's_111;
                }
                (*wp).w_cline_row += n;
            }
        }
        i += 1;
    }
    check_cursor_moved(wp);
    if (*wp).w_valid & VALID_CHEIGHT == 0 {
        if all_invalid as ::core::ffi::c_int != 0
            || i == (*wp).w_lines_valid
            || i < (*wp).w_lines_valid
                && (!(*(*wp).w_lines.offset(i as isize)).wl_valid
                    || (*(*wp).w_lines.offset(i as isize)).wl_lnum != (*wp).w_cursor.lnum)
        {
            (*wp).w_cline_height = plines_win_full(
                wp,
                (*wp).w_cursor.lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut (*wp).w_cline_folded,
                true_0 != 0,
                true_0 != 0,
            );
        } else if i > (*wp).w_lines_valid {
            (*wp).w_cline_height = 0 as ::core::ffi::c_int;
            (*wp).w_cline_folded = hasFolding(
                wp,
                (*wp).w_cursor.lnum,
                ::core::ptr::null_mut::<linenr_T>(),
                ::core::ptr::null_mut::<linenr_T>(),
            );
        } else {
            (*wp).w_cline_height =
                (*(*wp).w_lines.offset(i as isize)).wl_size as ::core::ffi::c_int;
            (*wp).w_cline_folded = (*(*wp).w_lines.offset(i as isize)).wl_folded;
        }
    }
    redraw_for_cursorline(wp);
    (*wp).w_valid |= VALID_CROW | VALID_CHEIGHT;
}
#[no_mangle]
pub unsafe extern "C" fn validate_virtcol(mut wp: *mut win_T) {
    check_cursor_moved(wp);
    if (*wp).w_valid & VALID_VIRTCOL != 0 {
        return;
    }
    getvvcol(
        wp,
        &raw mut (*wp).w_cursor,
        ::core::ptr::null_mut::<colnr_T>(),
        &raw mut (*wp).w_virtcol,
        ::core::ptr::null_mut::<colnr_T>(),
    );
    redraw_for_cursorcolumn(wp);
    (*wp).w_valid |= VALID_VIRTCOL;
}
#[no_mangle]
pub unsafe extern "C" fn validate_cheight(mut wp: *mut win_T) {
    check_cursor_moved(wp);
    if (*wp).w_valid & VALID_CHEIGHT != 0 {
        return;
    }
    (*wp).w_cline_height = plines_win_full(
        wp,
        (*wp).w_cursor.lnum,
        ::core::ptr::null_mut::<linenr_T>(),
        &raw mut (*wp).w_cline_folded,
        true_0 != 0,
        true_0 != 0,
    );
    (*wp).w_valid |= VALID_CHEIGHT;
}
#[no_mangle]
pub unsafe extern "C" fn validate_cursor_col(mut wp: *mut win_T) {
    validate_virtcol(wp);
    if (*wp).w_valid & VALID_WCOL != 0 {
        return;
    }
    let mut col: colnr_T = (*wp).w_virtcol;
    let mut off: colnr_T = win_col_off(wp);
    col += off;
    let mut width: ::core::ffi::c_int =
        (*wp).w_view_width - off as ::core::ffi::c_int + win_col_off2(wp);
    if (*wp).w_onebuf_opt.wo_wrap != 0
        && col >= (*wp).w_view_width
        && width > 0 as ::core::ffi::c_int
    {
        col -= ((col as ::core::ffi::c_int - (*wp).w_view_width) / width + 1 as ::core::ffi::c_int)
            * width;
    }
    if col > (*wp).w_leftcol {
        col -= (*wp).w_leftcol;
    } else {
        col = 0 as ::core::ffi::c_int as colnr_T;
    }
    (*wp).w_wcol = col as ::core::ffi::c_int;
    (*wp).w_valid |= VALID_WCOL;
}
#[no_mangle]
pub unsafe extern "C" fn win_col_off(mut wp: *mut win_T) -> ::core::ffi::c_int {
    return (if (*wp).w_onebuf_opt.wo_nu != 0
        || (*wp).w_onebuf_opt.wo_rnu != 0
        || *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
    {
        number_width(wp)
            + (*(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int == NUL) as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) + (if wp != cmdwin_win.get() {
        0 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    }) + win_fdccol_count(wp)
        + (*wp).w_scwidth * SIGN_WIDTH as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn win_col_off2(mut wp: *mut win_T) -> ::core::ffi::c_int {
    if ((*wp).w_onebuf_opt.wo_nu != 0
        || (*wp).w_onebuf_opt.wo_rnu != 0
        || *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL)
        && !vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null()
    {
        return number_width(wp)
            + (*(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int == NUL) as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn curs_columns(mut wp: *mut win_T, mut may_scroll: ::core::ffi::c_int) {
    let mut startcol: colnr_T = 0;
    let mut endcol: colnr_T = 0;
    update_topline(wp);
    if (*wp).w_valid & VALID_CROW == 0 {
        curs_rows(wp);
    }
    if (*wp).w_cline_folded {
        endcol = (*wp).w_leftcol;
        (*wp).w_virtcol = endcol;
        startcol = (*wp).w_virtcol;
    } else {
        getvvcol(
            wp,
            &raw mut (*wp).w_cursor,
            &raw mut startcol,
            &raw mut (*wp).w_virtcol,
            &raw mut endcol,
        );
    }
    if startcol > dollar_vcol.get() {
        dollar_vcol.set(-1 as ::core::ffi::c_int as colnr_T);
    }
    let mut extra: ::core::ffi::c_int = win_col_off(wp);
    (*wp).w_wcol = (*wp).w_virtcol as ::core::ffi::c_int + extra;
    endcol += extra;
    (*wp).w_wrow = (*wp).w_cline_row;
    let mut n: ::core::ffi::c_int = 0;
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - extra;
    let mut width2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut did_sub_skipcol: bool = false_0 != 0;
    if width1 <= 0 as ::core::ffi::c_int {
        (*wp).w_wcol = (*wp).w_view_width - 1 as ::core::ffi::c_int;
        if (*wp).w_onebuf_opt.wo_wrap != 0 {
            (*wp).w_wrow = (*wp).w_view_height - 1 as ::core::ffi::c_int;
        } else {
            (*wp).w_wrow = (*wp).w_view_height - 1 as ::core::ffi::c_int - (*wp).w_empty_rows;
        }
    } else if (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_view_width != 0 as ::core::ffi::c_int {
        width2 = width1 + win_col_off2(wp);
        if (*wp).w_cursor.lnum == (*wp).w_topline
            && (*wp).w_skipcol > 0 as ::core::ffi::c_int
            && (*wp).w_wcol >= (*wp).w_skipcol
        {
            if (*wp).w_skipcol <= width1 {
                (*wp).w_wcol -= width2;
            } else {
                (*wp).w_wcol -= width2
                    * (((*wp).w_skipcol as ::core::ffi::c_int - width1) / width2
                        + 1 as ::core::ffi::c_int);
            }
            did_sub_skipcol = true_0 != 0;
        }
        if (*wp).w_wcol >= (*wp).w_view_width {
            n = ((*wp).w_wcol - (*wp).w_view_width) / width2 + 1 as ::core::ffi::c_int;
            (*wp).w_wcol -= n * width2;
            (*wp).w_wrow += n;
        }
    } else if may_scroll != 0 && !(*wp).w_cline_folded {
        let mut siso: int64_t = get_sidescrolloff_value(wp);
        let mut off_left: int64_t = (startcol - (*wp).w_leftcol) as int64_t - siso;
        let mut off_right: int64_t = (endcol - (*wp).w_leftcol) as int64_t
            - ((*wp).w_view_width as int64_t - siso)
            + 1 as int64_t;
        if off_left < 0 as int64_t || off_right > 0 as int64_t {
            let mut diff: int64_t = if off_left < 0 as int64_t {
                -off_left
            } else {
                off_right
            };
            let mut new_leftcol: ::core::ffi::c_int = 0;
            if p_ss.get() == 0 as OptInt
                || diff >= (width1 / 2 as ::core::ffi::c_int) as int64_t
                || off_right >= off_left
            {
                new_leftcol = (*wp).w_wcol - extra - width1 / 2 as ::core::ffi::c_int;
            } else {
                if diff < p_ss.get() {
                    '_c2rust_label: {
                        if p_ss.get() <= 2147483647 as OptInt {
                        } else {
                            __assert_fail(
                                b"p_ss <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/move.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                903 as ::core::ffi::c_uint,
                                b"void curs_columns(win_T *, int)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    diff = p_ss.get() as int64_t;
                }
                if off_left < 0 as int64_t {
                    new_leftcol =
                        (*wp).w_leftcol as ::core::ffi::c_int - diff as ::core::ffi::c_int;
                } else {
                    new_leftcol =
                        (*wp).w_leftcol as ::core::ffi::c_int + diff as ::core::ffi::c_int;
                }
            }
            new_leftcol = if new_leftcol > 0 as ::core::ffi::c_int {
                new_leftcol
            } else {
                0 as ::core::ffi::c_int
            };
            if new_leftcol != (*wp).w_leftcol {
                (*wp).w_leftcol = new_leftcol as colnr_T;
                win_check_anchored_floats(wp);
                redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
            }
        }
        (*wp).w_wcol -= (*wp).w_leftcol as ::core::ffi::c_int;
    } else if (*wp).w_wcol > (*wp).w_leftcol {
        (*wp).w_wcol -= (*wp).w_leftcol as ::core::ffi::c_int;
    } else {
        (*wp).w_wcol = 0 as ::core::ffi::c_int;
    }
    if (*wp).w_cursor.lnum == (*wp).w_topline {
        (*wp).w_wrow += (*wp).w_topfill;
    } else {
        (*wp).w_wrow += win_get_fill(wp, (*wp).w_cursor.lnum);
    }
    let mut plines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut so: int64_t = get_scrolloff_value(wp);
    let mut prev_skipcol: colnr_T = (*wp).w_skipcol;
    if ((*wp).w_wrow >= (*wp).w_view_height
        || (prev_skipcol > 0 as ::core::ffi::c_int
            || (*wp).w_wrow as int64_t + so >= (*wp).w_view_height as int64_t)
            && {
                plines = plines_win_nofill(wp, (*wp).w_cursor.lnum, false_0 != 0);
                plines - 1 as ::core::ffi::c_int >= (*wp).w_view_height
            })
        && (*wp).w_view_height != 0 as ::core::ffi::c_int
        && (*wp).w_cursor.lnum == (*wp).w_topline
        && width2 > 0 as ::core::ffi::c_int
        && (*wp).w_view_width != 0 as ::core::ffi::c_int
    {
        extra = 0 as ::core::ffi::c_int;
        if (*wp).w_skipcol as int64_t + so * width2 as int64_t > (*wp).w_virtcol as int64_t {
            extra = 1 as ::core::ffi::c_int;
        }
        if plines == 0 as ::core::ffi::c_int {
            plines = plines_win(wp, (*wp).w_cursor.lnum, false_0 != 0);
        }
        plines -= 1;
        if plines as int64_t > (*wp).w_wrow as int64_t + so {
            '_c2rust_label_0: {
                if (*wp).w_wrow as int64_t + so <= 2147483647 as int64_t {
                } else {
                    __assert_fail(
                        b"wp->w_wrow + so <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/move.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        964 as ::core::ffi::c_uint,
                        b"void curs_columns(win_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            n = ((*wp).w_wrow as int64_t + so) as ::core::ffi::c_int;
        } else {
            n = plines;
        }
        if n as int64_t
            >= ((*wp).w_view_height + (*wp).w_skipcol as ::core::ffi::c_int / width2) as int64_t
                - so
        {
            extra += 2 as ::core::ffi::c_int;
        }
        if extra == 3 as ::core::ffi::c_int || (*wp).w_view_height as int64_t <= so * 2 as int64_t {
            n = (*wp).w_virtcol as ::core::ffi::c_int / width2;
            if n > (*wp).w_view_height / 2 as ::core::ffi::c_int {
                n -= (*wp).w_view_height / 2 as ::core::ffi::c_int;
            } else {
                n = 0 as ::core::ffi::c_int;
            }
            if n > plines - (*wp).w_view_height + 1 as ::core::ffi::c_int {
                n = plines - (*wp).w_view_height + 1 as ::core::ffi::c_int;
            }
            (*wp).w_skipcol = (if n > 0 as ::core::ffi::c_int {
                width1 + (n - 1 as ::core::ffi::c_int) * width2
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
        } else if extra == 1 as ::core::ffi::c_int {
            '_c2rust_label_1: {
                if so <= 2147483647 as int64_t {
                } else {
                    __assert_fail(
                        b"so <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/move.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        989 as ::core::ffi::c_uint,
                        b"void curs_columns(win_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            extra = (((*wp).w_skipcol as int64_t + so * width2 as int64_t
                - (*wp).w_virtcol as int64_t
                + width2 as int64_t
                - 1 as int64_t)
                / width2 as int64_t) as ::core::ffi::c_int;
            if extra > 0 as ::core::ffi::c_int {
                if extra * width2 > (*wp).w_skipcol {
                    extra = (*wp).w_skipcol as ::core::ffi::c_int / width2;
                }
                (*wp).w_skipcol -= extra * width2;
            }
        } else if extra == 2 as ::core::ffi::c_int {
            endcol = ((n - (*wp).w_view_height + 1 as ::core::ffi::c_int) * width2) as colnr_T;
            while endcol > (*wp).w_virtcol {
                endcol -= width2;
            }
            (*wp).w_skipcol = if (*wp).w_skipcol > endcol {
                (*wp).w_skipcol
            } else {
                endcol
            };
        }
        if did_sub_skipcol {
            (*wp).w_wrow -= ((*wp).w_skipcol as ::core::ffi::c_int
                - prev_skipcol as ::core::ffi::c_int)
                / width2;
        } else {
            (*wp).w_wrow -= (*wp).w_skipcol as ::core::ffi::c_int / width2;
        }
        if (*wp).w_wrow >= (*wp).w_view_height {
            extra = (*wp).w_wrow - (*wp).w_view_height + 1 as ::core::ffi::c_int;
            (*wp).w_skipcol += extra * width2;
            (*wp).w_wrow -= extra;
        }
        extra =
            (prev_skipcol as ::core::ffi::c_int - (*wp).w_skipcol as ::core::ffi::c_int) / width2;
        if !(*wp).w_grid.target.is_null() {
            win_scroll_lines(wp, 0 as ::core::ffi::c_int, extra);
        }
    } else if (*wp).w_onebuf_opt.wo_sms == 0 {
        (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
    }
    if prev_skipcol != (*wp).w_skipcol {
        redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
    }
    redraw_for_cursorcolumn(wp);
    (*wp).w_valid_leftcol = (*wp).w_leftcol;
    (*wp).w_valid_skipcol = (*wp).w_skipcol;
    (*wp).w_valid |= VALID_WCOL | VALID_WROW | VALID_VIRTCOL;
}
#[no_mangle]
pub unsafe extern "C" fn textpos2screenpos(
    mut wp: *mut win_T,
    mut pos: *mut pos_T,
    mut rowp: *mut ::core::ffi::c_int,
    mut scolp: *mut ::core::ffi::c_int,
    mut ccolp: *mut ::core::ffi::c_int,
    mut ecolp: *mut ::core::ffi::c_int,
    mut local: bool,
) {
    let mut scol: colnr_T = 0 as colnr_T;
    let mut ccol: colnr_T = 0 as colnr_T;
    let mut ecol: colnr_T = 0 as colnr_T;
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut coloff: colnr_T = 0 as colnr_T;
    let mut visible_row: bool = false_0 != 0;
    let mut is_folded: bool = false_0 != 0;
    let mut lnum: linenr_T = (*pos).lnum;
    if lnum >= (*wp).w_topline && lnum <= (*wp).w_botline {
        is_folded = hasFolding(wp, lnum, &raw mut lnum, ::core::ptr::null_mut::<linenr_T>());
        row = plines_m_win(wp, (*wp).w_topline, lnum - 1 as linenr_T, INT_MAX);
        row -= adjust_plines_for_skipcol(wp);
        row += if lnum == (*wp).w_topline {
            (*wp).w_topfill
        } else {
            win_get_fill(wp, lnum)
        };
        visible_row = true_0 != 0;
    } else if !local || lnum < (*wp).w_topline {
        row = 0 as ::core::ffi::c_int;
    } else {
        row = (*wp).w_view_height - 1 as ::core::ffi::c_int;
    }
    let mut existing_row: bool =
        lnum > 0 as linenr_T && lnum <= (*(*wp).w_buffer).b_ml.ml_line_count;
    if (local as ::core::ffi::c_int != 0 || visible_row as ::core::ffi::c_int != 0)
        && existing_row as ::core::ffi::c_int != 0
    {
        let off: colnr_T = win_col_off(wp);
        if is_folded {
            row += (if local as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                (*wp).w_winrow + (*wp).w_winrow_off
            }) + 1 as ::core::ffi::c_int;
            coloff = (if local as ::core::ffi::c_int != 0 {
                0 as colnr_T
            } else {
                (*wp).w_wincol as colnr_T + (*wp).w_wincol_off as colnr_T
            }) + 1 as colnr_T
                + off;
        } else {
            '_c2rust_label: {
                if lnum == (*pos).lnum {
                } else {
                    __assert_fail(
                        b"lnum == pos->lnum\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/move.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1087 as ::core::ffi::c_uint,
                        b"void textpos2screenpos(win_T *, pos_T *, int *, int *, int *, int *, _Bool)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            getvcol(wp, pos, &raw mut scol, &raw mut ccol, &raw mut ecol);
            let mut col: colnr_T = scol;
            col += off;
            let mut width: ::core::ffi::c_int =
                (*wp).w_view_width - off as ::core::ffi::c_int + win_col_off2(wp);
            if (*wp).w_onebuf_opt.wo_wrap != 0
                && col >= (*wp).w_view_width
                && width > 0 as ::core::ffi::c_int
            {
                let mut rowoff: ::core::ffi::c_int = if visible_row as ::core::ffi::c_int != 0 {
                    (col as ::core::ffi::c_int - (*wp).w_view_width) / width
                        + 1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                col -= rowoff * width;
                row += rowoff;
            }
            col -= (*wp).w_leftcol;
            if col >= 0 as ::core::ffi::c_int
                && col < (*wp).w_view_width
                && row >= 0 as ::core::ffi::c_int
                && row < (*wp).w_view_height
            {
                coloff = (col as ::core::ffi::c_int - scol as ::core::ffi::c_int
                    + (if local as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        (*wp).w_wincol + (*wp).w_wincol_off
                    })
                    + 1 as ::core::ffi::c_int) as colnr_T;
                row += (if local as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    (*wp).w_winrow + (*wp).w_winrow_off
                }) + 1 as ::core::ffi::c_int;
            } else {
                ecol = 0 as ::core::ffi::c_int as colnr_T;
                ccol = ecol;
                scol = ccol;
                if local {
                    coloff = (if col < 0 as ::core::ffi::c_int {
                        -1 as ::core::ffi::c_int
                    } else {
                        (*wp).w_view_width + 1 as ::core::ffi::c_int
                    }) as colnr_T;
                } else {
                    row = 0 as ::core::ffi::c_int;
                }
            }
        }
    }
    *rowp = row;
    *scolp = (scol + coloff) as ::core::ffi::c_int;
    *ccolp = (ccol + coloff) as ::core::ffi::c_int;
    *ecolp = (ecol + coloff) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn f_screenpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let mut dict: *mut dict_T = (*rettv).vval.v_dict;
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        return;
    }
    let mut pos: pos_T = pos_T {
        lnum: tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as linenr_T,
        col: tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) as colnr_T
            - 1 as colnr_T,
        coladd: 0 as colnr_T,
    };
    if pos.lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
        semsg(
            gettext(&raw const e_invalid_line_number_nr as *const ::core::ffi::c_char),
            pos.lnum,
        );
        return;
    }
    pos.col = (if pos.col > 0 as ::core::ffi::c_int {
        pos.col as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as colnr_T;
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut scol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ccol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ecol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    textpos2screenpos(
        wp,
        &raw mut pos,
        &raw mut row,
        &raw mut scol,
        &raw mut ccol,
        &raw mut ecol,
        false_0 != 0,
    );
    tv_dict_add_nr(
        dict,
        b"row\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        row as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"col\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        scol as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"curscol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        ccol as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"endcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        ecol as varnumber_T,
    );
}
unsafe extern "C" fn virtcol2col(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut vcol: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut offset: ::core::ffi::c_int = vcol2col(
        wp,
        lnum,
        vcol as colnr_T - 1 as colnr_T,
        ::core::ptr::null_mut::<colnr_T>(),
    );
    let mut line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
    let mut p: *mut ::core::ffi::c_char = line.offset(offset as isize);
    if *p as ::core::ffi::c_int == NUL {
        if p == line {
            return 0 as ::core::ffi::c_int;
        }
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
    }
    return (p.offset_from(line) + 1 as isize) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn f_virtcol2col(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    if tv_check_for_number_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_number_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        return;
    }
    let mut error: bool = false_0 != 0;
    let mut lnum: linenr_T = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut error,
    ) as linenr_T;
    if error as ::core::ffi::c_int != 0
        || lnum < 0 as linenr_T
        || lnum > (*(*wp).w_buffer).b_ml.ml_line_count
    {
        return;
    }
    let mut screencol: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(2 as ::core::ffi::c_int as isize),
        &raw mut error,
    ) as ::core::ffi::c_int;
    if error as ::core::ffi::c_int != 0 || screencol < 0 as ::core::ffi::c_int {
        return;
    }
    (*rettv).vval.v_number = virtcol2col(wp, lnum, screencol) as varnumber_T;
}
unsafe extern "C" fn cursor_correct_sms(mut wp: *mut win_T) {
    if (*wp).w_onebuf_opt.wo_sms == 0
        || (*wp).w_onebuf_opt.wo_wrap == 0
        || (*wp).w_cursor.lnum != (*wp).w_topline
    {
        return;
    }
    let mut so: int64_t = get_scrolloff_value(wp);
    let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
    let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
    let mut so_cols: int64_t = if so == 0 as int64_t {
        0 as int64_t
    } else {
        width1 as int64_t + (so - 1 as int64_t) * width2 as int64_t
    };
    let mut space_cols: ::core::ffi::c_int =
        ((*wp).w_view_height - 1 as ::core::ffi::c_int) * width2;
    let mut size: ::core::ffi::c_int = if so == 0 as int64_t {
        0 as ::core::ffi::c_int
    } else {
        linetabsize_eol(wp, (*wp).w_topline)
    };
    if (*wp).w_topline == 1 as linenr_T && (*wp).w_skipcol == 0 as ::core::ffi::c_int {
        so_cols = 0 as int64_t;
    } else if so_cols > (space_cols / 2 as ::core::ffi::c_int) as int64_t {
        so_cols = (space_cols / 2 as ::core::ffi::c_int) as int64_t;
    }
    while so_cols > size as int64_t
        && so_cols - width2 as int64_t >= width1 as int64_t
        && width1 > 0 as ::core::ffi::c_int
    {
        so_cols -= width2 as int64_t;
    }
    if so_cols >= width1 as int64_t && so_cols > size as int64_t {
        so_cols -= width1 as int64_t;
    }
    let mut overlap: ::core::ffi::c_int = if (*wp).w_skipcol == 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        sms_marker_overlap(wp, (*wp).w_view_width - width2)
    };
    let mut top: int64_t = (*wp).w_skipcol as int64_t
        + (if so_cols != 0 as int64_t {
            so_cols
        } else {
            overlap as int64_t
        });
    let mut bot: int64_t = ((*wp).w_skipcol as ::core::ffi::c_int
        + width1
        + ((*wp).w_view_height - 1 as ::core::ffi::c_int) * width2)
        as int64_t
        - so_cols;
    validate_virtcol(wp);
    let mut col: colnr_T = (*wp).w_virtcol;
    if (col as int64_t) < top {
        if col < width1 {
            col += width1;
        }
        while width2 > 0 as ::core::ffi::c_int && (col as int64_t) < top {
            col += width2;
        }
    } else {
        while width2 > 0 as ::core::ffi::c_int && col as int64_t >= bot {
            col -= width2;
        }
    }
    if col != (*wp).w_virtcol {
        (*wp).w_curswant = col;
        let mut rc: ::core::ffi::c_int = coladvance(wp, (*wp).w_curswant);
        (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL);
        if rc == FAIL
            && (*wp).w_skipcol > 0 as ::core::ffi::c_int
            && (*wp).w_cursor.lnum < (*(*wp).w_buffer).b_ml.ml_line_count
        {
            validate_virtcol(wp);
            if (*wp).w_virtcol < (*wp).w_skipcol as ::core::ffi::c_int + overlap {
                (*wp).w_cursor.lnum += 1;
                (*wp).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                (*wp).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                (*wp).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
                (*wp).w_valid &= !VALID_VIRTCOL;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn scroll_redraw(mut up: ::core::ffi::c_int, mut count: linenr_T) {
    let mut prev_topline: linenr_T = (*curwin.get()).w_topline;
    let mut prev_skipcol: ::core::ffi::c_int = (*curwin.get()).w_skipcol as ::core::ffi::c_int;
    let mut prev_topfill: ::core::ffi::c_int = (*curwin.get()).w_topfill;
    let mut prev_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut moved: bool = if up != 0 {
        scrollup(curwin.get(), count, true_0 != 0) as ::core::ffi::c_int
    } else {
        scrolldown(curwin.get(), count, true_0) as ::core::ffi::c_int
    } != 0;
    if get_scrolloff_value(curwin.get()) > 0 as int64_t {
        cursor_correct(curwin.get());
        check_cursor_moved(curwin.get());
        (*curwin.get()).w_valid |= VALID_TOPLINE;
        while (*curwin.get()).w_topline == prev_topline
            && (*curwin.get()).w_skipcol == prev_skipcol
            && (*curwin.get()).w_topfill == prev_topfill
        {
            if up != 0 {
                if (*curwin.get()).w_cursor.lnum > prev_lnum
                    || cursor_down(1 as ::core::ffi::c_int, false_0 != 0) == FAIL
                {
                    break;
                }
            } else if (*curwin.get()).w_cursor.lnum < prev_lnum
                || prev_topline as ::core::ffi::c_long == 1 as ::core::ffi::c_long
                || cursor_up(1 as linenr_T, false_0 != 0) == FAIL
            {
                break;
            }
            check_cursor_moved(curwin.get());
            (*curwin.get()).w_valid |= VALID_TOPLINE;
        }
    }
    if moved {
        (*curwin.get()).w_viewport_invalid = true_0 != 0;
    }
    cursor_correct_sms(curwin.get());
    if (*curwin.get()).w_cursor.lnum != prev_lnum {
        coladvance(curwin.get(), (*curwin.get()).w_curswant);
    }
    redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn scrolldown(
    mut wp: *mut win_T,
    mut line_count: linenr_T,
    mut byfold: ::core::ffi::c_int,
) -> bool {
    let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut width1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut width2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
    if do_sms {
        width1 = (*wp).w_view_width - win_col_off(wp);
        width2 = width1 + win_col_off2(wp);
    }
    hasFolding(
        wp,
        (*wp).w_topline,
        &raw mut (*wp).w_topline,
        ::core::ptr::null_mut::<linenr_T>(),
    );
    validate_cursor(wp);
    let mut todo: ::core::ffi::c_int = line_count as ::core::ffi::c_int;
    while todo > 0 as ::core::ffi::c_int {
        let mut can_fill: bool = (*wp).w_topfill < (*wp).w_view_height - 1 as ::core::ffi::c_int
            && (*wp).w_topfill < win_get_fill(wp, (*wp).w_topline);
        if (*wp).w_topline == 1 as linenr_T && !can_fill && (!do_sms || (*wp).w_skipcol < width1) {
            break;
        }
        if do_sms as ::core::ffi::c_int != 0 && (*wp).w_skipcol >= width1 {
            if (*wp).w_skipcol >= width1 + width2 {
                (*wp).w_skipcol -= width2;
            } else {
                (*wp).w_skipcol -= width1;
            }
            redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
            done += 1;
        } else if can_fill {
            (*wp).w_topfill += 1;
            done += 1;
        } else {
            (*wp).w_topline -= 1;
            (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
            (*wp).w_topfill = 0 as ::core::ffi::c_int;
            let mut first: linenr_T = 0;
            if hasFolding(
                wp,
                (*wp).w_topline,
                &raw mut first,
                ::core::ptr::null_mut::<linenr_T>(),
            ) {
                done += !decor_conceal_line(
                    wp,
                    first as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    false_0 != 0,
                ) as ::core::ffi::c_int;
                if byfold == 0 {
                    todo -= ((*wp).w_topline - first - 1 as linenr_T) as ::core::ffi::c_int;
                }
                (*wp).w_botline -= (*wp).w_topline - first;
                (*wp).w_topline = first;
            } else if decor_conceal_line(
                wp,
                (*wp).w_topline as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) {
                todo += 1;
            } else if do_sms {
                let mut size: ::core::ffi::c_int = linetabsize_eol(wp, (*wp).w_topline);
                if size > width1 {
                    (*wp).w_skipcol = width1 as colnr_T;
                    size -= width1;
                    redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
                }
                while size > width2 {
                    (*wp).w_skipcol += width2;
                    size -= width2;
                }
                done += 1;
            } else {
                done += plines_win_nofill(wp, (*wp).w_topline, true_0 != 0);
            }
        }
        (*wp).w_botline -= 1;
        invalidate_botline_win(wp);
        todo -= 1;
    }
    while (*wp).w_topline > 1 as linenr_T
        && decor_conceal_line(
            wp,
            (*wp).w_topline as ::core::ffi::c_int - 2 as ::core::ffi::c_int,
            false_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        (*wp).w_topline -= 1;
        hasFolding(
            wp,
            (*wp).w_topline,
            &raw mut (*wp).w_topline,
            ::core::ptr::null_mut::<linenr_T>(),
        );
    }
    (*wp).w_wrow += done;
    (*wp).w_cline_row += done;
    if (*wp).w_cursor.lnum == (*wp).w_topline {
        (*wp).w_cline_row = 0 as ::core::ffi::c_int;
    }
    check_topfill(wp, true_0 != 0);
    let mut wrow: ::core::ffi::c_int = (*wp).w_wrow;
    if (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_view_width != 0 as ::core::ffi::c_int {
        validate_virtcol(wp);
        validate_cheight(wp);
        wrow += (*wp).w_cline_height
            - 1 as ::core::ffi::c_int
            - (*wp).w_virtcol as ::core::ffi::c_int / (*wp).w_view_width;
    }
    let mut moved: bool = false_0 != 0;
    while wrow >= (*wp).w_view_height && (*wp).w_cursor.lnum > 1 as linenr_T {
        let mut first_0: linenr_T = 0;
        if hasFolding(
            wp,
            (*wp).w_cursor.lnum,
            &raw mut first_0,
            ::core::ptr::null_mut::<linenr_T>(),
        ) {
            wrow -= !decor_conceal_line(
                wp,
                (*wp).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int;
            (*wp).w_cursor.lnum = if first_0 - 1 as linenr_T > 1 as linenr_T {
                first_0 - 1 as linenr_T
            } else {
                1 as linenr_T
            };
        } else {
            let c2rust_fresh0 = (*wp).w_cursor.lnum;
            (*wp).w_cursor.lnum = (*wp).w_cursor.lnum - 1;
            wrow -= plines_win(wp, c2rust_fresh0, true_0 != 0);
        }
        (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL);
        moved = true_0 != 0;
    }
    if moved {
        foldAdjustCursor(wp);
        coladvance(wp, (*wp).w_curswant);
    }
    (*wp).w_cursor.lnum = if (*wp).w_cursor.lnum > (*wp).w_topline {
        (*wp).w_cursor.lnum
    } else {
        (*wp).w_topline
    };
    return moved;
}
#[no_mangle]
pub unsafe extern "C" fn scrollup(
    mut wp: *mut win_T,
    mut line_count: linenr_T,
    mut byfold: bool,
) -> bool {
    let mut topline: linenr_T = (*wp).w_topline;
    let mut botline: linenr_T = (*wp).w_botline;
    let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
    if do_sms as ::core::ffi::c_int != 0
        || byfold as ::core::ffi::c_int != 0 && win_lines_concealed(wp) as ::core::ffi::c_int != 0
        || win_may_fill(wp) as ::core::ffi::c_int != 0
    {
        let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
        let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let prev_skipcol: colnr_T = (*wp).w_skipcol;
        if do_sms {
            size = linetabsize_eol(wp, (*wp).w_topline);
        }
        let mut todo: ::core::ffi::c_int = line_count as ::core::ffi::c_int;
        while todo > 0 as ::core::ffi::c_int {
            todo += decor_conceal_line(
                wp,
                (*wp).w_topline as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int;
            if (*wp).w_topfill > 0 as ::core::ffi::c_int {
                (*wp).w_topfill -= 1;
            } else {
                let mut lnum: linenr_T = (*wp).w_topline;
                if byfold {
                    hasFolding(wp, lnum, ::core::ptr::null_mut::<linenr_T>(), &raw mut lnum);
                }
                if lnum == (*wp).w_topline && do_sms as ::core::ffi::c_int != 0 {
                    let mut add: ::core::ffi::c_int = if (*wp).w_skipcol > 0 as ::core::ffi::c_int {
                        width2
                    } else {
                        width1
                    };
                    (*wp).w_skipcol += add;
                    if (*wp).w_skipcol >= size {
                        if lnum == (*(*wp).w_buffer).b_ml.ml_line_count {
                            (*wp).w_skipcol -= add;
                            break;
                        } else {
                            lnum += 1;
                        }
                    }
                } else {
                    if lnum >= (*(*wp).w_buffer).b_ml.ml_line_count {
                        break;
                    }
                    lnum += 1;
                }
                if lnum > (*wp).w_topline {
                    (*wp).w_botline += lnum - (*wp).w_topline;
                    (*wp).w_topline = lnum;
                    (*wp).w_topfill = win_get_fill(wp, lnum);
                    (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
                    if todo > 1 as ::core::ffi::c_int && do_sms as ::core::ffi::c_int != 0 {
                        size = linetabsize_eol(wp, (*wp).w_topline);
                    }
                }
            }
            todo -= 1;
        }
        if prev_skipcol > 0 as ::core::ffi::c_int || (*wp).w_skipcol > 0 as ::core::ffi::c_int {
            redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
        }
    } else {
        (*wp).w_topline += line_count;
        (*wp).w_botline += line_count;
    }
    (*wp).w_topline = if (*wp).w_topline < (*(*wp).w_buffer).b_ml.ml_line_count {
        (*wp).w_topline
    } else {
        (*(*wp).w_buffer).b_ml.ml_line_count
    };
    (*wp).w_botline = if (*wp).w_botline < (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T {
        (*wp).w_botline
    } else {
        (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T
    };
    check_topfill(wp, false_0 != 0);
    hasFolding(
        wp,
        (*wp).w_topline,
        &raw mut (*wp).w_topline,
        ::core::ptr::null_mut::<linenr_T>(),
    );
    (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE);
    if (*wp).w_cursor.lnum < (*wp).w_topline {
        (*wp).w_cursor.lnum = (*wp).w_topline;
        (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW | VALID_VIRTCOL);
        coladvance(wp, (*wp).w_curswant);
    }
    let mut moved: bool = topline != (*wp).w_topline || botline != (*wp).w_botline;
    return moved;
}
#[no_mangle]
pub unsafe extern "C" fn adjust_skipcol() {
    if (*curwin.get()).w_onebuf_opt.wo_wrap == 0
        || (*curwin.get()).w_onebuf_opt.wo_sms == 0
        || (*curwin.get()).w_cursor.lnum != (*curwin.get()).w_topline
    {
        return;
    }
    let mut width1: ::core::ffi::c_int = (*curwin.get()).w_view_width - win_col_off(curwin.get());
    if width1 <= 0 as ::core::ffi::c_int {
        return;
    }
    let mut width2: ::core::ffi::c_int = width1 + win_col_off2(curwin.get());
    let mut so: int64_t = get_scrolloff_value(curwin.get());
    let mut scrolloff_cols: int64_t = if so == 0 as int64_t {
        0 as int64_t
    } else {
        width1 as int64_t + (so - 1 as int64_t) * width2 as int64_t
    };
    let mut scrolled: bool = false_0 != 0;
    validate_cheight(curwin.get());
    if (*curwin.get()).w_cline_height == (*curwin.get()).w_view_height
        && plines_win(curwin.get(), (*curwin.get()).w_cursor.lnum, false_0 != 0)
            <= (*curwin.get()).w_view_height
    {
        reset_skipcol(curwin.get());
        return;
    }
    validate_virtcol(curwin.get());
    let mut overlap: ::core::ffi::c_int =
        sms_marker_overlap(curwin.get(), (*curwin.get()).w_view_width - width2);
    while (*curwin.get()).w_skipcol > 0 as ::core::ffi::c_int
        && ((*curwin.get()).w_virtcol as int64_t)
            < ((*curwin.get()).w_skipcol as ::core::ffi::c_int + overlap) as int64_t
                + scrolloff_cols
    {
        if (*curwin.get()).w_skipcol >= width1 + width2 {
            (*curwin.get()).w_skipcol -= width2;
        } else {
            (*curwin.get()).w_skipcol -= width1;
        }
        scrolled = true_0 != 0;
    }
    if scrolled {
        validate_virtcol(curwin.get());
        redraw_later(curwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
        return;
    }
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut col: int64_t = (*curwin.get()).w_virtcol as int64_t + scrolloff_cols;
    if scrolloff_cols > 0 as int64_t {
        let mut size: ::core::ffi::c_int = linetabsize_eol(curwin.get(), (*curwin.get()).w_topline);
        size = width1 + width2 * ((size - width1 + width2 - 1 as ::core::ffi::c_int) / width2);
        while col > size as int64_t {
            col -= width2 as int64_t;
        }
    }
    col -= (*curwin.get()).w_skipcol as int64_t;
    if col >= width1 as int64_t {
        col -= width1 as int64_t;
        row += 1;
    }
    if col > width2 as int64_t {
        row += (col / width2 as int64_t) as ::core::ffi::c_int;
    }
    if row >= (*curwin.get()).w_view_height {
        if (*curwin.get()).w_skipcol == 0 as ::core::ffi::c_int {
            (*curwin.get()).w_skipcol += width1;
            row -= 1;
        }
        if row >= (*curwin.get()).w_view_height {
            (*curwin.get()).w_skipcol += (row - (*curwin.get()).w_view_height) * width2;
        }
        redraw_later(curwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn check_topfill(mut wp: *mut win_T, mut down: bool) {
    if (*wp).w_topfill > 0 as ::core::ffi::c_int {
        let mut n: ::core::ffi::c_int = plines_win_nofill(wp, (*wp).w_topline, true_0 != 0);
        if (*wp).w_topfill + n > (*wp).w_view_height {
            if down as ::core::ffi::c_int != 0 && (*wp).w_topline > 1 as linenr_T {
                (*wp).w_topline -= 1;
                (*wp).w_topfill = 0 as ::core::ffi::c_int;
            } else {
                (*wp).w_topfill = (*wp).w_view_height - n;
                (*wp).w_topfill = if (*wp).w_topfill > 0 as ::core::ffi::c_int {
                    (*wp).w_topfill
                } else {
                    0 as ::core::ffi::c_int
                };
            }
        }
    }
    win_check_anchored_floats(wp);
}
#[no_mangle]
pub unsafe extern "C" fn scrolldown_clamp() {
    let mut can_fill: bool =
        (*curwin.get()).w_topfill < win_get_fill(curwin.get(), (*curwin.get()).w_topline);
    if (*curwin.get()).w_topline <= 1 as linenr_T && !can_fill {
        return;
    }
    validate_cursor(curwin.get());
    let mut end_row: ::core::ffi::c_int = (*curwin.get()).w_wrow;
    if can_fill {
        end_row += 1;
    } else {
        end_row += plines_win_nofill(
            curwin.get(),
            (*curwin.get()).w_topline - 1 as linenr_T,
            true_0 != 0,
        );
    }
    if (*curwin.get()).w_onebuf_opt.wo_wrap != 0
        && (*curwin.get()).w_view_width != 0 as ::core::ffi::c_int
    {
        validate_cheight(curwin.get());
        validate_virtcol(curwin.get());
        end_row += (*curwin.get()).w_cline_height
            - 1 as ::core::ffi::c_int
            - (*curwin.get()).w_virtcol as ::core::ffi::c_int / (*curwin.get()).w_view_width;
    }
    if (end_row as int64_t)
        < (*curwin.get()).w_view_height as int64_t - get_scrolloff_value(curwin.get())
    {
        if can_fill {
            (*curwin.get()).w_topfill += 1;
            check_topfill(curwin.get(), true_0 != 0);
        } else {
            (*curwin.get()).w_topline -= 1;
            (*curwin.get()).w_topfill = 0 as ::core::ffi::c_int;
        }
        hasFolding(
            curwin.get(),
            (*curwin.get()).w_topline,
            &raw mut (*curwin.get()).w_topline,
            ::core::ptr::null_mut::<linenr_T>(),
        );
        (*curwin.get()).w_botline -= 1;
        (*curwin.get()).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE);
    }
}
#[no_mangle]
pub unsafe extern "C" fn scrollup_clamp() {
    if (*curwin.get()).w_topline == (*curbuf.get()).b_ml.ml_line_count
        && (*curwin.get()).w_topfill == 0 as ::core::ffi::c_int
    {
        return;
    }
    validate_cursor(curwin.get());
    let mut start_row: ::core::ffi::c_int = (*curwin.get()).w_wrow
        - plines_win_nofill(curwin.get(), (*curwin.get()).w_topline, true_0 != 0)
        - (*curwin.get()).w_topfill;
    if (*curwin.get()).w_onebuf_opt.wo_wrap != 0
        && (*curwin.get()).w_view_width != 0 as ::core::ffi::c_int
    {
        validate_virtcol(curwin.get());
        start_row -= (*curwin.get()).w_virtcol as ::core::ffi::c_int / (*curwin.get()).w_view_width;
    }
    if start_row as int64_t >= get_scrolloff_value(curwin.get()) {
        if (*curwin.get()).w_topfill > 0 as ::core::ffi::c_int {
            (*curwin.get()).w_topfill -= 1;
        } else {
            hasFolding(
                curwin.get(),
                (*curwin.get()).w_topline,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut (*curwin.get()).w_topline,
            );
            (*curwin.get()).w_topline += 1;
        }
        (*curwin.get()).w_botline += 1;
        (*curwin.get()).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE);
    }
}
unsafe extern "C" fn topline_back_winheight(
    mut wp: *mut win_T,
    mut lp: *mut lineoff_T,
    mut winheight: ::core::ffi::c_int,
) {
    if (*lp).fill < win_get_fill(wp, (*lp).lnum) {
        (*lp).fill += 1;
        (*lp).height = 1 as ::core::ffi::c_int;
    } else {
        (*lp).lnum -= 1;
        (*lp).fill = 0 as ::core::ffi::c_int;
        if (*lp).lnum < 1 as linenr_T {
            (*lp).height = MAXCOL as ::core::ffi::c_int;
        } else if hasFolding(
            wp,
            (*lp).lnum,
            &raw mut (*lp).lnum,
            ::core::ptr::null_mut::<linenr_T>(),
        ) {
            (*lp).height = !decor_conceal_line(
                wp,
                (*lp).lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int;
        } else {
            (*lp).height = plines_win_nofill(wp, (*lp).lnum, winheight != 0);
        }
    };
}
unsafe extern "C" fn topline_back(mut wp: *mut win_T, mut lp: *mut lineoff_T) {
    topline_back_winheight(wp, lp, true_0);
}
unsafe extern "C" fn botline_forw(mut wp: *mut win_T, mut lp: *mut lineoff_T) {
    if (*lp).fill < win_get_fill(wp, (*lp).lnum + 1 as linenr_T) {
        (*lp).fill += 1;
        (*lp).height = 1 as ::core::ffi::c_int;
    } else {
        (*lp).lnum += 1;
        (*lp).fill = 0 as ::core::ffi::c_int;
        '_c2rust_label: {
            if !(*wp).w_buffer.is_null() {
            } else {
                __assert_fail(
                    b"wp->w_buffer != 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/move.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1768 as ::core::ffi::c_uint,
                    b"void botline_forw(win_T *, lineoff_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if (*lp).lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
            (*lp).height = MAXCOL as ::core::ffi::c_int;
        } else if hasFolding(
            wp,
            (*lp).lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*lp).lnum,
        ) {
            (*lp).height = !decor_conceal_line(
                wp,
                (*lp).lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int;
        } else {
            (*lp).height = plines_win_nofill(wp, (*lp).lnum, true_0 != 0);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn scroll_cursor_top(
    mut wp: *mut win_T,
    mut min_scroll: ::core::ffi::c_int,
    mut always: ::core::ffi::c_int,
) {
    let mut old_topline: linenr_T = (*wp).w_topline;
    let mut old_skipcol: ::core::ffi::c_int = (*wp).w_skipcol as ::core::ffi::c_int;
    let mut old_topfill: linenr_T = (*wp).w_topfill as linenr_T;
    let mut off: int64_t = get_scrolloff_value(wp);
    if mouse_dragging.get() > 0 as ::core::ffi::c_int {
        off = (mouse_dragging.get() - 1 as ::core::ffi::c_int) as int64_t;
    }
    validate_cheight(wp);
    let mut scrolled: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut used: ::core::ffi::c_int = (*wp).w_cline_height;
    if (*wp).w_cursor.lnum < (*wp).w_topline {
        scrolled = used;
    }
    let mut top: linenr_T = 0;
    let mut bot: linenr_T = 0;
    if hasFolding(wp, (*wp).w_cursor.lnum, &raw mut top, &raw mut bot) {
        top -= 1;
        bot += 1;
    } else {
        top = (*wp).w_cursor.lnum - 1 as linenr_T;
        bot = (*wp).w_cursor.lnum + 1 as linenr_T;
    }
    let mut new_topline: linenr_T = top + 1 as linenr_T;
    let mut extra: ::core::ffi::c_int = win_get_fill(wp, (*wp).w_cursor.lnum);
    while top > 0 as linenr_T {
        let mut i: ::core::ffi::c_int = plines_win_nofill(wp, top, true_0 != 0);
        hasFolding(wp, top, &raw mut top, ::core::ptr::null_mut::<linenr_T>());
        if top < (*wp).w_topline {
            scrolled += i;
        }
        if (new_topline >= (*wp).w_topline || scrolled > min_scroll) && extra as int64_t >= off {
            break;
        }
        used += i;
        if (extra + i) as int64_t <= off && bot < (*(*wp).w_buffer).b_ml.ml_line_count {
            used += plines_win_full(
                wp,
                bot,
                &raw mut bot,
                ::core::ptr::null_mut::<bool>(),
                true_0 != 0,
                true_0 != 0,
            );
        }
        if used > (*wp).w_view_height {
            break;
        }
        extra += i;
        new_topline = top;
        top -= 1;
        bot += 1;
    }
    if used > (*wp).w_view_height {
        scroll_cursor_halfway(wp, false_0 != 0, false_0 != 0);
    } else {
        if new_topline < (*wp).w_topline || always != 0 {
            (*wp).w_topline = new_topline;
        }
        (*wp).w_topline = if (*wp).w_topline < (*wp).w_cursor.lnum {
            (*wp).w_topline
        } else {
            (*wp).w_cursor.lnum
        };
        (*wp).w_topfill = win_get_fill(wp, (*wp).w_topline);
        if (*wp).w_topfill > 0 as ::core::ffi::c_int && extra as int64_t > off {
            (*wp).w_topfill -= extra - off as ::core::ffi::c_int;
            (*wp).w_topfill = if (*wp).w_topfill > 0 as ::core::ffi::c_int {
                (*wp).w_topfill
            } else {
                0 as ::core::ffi::c_int
            };
        }
        check_topfill(wp, false_0 != 0);
        if (*wp).w_topline != old_topline {
            reset_skipcol(wp);
        } else if (*wp).w_topline == (*wp).w_cursor.lnum {
            validate_virtcol(wp);
            if (*wp).w_skipcol >= (*wp).w_virtcol {
                reset_skipcol(wp);
            }
        }
        if (*wp).w_topline != old_topline
            || (*wp).w_skipcol != old_skipcol
            || (*wp).w_topfill as linenr_T != old_topfill
        {
            (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
        }
        (*wp).w_valid |= VALID_TOPLINE;
        (*wp).w_viewport_invalid = true_0 != 0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn set_empty_rows(mut wp: *mut win_T, mut used: ::core::ffi::c_int) {
    (*wp).w_filler_rows = 0 as ::core::ffi::c_int;
    if used == 0 as ::core::ffi::c_int {
        (*wp).w_empty_rows = 0 as ::core::ffi::c_int;
    } else {
        (*wp).w_empty_rows = (*wp).w_view_height - used;
        if (*wp).w_botline <= (*(*wp).w_buffer).b_ml.ml_line_count {
            (*wp).w_filler_rows = win_get_fill(wp, (*wp).w_botline);
            if (*wp).w_empty_rows > (*wp).w_filler_rows {
                (*wp).w_empty_rows -= (*wp).w_filler_rows;
            } else {
                (*wp).w_filler_rows = (*wp).w_empty_rows;
                (*wp).w_empty_rows = 0 as ::core::ffi::c_int;
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn scroll_cursor_bot(
    mut wp: *mut win_T,
    mut min_scroll: ::core::ffi::c_int,
    mut set_topbot: bool,
) {
    let mut loff: lineoff_T = lineoff_T {
        lnum: 0,
        fill: 0,
        height: 0,
    };
    let mut old_topline: linenr_T = (*wp).w_topline;
    let mut old_skipcol: ::core::ffi::c_int = (*wp).w_skipcol as ::core::ffi::c_int;
    let mut old_topfill: ::core::ffi::c_int = (*wp).w_topfill;
    let mut old_botline: linenr_T = (*wp).w_botline;
    let mut old_valid: ::core::ffi::c_int = (*wp).w_valid;
    let mut old_empty_rows: ::core::ffi::c_int = (*wp).w_empty_rows;
    let mut cln: linenr_T = (*wp).w_cursor.lnum;
    let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
    if set_topbot {
        let mut used: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cln_last: linenr_T = cln;
        hasFolding(
            wp,
            cln,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut cln_last,
        );
        (*wp).w_botline = cln_last + 1 as linenr_T;
        loff.lnum = cln_last + 1 as linenr_T;
        loff.fill = 0 as ::core::ffi::c_int;
        loop {
            topline_back_winheight(wp, &raw mut loff, false_0);
            if loff.height == MAXCOL as ::core::ffi::c_int {
                break;
            }
            if used + loff.height > (*wp).w_view_height {
                if do_sms {
                    if used < (*wp).w_view_height {
                        let mut plines_offset: ::core::ffi::c_int =
                            used + loff.height - (*wp).w_view_height;
                        used = (*wp).w_view_height;
                        (*wp).w_topfill = loff.fill;
                        (*wp).w_topline = loff.lnum;
                        (*wp).w_skipcol = skipcol_from_plines(wp, plines_offset) as colnr_T;
                    }
                }
                break;
            } else {
                (*wp).w_topfill = loff.fill;
                (*wp).w_topline = loff.lnum;
                used += loff.height;
            }
        }
        set_empty_rows(wp, used);
        (*wp).w_valid |= VALID_BOTLINE | VALID_BOTLINE_AP;
        if (*wp).w_topline != old_topline
            || (*wp).w_topfill != old_topfill
            || (*wp).w_skipcol != old_skipcol
            || (*wp).w_skipcol != 0 as ::core::ffi::c_int
        {
            (*wp).w_valid &= !(VALID_WROW | VALID_CROW);
            if (*wp).w_skipcol != old_skipcol {
                redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
            } else {
                reset_skipcol(wp);
            }
        }
    } else {
        validate_botline_win(wp);
    }
    let mut used_0: ::core::ffi::c_int = plines_win_nofill(wp, cln, true_0 != 0);
    let mut scrolled: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if cln >= (*wp).w_botline {
        scrolled = used_0;
        if cln == (*wp).w_botline {
            scrolled -= (*wp).w_empty_rows;
        }
        if do_sms {
            let mut top_plines: ::core::ffi::c_int =
                plines_win_nofill(wp, (*wp).w_topline, false_0 != 0);
            let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
            if width1 > 0 as ::core::ffi::c_int {
                let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
                let mut skip_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if (*wp).w_skipcol > width1 {
                    skip_lines += ((*wp).w_skipcol as ::core::ffi::c_int - width1) / width2
                        + 1 as ::core::ffi::c_int;
                } else if (*wp).w_skipcol > 0 as ::core::ffi::c_int {
                    skip_lines = 1 as ::core::ffi::c_int;
                }
                top_plines -= skip_lines;
                if top_plines > (*wp).w_view_height {
                    scrolled += top_plines - (*wp).w_view_height;
                }
            }
        }
    }
    let mut boff: lineoff_T = lineoff_T {
        lnum: 0,
        fill: 0,
        height: 0,
    };
    if !hasFolding(
        wp,
        (*wp).w_cursor.lnum,
        &raw mut loff.lnum,
        &raw mut boff.lnum,
    ) {
        loff.lnum = cln;
        boff.lnum = cln;
    }
    loff.fill = 0 as ::core::ffi::c_int;
    boff.fill = 0 as ::core::ffi::c_int;
    let mut fill_below_window: ::core::ffi::c_int =
        win_get_fill(wp, (*wp).w_botline) - (*wp).w_filler_rows;
    let mut extra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut so: int64_t = get_scrolloff_value(wp);
    while loff.lnum > 1 as linenr_T {
        if ((scrolled <= 0 as ::core::ffi::c_int || scrolled >= min_scroll)
            && extra as int64_t
                >= (if mouse_dragging.get() > 0 as ::core::ffi::c_int {
                    (mouse_dragging.get() - 1 as ::core::ffi::c_int) as int64_t
                } else {
                    so
                })
            || boff.lnum + 1 as linenr_T > (*(*wp).w_buffer).b_ml.ml_line_count)
            && loff.lnum <= (*wp).w_botline
            && (loff.lnum < (*wp).w_botline || loff.fill >= fill_below_window)
        {
            break;
        }
        topline_back(wp, &raw mut loff);
        if loff.height == MAXCOL as ::core::ffi::c_int {
            used_0 = MAXCOL as ::core::ffi::c_int;
        } else {
            used_0 += loff.height;
        }
        if used_0 > (*wp).w_view_height {
            break;
        }
        if loff.lnum >= (*wp).w_botline
            && (loff.lnum > (*wp).w_botline || loff.fill <= fill_below_window)
        {
            scrolled += loff.height;
            if loff.lnum == (*wp).w_botline && loff.fill == 0 as ::core::ffi::c_int {
                scrolled -= (*wp).w_empty_rows;
            }
        }
        if boff.lnum >= (*(*wp).w_buffer).b_ml.ml_line_count {
            continue;
        }
        botline_forw(wp, &raw mut boff);
        '_c2rust_label: {
            if boff.height != MAXCOL as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"boff.height != MAXCOL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/move.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2067 as ::core::ffi::c_uint,
                    b"void scroll_cursor_bot(win_T *, int, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        used_0 += boff.height;
        if used_0 > (*wp).w_view_height {
            break;
        }
        if (extra as int64_t)
            < (if mouse_dragging.get() > 0 as ::core::ffi::c_int {
                (mouse_dragging.get() - 1 as ::core::ffi::c_int) as int64_t
            } else {
                so
            })
            || scrolled < min_scroll
        {
            extra += boff.height;
            if boff.lnum >= (*wp).w_botline
                || boff.lnum + 1 as linenr_T == (*wp).w_botline && boff.fill > (*wp).w_filler_rows
            {
                scrolled += boff.height;
                if boff.lnum == (*wp).w_botline && boff.fill == 0 as ::core::ffi::c_int {
                    scrolled -= (*wp).w_empty_rows;
                }
            }
        }
    }
    let mut line_count: linenr_T = 0;
    if scrolled <= 0 as ::core::ffi::c_int {
        line_count = 0 as ::core::ffi::c_int as linenr_T;
    } else if used_0 > (*wp).w_view_height {
        line_count = used_0 as linenr_T;
    } else {
        line_count = 0 as ::core::ffi::c_int as linenr_T;
        boff.fill = (*wp).w_topfill;
        boff.lnum = (*wp).w_topline - 1 as linenr_T;
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < scrolled && boff.lnum < (*wp).w_botline {
            botline_forw(wp, &raw mut boff);
            i += boff.height;
            line_count += 1;
        }
        if i < scrolled {
            line_count = 9999 as ::core::ffi::c_int as linenr_T;
        }
    }
    if line_count >= (*wp).w_view_height as linenr_T && line_count > min_scroll as linenr_T {
        scroll_cursor_halfway(wp, false_0 != 0, true_0 != 0);
    } else if line_count > 0 as linenr_T {
        if do_sms {
            scrollup(wp, scrolled as linenr_T, true_0 != 0);
        } else {
            scrollup(wp, line_count, true_0 != 0);
        }
    }
    if (*wp).w_topline == old_topline
        && (*wp).w_skipcol == old_skipcol
        && set_topbot as ::core::ffi::c_int != 0
    {
        (*wp).w_botline = old_botline;
        (*wp).w_empty_rows = old_empty_rows;
        (*wp).w_valid = old_valid;
    }
    (*wp).w_valid |= VALID_TOPLINE;
    (*wp).w_viewport_invalid = true_0 != 0;
    if set_topbot {
        cursor_correct_sms(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn scroll_cursor_halfway(
    mut wp: *mut win_T,
    mut atend: bool,
    mut prefer_above: bool,
) {
    let mut old_topline: linenr_T = (*wp).w_topline;
    let mut loff: lineoff_T = lineoff_T {
        lnum: (*wp).w_cursor.lnum,
        fill: 0,
        height: 0,
    };
    let mut boff: lineoff_T = lineoff_T {
        lnum: (*wp).w_cursor.lnum,
        fill: 0,
        height: 0,
    };
    hasFolding(wp, loff.lnum, &raw mut loff.lnum, &raw mut boff.lnum);
    let mut used: ::core::ffi::c_int = plines_win_nofill(wp, loff.lnum, true_0 != 0);
    loff.fill = 0 as ::core::ffi::c_int;
    boff.fill = 0 as ::core::ffi::c_int;
    let mut topline: linenr_T = loff.lnum;
    let mut skipcol: colnr_T = 0 as colnr_T;
    let mut want_height: ::core::ffi::c_int = 0;
    let mut do_sms: bool = (*wp).w_onebuf_opt.wo_wrap != 0 && (*wp).w_onebuf_opt.wo_sms != 0;
    if do_sms {
        if atend {
            want_height = ((*wp).w_view_height - used) / 2 as ::core::ffi::c_int;
            used = 0 as ::core::ffi::c_int;
        } else {
            want_height = (*wp).w_view_height;
        }
    }
    let mut topfill: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while topline > 1 as linenr_T {
        if do_sms {
            topline_back_winheight(wp, &raw mut loff, false_0);
            if loff.height == MAXCOL as ::core::ffi::c_int {
                break;
            }
            used += loff.height;
            if !atend && boff.lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
                botline_forw(wp, &raw mut boff);
                used += boff.height;
            }
            if used > want_height {
                if used - loff.height < want_height {
                    topline = loff.lnum;
                    topfill = loff.fill;
                    skipcol = skipcol_from_plines(wp, used - want_height) as colnr_T;
                }
                break;
            } else {
                topline = loff.lnum;
                topfill = loff.fill;
            }
        } else {
            let mut done: bool = false_0 != 0;
            let mut above: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut below: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while round <= 2 as ::core::ffi::c_int {
                if if prefer_above as ::core::ffi::c_int != 0 {
                    (round == 2 as ::core::ffi::c_int && below < above) as ::core::ffi::c_int
                } else {
                    (round == 1 as ::core::ffi::c_int && below <= above) as ::core::ffi::c_int
                } != 0
                {
                    if boff.lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
                        botline_forw(wp, &raw mut boff);
                        used += boff.height;
                        if used > (*wp).w_view_height {
                            done = true_0 != 0;
                            break;
                        } else {
                            below += boff.height;
                        }
                    } else {
                        below += 1;
                        if atend {
                            used += 1;
                        }
                    }
                }
                if if prefer_above as ::core::ffi::c_int != 0 {
                    (round == 1 as ::core::ffi::c_int && below >= above) as ::core::ffi::c_int
                } else {
                    (round == 1 as ::core::ffi::c_int && below > above) as ::core::ffi::c_int
                } != 0
                {
                    topline_back(wp, &raw mut loff);
                    if loff.height == MAXCOL as ::core::ffi::c_int {
                        used = MAXCOL as ::core::ffi::c_int;
                    } else {
                        used += loff.height;
                    }
                    if used > (*wp).w_view_height {
                        done = true_0 != 0;
                        break;
                    } else {
                        above += loff.height;
                        topline = loff.lnum;
                        topfill = loff.fill;
                    }
                }
                round += 1;
            }
            if done {
                break;
            }
        }
    }
    if !hasFolding(
        wp,
        topline,
        &raw mut (*wp).w_topline,
        ::core::ptr::null_mut::<linenr_T>(),
    ) && ((*wp).w_topline != topline
        || skipcol != 0 as ::core::ffi::c_int
        || (*wp).w_skipcol != 0 as ::core::ffi::c_int)
    {
        (*wp).w_topline = topline;
        if skipcol != 0 as ::core::ffi::c_int {
            (*wp).w_skipcol = skipcol;
            redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
        } else if do_sms {
            reset_skipcol(wp);
        }
    }
    (*wp).w_topfill = topfill;
    if old_topline > (*wp).w_topline + (*wp).w_view_height as linenr_T {
        (*wp).w_botfill = false_0 != 0;
    }
    check_topfill(wp, false_0 != 0);
    (*wp).w_valid &= !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
    (*wp).w_valid |= VALID_TOPLINE;
}
#[no_mangle]
pub unsafe extern "C" fn cursor_correct(mut wp: *mut win_T) {
    let mut above_wanted: int64_t = get_scrolloff_value(wp);
    let mut below_wanted: int64_t = get_scrolloff_value(wp);
    if mouse_dragging.get() > 0 as ::core::ffi::c_int {
        above_wanted = (mouse_dragging.get() - 1 as ::core::ffi::c_int) as int64_t;
        below_wanted = (mouse_dragging.get() - 1 as ::core::ffi::c_int) as int64_t;
    }
    if (*wp).w_topline == 1 as linenr_T {
        above_wanted = 0 as int64_t;
        let mut max_off: ::core::ffi::c_int = (*wp).w_view_height / 2 as ::core::ffi::c_int;
        below_wanted = if below_wanted < max_off as int64_t {
            below_wanted
        } else {
            max_off as int64_t
        };
    }
    validate_botline_win(wp);
    if (*wp).w_botline == (*(*wp).w_buffer).b_ml.ml_line_count + 1 as linenr_T
        && mouse_dragging.get() == 0 as ::core::ffi::c_int
    {
        below_wanted = 0 as int64_t;
        let mut max_off_0: ::core::ffi::c_int =
            ((*wp).w_view_height - 1 as ::core::ffi::c_int) / 2 as ::core::ffi::c_int;
        above_wanted = if above_wanted < max_off_0 as int64_t {
            above_wanted
        } else {
            max_off_0 as int64_t
        };
    }
    let mut cln: linenr_T = (*wp).w_cursor.lnum;
    if cln as int64_t >= (*wp).w_topline as int64_t + above_wanted
        && (cln as int64_t) < (*wp).w_botline as int64_t - below_wanted
        && !win_lines_concealed(wp)
    {
        return;
    }
    if (*wp).w_onebuf_opt.wo_sms != 0 && (*wp).w_onebuf_opt.wo_wrap == 0 {
        if (*wp).w_cline_height == (*wp).w_view_height {
            reset_skipcol(wp);
            return;
        }
    }
    let mut topline: linenr_T = (*wp).w_topline;
    let mut botline: linenr_T = (*wp).w_botline - 1 as linenr_T;
    let mut above: ::core::ffi::c_int = (*wp).w_topfill;
    let mut below: ::core::ffi::c_int = (*wp).w_filler_rows;
    while ((above as int64_t) < above_wanted || (below as int64_t) < below_wanted)
        && topline < botline
    {
        if (below as int64_t) < below_wanted && (below <= above || above as int64_t >= above_wanted)
        {
            below += plines_win_full(
                wp,
                botline,
                ::core::ptr::null_mut::<linenr_T>(),
                ::core::ptr::null_mut::<bool>(),
                true_0 != 0,
                true_0 != 0,
            );
            hasFolding(
                wp,
                botline,
                &raw mut botline,
                ::core::ptr::null_mut::<linenr_T>(),
            );
            botline -= 1;
        }
        if (above as int64_t) < above_wanted && (above < below || below as int64_t >= below_wanted)
        {
            above += plines_win_nofill(wp, topline, true_0 != 0);
            hasFolding(
                wp,
                topline,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut topline,
            );
            if topline < botline {
                above += win_get_fill(wp, topline + 1 as linenr_T);
            }
            topline += 1;
        }
    }
    if topline == botline || botline == 0 as linenr_T {
        (*wp).w_cursor.lnum = topline;
    } else if topline > botline {
        (*wp).w_cursor.lnum = botline;
    } else {
        if cln < topline && (*wp).w_topline > 1 as linenr_T {
            (*wp).w_cursor.lnum = topline;
            (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW);
        }
        if cln > botline && (*wp).w_botline <= (*(*wp).w_buffer).b_ml.ml_line_count {
            (*wp).w_cursor.lnum = botline;
            (*wp).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_CHEIGHT | VALID_CROW);
        }
    }
    check_cursor_moved(wp);
    (*wp).w_valid |= VALID_TOPLINE;
    (*wp).w_viewport_invalid = true_0 != 0;
}
unsafe extern "C" fn get_scroll_overlap(mut dir: Direction) -> ::core::ffi::c_int {
    let mut loff: lineoff_T = lineoff_T {
        lnum: 0,
        fill: 0,
        height: 0,
    };
    let mut min_height: ::core::ffi::c_int =
        (*curwin.get()).w_view_height - 2 as ::core::ffi::c_int;
    validate_botline_win(curwin.get());
    if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int
        && (*curwin.get()).w_topline == 1 as linenr_T
        || dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
            && (*curwin.get()).w_botline > (*curbuf.get()).b_ml.ml_line_count
    {
        return min_height + 2 as ::core::ffi::c_int;
    }
    loff.lnum = if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        (*curwin.get()).w_botline
    } else {
        (*curwin.get()).w_topline - 1 as linenr_T
    };
    loff.fill = win_get_fill(
        curwin.get(),
        loff.lnum
            + (dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
    ) - (if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        (*curwin.get()).w_filler_rows
    } else {
        (*curwin.get()).w_topfill
    });
    loff.height = if loff.fill > 0 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else {
        plines_win_nofill(curwin.get(), loff.lnum, true_0 != 0)
    };
    let mut h1: ::core::ffi::c_int = loff.height;
    if h1 > min_height {
        return min_height + 2 as ::core::ffi::c_int;
    }
    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        topline_back(curwin.get(), &raw mut loff);
    } else {
        botline_forw(curwin.get(), &raw mut loff);
    }
    let mut h2: ::core::ffi::c_int = loff.height;
    if h2 == MAXCOL as ::core::ffi::c_int || h2 + h1 > min_height {
        return min_height + 2 as ::core::ffi::c_int;
    }
    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        topline_back(curwin.get(), &raw mut loff);
    } else {
        botline_forw(curwin.get(), &raw mut loff);
    }
    let mut h3: ::core::ffi::c_int = loff.height;
    if h3 == MAXCOL as ::core::ffi::c_int || h3 + h2 > min_height {
        return min_height + 2 as ::core::ffi::c_int;
    }
    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        topline_back(curwin.get(), &raw mut loff);
    } else {
        botline_forw(curwin.get(), &raw mut loff);
    }
    let mut h4: ::core::ffi::c_int = loff.height;
    if h4 == MAXCOL as ::core::ffi::c_int || h4 + h3 + h2 > min_height || h3 + h2 + h1 > min_height
    {
        return min_height + 1 as ::core::ffi::c_int;
    } else {
        return min_height;
    };
}
unsafe extern "C" fn scroll_with_sms(
    mut dir: Direction,
    mut count: ::core::ffi::c_int,
    mut curscount: *mut ::core::ffi::c_int,
) -> bool {
    let mut prev_sms: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_sms;
    let mut prev_skipcol: colnr_T = (*curwin.get()).w_skipcol;
    let mut prev_topline: linenr_T = (*curwin.get()).w_topline;
    let mut prev_topfill: ::core::ffi::c_int = (*curwin.get()).w_topfill;
    (*curwin.get()).w_onebuf_opt.wo_sms = true_0;
    scroll_redraw(
        (dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
        count as linenr_T,
    );
    if prev_sms == 0 && (*curwin.get()).w_skipcol > 0 as ::core::ffi::c_int {
        let mut fixdir: ::core::ffi::c_int = dir as ::core::ffi::c_int;
        if labs(((*curwin.get()).w_topline - prev_topline) as ::core::ffi::c_long)
            > (dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int) as ::core::ffi::c_int
                as ::core::ffi::c_long
        {
            fixdir = dir as ::core::ffi::c_int * -1 as ::core::ffi::c_int;
        }
        let mut width1: ::core::ffi::c_int =
            (*curwin.get()).w_view_width - win_col_off(curwin.get());
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(curwin.get());
        count = 1 as ::core::ffi::c_int
            + ((*curwin.get()).w_skipcol as ::core::ffi::c_int - width1 - 1 as ::core::ffi::c_int)
                / width2;
        if fixdir == FORWARD as ::core::ffi::c_int {
            count = 1 as ::core::ffi::c_int
                + (linetabsize_eol(curwin.get(), (*curwin.get()).w_topline)
                    - (*curwin.get()).w_skipcol as ::core::ffi::c_int
                    - width1
                    + width2
                    - 1 as ::core::ffi::c_int)
                    / width2;
        }
        scroll_redraw(
            (fixdir == FORWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
            count as linenr_T,
        );
        *curscount += count
            * (if fixdir == dir as ::core::ffi::c_int {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            });
    }
    (*curwin.get()).w_onebuf_opt.wo_sms = prev_sms;
    return (*curwin.get()).w_topline != prev_topline
        || (*curwin.get()).w_topfill != prev_topfill
        || (*curwin.get()).w_skipcol != prev_skipcol;
}
#[no_mangle]
pub unsafe extern "C" fn pagescroll(
    mut dir: Direction,
    mut count: ::core::ffi::c_int,
    mut half: bool,
) -> ::core::ffi::c_int {
    let mut did_move: bool = false_0 != 0;
    let mut buflen: ::core::ffi::c_int = (*curbuf.get()).b_ml.ml_line_count as ::core::ffi::c_int;
    let mut prev_col: colnr_T = (*curwin.get()).w_cursor.col;
    let mut prev_curswant: colnr_T = (*curwin.get()).w_curswant;
    let mut prev_lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut oa: oparg_T = oparg_T {
        op_type: 0 as ::core::ffi::c_int,
        regname: 0,
        motion_type: kMTCharWise,
        motion_force: 0,
        use_reg_one: false,
        inclusive: false,
        end_adjusted: false,
        start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        end: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cursor_start: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        line_count: 0,
        empty: false,
        is_VIsual: false,
        start_vcol: 0,
        end_vcol: 0,
        prev_opcount: 0,
        prev_count0: 0,
        excl_tr_ws: false,
    };
    let mut ca: cmdarg_T = cmdarg_T {
        oap: ::core::ptr::null_mut::<oparg_T>(),
        prechar: 0,
        cmdchar: 0,
        nchar: 0,
        nchar_composing: [0; 32],
        nchar_len: 0,
        extra_char: 0,
        opcount: 0,
        count0: 0,
        count1: 0,
        arg: 0,
        retval: 0,
        searchbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    ca.oap = &raw mut oa;
    if half {
        if count != 0 {
            (*curwin.get()).w_onebuf_opt.wo_scr = (if (*curwin.get()).w_view_height < count {
                (*curwin.get()).w_view_height
            } else {
                count
            }) as OptInt;
        }
        count = if (*curwin.get()).w_view_height
            < (*curwin.get()).w_onebuf_opt.wo_scr as ::core::ffi::c_int
        {
            (*curwin.get()).w_view_height
        } else {
            (*curwin.get()).w_onebuf_opt.wo_scr as ::core::ffi::c_int
        };
        let mut curscount: ::core::ffi::c_int = count;
        if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
            && ((*curwin.get()).w_topline
                + (*curwin.get()).w_view_height as linenr_T
                + count as linenr_T
                > buflen as linenr_T
                || win_lines_concealed(curwin.get()) as ::core::ffi::c_int != 0)
        {
            let mut n: ::core::ffi::c_int = plines_correct_topline(
                curwin.get(),
                (*curwin.get()).w_topline,
                ::core::ptr::null_mut::<linenr_T>(),
                false_0 != 0,
                ::core::ptr::null_mut::<bool>(),
            );
            if n - count < (*curwin.get()).w_view_height
                && (*curwin.get()).w_topline < buflen as linenr_T
            {
                n += plines_m_win(
                    curwin.get(),
                    (*curwin.get()).w_topline + 1 as linenr_T,
                    buflen as linenr_T,
                    (*curwin.get()).w_view_height + count,
                );
            }
            if n < (*curwin.get()).w_view_height + count {
                count = n - (*curwin.get()).w_view_height;
            }
        }
        if count > 0 as ::core::ffi::c_int {
            did_move = scroll_with_sms(dir, count, &raw mut curscount);
            (*curwin.get()).w_cursor.lnum = prev_lnum;
            (*curwin.get()).w_cursor.col = prev_col;
            (*curwin.get()).w_curswant = prev_curswant;
        }
        if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 {
            nv_screengo(
                &raw mut oa,
                dir as ::core::ffi::c_int,
                curscount,
                true_0 != 0,
            );
        } else if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            cursor_down_inner(curwin.get(), curscount, true_0 != 0);
        } else {
            cursor_up_inner(curwin.get(), curscount as linenr_T, true_0 != 0);
        }
    } else {
        count *= if firstwin.get() == lastwin.get()
            && p_window.get() > 0 as OptInt
            && p_window.get() < (Rows.get() - 1 as ::core::ffi::c_int) as OptInt
        {
            if 1 as ::core::ffi::c_int
                > p_window.get() as ::core::ffi::c_int - 2 as ::core::ffi::c_int
            {
                1 as ::core::ffi::c_int
            } else {
                p_window.get() as ::core::ffi::c_int - 2 as ::core::ffi::c_int
            }
        } else {
            get_scroll_overlap(dir)
        };
        did_move = scroll_with_sms(dir, count, &raw mut count);
        if did_move {
            validate_botline_win(curwin.get());
            let mut lnum: linenr_T = if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                (*curwin.get()).w_topline
            } else {
                (*curwin.get()).w_botline - 1 as linenr_T
            };
            (*curwin.get()).w_cursor.lnum = if lnum > 1 as linenr_T {
                lnum
            } else {
                1 as linenr_T
            };
        }
    }
    if get_scrolloff_value(curwin.get()) > 0 as int64_t {
        cursor_correct(curwin.get());
    }
    foldAdjustCursor(curwin.get());
    did_move = did_move as ::core::ffi::c_int != 0
        || prev_col != (*curwin.get()).w_cursor.col
        || prev_lnum != (*curwin.get()).w_cursor.lnum;
    if !did_move {
        beep_flush();
    } else if (*curwin.get()).w_onebuf_opt.wo_sms == 0 {
        beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    } else if p_sol.get() != 0 {
        nv_g_home_m_cmd(&raw mut ca);
    }
    return if did_move as ::core::ffi::c_int != 0 {
        OK
    } else {
        FAIL
    };
}
#[no_mangle]
pub unsafe extern "C" fn do_check_cursorbind() {
    static prev_curwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
    static prev_cursor: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    });
    if curwin.get() == prev_curwin.get()
        && equalpos((*curwin.get()).w_cursor, prev_cursor.get()) as ::core::ffi::c_int != 0
    {
        return;
    }
    prev_curwin.set(curwin.get());
    prev_cursor.set((*curwin.get()).w_cursor);
    let mut line: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut col: colnr_T = (*curwin.get()).w_cursor.col;
    let mut coladd: colnr_T = (*curwin.get()).w_cursor.coladd;
    let mut curswant: colnr_T = (*curwin.get()).w_curswant;
    let mut set_curswant: bool = (*curwin.get()).w_set_curswant != 0;
    let mut old_curwin: *mut win_T = curwin.get();
    let mut old_curbuf: *mut buf_T = curbuf.get();
    let mut old_VIsual_select: ::core::ffi::c_int = VIsual_select.get() as ::core::ffi::c_int;
    let mut old_VIsual_active: ::core::ffi::c_int = VIsual_active.get() as ::core::ffi::c_int;
    VIsual_active.set(false_0 != 0);
    VIsual_select.set(VIsual_active.get());
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        curwin.set(wp);
        curbuf.set((*curwin.get()).w_buffer);
        if curwin.get() != old_curwin && (*curwin.get()).w_onebuf_opt.wo_crb != 0 {
            if (*curwin.get()).w_onebuf_opt.wo_diff != 0 {
                (*curwin.get()).w_cursor.lnum = diff_get_corresponding_line(old_curbuf, line);
            } else {
                (*curwin.get()).w_cursor.lnum = line;
            }
            (*curwin.get()).w_cursor.col = col;
            (*curwin.get()).w_cursor.coladd = coladd;
            (*curwin.get()).w_curswant = curswant;
            (*curwin.get()).w_set_curswant = set_curswant as ::core::ffi::c_int;
            let mut restart_edit_save: ::core::ffi::c_int = restart_edit.get();
            restart_edit.set(true_0);
            check_cursor(curwin.get());
            if (*curwin.get()).w_onebuf_opt.wo_scb == 0 {
                validate_cursor(curwin.get());
            }
            restart_edit.set(restart_edit_save);
            mb_adjust_cursor();
            redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
            if (*curwin.get()).w_onebuf_opt.wo_scb == 0 {
                update_topline(curwin.get());
            }
            (*curwin.get()).w_redr_status = true_0 != 0;
        }
        wp = (*wp).w_next;
    }
    VIsual_select.set(old_VIsual_select != 0);
    VIsual_active.set(old_VIsual_active != 0);
    curwin.set(old_curwin);
    curbuf.set(old_curbuf);
}
pub const CPO_NUMCOL: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
