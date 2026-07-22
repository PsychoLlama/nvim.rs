use crate::src::nvim::buffer::{bt_prompt, bt_quickfix};
use crate::src::nvim::charset::vim_iswordc;
use crate::src::nvim::cursor::{coladvance, get_cursor_pos_ptr, set_leftcol};
use crate::src::nvim::decoration::decor_conceal_line;
use crate::src::nvim::drawscreen::{
    redraw_curbuf_later, redraw_later, redraw_statuslines, setcursor, update_screen,
};
use crate::src::nvim::edit::{set_can_cindent, start_arrow, undisplay_dollar};
use crate::src::nvim::eval::typval::{tv_clear, tv_dict_add_nr, tv_dict_alloc_ret};
use crate::src::nvim::eval_1::{call_vim_function, eval_has_provider};
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, tabpage_close, tabpage_close_other, tabpage_new};
use crate::src::nvim::fold::{closeFold, hasFolding, openFold};
use crate::src::nvim::getchar::{
    safe_vgetc, stuffReadbuff, stuffcharReadbuff, stuffnumReadbuff, vpeekc, vungetc,
    AppendCharToRedobuff,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{get_win_by_grid_handle, grid_adjust};
use crate::src::nvim::keycodes::get_mouse_button;
use crate::src::nvim::main::{
    cmdwin_type, cmdwin_win, curbuf, curtab, curwin, first_tabpage, firstwin, mod_mask,
    mode_displayed, mouse_col, mouse_dragging, mouse_grid, mouse_past_bottom, mouse_past_eol,
    mouse_row, msg_grid, msg_grid_pos, msg_silent, p_ch, p_mousem, p_mousescroll_hor,
    p_mousescroll_vert, p_sel, p_smd, pum_grid, redraw_cmdline, restart_edit, tab_page_click_defs,
    topframe, where_paste_started, Columns, KeyStuffed, Rows, State, VIsual, VIsual_active,
    VIsual_mode, VIsual_reselect, VIsual_select,
};
use crate::src::nvim::mbyte::{
    mb_get_class, utf8len_tab, utf_head_off, utf_ptr2CharInfo_impl, utfc_next_impl, utfc_ptr2len,
};
use crate::src::nvim::memline::{gchar_pos, inc, ml_get, ml_get_buf};
use crate::src::nvim::menu::show_popupmenu;
use crate::src::nvim::message::siemsg;
use crate::src::nvim::normal::{
    clearop, clearopbeep, end_visual_mode, may_start_select, nv_scroll_line, prep_redo,
};
use crate::src::nvim::ops::clear_oparg;
use crate::src::nvim::option::get_scrolloff_value;
use crate::src::nvim::os::libc::{__assert_fail, abs, memset, strcmp};
use crate::src::nvim::plines::{
    charsize_fast, charsize_regular, getvcol, getvcols, init_charsize_arg, plines_win,
    plines_win_nofill, win_chartabsize, win_get_fill, win_may_fill,
};
use crate::src::nvim::popupmenu::pum_visible;
use crate::src::nvim::r#move::{
    check_topfill, pagescroll, scroll_redraw, win_col_off, win_col_off2,
};
use crate::src::nvim::register::{do_put, insert_reg, yank_register_mline};
use crate::src::nvim::search::findmatch;
use crate::src::nvim::state::virtual_active;
use crate::src::nvim::statusline::stl_connected;
use crate::src::nvim::strings::vim_strchr;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, Array, BoolVarValue, Boolean,
    BufUpdateCallbacks, CSType, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5,
    ChangedtickDictItem, CharInfo, CharSize, CharsizeArg, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict,
    Direction, Error, ErrorType, EvalFuncData, ExtmarkUndoObject, FileID, Float, FloatAnchor,
    FloatRelative, GridView, Integer, Intersection, KeyValuePair, LuaRef, MTKey, MTNode, MTPos,
    MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t,
    MarkTree, MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_13, MotionType,
    MsgpackRpcRequestHandler, Object, ObjectType, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, StrCharInfo, String_0, Terminal, Timestamp,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T,
    bufstate_T, chunksize_T, cmdarg_T, colnr_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_extra, key_value_pair,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, oparg_T, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    uintptr_t, undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S,
    winopt_T, wline_T, xfmark_T, yankreg_T, QUEUE,
};
use crate::src::nvim::ui::{ui_check_mouse, ui_cursor_shape, ui_flush, ui_mouse_has};
use crate::src::nvim::ui_compositor::ui_comp_mouse_focus;
use crate::src::nvim::window::{
    find_tabpage, global_stl_height, goto_tabpage, tabpage_index, tabpage_move,
    win_drag_status_line, win_drag_vsep_line, win_enter, win_fdccol_count, win_valid,
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
pub const MODE_SHOWMATCH: C2Rust_Unnamed_15 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_15 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_15 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_15 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_15 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_15 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_15 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_15 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_15 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_15 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_15 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_15 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_15 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_15 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_15 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_15 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_15 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_15 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_15 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_15 = 1;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
pub const KE_IGNORE: key_extra = 53;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_MOUSE: key_extra = 43;
pub const KE_S_F37: key_extra = 42;
pub const KE_S_F36: key_extra = 41;
pub const KE_S_F35: key_extra = 40;
pub const KE_S_F34: key_extra = 39;
pub const KE_S_F33: key_extra = 38;
pub const KE_S_F32: key_extra = 37;
pub const KE_S_F31: key_extra = 36;
pub const KE_S_F30: key_extra = 35;
pub const KE_S_F29: key_extra = 34;
pub const KE_S_F28: key_extra = 33;
pub const KE_S_F27: key_extra = 32;
pub const KE_S_F26: key_extra = 31;
pub const KE_S_F25: key_extra = 30;
pub const KE_S_F24: key_extra = 29;
pub const KE_S_F23: key_extra = 28;
pub const KE_S_F22: key_extra = 27;
pub const KE_S_F21: key_extra = 26;
pub const KE_S_F20: key_extra = 25;
pub const KE_S_F19: key_extra = 24;
pub const KE_S_F18: key_extra = 23;
pub const KE_S_F17: key_extra = 22;
pub const KE_S_F16: key_extra = 21;
pub const KE_S_F15: key_extra = 20;
pub const KE_S_F14: key_extra = 19;
pub const KE_S_F13: key_extra = 18;
pub const KE_S_F12: key_extra = 17;
pub const KE_S_F11: key_extra = 16;
pub const KE_S_F10: key_extra = 15;
pub const KE_S_F9: key_extra = 14;
pub const KE_S_F8: key_extra = 13;
pub const KE_S_F7: key_extra = 12;
pub const KE_S_F6: key_extra = 11;
pub const KE_S_F5: key_extra = 10;
pub const KE_S_F4: key_extra = 9;
pub const KE_S_F3: key_extra = 8;
pub const KE_S_F2: key_extra = 7;
pub const KE_S_F1: key_extra = 6;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const MOUSE_STATUSCOL: C2Rust_Unnamed_16 = 4096;
pub const MOUSE_WINBAR: C2Rust_Unnamed_16 = 2048;
pub const MOUSE_FOLD_OPEN: C2Rust_Unnamed_16 = 1024;
pub const MOUSE_FOLD_CLOSE: C2Rust_Unnamed_16 = 512;
pub const CURSOR_MOVED: C2Rust_Unnamed_16 = 256;
pub const IN_OTHER_WIN: C2Rust_Unnamed_16 = 8;
pub const IN_SEP_LINE: C2Rust_Unnamed_16 = 4;
pub const IN_STATUS_LINE: C2Rust_Unnamed_16 = 2;
pub const IN_BUFFER: C2Rust_Unnamed_16 = 1;
pub const IN_UNKNOWN: C2Rust_Unnamed_16 = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const MOUSE_RELEASED: C2Rust_Unnamed_17 = 32;
pub const MOUSE_MAY_STOP_VIS: C2Rust_Unnamed_17 = 16;
pub const MOUSE_SETPOS: C2Rust_Unnamed_17 = 8;
pub const MOUSE_DID_MOVE: C2Rust_Unnamed_17 = 4;
pub const MOUSE_MAY_VIS: C2Rust_Unnamed_17 = 2;
pub const MOUSE_FOCUS: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const MOUSE_X2: C2Rust_Unnamed_18 = 1024;
pub const MOUSE_X1: C2Rust_Unnamed_18 = 768;
pub const MOUSE_RELEASE: C2Rust_Unnamed_18 = 3;
pub const MOUSE_RIGHT: C2Rust_Unnamed_18 = 2;
pub const MOUSE_MIDDLE: C2Rust_Unnamed_18 = 1;
pub const MOUSE_LEFT: C2Rust_Unnamed_18 = 0;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_int;
pub const MSCR_RIGHT: C2Rust_Unnamed_19 = -2;
pub const MSCR_LEFT: C2Rust_Unnamed_19 = -1;
pub const MSCR_UP: C2Rust_Unnamed_19 = 1;
pub const MSCR_DOWN: C2Rust_Unnamed_19 = 0;
pub const PUT_CURSEND: C2Rust_Unnamed_20 = 2;
pub const PUT_FIXINDENT: C2Rust_Unnamed_20 = 1;
pub const OP_NOP: C2Rust_Unnamed_21 = 0;
pub const kCharsizeFast: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_20 = 64;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_20 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_20 = 16;
pub const PUT_LINE: C2Rust_Unnamed_20 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_20 = 4;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const OP_NR_SUB: C2Rust_Unnamed_21 = 29;
pub const OP_NR_ADD: C2Rust_Unnamed_21 = 28;
pub const OP_FUNCTION: C2Rust_Unnamed_21 = 27;
pub const OP_FORMAT2: C2Rust_Unnamed_21 = 26;
pub const OP_FOLDDELREC: C2Rust_Unnamed_21 = 25;
pub const OP_FOLDDEL: C2Rust_Unnamed_21 = 24;
pub const OP_FOLDCLOSEREC: C2Rust_Unnamed_21 = 23;
pub const OP_FOLDCLOSE: C2Rust_Unnamed_21 = 22;
pub const OP_FOLDOPENREC: C2Rust_Unnamed_21 = 21;
pub const OP_FOLDOPEN: C2Rust_Unnamed_21 = 20;
pub const OP_FOLD: C2Rust_Unnamed_21 = 19;
pub const OP_APPEND: C2Rust_Unnamed_21 = 18;
pub const OP_INSERT: C2Rust_Unnamed_21 = 17;
pub const OP_REPLACE: C2Rust_Unnamed_21 = 16;
pub const OP_ROT13: C2Rust_Unnamed_21 = 15;
pub const OP_JOIN_NS: C2Rust_Unnamed_21 = 14;
pub const OP_JOIN: C2Rust_Unnamed_21 = 13;
pub const OP_LOWER: C2Rust_Unnamed_21 = 12;
pub const OP_UPPER: C2Rust_Unnamed_21 = 11;
pub const OP_COLON: C2Rust_Unnamed_21 = 10;
pub const OP_FORMAT: C2Rust_Unnamed_21 = 9;
pub const OP_INDENT: C2Rust_Unnamed_21 = 8;
pub const OP_TILDE: C2Rust_Unnamed_21 = 7;
pub const OP_FILTER: C2Rust_Unnamed_21 = 6;
pub const OP_RSHIFT: C2Rust_Unnamed_21 = 5;
pub const OP_LSHIFT: C2Rust_Unnamed_21 = 4;
pub const OP_CHANGE: C2Rust_Unnamed_21 = 3;
pub const OP_YANK: C2Rust_Unnamed_21 = 2;
pub const OP_DELETE: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const kCharsizeRegular: C2Rust_Unnamed_22 = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_G: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_R: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const Ctrl_T: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_RSB: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const VALID_WROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VALID_BOTLINE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VALID_BOTLINE_AP: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn lt(mut a: pos_T, mut b: pos_T) -> bool {
    if a.lnum != b.lnum {
        return a.lnum < b.lnum;
    } else if a.col != b.col {
        return a.col < b.col;
    } else {
        return a.coladd < b.coladd;
    };
}
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
#[inline(always)]
unsafe extern "C" fn ltoreq(mut a: pos_T, mut b: pos_T) -> bool {
    return lt(a, b) as ::core::ffi::c_int != 0 || equalpos(a, b) as ::core::ffi::c_int != 0;
}
pub const MOUSE_VISUAL: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MOD_MASK_SHIFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_ALT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MOD_MASK_META: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MOD_MASK_2CLICK: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MOD_MASK_3CLICK: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MOD_MASK_4CLICK: ::core::ffi::c_int = 0x60 as ::core::ffi::c_int;
pub const MOD_MASK_MULTI_CLICK: ::core::ffi::c_int =
    MOD_MASK_2CLICK | MOD_MASK_3CLICK | MOD_MASK_4CLICK;
#[inline(always)]
unsafe extern "C" fn utf_ptr2CharInfo(p_in: *const ::core::ffi::c_char) -> CharInfo {
    let p: *const uint8_t = p_in as *const uint8_t;
    let first: uint8_t = *p;
    if (first as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        return CharInfo {
            value: first as int32_t,
            len: 1 as ::core::ffi::c_int,
        };
    } else {
        let mut len: ::core::ffi::c_int =
            (*utf8len_tab.ptr())[first as usize] as ::core::ffi::c_int;
        let code_point: int32_t = utf_ptr2CharInfo_impl(p, len as uintptr_t);
        if code_point < 0 as int32_t {
            len = 1 as ::core::ffi::c_int;
        }
        return CharInfo {
            value: code_point,
            len: len,
        };
    };
}
#[inline(always)]
unsafe extern "C" fn utfc_next(mut cur: StrCharInfo) -> StrCharInfo {
    let mut next: *mut uint8_t = cur.ptr.offset(cur.chr.len as isize) as *mut uint8_t;
    if ((*next as ::core::ffi::c_uint) < 0x80 as ::core::ffi::c_uint) as ::core::ffi::c_int
        as ::core::ffi::c_long
        != 0
    {
        return StrCharInfo {
            ptr: next as *mut ::core::ffi::c_char,
            chr: CharInfo {
                value: *next as int32_t,
                len: 1 as ::core::ffi::c_int,
            },
        };
    }
    return utfc_next_impl(cur);
}
#[inline(always)]
unsafe extern "C" fn utf_ptr2StrCharInfo(mut ptr: *mut ::core::ffi::c_char) -> StrCharInfo {
    return StrCharInfo {
        ptr: ptr,
        chr: utf_ptr2CharInfo(ptr),
    };
}
static orig_topline: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
static orig_topfill: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
unsafe extern "C" fn get_mouse_class(mut p: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    if (*utf8len_tab.ptr())[*p.offset(0 as ::core::ffi::c_int as isize) as uint8_t as usize]
        as ::core::ffi::c_int
        > 1 as ::core::ffi::c_int
    {
        return mb_get_class(p);
    }
    let c: ::core::ffi::c_int = *p as uint8_t as ::core::ffi::c_int;
    if c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if vim_iswordc(c) {
        return 2 as ::core::ffi::c_int;
    }
    if c != NUL
        && !vim_strchr(b"-+*/%<>&|^!=\0".as_ptr() as *const ::core::ffi::c_char, c).is_null()
    {
        return 1 as ::core::ffi::c_int;
    }
    return c;
}
unsafe extern "C" fn find_start_of_word(mut pos: *mut pos_T) {
    let mut line: *mut ::core::ffi::c_char = ml_get((*pos).lnum);
    let mut cclass: ::core::ffi::c_int = get_mouse_class(line.offset((*pos).col as isize));
    while (*pos).col > 0 as ::core::ffi::c_int {
        let mut col: ::core::ffi::c_int =
            (*pos).col as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        col -= utf_head_off(line, line.offset(col as isize));
        if get_mouse_class(line.offset(col as isize)) != cclass {
            break;
        }
        (*pos).col = col as colnr_T;
    }
}
unsafe extern "C" fn find_end_of_word(mut pos: *mut pos_T) {
    let mut line: *mut ::core::ffi::c_char = ml_get((*pos).lnum);
    if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && (*pos).col > 0 as ::core::ffi::c_int
    {
        (*pos).col -= 1;
        (*pos).col -= utf_head_off(line, line.offset((*pos).col as isize));
    }
    let mut cclass: ::core::ffi::c_int = get_mouse_class(line.offset((*pos).col as isize));
    while *line.offset((*pos).col as isize) as ::core::ffi::c_int != NUL {
        let mut col: ::core::ffi::c_int =
            (*pos).col as ::core::ffi::c_int + utfc_ptr2len(line.offset((*pos).col as isize));
        if get_mouse_class(line.offset(col as isize)) != cclass {
            if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                (*pos).col = col as colnr_T;
            }
            break;
        } else {
            (*pos).col = col as colnr_T;
        }
    }
}
unsafe extern "C" fn move_tab_to_mouse() {
    let mut tabnr: ::core::ffi::c_int =
        (*(*tab_page_click_defs.ptr()).offset(mouse_col.get() as isize)).tabnr;
    if tabnr <= 0 as ::core::ffi::c_int {
        tabpage_move(9999 as ::core::ffi::c_int);
    } else if tabnr < tabpage_index(curtab.get()) {
        tabpage_move(tabnr - 1 as ::core::ffi::c_int);
    } else {
        tabpage_move(tabnr);
    };
}
unsafe extern "C" fn mouse_tab_close(mut c1: ::core::ffi::c_int) {
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if c1 == 999 as ::core::ffi::c_int {
        tp = curtab.get();
    } else {
        tp = find_tabpage(c1);
    }
    if tp == curtab.get() {
        if !(*first_tabpage.get()).tp_next.is_null() {
            tabpage_close(false_0);
        }
    } else if !tp.is_null() {
        tabpage_close_other(tp, false_0);
    }
}
static got_click: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
unsafe extern "C" fn call_click_def_func(
    mut click_defs: *mut StlClickDefinition,
    mut col: ::core::ffi::c_int,
    mut which_button: ::core::ffi::c_int,
) {
    let mut c2rust_lvalue: [::core::ffi::c_char; 5] = [
        (if mod_mask.get() & MOD_MASK_SHIFT != 0 {
            's' as ::core::ffi::c_int
        } else {
            ' ' as ::core::ffi::c_int
        }) as ::core::ffi::c_char,
        (if mod_mask.get() & MOD_MASK_CTRL != 0 {
            'c' as ::core::ffi::c_int
        } else {
            ' ' as ::core::ffi::c_int
        }) as ::core::ffi::c_char,
        (if mod_mask.get() & MOD_MASK_ALT != 0 {
            'a' as ::core::ffi::c_int
        } else {
            ' ' as ::core::ffi::c_int
        }) as ::core::ffi::c_char,
        (if mod_mask.get() & MOD_MASK_META != 0 {
            'm' as ::core::ffi::c_int
        } else {
            ' ' as ::core::ffi::c_int
        }) as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
    ];
    let mut argv: [typval_T; 4] = [
        typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                v_number: (*click_defs.offset(col as isize)).tabnr as varnumber_T,
            },
        },
        typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                v_number: (if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_4CLICK {
                    4 as ::core::ffi::c_int
                } else if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_3CLICK {
                    3 as ::core::ffi::c_int
                } else if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) as varnumber_T,
            },
        },
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                v_string: (if which_button == MOUSE_LEFT as ::core::ffi::c_int {
                    b"l\0".as_ptr() as *const ::core::ffi::c_char
                } else if which_button == MOUSE_RIGHT as ::core::ffi::c_int {
                    b"r\0".as_ptr() as *const ::core::ffi::c_char
                } else if which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
                    b"m\0".as_ptr() as *const ::core::ffi::c_char
                } else if which_button == MOUSE_X1 as ::core::ffi::c_int {
                    b"x1\0".as_ptr() as *const ::core::ffi::c_char
                } else if which_button == MOUSE_X2 as ::core::ffi::c_int {
                    b"x2\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"?\0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char,
            },
        },
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                v_string: &raw mut c2rust_lvalue as *mut ::core::ffi::c_char,
            },
        },
    ];
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    call_vim_function(
        (*click_defs.offset(col as isize)).func,
        ::core::mem::size_of::<[typval_T; 4]>()
            .wrapping_div(::core::mem::size_of::<typval_T>())
            .wrapping_div(
                (::core::mem::size_of::<[typval_T; 4]>()
                    .wrapping_rem(::core::mem::size_of::<typval_T>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int,
        &raw mut argv as *mut typval_T,
        &raw mut rettv,
    );
    tv_clear(&raw mut rettv);
    got_click.set(false_0 != 0);
}
unsafe extern "C" fn get_fpos_of_mouse(mut mpos: *mut pos_T) -> ::core::ffi::c_int {
    let mut grid: ::core::ffi::c_int = mouse_grid.get();
    let mut row: ::core::ffi::c_int = mouse_row.get();
    let mut col: ::core::ffi::c_int = mouse_col.get();
    if row < 0 as ::core::ffi::c_int || col < 0 as ::core::ffi::c_int {
        return IN_UNKNOWN as ::core::ffi::c_int;
    }
    let mut wp: *mut win_T = mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
    if wp.is_null() {
        return IN_UNKNOWN as ::core::ffi::c_int;
    }
    let mut winrow: ::core::ffi::c_int = row;
    let mut wincol: ::core::ffi::c_int = col;
    let mut below_buffer: bool =
        mouse_comp_pos(wp, &raw mut row, &raw mut col, &raw mut (*mpos).lnum);
    if !below_buffer
        && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
        && (if (*wp).w_onebuf_opt.wo_rl != 0 {
            (wincol >= (*wp).w_view_width - win_col_off(wp)) as ::core::ffi::c_int
        } else {
            (wincol < win_col_off(wp)) as ::core::ffi::c_int
        }) != 0
    {
        return MOUSE_STATUSCOL as ::core::ffi::c_int;
    }
    if winrow >= (*wp).w_view_height + (*wp).w_status_height {
        if mouse_grid.get() <= 1 as ::core::ffi::c_int
            && (mouse_row.get() as OptInt) < Rows.get() as OptInt - p_ch.get()
            && mouse_row.get() as OptInt
                >= Rows.get() as OptInt - p_ch.get() - global_stl_height() as OptInt
        {
            return IN_STATUS_LINE as ::core::ffi::c_int;
        }
        return IN_UNKNOWN as ::core::ffi::c_int;
    } else if winrow >= (*wp).w_view_height {
        return IN_STATUS_LINE as ::core::ffi::c_int;
    }
    if winrow < 0 as ::core::ffi::c_int && winrow + (*wp).w_winbar_height >= 0 as ::core::ffi::c_int
    {
        return MOUSE_WINBAR as ::core::ffi::c_int;
    }
    if wincol >= (*wp).w_view_width {
        return IN_SEP_LINE as ::core::ffi::c_int;
    }
    if wp != curwin.get() || below_buffer as ::core::ffi::c_int != 0 {
        return IN_UNKNOWN as ::core::ffi::c_int;
    }
    (*mpos).col = vcol2col(wp, (*mpos).lnum, col as colnr_T, &raw mut (*mpos).coladd);
    return IN_BUFFER as ::core::ffi::c_int;
}
unsafe extern "C" fn do_popup(
    mut which_button: ::core::ffi::c_int,
    mut m_pos_flag: ::core::ffi::c_int,
    mut m_pos: pos_T,
) -> ::core::ffi::c_int {
    let mut jump_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if strcmp(
        p_mousem.get(),
        b"popup_setpos\0".as_ptr() as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        if VIsual_active.get() {
            if m_pos_flag != IN_BUFFER as ::core::ffi::c_int {
                jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
            } else if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                if (*curwin.get()).w_cursor.lnum <= (*VIsual.ptr()).lnum
                    && (m_pos.lnum < (*curwin.get()).w_cursor.lnum
                        || (*VIsual.ptr()).lnum < m_pos.lnum)
                    || (*VIsual.ptr()).lnum < (*curwin.get()).w_cursor.lnum
                        && (m_pos.lnum < (*VIsual.ptr()).lnum
                            || (*curwin.get()).w_cursor.lnum < m_pos.lnum)
                {
                    jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                }
            } else if ltoreq((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0
                && (lt(m_pos, (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
                    || lt(VIsual.get(), m_pos) as ::core::ffi::c_int != 0)
                || lt(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
                    && (lt(m_pos, VIsual.get()) as ::core::ffi::c_int != 0
                        || lt((*curwin.get()).w_cursor, m_pos) as ::core::ffi::c_int != 0)
            {
                jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
            } else if VIsual_mode.get() == Ctrl_V {
                let mut leftcol: colnr_T = 0;
                let mut rightcol: colnr_T = 0;
                getvcols(
                    curwin.get(),
                    &raw mut (*curwin.get()).w_cursor,
                    VIsual.ptr(),
                    &raw mut leftcol,
                    &raw mut rightcol,
                );
                getvcol(
                    curwin.get(),
                    &raw mut m_pos,
                    ::core::ptr::null_mut::<colnr_T>(),
                    &raw mut m_pos.col,
                    ::core::ptr::null_mut::<colnr_T>(),
                );
                if m_pos.col < leftcol || m_pos.col > rightcol {
                    jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                }
            }
        } else {
            jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
        }
    }
    if jump_flags != 0 {
        jump_flags = jump_to_mouse(jump_flags, ::core::ptr::null_mut::<bool>(), which_button);
        redraw_curbuf_later(if VIsual_active.get() as ::core::ffi::c_int != 0 {
            UPD_INVERTED as ::core::ffi::c_int
        } else {
            UPD_VALID as ::core::ffi::c_int
        });
        update_screen();
        setcursor();
        ui_flush();
    }
    show_popupmenu();
    got_click.set(false_0 != 0);
    return jump_flags;
}
pub unsafe extern "C" fn do_mouse(
    mut oap: *mut oparg_T,
    mut c: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut fixindent: bool,
) -> bool {
    let mut which_button: ::core::ffi::c_int = 0;
    let mut is_click: bool = false;
    let mut is_drag: bool = false;
    static in_tab_line: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    static orig_cursor: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    });
    loop {
        which_button = get_mouse_button(
            (-c as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint)
                as ::core::ffi::c_int,
            &raw mut is_click,
            &raw mut is_drag,
        );
        if !is_drag {
            break;
        }
        if !(KeyStuffed.get() == 0 && vpeekc() != NUL) {
            break;
        }
        let mut nc: ::core::ffi::c_int = 0;
        let mut save_mouse_grid: ::core::ffi::c_int = mouse_grid.get();
        let mut save_mouse_row: ::core::ffi::c_int = mouse_row.get();
        let mut save_mouse_col: ::core::ffi::c_int = mouse_col.get();
        nc = safe_vgetc();
        if c == nc {
            continue;
        }
        vungetc(nc);
        mouse_grid.set(save_mouse_grid);
        mouse_row.set(save_mouse_row);
        mouse_col.set(save_mouse_col);
        break;
    }
    if c == -(253 as ::core::ffi::c_int
        + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
    {
        return false_0 != 0;
    }
    if is_click {
        got_click.set(true_0 != 0);
    } else {
        if !got_click.get() {
            return false_0 != 0;
        }
        if !is_drag {
            got_click.set(false_0 != 0);
            if in_tab_line.get() {
                in_tab_line.set(false_0 != 0);
                return false_0 != 0;
            }
        }
    }
    if is_click as ::core::ffi::c_int != 0
        && mod_mask.get() & MOD_MASK_CTRL != 0
        && which_button == MOUSE_RIGHT as ::core::ffi::c_int
    {
        if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
            stuffcharReadbuff(Ctrl_O);
        }
        if count > 1 as ::core::ffi::c_int {
            stuffnumReadbuff(count);
        }
        stuffcharReadbuff(Ctrl_T);
        got_click.set(false_0 != 0);
        return false_0 != 0;
    }
    if mod_mask.get() & MOD_MASK_CTRL != 0 && which_button != MOUSE_LEFT as ::core::ffi::c_int {
        return false_0 != 0;
    }
    if mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL | MOD_MASK_ALT | MOD_MASK_META) != 0
        && (!is_click
            || mod_mask.get() & MOD_MASK_MULTI_CLICK != 0
            || which_button == MOUSE_MIDDLE as ::core::ffi::c_int)
        && !(mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_ALT) != 0
            && mouse_model_popup() as ::core::ffi::c_int != 0
            && which_button == MOUSE_LEFT as ::core::ffi::c_int)
        && !(mod_mask.get() & MOD_MASK_ALT != 0
            && !mouse_model_popup()
            && which_button == MOUSE_RIGHT as ::core::ffi::c_int)
    {
        return false_0 != 0;
    }
    if !is_click && which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut regname: ::core::ffi::c_int = if !oap.is_null() {
        (*oap).regname
    } else {
        0 as ::core::ffi::c_int
    };
    if which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
        if State.get() == MODE_NORMAL as ::core::ffi::c_int {
            if !oap.is_null() && (*oap).op_type != OP_NOP as ::core::ffi::c_int {
                clearopbeep(oap);
                return false_0 != 0;
            }
            if VIsual_active.get() {
                if VIsual_select.get() {
                    stuffcharReadbuff(Ctrl_G);
                    stuffReadbuff(b"\"+p\0".as_ptr() as *const ::core::ffi::c_char);
                } else {
                    stuffcharReadbuff('y' as ::core::ffi::c_int);
                    stuffcharReadbuff(
                        -(253 as ::core::ffi::c_int
                            + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
                    );
                }
                return false_0 != 0;
            }
        } else if State.get() & MODE_INSERT as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
            if regname == '.' as ::core::ffi::c_int {
                insert_reg(regname, ::core::ptr::null_mut::<yankreg_T>(), true_0 != 0);
            } else {
                if regname == 0 as ::core::ffi::c_int
                    && eval_has_provider(
                        b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
                        false_0 != 0,
                    ) as ::core::ffi::c_int
                        != 0
                {
                    regname = '*' as ::core::ffi::c_int;
                }
                let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
                if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0
                    && !yank_register_mline(regname, &raw mut reg)
                {
                    insert_reg(regname, reg, true_0 != 0);
                } else {
                    do_put(
                        regname,
                        reg,
                        BACKWARD as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        (if fixindent as ::core::ffi::c_int != 0 {
                            PUT_FIXINDENT as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) | PUT_CURSEND as ::core::ffi::c_int,
                    );
                    AppendCharToRedobuff(Ctrl_R);
                    AppendCharToRedobuff(if fixindent as ::core::ffi::c_int != 0 {
                        Ctrl_P
                    } else {
                        Ctrl_O
                    });
                    AppendCharToRedobuff(if regname == 0 as ::core::ffi::c_int {
                        '"' as ::core::ffi::c_int
                    } else {
                        regname
                    });
                }
            }
            return false_0 != 0;
        }
    }
    let mut jump_flags: ::core::ffi::c_int = if is_click as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        MOUSE_FOCUS as ::core::ffi::c_int | MOUSE_DID_MOVE as ::core::ffi::c_int
    };
    let mut old_curwin: *mut win_T = curwin.get();
    if !(*tab_page_click_defs.ptr()).is_null() {
        if mouse_grid.get() <= 1 as ::core::ffi::c_int
            && mouse_row.get() == 0 as ::core::ffi::c_int
            && (*firstwin.get()).w_winrow > 0 as ::core::ffi::c_int
        {
            if is_drag {
                if in_tab_line.get() {
                    move_tab_to_mouse();
                }
                return false_0 != 0;
            }
            if is_click as ::core::ffi::c_int != 0
                && cmdwin_type.get() == 0 as ::core::ffi::c_int
                && mouse_col.get() < Columns.get()
            {
                let mut tabnr: ::core::ffi::c_int =
                    (*(*tab_page_click_defs.ptr()).offset(mouse_col.get() as isize)).tabnr;
                in_tab_line.set(true_0 != 0);
                's_464: {
                    match (*(*tab_page_click_defs.ptr()).offset(mouse_col.get() as isize)).type_0
                        as ::core::ffi::c_uint
                    {
                        1 => {
                            if which_button != MOUSE_MIDDLE as ::core::ffi::c_int {
                                if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                                    end_visual_mode();
                                    tabpage_new();
                                    tabpage_move(if tabnr == 0 as ::core::ffi::c_int {
                                        9999 as ::core::ffi::c_int
                                    } else {
                                        tabnr - 1 as ::core::ffi::c_int
                                    });
                                } else {
                                    goto_tabpage(tabnr);
                                    if curwin.get() != old_curwin {
                                        end_visual_mode();
                                    }
                                }
                                break 's_464;
                            }
                        }
                        2 => {}
                        3 => {
                            call_click_def_func(
                                tab_page_click_defs.get(),
                                mouse_col.get(),
                                which_button,
                            );
                            break 's_464;
                        }
                        0 | _ => {
                            break 's_464;
                        }
                    }
                    mouse_tab_close(tabnr);
                }
            }
            return true_0 != 0;
        } else if is_drag as ::core::ffi::c_int != 0 && in_tab_line.get() as ::core::ffi::c_int != 0
        {
            move_tab_to_mouse();
            return false_0 != 0;
        }
    }
    let mut m_pos_flag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut m_pos: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0,
        coladd: 0,
    };
    if mouse_model_popup() {
        m_pos_flag = get_fpos_of_mouse(&raw mut m_pos);
        if m_pos_flag
            & (IN_STATUS_LINE as ::core::ffi::c_int
                | MOUSE_WINBAR as ::core::ffi::c_int
                | MOUSE_STATUSCOL as ::core::ffi::c_int)
            == 0
            && which_button == MOUSE_RIGHT as ::core::ffi::c_int
            && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
        {
            if !is_click {
                return false_0 != 0;
            }
            return do_popup(which_button, m_pos_flag, m_pos) & CURSOR_MOVED as ::core::ffi::c_int
                != 0;
        }
        if m_pos_flag
            & (IN_STATUS_LINE as ::core::ffi::c_int
                | MOUSE_WINBAR as ::core::ffi::c_int
                | MOUSE_STATUSCOL as ::core::ffi::c_int)
            == 0
            && (which_button == MOUSE_LEFT as ::core::ffi::c_int
                && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_ALT) != 0)
        {
            which_button = MOUSE_RIGHT as ::core::ffi::c_int;
            (*mod_mask.ptr()) &= !MOD_MASK_SHIFT;
        }
    }
    let mut end_visual: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0,
        coladd: 0,
    };
    let mut start_visual: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0,
        coladd: 0,
    };
    let mut mouse_can_visual: bool = ui_mouse_has(MOUSE_VISUAL);
    if State.get() & (MODE_NORMAL as ::core::ffi::c_int | MODE_INSERT as ::core::ffi::c_int) != 0
        && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
    {
        if which_button == MOUSE_LEFT as ::core::ffi::c_int
            && mouse_can_visual as ::core::ffi::c_int != 0
        {
            if is_click {
                if VIsual_active.get() {
                    jump_flags |= MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                }
            } else {
                jump_flags |= MOUSE_MAY_VIS as ::core::ffi::c_int;
            }
        } else if which_button == MOUSE_RIGHT as ::core::ffi::c_int
            && mouse_can_visual as ::core::ffi::c_int != 0
        {
            if is_click as ::core::ffi::c_int != 0 && VIsual_active.get() as ::core::ffi::c_int != 0
            {
                if lt((*curwin.get()).w_cursor, VIsual.get()) {
                    start_visual = (*curwin.get()).w_cursor;
                    end_visual = VIsual.get();
                } else {
                    start_visual = VIsual.get();
                    end_visual = (*curwin.get()).w_cursor;
                }
            }
            jump_flags |= MOUSE_MAY_VIS as ::core::ffi::c_int;
            jump_flags |= MOUSE_FOCUS as ::core::ffi::c_int;
        } else if which_button == MOUSE_RIGHT as ::core::ffi::c_int {
            jump_flags |= MOUSE_FOCUS as ::core::ffi::c_int;
        }
    }
    if !is_drag && !oap.is_null() && (*oap).op_type != OP_NOP as ::core::ffi::c_int {
        got_click.set(false_0 != 0);
        (*oap).motion_type = kMTCharWise;
    }
    if !is_click && !is_drag {
        jump_flags |= MOUSE_RELEASED as ::core::ffi::c_int;
    }
    let mut old_active: ::core::ffi::c_int = VIsual_active.get() as ::core::ffi::c_int;
    let mut save_cursor: pos_T = (*curwin.get()).w_cursor;
    if !VIsual_active.get() || mouse_can_visual as ::core::ffi::c_int != 0 {
        jump_flags = jump_to_mouse(
            jump_flags,
            if oap.is_null() {
                ::core::ptr::null_mut::<bool>()
            } else {
                &raw mut (*oap).inclusive
            },
            which_button,
        );
    }
    let mut moved: bool = jump_flags & CURSOR_MOVED as ::core::ffi::c_int != 0;
    let mut in_winbar: bool = jump_flags & MOUSE_WINBAR as ::core::ffi::c_int != 0;
    let mut in_statuscol: bool = jump_flags & MOUSE_STATUSCOL as ::core::ffi::c_int != 0;
    let mut in_status_line: bool = jump_flags & IN_STATUS_LINE as ::core::ffi::c_int != 0;
    let mut in_global_statusline: bool =
        in_status_line as ::core::ffi::c_int != 0 && global_stl_height() > 0 as ::core::ffi::c_int;
    let mut in_sep_line: bool = jump_flags & IN_SEP_LINE as ::core::ffi::c_int != 0;
    if (in_winbar as ::core::ffi::c_int != 0
        || in_status_line as ::core::ffi::c_int != 0
        || in_statuscol as ::core::ffi::c_int != 0)
        && is_click as ::core::ffi::c_int != 0
    {
        let mut click_grid: ::core::ffi::c_int = mouse_grid.get();
        let mut click_row: ::core::ffi::c_int = mouse_row.get();
        let mut click_col: ::core::ffi::c_int = mouse_col.get();
        let mut wp: *mut win_T =
            mouse_find_win_inner(&raw mut click_grid, &raw mut click_row, &raw mut click_col);
        if wp.is_null() {
            return false_0 != 0;
        }
        let mut click_defs: *mut StlClickDefinition = if in_status_line as ::core::ffi::c_int != 0 {
            (*wp).w_status_click_defs
        } else if in_winbar as ::core::ffi::c_int != 0 {
            (*wp).w_winbar_click_defs
        } else {
            (*wp).w_statuscol_click_defs
        };
        if in_global_statusline {
            click_defs = (*curwin.get()).w_status_click_defs;
            click_col = mouse_col.get();
        }
        if in_statuscol as ::core::ffi::c_int != 0 && (*wp).w_onebuf_opt.wo_rl != 0 {
            click_col = (*wp).w_view_width - click_col - 1 as ::core::ffi::c_int;
        }
        if in_statuscol as ::core::ffi::c_int != 0
            && click_col >= (*wp).w_statuscol_click_defs_size as ::core::ffi::c_int
            || in_status_line as ::core::ffi::c_int != 0
                && click_col
                    >= (*(if in_global_statusline as ::core::ffi::c_int != 0 {
                        curwin.get()
                    } else {
                        wp
                    }))
                    .w_status_click_defs_size as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        if !click_defs.is_null() {
            match (*click_defs.offset(click_col as isize)).type_0 as ::core::ffi::c_uint {
                0 => {
                    if in_statuscol as ::core::ffi::c_int != 0
                        && mouse_model_popup() as ::core::ffi::c_int != 0
                        && which_button == MOUSE_RIGHT as ::core::ffi::c_int
                        && mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) == 0
                    {
                        do_popup(which_button, m_pos_flag, m_pos);
                    }
                }
                3 => {
                    call_click_def_func(click_defs, click_col, which_button);
                }
                _ => {
                    '_c2rust_label: {
                        if false
                            && !(b"winbar, statusline and statuscolumn only support %@ for clicks\0"
                                .as_ptr()
                                as *const ::core::ffi::c_char)
                                .is_null()
                        {
                        } else {
                            __assert_fail(
                                b"false && \"winbar, statusline and statuscolumn only support %@ for clicks\"\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/mouse.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                716 as ::core::ffi::c_uint,
                                b"_Bool do_mouse(oparg_T *, int, int, int, _Bool)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
            }
        }
        if !(in_statuscol as ::core::ffi::c_int != 0
            && jump_flags
                & (MOUSE_FOLD_CLOSE as ::core::ffi::c_int | MOUSE_FOLD_OPEN as ::core::ffi::c_int)
                != 0)
        {
            return false_0 != 0;
        }
    } else if in_winbar as ::core::ffi::c_int != 0 || in_statuscol as ::core::ffi::c_int != 0 {
        return false_0 != 0;
    }
    if curwin.get() != old_curwin
        && !oap.is_null()
        && (*oap).op_type != OP_NOP as ::core::ffi::c_int
    {
        clearop(oap);
    }
    if mod_mask.get() == 0 as ::core::ffi::c_int
        && !is_drag
        && jump_flags
            & (MOUSE_FOLD_CLOSE as ::core::ffi::c_int | MOUSE_FOLD_OPEN as ::core::ffi::c_int)
            != 0
        && which_button == MOUSE_LEFT as ::core::ffi::c_int
    {
        if jump_flags & MOUSE_FOLD_OPEN as ::core::ffi::c_int != 0 {
            openFold((*curwin.get()).w_cursor, 1 as ::core::ffi::c_int);
        } else {
            closeFold((*curwin.get()).w_cursor, 1 as ::core::ffi::c_int);
        }
        if curwin.get() == old_curwin {
            (*curwin.get()).w_cursor = save_cursor;
        }
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && is_drag as ::core::ffi::c_int != 0
        && get_scrolloff_value(curwin.get()) != 0
    {
        if mouse_row.get() == 0 as ::core::ffi::c_int {
            mouse_dragging.set(2 as ::core::ffi::c_int);
        } else {
            mouse_dragging.set(1 as ::core::ffi::c_int);
        }
    }
    if is_drag as ::core::ffi::c_int != 0
        && mouse_row.get() < 0 as ::core::ffi::c_int
        && !in_status_line
    {
        scroll_redraw(false_0, 1 as linenr_T);
        mouse_row.set(0 as ::core::ffi::c_int);
    }
    let mut old_mode: ::core::ffi::c_int = VIsual_mode.get();
    if start_visual.lnum != 0 {
        let mut diff: linenr_T = 0;
        if mod_mask.get() & MOD_MASK_ALT != 0 {
            VIsual_mode.set(Ctrl_V);
        }
        if VIsual_mode.get() == Ctrl_V {
            let mut leftcol: colnr_T = 0;
            let mut rightcol: colnr_T = 0;
            getvcols(
                curwin.get(),
                &raw mut start_visual,
                &raw mut end_visual,
                &raw mut leftcol,
                &raw mut rightcol,
            );
            if (*curwin.get()).w_curswant
                > (leftcol as ::core::ffi::c_int + rightcol as ::core::ffi::c_int)
                    / 2 as ::core::ffi::c_int
            {
                end_visual.col = leftcol;
            } else {
                end_visual.col = rightcol;
            }
            if (*curwin.get()).w_cursor.lnum
                >= (start_visual.lnum + end_visual.lnum) / 2 as linenr_T
            {
                end_visual.lnum = start_visual.lnum;
            }
            start_visual = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = end_visual;
            coladvance(curwin.get(), end_visual.col);
            VIsual.set((*curwin.get()).w_cursor);
            (*curwin.get()).w_cursor = start_visual;
        } else if lt((*curwin.get()).w_cursor, start_visual) {
            VIsual.set(end_visual);
        } else if lt(end_visual, (*curwin.get()).w_cursor) {
            VIsual.set(start_visual);
        } else if end_visual.lnum == start_visual.lnum {
            if (*curwin.get()).w_cursor.col - start_visual.col
                > end_visual.col - (*curwin.get()).w_cursor.col
            {
                VIsual.set(start_visual);
            } else {
                VIsual.set(end_visual);
            }
        } else {
            diff = (*curwin.get()).w_cursor.lnum
                - start_visual.lnum
                - (end_visual.lnum - (*curwin.get()).w_cursor.lnum);
            if diff > 0 as linenr_T {
                VIsual.set(start_visual);
            } else if diff < 0 as linenr_T {
                VIsual.set(end_visual);
            } else if (*curwin.get()).w_cursor.col
                < (start_visual.col as ::core::ffi::c_int + end_visual.col as ::core::ffi::c_int)
                    / 2 as ::core::ffi::c_int
            {
                VIsual.set(end_visual);
            } else {
                VIsual.set(start_visual);
            }
        }
    } else if State.get() & MODE_INSERT as ::core::ffi::c_int != 0
        && VIsual_active.get() as ::core::ffi::c_int != 0
    {
        stuffcharReadbuff(Ctrl_O);
    }
    if which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
        if regname == 0 as ::core::ffi::c_int
            && eval_has_provider(
                b"clipboard\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            ) as ::core::ffi::c_int
                != 0
        {
            regname = '*' as ::core::ffi::c_int;
        }
        let mut reg_0: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
        if yank_register_mline(regname, &raw mut reg_0) {
            if mouse_past_bottom.get() {
                dir = FORWARD as ::core::ffi::c_int;
            }
        } else if mouse_past_eol.get() {
            dir = FORWARD as ::core::ffi::c_int;
        }
        let mut c1: ::core::ffi::c_int = 0;
        let mut c2: ::core::ffi::c_int = 0;
        if fixindent {
            c1 = if dir == BACKWARD as ::core::ffi::c_int {
                '[' as ::core::ffi::c_int
            } else {
                ']' as ::core::ffi::c_int
            };
            c2 = 'p' as ::core::ffi::c_int;
        } else {
            c1 = if dir == FORWARD as ::core::ffi::c_int {
                'p' as ::core::ffi::c_int
            } else {
                'P' as ::core::ffi::c_int
            };
            c2 = NUL;
        }
        prep_redo(regname, count, NUL, c1, NUL, c2, NUL);
        if restart_edit.get() != 0 as ::core::ffi::c_int {
            where_paste_started.set((*curwin.get()).w_cursor);
        }
        do_put(
            regname,
            reg_0,
            dir,
            count,
            (if fixindent as ::core::ffi::c_int != 0 {
                PUT_FIXINDENT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | PUT_CURSEND as ::core::ffi::c_int,
        );
    } else if (mod_mask.get() & MOD_MASK_CTRL != 0
        || mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK)
        && bt_quickfix(curbuf.get()) as ::core::ffi::c_int != 0
    {
        if (*curwin.get()).w_llist_ref.is_null() {
            do_cmdline_cmd(b".cc\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            do_cmdline_cmd(b".ll\0".as_ptr() as *const ::core::ffi::c_char);
        }
        got_click.set(false_0 != 0);
    } else if mod_mask.get() & MOD_MASK_CTRL != 0
        || (*curbuf.get()).b_help as ::core::ffi::c_int != 0
            && mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK
    {
        if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
            stuffcharReadbuff(Ctrl_O);
        }
        stuffcharReadbuff(Ctrl_RSB);
        got_click.set(false_0 != 0);
    } else if mod_mask.get() & MOD_MASK_SHIFT != 0 {
        if State.get() & MODE_INSERT as ::core::ffi::c_int != 0
            || VIsual_active.get() as ::core::ffi::c_int != 0
                && VIsual_select.get() as ::core::ffi::c_int != 0
        {
            stuffcharReadbuff(Ctrl_O);
        }
        if which_button == MOUSE_LEFT as ::core::ffi::c_int {
            stuffcharReadbuff('*' as ::core::ffi::c_int);
        } else {
            stuffcharReadbuff('#' as ::core::ffi::c_int);
        }
    } else if !(in_status_line as ::core::ffi::c_int != 0 || in_sep_line as ::core::ffi::c_int != 0)
    {
        if mod_mask.get() & MOD_MASK_MULTI_CLICK != 0
            && State.get() & (MODE_NORMAL as ::core::ffi::c_int | MODE_INSERT as ::core::ffi::c_int)
                != 0
            && mouse_can_visual as ::core::ffi::c_int != 0
        {
            if is_click as ::core::ffi::c_int != 0 || !VIsual_active.get() {
                if VIsual_active.get() {
                    orig_cursor.set(VIsual.get());
                } else {
                    VIsual.set((*curwin.get()).w_cursor);
                    orig_cursor.set(VIsual.get());
                    VIsual_active.set(true_0 != 0);
                    VIsual_reselect.set(true_0);
                    may_start_select('o' as ::core::ffi::c_int);
                    setmouse();
                }
                if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                    if mod_mask.get() & MOD_MASK_ALT != 0 {
                        VIsual_mode.set(Ctrl_V);
                    } else {
                        VIsual_mode.set('v' as ::core::ffi::c_int);
                    }
                } else if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_3CLICK {
                    VIsual_mode.set('V' as ::core::ffi::c_int);
                } else if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_4CLICK {
                    VIsual_mode.set(Ctrl_V);
                }
            }
            if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
                if is_click {
                    end_visual = (*curwin.get()).w_cursor;
                    let mut gc: ::core::ffi::c_int = 0;
                    loop {
                        gc = gchar_pos(&raw mut end_visual);
                        if !ascii_iswhite(gc) {
                            break;
                        }
                        inc(&raw mut end_visual);
                    }
                    if !oap.is_null() {
                        (*oap).motion_type = kMTCharWise;
                    }
                    if !oap.is_null()
                        && VIsual_mode.get() == 'v' as ::core::ffi::c_int
                        && !vim_iswordc(gchar_pos(&raw mut end_visual))
                        && equalpos((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int
                            != 0
                        && {
                            pos = findmatch(oap, NUL);
                            !pos.is_null()
                        }
                    {
                        (*curwin.get()).w_cursor = *pos;
                        if (*oap).motion_type as ::core::ffi::c_int
                            == kMTLineWise as ::core::ffi::c_int
                        {
                            VIsual_mode.set('V' as ::core::ffi::c_int);
                        } else if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                            if lt((*curwin.get()).w_cursor, VIsual.get()) {
                                (*VIsual.ptr()).col += 1;
                            } else {
                                (*curwin.get()).w_cursor.col += 1;
                            }
                        }
                    }
                }
                if pos.is_null()
                    && (is_click as ::core::ffi::c_int != 0 || is_drag as ::core::ffi::c_int != 0)
                {
                    if lt((*curwin.get()).w_cursor, orig_cursor.get()) {
                        find_start_of_word(&raw mut (*curwin.get()).w_cursor);
                        find_end_of_word(VIsual.ptr());
                    } else {
                        find_start_of_word(VIsual.ptr());
                        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                            && *get_cursor_pos_ptr() as ::core::ffi::c_int != NUL
                        {
                            (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
                        }
                        find_end_of_word(&raw mut (*curwin.get()).w_cursor);
                    }
                }
                (*curwin.get()).w_set_curswant = true_0;
            }
            if is_click {
                redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
            }
        } else if VIsual_active.get() as ::core::ffi::c_int != 0 && old_active == 0 {
            if mod_mask.get() & MOD_MASK_ALT != 0 {
                VIsual_mode.set(Ctrl_V);
            } else {
                VIsual_mode.set('v' as ::core::ffi::c_int);
            }
        }
    }
    if !VIsual_active.get() && old_active != 0 && mode_displayed.get() as ::core::ffi::c_int != 0
        || VIsual_active.get() as ::core::ffi::c_int != 0
            && p_smd.get() != 0
            && msg_silent.get() == 0 as ::core::ffi::c_int
            && (old_active == 0 || VIsual_mode.get() != old_mode)
    {
        redraw_cmdline.set(true_0 != 0);
    }
    return moved;
}
pub unsafe extern "C" fn ins_mouse(mut c: ::core::ffi::c_int) {
    let mut old_curwin: *mut win_T = curwin.get();
    undisplay_dollar();
    let mut tpos: pos_T = (*curwin.get()).w_cursor;
    if do_mouse(
        ::core::ptr::null_mut::<oparg_T>(),
        c,
        BACKWARD as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        false,
    ) {
        let mut new_curwin: *mut win_T = curwin.get();
        if curwin.get() != old_curwin && win_valid(old_curwin) as ::core::ffi::c_int != 0 {
            curwin.set(old_curwin);
            curbuf.set((*curwin.get()).w_buffer);
            if bt_prompt(curbuf.get()) {
                (*curbuf.get()).b_prompt_insert = 'A' as ::core::ffi::c_int;
            }
        }
        start_arrow(if curwin.get() == old_curwin {
            &raw mut tpos
        } else {
            ::core::ptr::null_mut::<pos_T>()
        });
        if curwin.get() != new_curwin && win_valid(new_curwin) as ::core::ffi::c_int != 0 {
            curwin.set(new_curwin);
            curbuf.set((*curwin.get()).w_buffer);
        }
        set_can_cindent(true_0 != 0);
    }
    redraw_statuslines();
}
pub unsafe extern "C" fn do_mousescroll(mut cap: *mut cmdarg_T) {
    let mut shift_or_ctrl: bool = mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0;
    if (*cap).arg == MSCR_UP as ::core::ffi::c_int || (*cap).arg == MSCR_DOWN as ::core::ffi::c_int
    {
        if State.get() & MODE_NORMAL as ::core::ffi::c_int != 0
            && shift_or_ctrl as ::core::ffi::c_int != 0
        {
            pagescroll(
                (if (*cap).arg != 0 {
                    FORWARD as ::core::ffi::c_int
                } else {
                    BACKWARD as ::core::ffi::c_int
                }) as Direction,
                1 as ::core::ffi::c_int,
                false_0 != 0,
            );
        } else {
            if shift_or_ctrl {
                (*cap).count1 =
                    ((*curwin.get()).w_botline - (*curwin.get()).w_topline) as ::core::ffi::c_int;
            } else {
                (*cap).count1 = p_mousescroll_vert.get() as ::core::ffi::c_int;
            }
            if (*cap).count1 > 0 as ::core::ffi::c_int {
                (*cap).count0 = (*cap).count1;
                nv_scroll_line(cap);
            }
        }
    } else {
        let mut step: ::core::ffi::c_int = if shift_or_ctrl as ::core::ffi::c_int != 0 {
            (*curwin.get()).w_view_width
        } else {
            p_mousescroll_hor.get() as ::core::ffi::c_int
        };
        let mut leftcol: colnr_T = (*curwin.get()).w_leftcol
            + (if (*cap).arg == MSCR_RIGHT as ::core::ffi::c_int {
                -(step as colnr_T)
            } else {
                step as colnr_T
            });
        leftcol = (if leftcol > 0 as ::core::ffi::c_int {
            leftcol as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as colnr_T;
        do_mousescroll_horiz(leftcol);
    };
}
pub unsafe extern "C" fn ins_mousescroll(mut dir: ::core::ffi::c_int) {
    let mut cap: cmdarg_T = cmdarg_T {
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
    let mut oa: oparg_T = oparg_T {
        op_type: 0,
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
    memset(
        &raw mut cap as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmdarg_T>(),
    );
    clear_oparg(&raw mut oa);
    cap.oap = &raw mut oa;
    cap.arg = dir;
    match dir {
        1 => {
            cap.cmdchar = -(253 as ::core::ffi::c_int
                + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        0 => {
            cap.cmdchar = -(253 as ::core::ffi::c_int
                + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        -1 => {
            cap.cmdchar = -(253 as ::core::ffi::c_int
                + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        -2 => {
            cap.cmdchar = -(253 as ::core::ffi::c_int
                + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        _ => {
            siemsg(
                b"Invalid ins_mousescroll() argument: %d\0".as_ptr() as *const ::core::ffi::c_char,
                dir,
            );
        }
    }
    let mut old_curwin: *mut win_T = curwin.get();
    if mouse_row.get() >= 0 as ::core::ffi::c_int && mouse_col.get() >= 0 as ::core::ffi::c_int {
        let mut grid: ::core::ffi::c_int = mouse_grid.get();
        let mut row: ::core::ffi::c_int = mouse_row.get();
        let mut col: ::core::ffi::c_int = mouse_col.get();
        curwin.set(mouse_find_win_inner(
            &raw mut grid,
            &raw mut row,
            &raw mut col,
        ));
        if (*curwin.ptr()).is_null() {
            curwin.set(old_curwin);
            return;
        }
        curbuf.set((*curwin.get()).w_buffer);
    }
    if curwin.get() == old_curwin {
        if pum_visible() {
            return;
        }
        undisplay_dollar();
    }
    let mut orig_cursor: pos_T = (*curwin.get()).w_cursor;
    do_mousescroll(&raw mut cap);
    (*curwin.get()).w_redr_status = true_0 != 0;
    curwin.set(old_curwin);
    curbuf.set((*curwin.get()).w_buffer);
    if !equalpos((*curwin.get()).w_cursor, orig_cursor) {
        start_arrow(&raw mut orig_cursor);
        set_can_cindent(true_0 != 0);
    }
}
pub unsafe extern "C" fn is_mouse_key(mut c: ::core::ffi::c_int) -> bool {
    return c
        == -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MIDDLERELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X1MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X1DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X1RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X2MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X2DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X2RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
}
unsafe extern "C" fn mouse_model_popup() -> bool {
    return *(*p_mousem.ptr()).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'p' as ::core::ffi::c_int;
}
static dragwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub unsafe extern "C" fn reset_dragwin() {
    dragwin.set(::core::ptr::null_mut::<win_T>());
}
pub unsafe extern "C" fn jump_to_mouse(
    mut flags: ::core::ffi::c_int,
    mut inclusive: *mut bool,
    mut which_button: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    static status_line_offset: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static sep_line_offset: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static on_status_line: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    static on_sep_line: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    static on_winbar: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    static on_statuscol: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    static prev_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
    static prev_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(-1 as ::core::ffi::c_int);
    static did_drag: GlobalCell<::core::ffi::c_int> = GlobalCell::new(false_0);
    let mut count: ::core::ffi::c_int = 0;
    let mut first: bool = false;
    let mut row: ::core::ffi::c_int = mouse_row.get();
    let mut col: ::core::ffi::c_int = mouse_col.get();
    let mut grid: ::core::ffi::c_int = mouse_grid.get();
    let mut fdc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut keep_focus: bool = flags & MOUSE_FOCUS as ::core::ffi::c_int != 0;
    mouse_past_bottom.set(false_0 != 0);
    mouse_past_eol.set(false_0 != 0);
    if flags & MOUSE_RELEASED as ::core::ffi::c_int != 0 {
        if !(*dragwin.ptr()).is_null() && did_drag.get() == 0 {
            flags &= !(MOUSE_FOCUS as ::core::ffi::c_int | MOUSE_DID_MOVE as ::core::ffi::c_int);
        }
        dragwin.set(::core::ptr::null_mut::<win_T>());
        did_drag.set(false_0);
    }
    if !(flags & MOUSE_DID_MOVE as ::core::ffi::c_int != 0
        && prev_row.get() == mouse_row.get()
        && prev_col.get() == mouse_col.get())
    {
        prev_row.set(mouse_row.get());
        prev_col.set(mouse_col.get());
        if flags & MOUSE_SETPOS as ::core::ffi::c_int == 0 {
            if row < 0 as ::core::ffi::c_int || col < 0 as ::core::ffi::c_int {
                return IN_UNKNOWN as ::core::ffi::c_int;
            }
            let mut wp: *mut win_T =
                mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
            if wp.is_null() {
                return IN_UNKNOWN as ::core::ffi::c_int;
            }
            let mut below_window: bool =
                grid == DEFAULT_GRID_HANDLE && row + (*wp).w_winbar_height >= (*wp).w_height;
            on_status_line.set(
                below_window as ::core::ffi::c_int != 0
                    && row + (*wp).w_winbar_height - (*wp).w_height + 1 as ::core::ffi::c_int
                        == 1 as ::core::ffi::c_int,
            );
            on_sep_line.set(
                grid == DEFAULT_GRID_HANDLE
                    && col >= (*wp).w_width
                    && col - (*wp).w_width + 1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int,
            );
            on_winbar.set(
                row < 0 as ::core::ffi::c_int
                    && row + (*wp).w_winbar_height >= 0 as ::core::ffi::c_int,
            );
            on_statuscol.set(
                !below_window
                    && !on_status_line.get()
                    && !on_sep_line.get()
                    && !on_winbar.get()
                    && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
                    && (if (*wp).w_onebuf_opt.wo_rl != 0 {
                        (col >= (*wp).w_view_width - win_col_off(wp)) as ::core::ffi::c_int
                    } else {
                        (col < win_col_off(wp)) as ::core::ffi::c_int
                    }) != 0,
            );
            if on_status_line.get() as ::core::ffi::c_int != 0
                && on_sep_line.get() as ::core::ffi::c_int != 0
            {
                if stl_connected(wp) {
                    on_sep_line.set(false_0 != 0);
                } else {
                    on_status_line.set(false_0 != 0);
                }
            }
            if keep_focus {
                row = mouse_row.get();
                col = mouse_col.get();
                grid = mouse_grid.get();
            }
            let mut old_curwin: *mut win_T = curwin.get();
            let mut old_cursor: pos_T = (*curwin.get()).w_cursor;
            if !keep_focus {
                if on_winbar.get() {
                    return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_WINBAR as ::core::ffi::c_int;
                }
                if !on_statuscol.get() {
                    fdc = win_fdccol_count(wp);
                    dragwin.set(::core::ptr::null_mut::<win_T>());
                    if below_window {
                        status_line_offset.set(
                            row + (*wp).w_winbar_height - (*wp).w_height + 1 as ::core::ffi::c_int,
                        );
                        dragwin.set(wp);
                    } else {
                        status_line_offset.set(0 as ::core::ffi::c_int);
                    }
                    if grid == DEFAULT_GRID_HANDLE && col >= (*wp).w_width {
                        sep_line_offset.set(col - (*wp).w_width + 1 as ::core::ffi::c_int);
                        dragwin.set(wp);
                    } else {
                        sep_line_offset.set(0 as ::core::ffi::c_int);
                    }
                    if status_line_offset.get() != 0 && sep_line_offset.get() != 0 {
                        if stl_connected(wp) {
                            sep_line_offset.set(0 as ::core::ffi::c_int);
                        } else {
                            status_line_offset.set(0 as ::core::ffi::c_int);
                        }
                    }
                    if VIsual_active.get() as ::core::ffi::c_int != 0
                        && ((*wp).w_buffer != (*curwin.get()).w_buffer
                            || status_line_offset.get() == 0
                                && sep_line_offset.get() == 0
                                && (if (*wp).w_onebuf_opt.wo_rl != 0 {
                                    (col < (*wp).w_view_width - fdc) as ::core::ffi::c_int
                                } else {
                                    (col >= fdc
                                        + (if wp != cmdwin_win.get() {
                                            0 as ::core::ffi::c_int
                                        } else {
                                            1 as ::core::ffi::c_int
                                        }))
                                        as ::core::ffi::c_int
                                }) != 0
                                && flags & MOUSE_MAY_STOP_VIS as ::core::ffi::c_int != 0)
                    {
                        end_visual_mode();
                        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
                    }
                    if cmdwin_type.get() != 0 as ::core::ffi::c_int && wp != cmdwin_win.get() {
                        sep_line_offset.set(0 as ::core::ffi::c_int);
                        row = 0 as ::core::ffi::c_int;
                        col += (*wp).w_wincol;
                        wp = cmdwin_win.get();
                    }
                    if (*dragwin.ptr()).is_null()
                        || flags & MOUSE_RELEASED as ::core::ffi::c_int != 0
                    {
                        win_enter(wp, true_0 != 0);
                    }
                    if curwin.get() != old_curwin {
                        set_mouse_topline(curwin.get());
                    }
                    if status_line_offset.get() != 0 {
                        if curwin.get() == old_curwin {
                            return IN_STATUS_LINE as ::core::ffi::c_int;
                        }
                        return IN_STATUS_LINE as ::core::ffi::c_int
                            | CURSOR_MOVED as ::core::ffi::c_int;
                    }
                    if sep_line_offset.get() != 0 {
                        if curwin.get() == old_curwin {
                            return IN_SEP_LINE as ::core::ffi::c_int;
                        }
                        return IN_SEP_LINE as ::core::ffi::c_int
                            | CURSOR_MOVED as ::core::ffi::c_int;
                    }
                    (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_topline;
                }
            } else if status_line_offset.get() != 0 {
                if which_button == MOUSE_LEFT as ::core::ffi::c_int && !(*dragwin.ptr()).is_null() {
                    count = row - (*dragwin.get()).w_winrow - (*dragwin.get()).w_height
                        + 1 as ::core::ffi::c_int
                        - status_line_offset.get();
                    win_drag_status_line(dragwin.get(), count);
                    (*did_drag.ptr()) |= count;
                }
                return IN_STATUS_LINE as ::core::ffi::c_int;
            } else if sep_line_offset.get() != 0 && which_button == MOUSE_LEFT as ::core::ffi::c_int
            {
                if !(*dragwin.ptr()).is_null() {
                    count = col - (*dragwin.get()).w_wincol - (*dragwin.get()).w_width
                        + 1 as ::core::ffi::c_int
                        - sep_line_offset.get();
                    win_drag_vsep_line(dragwin.get(), count);
                    (*did_drag.ptr()) |= count;
                }
                return IN_SEP_LINE as ::core::ffi::c_int;
            } else if on_status_line.get() as ::core::ffi::c_int != 0
                && which_button == MOUSE_RIGHT as ::core::ffi::c_int
            {
                return IN_STATUS_LINE as ::core::ffi::c_int;
            } else if on_winbar.get() as ::core::ffi::c_int != 0
                && which_button == MOUSE_RIGHT as ::core::ffi::c_int
            {
                return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_WINBAR as ::core::ffi::c_int;
            } else if on_statuscol.get() as ::core::ffi::c_int != 0
                && which_button == MOUSE_RIGHT as ::core::ffi::c_int
            {
                return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_STATUSCOL as ::core::ffi::c_int;
            } else {
                if flags & MOUSE_MAY_STOP_VIS as ::core::ffi::c_int != 0 {
                    end_visual_mode();
                    redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
                }
                if grid == 0 as ::core::ffi::c_int {
                    row -=
                        (*curwin.get()).w_grid_alloc.comp_row + (*curwin.get()).w_grid.row_offset;
                    col -=
                        (*curwin.get()).w_grid_alloc.comp_col + (*curwin.get()).w_grid.col_offset;
                } else if grid != DEFAULT_GRID_HANDLE {
                    row -= (*curwin.get()).w_grid.row_offset;
                    col -= (*curwin.get()).w_grid.col_offset;
                }
                if row < 0 as ::core::ffi::c_int {
                    count = 0 as ::core::ffi::c_int;
                    first = true_0 != 0;
                    while (*curwin.get()).w_topline > 1 as linenr_T {
                        if (*curwin.get()).w_topfill
                            < win_get_fill(curwin.get(), (*curwin.get()).w_topline)
                        {
                            count += 1;
                        } else {
                            count += plines_win(
                                curwin.get(),
                                (*curwin.get()).w_topline - 1 as linenr_T,
                                true_0 != 0,
                            );
                        }
                        if !first && count > -row {
                            break;
                        }
                        first = false_0 != 0;
                        hasFolding(
                            curwin.get(),
                            (*curwin.get()).w_topline,
                            &raw mut (*curwin.get()).w_topline,
                            ::core::ptr::null_mut::<linenr_T>(),
                        );
                        if (*curwin.get()).w_topfill
                            < win_get_fill(curwin.get(), (*curwin.get()).w_topline)
                        {
                            (*curwin.get()).w_topfill += 1;
                        } else {
                            (*curwin.get()).w_topline -= 1;
                            (*curwin.get()).w_topfill = 0 as ::core::ffi::c_int;
                        }
                    }
                    check_topfill(curwin.get(), false_0 != 0);
                    (*curwin.get()).w_valid &=
                        !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
                    redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
                    row = 0 as ::core::ffi::c_int;
                } else if row >= (*curwin.get()).w_view_height {
                    count = 0 as ::core::ffi::c_int;
                    first = true_0 != 0;
                    while (*curwin.get()).w_topline < (*curbuf.get()).b_ml.ml_line_count {
                        if (*curwin.get()).w_topfill > 0 as ::core::ffi::c_int {
                            count += 1;
                        } else {
                            count +=
                                plines_win(curwin.get(), (*curwin.get()).w_topline, true_0 != 0);
                        }
                        if !first
                            && count > row - (*curwin.get()).w_view_height + 1 as ::core::ffi::c_int
                        {
                            break;
                        }
                        first = false_0 != 0;
                        if (*curwin.get()).w_topfill > 0 as ::core::ffi::c_int {
                            (*curwin.get()).w_topfill -= 1;
                        } else {
                            if hasFolding(
                                curwin.get(),
                                (*curwin.get()).w_topline,
                                ::core::ptr::null_mut::<linenr_T>(),
                                &raw mut (*curwin.get()).w_topline,
                            ) as ::core::ffi::c_int
                                != 0
                                && (*curwin.get()).w_topline == (*curbuf.get()).b_ml.ml_line_count
                            {
                                break;
                            }
                            (*curwin.get()).w_topline += 1;
                            (*curwin.get()).w_topfill =
                                win_get_fill(curwin.get(), (*curwin.get()).w_topline);
                        }
                    }
                    check_topfill(curwin.get(), false_0 != 0);
                    redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
                    (*curwin.get()).w_valid &=
                        !(VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP);
                    row = (*curwin.get()).w_view_height - 1 as ::core::ffi::c_int;
                } else if row == 0 as ::core::ffi::c_int {
                    if mouse_dragging.get() > 0 as ::core::ffi::c_int
                        && (*curwin.get()).w_cursor.lnum
                            == (*(*curwin.get()).w_buffer).b_ml.ml_line_count
                        && (*curwin.get()).w_cursor.lnum == (*curwin.get()).w_topline
                    {
                        (*curwin.get()).w_valid &= !VALID_TOPLINE;
                    }
                }
            }
            let mut col_from_screen: colnr_T = -1 as colnr_T;
            let mut mouse_fold_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            mouse_check_grid(&raw mut col_from_screen, &raw mut mouse_fold_flags);
            if mouse_comp_pos(
                curwin.get(),
                &raw mut row,
                &raw mut col,
                &raw mut (*curwin.get()).w_cursor.lnum,
            ) {
                mouse_past_bottom.set(true_0 != 0);
            }
            if flags & MOUSE_MAY_VIS as ::core::ffi::c_int != 0 && !VIsual_active.get() {
                VIsual.set(old_cursor);
                VIsual_active.set(true_0 != 0);
                VIsual_reselect.set(true_0);
                may_start_select('o' as ::core::ffi::c_int);
                setmouse();
                if p_smd.get() != 0 && msg_silent.get() == 0 as ::core::ffi::c_int {
                    redraw_cmdline.set(true_0 != 0);
                }
            }
            if col_from_screen >= 0 as ::core::ffi::c_int {
                col = col_from_screen as ::core::ffi::c_int;
            }
            (*curwin.get()).w_curswant = col as colnr_T;
            (*curwin.get()).w_set_curswant = false_0;
            if coladvance(curwin.get(), col as colnr_T) == FAIL {
                if !inclusive.is_null() {
                    *inclusive = true_0 != 0;
                }
                mouse_past_eol.set(true_0 != 0);
            } else if !inclusive.is_null() {
                *inclusive = false_0 != 0;
            }
            count = if on_statuscol.get() as ::core::ffi::c_int != 0 {
                IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_STATUSCOL as ::core::ffi::c_int
            } else {
                IN_BUFFER as ::core::ffi::c_int
            };
            if curwin.get() != old_curwin
                || (*curwin.get()).w_cursor.lnum != old_cursor.lnum
                || (*curwin.get()).w_cursor.col != old_cursor.col
            {
                count |= CURSOR_MOVED as ::core::ffi::c_int;
            }
            count |= mouse_fold_flags;
            return count;
        }
    }
    if status_line_offset.get() != 0 {
        return IN_STATUS_LINE as ::core::ffi::c_int;
    }
    if sep_line_offset.get() != 0 {
        return IN_SEP_LINE as ::core::ffi::c_int;
    }
    if on_winbar.get() {
        return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_WINBAR as ::core::ffi::c_int;
    }
    if on_statuscol.get() {
        return IN_OTHER_WIN as ::core::ffi::c_int | MOUSE_STATUSCOL as ::core::ffi::c_int;
    }
    if flags & MOUSE_MAY_STOP_VIS as ::core::ffi::c_int != 0 {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    return IN_BUFFER as ::core::ffi::c_int;
}
unsafe extern "C" fn do_mousescroll_horiz(mut leftcol: colnr_T) -> bool {
    if (*curwin.get()).w_onebuf_opt.wo_wrap != 0 {
        return false_0 != 0;
    }
    if (*curwin.get()).w_leftcol == leftcol {
        return false_0 != 0;
    }
    if !virtual_active(curwin.get()) && leftcol > scroll_line_len((*curwin.get()).w_cursor.lnum) {
        (*curwin.get()).w_cursor.lnum = find_longest_lnum();
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    return set_leftcol(leftcol);
}
pub unsafe extern "C" fn nv_mousescroll(mut cap: *mut cmdarg_T) {
    let old_curwin: *mut win_T = curwin.get();
    if mouse_row.get() >= 0 as ::core::ffi::c_int && mouse_col.get() >= 0 as ::core::ffi::c_int {
        let mut grid: ::core::ffi::c_int = mouse_grid.get();
        let mut row: ::core::ffi::c_int = mouse_row.get();
        let mut col: ::core::ffi::c_int = mouse_col.get();
        curwin.set(mouse_find_win_inner(
            &raw mut grid,
            &raw mut row,
            &raw mut col,
        ));
        if (*curwin.ptr()).is_null() {
            curwin.set(old_curwin);
            return;
        }
        curbuf.set((*curwin.get()).w_buffer);
    }
    do_mousescroll(cap);
    (*curwin.get()).w_redr_status = true_0 != 0;
    curwin.set(old_curwin);
    curbuf.set((*curwin.get()).w_buffer);
}
pub unsafe extern "C" fn nv_mouse(mut cap: *mut cmdarg_T) {
    do_mouse(
        (*cap).oap,
        (*cap).cmdchar,
        BACKWARD as ::core::ffi::c_int,
        (*cap).count1,
        false,
    );
}
pub unsafe extern "C" fn mouse_comp_pos(
    mut win: *mut win_T,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
    mut lnump: *mut linenr_T,
) -> bool {
    let mut col: ::core::ffi::c_int = *colp;
    let mut row: ::core::ffi::c_int = *rowp;
    let mut retval: bool = false_0 != 0;
    let mut count: ::core::ffi::c_int = 0;
    if (*win).w_onebuf_opt.wo_rl != 0 {
        col = (*win).w_view_width - 1 as ::core::ffi::c_int - col;
    }
    let mut lnum: linenr_T = (*win).w_topline;
    while row > 0 as ::core::ffi::c_int {
        if win_may_fill(win) {
            row -= if lnum == (*win).w_topline {
                (*win).w_topfill
            } else {
                win_get_fill(win, lnum)
            };
            count = plines_win_nofill(win, lnum, false_0 != 0);
        } else {
            count = plines_win(win, lnum, false_0 != 0);
        }
        if (*win).w_skipcol > 0 as ::core::ffi::c_int && lnum == (*win).w_topline {
            let mut width1: ::core::ffi::c_int = (*win).w_view_width - win_col_off(win);
            if width1 > 0 as ::core::ffi::c_int {
                let mut skip_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if (*win).w_skipcol > width1 {
                    skip_lines = ((*win).w_skipcol as ::core::ffi::c_int - width1)
                        / (width1 + win_col_off2(win))
                        + 1 as ::core::ffi::c_int;
                } else if (*win).w_skipcol > 0 as ::core::ffi::c_int {
                    skip_lines = 1 as ::core::ffi::c_int;
                }
                count -= skip_lines;
            }
        }
        if count > row {
            break;
        }
        hasFolding(
            win,
            lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut lnum,
        );
        if lnum == (*(*win).w_buffer).b_ml.ml_line_count {
            retval = true_0 != 0;
            break;
        } else {
            row -= count;
            lnum += 1;
        }
    }
    while lnum < (*(*win).w_buffer).b_ml.ml_line_count
        && decor_conceal_line(
            win,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            false_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        lnum += 1;
        hasFolding(
            win,
            lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut lnum,
        );
    }
    if !retval {
        let mut off: ::core::ffi::c_int = win_col_off(win) - win_col_off2(win);
        col = if col > off { col } else { off };
        col += row * ((*win).w_view_width - off);
        if lnum == (*win).w_topline {
            col += (*win).w_skipcol as ::core::ffi::c_int;
        }
    }
    if (*win).w_onebuf_opt.wo_wrap == 0 {
        col += (*win).w_leftcol as ::core::ffi::c_int;
    }
    col -= win_col_off(win);
    col = if col > 0 as ::core::ffi::c_int {
        col
    } else {
        0 as ::core::ffi::c_int
    };
    *colp = col;
    *rowp = row;
    *lnump = lnum;
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn mouse_find_win_inner(
    mut gridp: *mut ::core::ffi::c_int,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
) -> *mut win_T {
    let mut wp_grid: *mut win_T = mouse_find_grid_win(gridp, rowp, colp);
    if !wp_grid.is_null() {
        return wp_grid;
    } else if *gridp > 1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<win_T>();
    }
    let mut fp: *mut frame_T = topframe.get();
    *rowp -= (*firstwin.get()).w_winrow;
    while (*fp).fr_layout as ::core::ffi::c_int != FR_LEAF {
        if (*fp).fr_layout as ::core::ffi::c_int == FR_ROW {
            fp = (*fp).fr_child;
            while !(*fp).fr_next.is_null() {
                if *colp < (*fp).fr_width {
                    break;
                }
                *colp -= (*fp).fr_width;
                fp = (*fp).fr_next;
            }
        } else {
            fp = (*fp).fr_child;
            while !(*fp).fr_next.is_null() {
                if *rowp < (*fp).fr_height {
                    break;
                }
                *rowp -= (*fp).fr_height;
                fp = (*fp).fr_next;
            }
        }
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if wp == (*fp).fr_win {
            *rowp -= (*wp).w_winbar_height;
            return wp;
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
pub unsafe extern "C" fn mouse_find_win_outer(
    mut gridp: *mut ::core::ffi::c_int,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
) -> *mut win_T {
    let mut wp: *mut win_T = mouse_find_win_inner(gridp, rowp, colp);
    if !wp.is_null() {
        *rowp += (*wp).w_winrow_off;
        *colp += (*wp).w_wincol_off;
    }
    return wp;
}
unsafe extern "C" fn mouse_find_grid_win(
    mut gridp: *mut ::core::ffi::c_int,
    mut rowp: *mut ::core::ffi::c_int,
    mut colp: *mut ::core::ffi::c_int,
) -> *mut win_T {
    if *gridp == (*msg_grid.ptr()).handle {
        *rowp += msg_grid_pos.get();
        *gridp = DEFAULT_GRID_HANDLE;
    } else if *gridp > 1 as ::core::ffi::c_int {
        let mut wp: *mut win_T = get_win_by_grid_handle(*gridp as handle_T);
        if !wp.is_null()
            && !(*wp).w_grid_alloc.chars.is_null()
            && !((*wp).w_floating as ::core::ffi::c_int != 0 && !(*wp).w_config.mouse)
        {
            *rowp = if *rowp - (*wp).w_grid.row_offset
                < (*wp).w_view_height - 1 as ::core::ffi::c_int
            {
                *rowp - (*wp).w_grid.row_offset
            } else {
                (*wp).w_view_height - 1 as ::core::ffi::c_int
            };
            *colp =
                if *colp - (*wp).w_grid.col_offset < (*wp).w_view_width - 1 as ::core::ffi::c_int {
                    *colp - (*wp).w_grid.col_offset
                } else {
                    (*wp).w_view_width - 1 as ::core::ffi::c_int
                };
            return wp;
        }
    } else if *gridp == 0 as ::core::ffi::c_int {
        let mut grid: *mut ScreenGrid = ui_comp_mouse_focus(*rowp, *colp);
        if grid == pum_grid.ptr() {
            *gridp = (*grid).handle as ::core::ffi::c_int;
            *rowp -= (*grid).comp_row;
            *colp -= (*grid).comp_col;
            return ::core::ptr::null_mut::<win_T>();
        } else {
            let mut wp_0: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp_0.is_null() {
                if &raw mut (*wp_0).w_grid_alloc != grid {
                    wp_0 = (*wp_0).w_next;
                } else {
                    *gridp = (*grid).handle as ::core::ffi::c_int;
                    *rowp -= (*wp_0).w_winrow + (*wp_0).w_grid.row_offset;
                    *colp -= (*wp_0).w_wincol + (*wp_0).w_grid.col_offset;
                    return wp_0;
                }
            }
        }
        *gridp = DEFAULT_GRID_HANDLE;
    }
    return ::core::ptr::null_mut::<win_T>();
}
pub unsafe extern "C" fn vcol2col(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut vcol: colnr_T,
    mut coladdp: *mut colnr_T,
) -> colnr_T {
    let mut line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
    let mut csarg: CharsizeArg = CharsizeArg {
        win: ::core::ptr::null_mut::<win_T>(),
        line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        use_tabstop: false,
        indent_width: 0,
        virt_row: 0,
        cur_text_width_left: 0,
        cur_text_width_right: 0,
        max_head_vcol: 0,
        iter: [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_13 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1],
    };
    let mut cstype: CSType = init_charsize_arg(&raw mut csarg, wp, lnum, line);
    let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
    let mut cur_vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while cur_vcol < vcol && *ci.ptr as ::core::ffi::c_int != NUL {
        let mut next_vcol: ::core::ffi::c_int =
            cur_vcol + win_charsize(cstype, cur_vcol, ci.ptr, ci.chr.value, &raw mut csarg).width;
        if next_vcol > vcol {
            break;
        }
        cur_vcol = next_vcol;
        ci = utfc_next(ci);
    }
    if !coladdp.is_null() {
        *coladdp = (vcol as ::core::ffi::c_int - cur_vcol) as colnr_T;
    }
    return ci.ptr.offset_from(line) as colnr_T;
}
pub unsafe extern "C" fn setmouse() {
    ui_cursor_shape();
    ui_check_mouse();
}
unsafe extern "C" fn set_mouse_topline(mut wp: *mut win_T) {
    orig_topline.set((*wp).w_topline);
    orig_topfill.set((*wp).w_topfill);
}
unsafe extern "C" fn scroll_line_len(mut lnum: linenr_T) -> colnr_T {
    let mut col: colnr_T = 0 as colnr_T;
    let mut line: *mut ::core::ffi::c_char = ml_get(lnum);
    if *line as ::core::ffi::c_int != NUL {
        loop {
            let mut numchar: ::core::ffi::c_int = win_chartabsize(curwin.get(), line, col);
            line = line.offset(utfc_ptr2len(line) as isize);
            if *line as ::core::ffi::c_int == NUL {
                break;
            }
            col += numchar;
        }
    }
    return col;
}
unsafe extern "C" fn find_longest_lnum() -> linenr_T {
    let mut ret: linenr_T = 0 as linenr_T;
    if (*curwin.get()).w_topline <= (*curwin.get()).w_cursor.lnum
        && (*curwin.get()).w_botline > (*curwin.get()).w_cursor.lnum
        && (*curwin.get()).w_botline <= (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T
    {
        let mut max: colnr_T = 0 as colnr_T;
        let mut lnum: linenr_T = (*curwin.get()).w_topline;
        while lnum < (*curwin.get()).w_botline {
            let mut len: colnr_T = scroll_line_len(lnum);
            if len > max {
                max = len;
                ret = lnum;
            } else if len == max
                && abs(lnum as ::core::ffi::c_int
                    - (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int)
                    < abs(ret as ::core::ffi::c_int
                        - (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int)
            {
                ret = lnum;
            }
            lnum += 1;
        }
    } else {
        ret = (*curwin.get()).w_cursor.lnum;
    }
    return ret;
}
unsafe extern "C" fn mouse_check_grid(
    mut vcolp: *mut colnr_T,
    mut flagsp: *mut ::core::ffi::c_int,
) {
    let mut click_grid: ::core::ffi::c_int = mouse_grid.get();
    let mut click_row: ::core::ffi::c_int = mouse_row.get();
    let mut click_col: ::core::ffi::c_int = mouse_col.get();
    if mouse_find_win_inner(&raw mut click_grid, &raw mut click_row, &raw mut click_col)
        != curwin.get()
        || (*curwin.get()).w_redr_type != 0 as ::core::ffi::c_int
    {
        return;
    }
    let mut start_row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut start_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut gp: *mut ScreenGrid = grid_adjust(
        &raw mut (*curwin.get()).w_grid,
        &raw mut start_row,
        &raw mut start_col,
    );
    if (*gp).handle != click_grid || (*gp).chars.is_null() {
        return;
    }
    click_row += start_row;
    click_col += start_col;
    if click_row < 0 as ::core::ffi::c_int
        || click_row >= (*gp).rows
        || click_col < 0 as ::core::ffi::c_int
        || click_col >= (*gp).cols
    {
        return;
    }
    let off: size_t =
        (*(*gp).line_offset.offset(click_row as isize)).wrapping_add(click_col as size_t);
    let mut col_from_screen: colnr_T = *(*gp).vcols.offset(off as isize);
    if col_from_screen >= 0 as ::core::ffi::c_int {
        *vcolp = col_from_screen;
    }
    if col_from_screen == -2 as ::core::ffi::c_int {
        *flagsp |= MOUSE_FOLD_OPEN as ::core::ffi::c_int;
    } else if col_from_screen == -3 as ::core::ffi::c_int {
        *flagsp |= MOUSE_FOLD_CLOSE as ::core::ffi::c_int;
    }
}
pub unsafe extern "C" fn f_getmousepos(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut row: ::core::ffi::c_int = mouse_row.get();
    let mut col: ::core::ffi::c_int = mouse_col.get();
    let mut grid: ::core::ffi::c_int = mouse_grid.get();
    let mut winid: varnumber_T = 0 as varnumber_T;
    let mut winrow: varnumber_T = 0 as varnumber_T;
    let mut wincol: varnumber_T = 0 as varnumber_T;
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut column: varnumber_T = 0 as varnumber_T;
    let mut coladd: colnr_T = 0 as colnr_T;
    tv_dict_alloc_ret(rettv);
    let mut d: *mut dict_T = (*rettv).vval.v_dict;
    tv_dict_add_nr(
        d,
        b"screenrow\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        mouse_row.get() as varnumber_T + 1 as varnumber_T,
    );
    tv_dict_add_nr(
        d,
        b"screencol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        mouse_col.get() as varnumber_T + 1 as varnumber_T,
    );
    let mut wp: *mut win_T = mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
    if !wp.is_null() {
        let mut height: ::core::ffi::c_int =
            (*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height;
        if row < height + (*wp).w_border_adj[2 as ::core::ffi::c_int as usize] {
            winid = (*wp).handle as varnumber_T;
            winrow = (row + 1 as ::core::ffi::c_int + (*wp).w_winrow_off) as varnumber_T;
            wincol = (col + 1 as ::core::ffi::c_int + (*wp).w_wincol_off) as varnumber_T;
            if row >= 0 as ::core::ffi::c_int
                && row < (*wp).w_height
                && col >= 0 as ::core::ffi::c_int
                && col < (*wp).w_width
            {
                mouse_comp_pos(wp, &raw mut row, &raw mut col, &raw mut lnum);
                col = vcol2col(wp, lnum, col as colnr_T, &raw mut coladd) as ::core::ffi::c_int;
                column = (col + 1 as ::core::ffi::c_int) as varnumber_T;
            }
        }
    }
    tv_dict_add_nr(
        d,
        b"winid\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        winid,
    );
    tv_dict_add_nr(
        d,
        b"winrow\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        winrow,
    );
    tv_dict_add_nr(
        d,
        b"wincol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        wincol,
    );
    tv_dict_add_nr(
        d,
        b"line\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        lnum as varnumber_T,
    );
    tv_dict_add_nr(
        d,
        b"column\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        column,
    );
    tv_dict_add_nr(
        d,
        b"coladd\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        coladd as varnumber_T,
    );
}
#[inline(always)]
unsafe extern "C" fn win_charsize(
    mut cstype: CSType,
    mut vcol: ::core::ffi::c_int,
    mut ptr: *mut ::core::ffi::c_char,
    mut chr: int32_t,
    mut csarg: *mut CharsizeArg,
) -> CharSize {
    if cstype as ::core::ffi::c_int == kCharsizeFast as ::core::ffi::c_int {
        return charsize_fast(csarg, ptr, vcol as colnr_T, chr);
    } else {
        return charsize_regular(csarg, ptr, vcol as colnr_T, chr);
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
