use crate::src::nvim::autocmd::apply_autocmds;
use crate::src::nvim::buffer::maketitle;
use crate::src::nvim::charset::{vim_isprintc, vim_strsize};
use crate::src::nvim::cmdexpand::cmdline_pum_display;
use crate::src::nvim::decoration::{
    buf_signcols_count_range, decor_conceal_line, decor_range_add_virt, decor_redraw_reset,
    decor_virt_lines, win_lines_concealed,
};
use crate::src::nvim::decoration_provider::{
    decor_providers_invoke_buf, decor_providers_invoke_end, decor_providers_invoke_win,
    decor_providers_start,
};
use crate::src::nvim::diff::diff_redraw;
use crate::src::nvim::digraph::keymap_str;
use crate::src::nvim::drawline::win_line;
use crate::src::nvim::eval::vars::set_vim_var_nr;
use crate::src::nvim::ex_getln::{
    cmdline_screen_cleared, compute_cmdrow, get_cmdline_info, redrawcmdline,
};
use crate::src::nvim::fold::{fold_info, foldmethodIsSyntax, hasAnyFolding, hasFolding};
use crate::src::nvim::getchar::char_avail;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{
    grid_adjust, grid_alloc, grid_clear, grid_clear_line, grid_del_lines, grid_draw_border,
    grid_ins_lines, grid_invalidate, grid_line_clear_end, grid_line_fill, grid_line_flush,
    grid_line_getchar, grid_line_mirror, grid_line_put_schar, grid_line_start,
    schar_cache_clear_if_full, win_grid_alloc,
};
use crate::src::nvim::highlight::{
    hl_combine_attr, update_window_hl, win_bg_attr, win_check_ns_hl,
};
use crate::src::nvim::highlight_group::highlight_changed;
use crate::src::nvim::insexpand::ins_compl_show_pum;
use crate::src::nvim::main::{
    clear_cmdline, cmdline_row, cmdline_was_last_drawn, curbuf, curtab, curwin, decor_state,
    default_grid, default_gridview, display_tick, do_redraw, dollar_vcol, dy_flags, edit_submode,
    edit_submode_extra, edit_submode_highl, edit_submode_pre, exiting, exmode_active,
    first_tabpage, firstwin, global_busy, got_int, hl_attr_active, lines_left, mode_displayed,
    msg_col, msg_did_scroll, msg_didany, msg_didout, msg_grid, msg_grid_scroll_discount,
    msg_no_more, msg_row, msg_scrolled, msg_scrolled_at_flush, msg_silent, must_redraw,
    must_redraw_pum, need_diff_redraw, need_highlight_changed, need_maketitle, need_wait_return,
    no_hlsearch, ns_hl_fast, p_ch, p_columns, p_cpo, p_hls, p_icon, p_lines, p_lz, p_paste, p_rdt,
    p_ri, p_ru, p_sc, p_sloc, p_smd, p_title, p_wbr, p_wmw, redraw_cmdline, redraw_mode,
    redraw_not_allowed, redraw_tabline, reg_recording, resizing_screen, restart_edit, ru_col,
    ru_wid, sc_col, screen_search_hl, search_hl_has_cursor_lnum, starting, stl_syntax,
    tab_page_click_defs, tab_page_click_defs_size, updating_screen, Columns, KeyTyped, NameBuff,
    RedrawingDisabled, Rows, State, VIsual, VIsual_active, VIsual_mode, VIsual_select,
};
use crate::src::nvim::mbyte::{utf_ptr2cells, utf_ptr2char};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::message::{
    msg_check_for_delay, msg_clr_cmdline, msg_clr_eos, msg_ext_flush_showmode, msg_ext_ui_flush,
    msg_grid_set_pos, msg_grid_validate, msg_puts_hl, msg_reset_scroll, msg_scrollsize,
    msg_use_grid, repeat_message,
};
use crate::src::nvim::normal::{clear_showcmd, do_check_scrollbind};
use crate::src::nvim::option::{get_ve_flags, shortmess};
use crate::src::nvim::os::libc::{__assert_fail, abs, gettext, snprintf};
use crate::src::nvim::plines::{
    getvcols, getvvcol, plines_m_win, plines_win, win_get_fill, win_may_fill,
};
use crate::src::nvim::popupmenu::{pum_check_clear, pum_drawn, pum_invalidate, pum_redraw};
use crate::src::nvim::profile::profile_setlimit;
use crate::src::nvim::r#match::{init_search_hl, prepare_search_hl};
use crate::src::nvim::r#move::{
    changed_line_abv_curs, changed_line_abv_curs_win, changed_window_setting, curs_columns,
    invalidate_botline_win, plines_correct_topline, set_empty_rows, update_curswant,
    update_topline, validate_cursor, validate_virtcol, win_col_off, win_col_off2,
};
use crate::src::nvim::search::last_pat_prog;
use crate::src::nvim::spell::spell_check_window;
use crate::src::nvim::state::get_real_state;
use crate::src::nvim::statusline::{
    draw_tabline, redraw_ruler, stl_alloc_click_defs, stl_clear_click_defs, win_redr_status,
    win_redr_winbar,
};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::syntax::{
    syn_set_timeout, syn_stack_apply_changes, syntax_check_changed, syntax_end_parsing,
    syntax_present,
};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, CmdRedraw, CmdlineColorChunk,
    CmdlineColors, CmdlineInfo, ColoredCmdline, DecorExt, DecorHighlightInline, DecorInlineData,
    DecorPriority, DecorPriorityInternal, DecorRange, DecorRangeKind, DecorRangeSlot,
    DecorRange_data as C2Rust_Unnamed_19, DecorRange_data_ui as C2Rust_Unnamed_20,
    DecorSignHighlight, DecorState, DecorState_ranges_i as C2Rust_Unnamed_21,
    DecorState_slots as C2Rust_Unnamed_22, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2,
    Direction, ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Integer,
    Intersection, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, MarkTreeIter,
    MarkTreeIter_s as C2Rust_Unnamed_15, MetaIndex, OptInt, ScopeDictDictItem, ScopeType,
    ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal, Timestamp, TriState, UIExtension,
    VarLockStatus, VarType, VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinExtmark, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, auto_event,
    bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, cmdline_info, colnr_T,
    dict_T, dictvar_S, diff_T, diffblock_S, disptick_T, event_T, expand_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, foldinfo_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, hlf_T, infoptr_T, int16_t, int32_t, int64_t,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T,
    ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T,
    schar_T, scid_T, sctx_T, size_t, spellvars_T, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, xfmark_T, xp_prefix_T, NS, QUEUE,
};
use crate::src::nvim::ui::{
    ui_call_grid_clear, ui_call_grid_resize, ui_call_msg_clear, ui_call_win_extmark, ui_flush,
    ui_grid_cursor_goto, ui_has,
};
use crate::src::nvim::ui_compositor::ui_comp_set_screen_valid;
use crate::src::nvim::version::{intro_message, may_show_intro};
use crate::src::nvim::window::{
    frame2win, global_stl_height, last_stl_height, min_rows, min_rows_for_all_tabpages,
    win_fdccol_count, win_new_screensize, win_ui_flush,
};
extern "C" {
    static win_extmark_arr: GlobalCell<C2Rust_Unnamed_23>;
    fn re_multiline(prog: *const regprog_T) -> ::core::ffi::c_int;
    fn vim_regfree(prog: *mut regprog_T);
    fn terminal_check_size(term: *mut Terminal);
    fn terminal_suspended(term: *const Terminal) -> bool;
}
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const SIGN_WIDTH: C2Rust_Unnamed = 2;
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
pub const MAXLNUM: C2Rust_Unnamed_13 = 2147483647;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_14 = 2147483647;
pub const HLF_COUNT: hlf_T = 76;
pub const HLF_PRE: hlf_T = 75;
pub const HLF_OK: hlf_T = 74;
pub const HLF_SO: hlf_T = 73;
pub const HLF_SE: hlf_T = 72;
pub const HLF_TSNC: hlf_T = 71;
pub const HLF_TS: hlf_T = 70;
pub const HLF_BFOOTER: hlf_T = 69;
pub const HLF_BTITLE: hlf_T = 68;
pub const HLF_CU: hlf_T = 67;
pub const HLF_WBRNC: hlf_T = 66;
pub const HLF_WBR: hlf_T = 65;
pub const HLF_BORDER: hlf_T = 64;
pub const HLF_MSG: hlf_T = 63;
pub const HLF_NFLOAT: hlf_T = 62;
pub const HLF_MSGSEP: hlf_T = 61;
pub const HLF_INACTIVE: hlf_T = 60;
pub const HLF_0: hlf_T = 59;
pub const HLF_QFL: hlf_T = 58;
pub const HLF_MC: hlf_T = 57;
pub const HLF_CUL: hlf_T = 56;
pub const HLF_CUC: hlf_T = 55;
pub const HLF_TPF: hlf_T = 54;
pub const HLF_TPS: hlf_T = 53;
pub const HLF_TP: hlf_T = 52;
pub const HLF_PBR: hlf_T = 51;
pub const HLF_PST: hlf_T = 50;
pub const HLF_PSB: hlf_T = 49;
pub const HLF_PSX: hlf_T = 48;
pub const HLF_PNX: hlf_T = 47;
pub const HLF_PSK: hlf_T = 46;
pub const HLF_PNK: hlf_T = 45;
pub const HLF_PMSI: hlf_T = 44;
pub const HLF_PMNI: hlf_T = 43;
pub const HLF_PSI: hlf_T = 42;
pub const HLF_PNI: hlf_T = 41;
pub const HLF_SPL: hlf_T = 40;
pub const HLF_SPR: hlf_T = 39;
pub const HLF_SPC: hlf_T = 38;
pub const HLF_SPB: hlf_T = 37;
pub const HLF_CONCEAL: hlf_T = 36;
pub const HLF_SC: hlf_T = 35;
pub const HLF_TXA: hlf_T = 34;
pub const HLF_TXD: hlf_T = 33;
pub const HLF_DED: hlf_T = 32;
pub const HLF_CHD: hlf_T = 31;
pub const HLF_ADD: hlf_T = 30;
pub const HLF_FC: hlf_T = 29;
pub const HLF_FL: hlf_T = 28;
pub const HLF_WM: hlf_T = 27;
pub const HLF_W: hlf_T = 26;
pub const HLF_VNC: hlf_T = 25;
pub const HLF_V: hlf_T = 24;
pub const HLF_T: hlf_T = 23;
pub const HLF_VSP: hlf_T = 22;
pub const HLF_C: hlf_T = 21;
pub const HLF_SNC: hlf_T = 20;
pub const HLF_S: hlf_T = 19;
pub const HLF_R: hlf_T = 18;
pub const HLF_CLF: hlf_T = 17;
pub const HLF_CLS: hlf_T = 16;
pub const HLF_CLN: hlf_T = 15;
pub const HLF_LNB: hlf_T = 14;
pub const HLF_LNA: hlf_T = 13;
pub const HLF_N: hlf_T = 12;
pub const HLF_CM: hlf_T = 11;
pub const HLF_M: hlf_T = 10;
pub const HLF_LC: hlf_T = 9;
pub const HLF_L: hlf_T = 8;
pub const HLF_I: hlf_T = 7;
pub const HLF_E: hlf_T = 6;
pub const HLF_D: hlf_T = 5;
pub const HLF_AT: hlf_T = 4;
pub const HLF_TERM: hlf_T = 3;
pub const HLF_EOB: hlf_T = 2;
pub const HLF_8: hlf_T = 1;
pub const HLF_NONE: hlf_T = 0;
pub const kMTMetaCount: MetaIndex = 5;
pub const kMTMetaConcealLines: MetaIndex = 4;
pub const kMTMetaSignText: MetaIndex = 3;
pub const kMTMetaSignHL: MetaIndex = 2;
pub const kMTMetaLines: MetaIndex = 1;
pub const kMTMetaInline: MetaIndex = 0;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub const NUM_EVENTS: auto_event = 145;
pub const EVENT_WINSCROLLED: auto_event = 144;
pub const EVENT_WINRESIZED: auto_event = 143;
pub const EVENT_WINNEWPRE: auto_event = 142;
pub const EVENT_WINNEW: auto_event = 141;
pub const EVENT_WINLEAVE: auto_event = 140;
pub const EVENT_WINENTER: auto_event = 139;
pub const EVENT_WINCLOSED: auto_event = 138;
pub const EVENT_VIMSUSPEND: auto_event = 137;
pub const EVENT_VIMRESUME: auto_event = 136;
pub const EVENT_VIMRESIZED: auto_event = 135;
pub const EVENT_VIMLEAVEPRE: auto_event = 134;
pub const EVENT_VIMLEAVE: auto_event = 133;
pub const EVENT_VIMENTER: auto_event = 132;
pub const EVENT_USER: auto_event = 131;
pub const EVENT_UILEAVE: auto_event = 130;
pub const EVENT_UIENTER: auto_event = 129;
pub const EVENT_TEXTYANKPOST: auto_event = 128;
pub const EVENT_TEXTCHANGEDT: auto_event = 127;
pub const EVENT_TEXTCHANGEDP: auto_event = 126;
pub const EVENT_TEXTCHANGEDI: auto_event = 125;
pub const EVENT_TEXTCHANGED: auto_event = 124;
pub const EVENT_TERMRESPONSE: auto_event = 123;
pub const EVENT_TERMREQUEST: auto_event = 122;
pub const EVENT_TERMOPEN: auto_event = 121;
pub const EVENT_TERMLEAVE: auto_event = 120;
pub const EVENT_TERMENTER: auto_event = 119;
pub const EVENT_TERMCLOSE: auto_event = 118;
pub const EVENT_TERMCHANGED: auto_event = 117;
pub const EVENT_TABNEWENTERED: auto_event = 116;
pub const EVENT_TABNEW: auto_event = 115;
pub const EVENT_TABLEAVE: auto_event = 114;
pub const EVENT_TABENTER: auto_event = 113;
pub const EVENT_TABCLOSEDPRE: auto_event = 112;
pub const EVENT_TABCLOSED: auto_event = 111;
pub const EVENT_SYNTAX: auto_event = 110;
pub const EVENT_SWAPEXISTS: auto_event = 109;
pub const EVENT_STDINREADPRE: auto_event = 108;
pub const EVENT_STDINREADPOST: auto_event = 107;
pub const EVENT_SPELLFILEMISSING: auto_event = 106;
pub const EVENT_SOURCEPRE: auto_event = 105;
pub const EVENT_SOURCEPOST: auto_event = 104;
pub const EVENT_SOURCECMD: auto_event = 103;
pub const EVENT_SIGNAL: auto_event = 102;
pub const EVENT_SHELLFILTERPOST: auto_event = 101;
pub const EVENT_SHELLCMDPOST: auto_event = 100;
pub const EVENT_SESSIONWRITEPOST: auto_event = 99;
pub const EVENT_SESSIONLOADPRE: auto_event = 98;
pub const EVENT_SESSIONLOADPOST: auto_event = 97;
pub const EVENT_SEARCHWRAPPED: auto_event = 96;
pub const EVENT_SAFESTATE: auto_event = 95;
pub const EVENT_REMOTEREPLY: auto_event = 94;
pub const EVENT_RECORDINGLEAVE: auto_event = 93;
pub const EVENT_RECORDINGENTER: auto_event = 92;
pub const EVENT_QUITPRE: auto_event = 91;
pub const EVENT_QUICKFIXCMDPRE: auto_event = 90;
pub const EVENT_QUICKFIXCMDPOST: auto_event = 89;
pub const EVENT_PROGRESS: auto_event = 88;
pub const EVENT_PACKCHANGEDPRE: auto_event = 87;
pub const EVENT_PACKCHANGED: auto_event = 86;
pub const EVENT_OPTIONSET: auto_event = 85;
pub const EVENT_MODECHANGED: auto_event = 84;
pub const EVENT_MENUPOPUP: auto_event = 83;
pub const EVENT_MARKSET: auto_event = 82;
pub const EVENT_LSPTOKENUPDATE: auto_event = 81;
pub const EVENT_LSPREQUEST: auto_event = 80;
pub const EVENT_LSPPROGRESS: auto_event = 79;
pub const EVENT_LSPNOTIFY: auto_event = 78;
pub const EVENT_LSPDETACH: auto_event = 77;
pub const EVENT_LSPATTACH: auto_event = 76;
pub const EVENT_INSERTLEAVEPRE: auto_event = 75;
pub const EVENT_INSERTLEAVE: auto_event = 74;
pub const EVENT_INSERTENTER: auto_event = 73;
pub const EVENT_INSERTCHARPRE: auto_event = 72;
pub const EVENT_INSERTCHANGE: auto_event = 71;
pub const EVENT_GUIFAILED: auto_event = 70;
pub const EVENT_GUIENTER: auto_event = 69;
pub const EVENT_FUNCUNDEFINED: auto_event = 68;
pub const EVENT_FOCUSLOST: auto_event = 67;
pub const EVENT_FOCUSGAINED: auto_event = 66;
pub const EVENT_FILTERWRITEPRE: auto_event = 65;
pub const EVENT_FILTERWRITEPOST: auto_event = 64;
pub const EVENT_FILTERREADPRE: auto_event = 63;
pub const EVENT_FILTERREADPOST: auto_event = 62;
pub const EVENT_FILEWRITEPRE: auto_event = 61;
pub const EVENT_FILEWRITEPOST: auto_event = 60;
pub const EVENT_FILEWRITECMD: auto_event = 59;
pub const EVENT_FILETYPE: auto_event = 58;
pub const EVENT_FILEREADPRE: auto_event = 57;
pub const EVENT_FILEREADPOST: auto_event = 56;
pub const EVENT_FILEREADCMD: auto_event = 55;
pub const EVENT_FILEENCODING: auto_event = 54;
pub const EVENT_FILECHANGEDSHELLPOST: auto_event = 53;
pub const EVENT_FILECHANGEDSHELL: auto_event = 52;
pub const EVENT_FILECHANGEDRO: auto_event = 51;
pub const EVENT_FILEAPPENDPRE: auto_event = 50;
pub const EVENT_FILEAPPENDPOST: auto_event = 49;
pub const EVENT_FILEAPPENDCMD: auto_event = 48;
pub const EVENT_EXITPRE: auto_event = 47;
pub const EVENT_ENCODINGCHANGED: auto_event = 46;
pub const EVENT_DIRCHANGEDPRE: auto_event = 45;
pub const EVENT_DIRCHANGED: auto_event = 44;
pub const EVENT_DIFFUPDATED: auto_event = 43;
pub const EVENT_DIAGNOSTICCHANGED: auto_event = 42;
pub const EVENT_CURSORMOVEDI: auto_event = 41;
pub const EVENT_CURSORMOVEDC: auto_event = 40;
pub const EVENT_CURSORMOVED: auto_event = 39;
pub const EVENT_CURSORHOLDI: auto_event = 38;
pub const EVENT_CURSORHOLD: auto_event = 37;
pub const EVENT_COMPLETEDONEPRE: auto_event = 36;
pub const EVENT_COMPLETEDONE: auto_event = 35;
pub const EVENT_COMPLETECHANGED: auto_event = 34;
pub const EVENT_COLORSCHEMEPRE: auto_event = 33;
pub const EVENT_COLORSCHEME: auto_event = 32;
pub const EVENT_CMDWINLEAVE: auto_event = 31;
pub const EVENT_CMDWINENTER: auto_event = 30;
pub const EVENT_CMDUNDEFINED: auto_event = 29;
pub const EVENT_CMDLINELEAVEPRE: auto_event = 28;
pub const EVENT_CMDLINELEAVE: auto_event = 27;
pub const EVENT_CMDLINEENTER: auto_event = 26;
pub const EVENT_CMDLINECHANGED: auto_event = 25;
pub const EVENT_CHANOPEN: auto_event = 24;
pub const EVENT_CHANINFO: auto_event = 23;
pub const EVENT_BUFWRITEPRE: auto_event = 22;
pub const EVENT_BUFWRITEPOST: auto_event = 21;
pub const EVENT_BUFWRITECMD: auto_event = 20;
pub const EVENT_BUFWRITE: auto_event = 19;
pub const EVENT_BUFWIPEOUT: auto_event = 18;
pub const EVENT_BUFWINLEAVE: auto_event = 17;
pub const EVENT_BUFWINENTER: auto_event = 16;
pub const EVENT_BUFUNLOAD: auto_event = 15;
pub const EVENT_BUFREADPRE: auto_event = 14;
pub const EVENT_BUFREADPOST: auto_event = 13;
pub const EVENT_BUFREADCMD: auto_event = 12;
pub const EVENT_BUFREAD: auto_event = 11;
pub const EVENT_BUFNEWFILE: auto_event = 10;
pub const EVENT_BUFNEW: auto_event = 9;
pub const EVENT_BUFMODIFIEDSET: auto_event = 8;
pub const EVENT_BUFLEAVE: auto_event = 7;
pub const EVENT_BUFHIDDEN: auto_event = 6;
pub const EVENT_BUFFILEPRE: auto_event = 5;
pub const EVENT_BUFFILEPOST: auto_event = 4;
pub const EVENT_BUFENTER: auto_event = 3;
pub const EVENT_BUFDELETE: auto_event = 2;
pub const EVENT_BUFCREATE: auto_event = 1;
pub const EVENT_BUFADD: auto_event = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kOptDyFlagMsgsep: C2Rust_Unnamed_16 = 8;
pub const kOptDyFlagUhex: C2Rust_Unnamed_16 = 4;
pub const kOptDyFlagTruncate: C2Rust_Unnamed_16 = 2;
pub const kOptDyFlagLastline: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kOptVeFlagNoneU: C2Rust_Unnamed_17 = 32;
pub const kOptVeFlagNone: C2Rust_Unnamed_17 = 16;
pub const kOptVeFlagOnemore: C2Rust_Unnamed_17 = 8;
pub const kOptVeFlagInsert: C2Rust_Unnamed_17 = 6;
pub const kOptVeFlagBlock: C2Rust_Unnamed_17 = 5;
pub const kOptVeFlagAll: C2Rust_Unnamed_17 = 4;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_18 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_18 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_18 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_18 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_18 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_18 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_18 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_18 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_18 = 79;
pub const SHM_OVER: C2Rust_Unnamed_18 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_18 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_18 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_18 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_18 = 97;
pub const SHM_WRI: C2Rust_Unnamed_18 = 119;
pub const SHM_LINES: C2Rust_Unnamed_18 = 108;
pub const SHM_MOD: C2Rust_Unnamed_18 = 109;
pub const SHM_RO: C2Rust_Unnamed_18 = 114;
pub const kCmdRedrawAll: CmdRedraw = 2;
pub const kCmdRedrawPos: CmdRedraw = 1;
pub const kCmdRedrawNone: CmdRedraw = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_23 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut WinExtmark,
}
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_24 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_24 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_24 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_24 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_24 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_24 = 20;
pub const UPD_VALID: C2Rust_Unnamed_24 = 10;
pub const MODE_CMDLINE: C2Rust_Unnamed_26 = 8;
pub const MODE_NORMAL: C2Rust_Unnamed_26 = 1;
pub const MODE_INSERT: C2Rust_Unnamed_26 = 16;
pub const MODE_VISUAL: C2Rust_Unnamed_26 = 2;
pub const kUIExtCount: UIExtension = 10;
pub const kUIFloatDebug: UIExtension = 9;
pub const kUITermColors: UIExtension = 8;
pub const kUIHlState: UIExtension = 7;
pub const kUIMultigrid: UIExtension = 6;
pub const kUILinegrid: UIExtension = 5;
pub const kUIMessages: UIExtension = 4;
pub const kUIWildmenu: UIExtension = 3;
pub const kUITabline: UIExtension = 2;
pub const kUIPopupmenu: UIExtension = 1;
pub const kUICmdline: UIExtension = 0;
pub const MODE_LANGMAP: C2Rust_Unnamed_26 = 32;
pub const REPLACE_FLAG: C2Rust_Unnamed_26 = 256;
pub const VREPLACE_FLAG: C2Rust_Unnamed_26 = 512;
pub const MODE_TERMINAL: C2Rust_Unnamed_26 = 128;
pub type WindowCorner = ::core::ffi::c_uint;
pub const WC_BOTTOM_RIGHT: WindowCorner = 3;
pub const WC_BOTTOM_LEFT: WindowCorner = 2;
pub const WC_TOP_RIGHT: WindowCorner = 1;
pub const WC_TOP_LEFT: WindowCorner = 0;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const DID_FOLD: C2Rust_Unnamed_25 = 3;
pub const DID_LINE: C2Rust_Unnamed_25 = 2;
pub const DID_NONE: C2Rust_Unnamed_25 = 1;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_26 = 20480;
pub const MODE_ASKMORE: C2Rust_Unnamed_26 = 12288;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_STARTTIME: VimVarIndex = 104;
pub const VV_VIRTNUM: VimVarIndex = 103;
pub const VV_RELNUM: VimVarIndex = 102;
pub const VV_LUA: VimVarIndex = 101;
pub const VV__NULL_BLOB: VimVarIndex = 100;
pub const VV__NULL_DICT: VimVarIndex = 99;
pub const VV__NULL_LIST: VimVarIndex = 98;
pub const VV__NULL_STRING: VimVarIndex = 97;
pub const VV_MSGPACK_TYPES: VimVarIndex = 96;
pub const VV_STDERR: VimVarIndex = 95;
pub const VV_VIM_DID_INIT: VimVarIndex = 94;
pub const VV_STACKTRACE: VimVarIndex = 93;
pub const VV_MAXCOL: VimVarIndex = 92;
pub const VV_EXITING: VimVarIndex = 91;
pub const VV_COLLATE: VimVarIndex = 90;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_ARGF: VimVarIndex = 88;
pub const VV_ECHOSPACE: VimVarIndex = 87;
pub const VV_VERSIONLONG: VimVarIndex = 86;
pub const VV_EVENT: VimVarIndex = 85;
pub const VV_TYPE_BLOB: VimVarIndex = 84;
pub const VV_TYPE_BOOL: VimVarIndex = 83;
pub const VV_TYPE_FLOAT: VimVarIndex = 82;
pub const VV_TYPE_DICT: VimVarIndex = 81;
pub const VV_TYPE_LIST: VimVarIndex = 80;
pub const VV_TYPE_FUNC: VimVarIndex = 79;
pub const VV_TYPE_STRING: VimVarIndex = 78;
pub const VV_TYPE_NUMBER: VimVarIndex = 77;
pub const VV_TESTING: VimVarIndex = 76;
pub const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub const VV_NUMBERSIZE: VimVarIndex = 74;
pub const VV_NUMBERMIN: VimVarIndex = 73;
pub const VV_NUMBERMAX: VimVarIndex = 72;
pub const VV_NULL: VimVarIndex = 71;
pub const VV_TRUE: VimVarIndex = 70;
pub const VV_FALSE: VimVarIndex = 69;
pub const VV_ERRORS: VimVarIndex = 68;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_COMPLETED_ITEM: VimVarIndex = 61;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_WINDOWID: VimVarIndex = 59;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub const VV_OP: VimVarIndex = 55;
pub const VV_MOUSE_COL: VimVarIndex = 54;
pub const VV_MOUSE_LNUM: VimVarIndex = 53;
pub const VV_MOUSE_WINID: VimVarIndex = 52;
pub const VV_MOUSE_WIN: VimVarIndex = 51;
pub const VV_CHAR: VimVarIndex = 50;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub const VV_SWAPCHOICE: VimVarIndex = 48;
pub const VV_SWAPNAME: VimVarIndex = 47;
pub const VV_SCROLLSTART: VimVarIndex = 46;
pub const VV_BEVAL_TEXT: VimVarIndex = 45;
pub const VV_BEVAL_COL: VimVarIndex = 44;
pub const VV_BEVAL_LNUM: VimVarIndex = 43;
pub const VV_BEVAL_WINID: VimVarIndex = 42;
pub const VV_BEVAL_WINNR: VimVarIndex = 41;
pub const VV_BEVAL_BUFNR: VimVarIndex = 40;
pub const VV_FCS_CHOICE: VimVarIndex = 39;
pub const VV_FCS_REASON: VimVarIndex = 38;
pub const VV_PROFILING: VimVarIndex = 37;
pub const VV_KEY: VimVarIndex = 36;
pub const VV_VAL: VimVarIndex = 35;
pub const VV_INSERTMODE: VimVarIndex = 34;
pub const VV_CMDBANG: VimVarIndex = 33;
pub const VV_REG: VimVarIndex = 32;
pub const VV_THROWPOINT: VimVarIndex = 31;
pub const VV_EXCEPTION: VimVarIndex = 30;
pub const VV_DYING: VimVarIndex = 29;
pub const VV_SEND_SERVER: VimVarIndex = 28;
pub const VV_PROGNAME: VimVarIndex = 27;
pub const VV_FOLDLEVEL: VimVarIndex = 26;
pub const VV_FOLDDASHES: VimVarIndex = 25;
pub const VV_FOLDEND: VimVarIndex = 24;
pub const VV_FOLDSTART: VimVarIndex = 23;
pub const VV_CMDARG: VimVarIndex = 22;
pub const VV_FNAME_DIFF: VimVarIndex = 21;
pub const VV_FNAME_NEW: VimVarIndex = 20;
pub const VV_FNAME_OUT: VimVarIndex = 19;
pub const VV_FNAME_IN: VimVarIndex = 18;
pub const VV_CC_TO: VimVarIndex = 17;
pub const VV_CC_FROM: VimVarIndex = 16;
pub const VV_CTYPE: VimVarIndex = 15;
pub const VV_LC_TIME: VimVarIndex = 14;
pub const VV_LANG: VimVarIndex = 13;
pub const VV_FNAME: VimVarIndex = 12;
pub const VV_TERMRESPONSE: VimVarIndex = 11;
pub const VV_TERMREQUEST: VimVarIndex = 10;
pub const VV_LNUM: VimVarIndex = 9;
pub const VV_VERSION: VimVarIndex = 8;
pub const VV_THIS_SESSION: VimVarIndex = 7;
pub const VV_SHELL_ERROR: VimVarIndex = 6;
pub const VV_STATUSMSG: VimVarIndex = 5;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const VV_ERRMSG: VimVarIndex = 3;
pub const VV_PREVCOUNT: VimVarIndex = 2;
pub const VV_COUNT1: VimVarIndex = 1;
pub const VV_COUNT: VimVarIndex = 0;
pub const SHOWCMD_COLS: C2Rust_Unnamed_27 = 10;
pub const MIN_COLUMNS: C2Rust_Unnamed_28 = 12;
pub const MODE_SETWSIZE: C2Rust_Unnamed_26 = 16384;
pub const MODE_HITRETURN: C2Rust_Unnamed_26 = 8193;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_26 = 24592;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_26 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_26 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_26 = 784;
pub const MODE_REPLACE: C2Rust_Unnamed_26 = 272;
pub const MAP_ALL_MODES: C2Rust_Unnamed_26 = 255;
pub const MODE_SELECT: C2Rust_Unnamed_26 = 64;
pub const MODE_OP_PENDING: C2Rust_Unnamed_26 = 4;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const STATUS_HEIGHT: C2Rust_Unnamed_28 = 1;
pub const MIN_LINES: C2Rust_Unnamed_28 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_BOTLINE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
pub const CPO_NUMCOL: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const SCL_NUM: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
static redraw_popupmenu: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static msg_grid_invalid: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static resizing_autocmd: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static conceal_cursor_used: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn conceal_check_cursor_line() {
    let mut should_conceal: bool = conceal_cursor_line(curwin.get());
    if (*curwin.get()).w_onebuf_opt.wo_cole <= 0 as OptInt
        || conceal_cursor_used.get() as ::core::ffi::c_int == should_conceal as ::core::ffi::c_int
    {
        return;
    }
    redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
    if decor_conceal_line(
        curwin.get(),
        (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        true_0 != 0,
    ) {
        changed_window_setting(curwin.get());
    }
    curs_columns(curwin.get(), true_0);
}
pub unsafe extern "C" fn default_grid_alloc() -> bool {
    static resizing: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if resizing.get() {
        return false_0 != 0;
    }
    resizing.set(true_0 != 0);
    if !(*default_grid.ptr()).chars.is_null()
        && Rows.get() == (*default_grid.ptr()).rows
        && Columns.get() == (*default_grid.ptr()).cols
        || Rows.get() == 0 as ::core::ffi::c_int
        || Columns.get() == 0 as ::core::ffi::c_int
    {
        resizing.set(false_0 != 0);
        return false_0 != 0;
    }
    grid_alloc(
        default_grid.ptr(),
        Rows.get(),
        Columns.get(),
        true_0 != 0,
        true_0 != 0,
    );
    stl_clear_click_defs(tab_page_click_defs.get(), tab_page_click_defs_size.get());
    tab_page_click_defs.set(stl_alloc_click_defs(
        tab_page_click_defs.get(),
        Columns.get(),
        tab_page_click_defs_size.ptr(),
    ));
    (*default_grid.ptr()).comp_height = Rows.get();
    (*default_grid.ptr()).comp_width = Columns.get();
    (*default_grid.ptr()).handle = DEFAULT_GRID_HANDLE as handle_T;
    resizing.set(false_0 != 0);
    return true_0 != 0;
}
pub unsafe extern "C" fn screenclear() {
    msg_check_for_delay(false_0 != 0);
    if starting.get() == NO_SCREEN || (*default_grid.ptr()).chars.is_null() {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*default_grid.ptr()).rows {
        grid_clear_line(
            default_grid.ptr(),
            *(*default_grid.ptr()).line_offset.offset(i as isize),
            (*default_grid.ptr()).cols,
            true_0 != 0,
        );
        i += 1;
    }
    ui_call_grid_clear(1 as Integer);
    ui_comp_set_screen_valid(true_0 != 0);
    ns_hl_fast.set(-1 as ::core::ffi::c_int as NS);
    clear_cmdline.set(false_0 != 0);
    mode_displayed.set(false_0 != 0);
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    cmdline_was_last_drawn.set(false_0 != 0);
    redraw_cmdline.set(true_0 != 0);
    redraw_tabline.set(true_0 != 0);
    redraw_popupmenu.set(true_0 != 0);
    pum_invalidate();
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_floating {
            (*wp).w_redr_type = UPD_CLEAR as ::core::ffi::c_int;
        }
        wp = (*wp).w_next;
    }
    if must_redraw.get() == UPD_CLEAR as ::core::ffi::c_int {
        must_redraw.set(UPD_NOT_VALID as ::core::ffi::c_int);
    }
    compute_cmdrow();
    msg_row.set(cmdline_row.get());
    msg_col.set(0 as ::core::ffi::c_int);
    msg_reset_scroll();
    msg_didany.set(false_0 != 0);
    msg_didout.set(false_0 != 0);
    if *(*hl_attr_active.ptr()).offset(HLF_MSG as ::core::ffi::c_int as isize)
        > 0 as ::core::ffi::c_int
        && msg_use_grid() as ::core::ffi::c_int != 0
        && !(*msg_grid.ptr()).chars.is_null()
    {
        grid_invalidate(msg_grid.ptr());
        msg_grid_validate();
        msg_grid_invalid.set(false_0 != 0);
        clear_cmdline.set(true_0 != 0);
    }
}
unsafe extern "C" fn cmdline_number_prompt() -> bool {
    return !ui_has(kUIMessages)
        && State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
        && !(*get_cmdline_info()).mouse_used.is_null();
}
#[no_mangle]
pub unsafe extern "C" fn screen_resize(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
) {
    if updating_screen.get() as ::core::ffi::c_int != 0
        || resizing_screen.get() as ::core::ffi::c_int != 0
        || cmdline_number_prompt() as ::core::ffi::c_int != 0
    {
        return;
    }
    if width < 0 as ::core::ffi::c_int || height < 0 as ::core::ffi::c_int {
        return;
    }
    if State.get() == MODE_HITRETURN as ::core::ffi::c_int
        || State.get() == MODE_SETWSIZE as ::core::ffi::c_int
    {
        State.set(MODE_SETWSIZE as ::core::ffi::c_int);
        return;
    }
    resizing_screen.set(true_0 != 0);
    Rows.set(height);
    Columns.set(width);
    check_screensize();
    if !ui_has(kUIMessages) {
        let mut max_p_ch: ::core::ffi::c_int =
            Rows.get() - min_rows(curtab.get()) + 1 as ::core::ffi::c_int;
        if p_ch.get() > 0 as OptInt && p_ch.get() > max_p_ch as OptInt {
            p_ch.set(
                (if max_p_ch > 1 as ::core::ffi::c_int {
                    max_p_ch
                } else {
                    1 as ::core::ffi::c_int
                }) as OptInt,
            );
            (*curtab.get()).tp_ch_used = p_ch.get();
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if tp != curtab.get() {
                let mut max_tp_ch: ::core::ffi::c_int =
                    Rows.get() - min_rows(tp as *mut tabpage_T) + 1 as ::core::ffi::c_int;
                if (*tp).tp_ch_used > 0 as OptInt && (*tp).tp_ch_used > max_tp_ch as OptInt {
                    (*tp).tp_ch_used = (if max_tp_ch > 1 as ::core::ffi::c_int {
                        max_tp_ch
                    } else {
                        1 as ::core::ffi::c_int
                    }) as OptInt;
                }
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
    height = Rows.get();
    width = Columns.get();
    p_lines.set(Rows.get() as OptInt);
    p_columns.set(Columns.get() as OptInt);
    ui_call_grid_resize(1 as Integer, width as Integer, height as Integer);
    let mut retry_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    resizing_autocmd.set(true_0 != 0);
    while default_grid_alloc() {
        ui_comp_set_screen_valid(false_0 != 0);
        if !(*msg_grid.ptr()).chars.is_null() {
            msg_grid_invalid.set(true_0 != 0);
        }
        (*RedrawingDisabled.ptr()) += 1;
        win_new_screensize();
        comp_col();
        (*RedrawingDisabled.ptr()) -= 1;
        retry_count += 1;
        if retry_count > 3 as ::core::ffi::c_int {
            break;
        }
        apply_autocmds(
            EVENT_VIMRESIZED,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    resizing_autocmd.set(false_0 != 0);
    redraw_all_later(UPD_CLEAR as ::core::ffi::c_int);
    if State.get() != MODE_ASKMORE as ::core::ffi::c_int
        && State.get() != MODE_EXTERNCMD as ::core::ffi::c_int
    {
        screenclear();
    }
    if starting.get() != NO_SCREEN {
        maketitle();
        changed_line_abv_curs();
        invalidate_botline_win(curwin.get());
        if State.get() == MODE_ASKMORE as ::core::ffi::c_int
            || State.get() == MODE_EXTERNCMD as ::core::ffi::c_int
            || exmode_active.get() as ::core::ffi::c_int != 0
            || State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
                && (*get_cmdline_info()).one_key as ::core::ffi::c_int != 0
        {
            if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
                update_screen();
            }
            if !(*msg_grid.ptr()).chars.is_null() {
                msg_grid_validate();
            }
            ui_comp_set_screen_valid(true_0 != 0);
            repeat_message();
        } else {
            if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
                do_check_scrollbind(true_0 != 0);
            }
            if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
                redraw_popupmenu.set(false_0 != 0);
                update_screen();
                redrawcmdline();
                if pum_drawn() {
                    cmdline_pum_display(false_0 != 0);
                }
            } else {
                update_topline(curwin.get());
                if pum_drawn() {
                    redraw_popupmenu.set(false_0 != 0);
                    ins_compl_show_pum();
                }
                update_screen();
                if redrawing() {
                    setcursor();
                }
            }
        }
        ui_flush();
    }
    resizing_screen.set(false_0 != 0);
}
pub unsafe extern "C" fn check_screensize() {
    Rows.set(
        if (if Rows.get() > min_rows_for_all_tabpages() {
            Rows.get()
        } else {
            min_rows_for_all_tabpages()
        }) < 1000 as ::core::ffi::c_int
        {
            if Rows.get() > min_rows_for_all_tabpages() {
                Rows.get()
            } else {
                min_rows_for_all_tabpages()
            }
        } else {
            1000 as ::core::ffi::c_int
        },
    );
    Columns.set(
        if (if Columns.get() > MIN_COLUMNS as ::core::ffi::c_int {
            Columns.get()
        } else {
            MIN_COLUMNS as ::core::ffi::c_int
        }) < 10000 as ::core::ffi::c_int
        {
            if Columns.get() > MIN_COLUMNS as ::core::ffi::c_int {
                Columns.get()
            } else {
                MIN_COLUMNS as ::core::ffi::c_int
            }
        } else {
            10000 as ::core::ffi::c_int
        },
    );
}
pub unsafe extern "C" fn redrawing() -> bool {
    return RedrawingDisabled.get() == 0
        && !(p_lz.get() != 0
            && char_avail() as ::core::ffi::c_int != 0
            && !KeyTyped.get()
            && !do_redraw.get());
}
pub unsafe extern "C" fn update_screen() -> ::core::ffi::c_int {
    static still_may_intro: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
    if still_may_intro.get() {
        if !may_show_intro() {
            redraw_later(firstwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
            still_may_intro.set(false_0 != 0);
        }
    }
    let mut is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
    if resizing_autocmd.get() as ::core::ffi::c_int != 0 || (*default_grid.ptr()).chars.is_null() {
        return FAIL;
    }
    if need_diff_redraw.get() {
        diff_redraw(true_0 != 0);
    }
    if !redrawing()
        || updating_screen.get() as ::core::ffi::c_int != 0
        || cmdline_number_prompt() as ::core::ffi::c_int != 0
    {
        return FAIL;
    }
    let mut type_0: ::core::ffi::c_int = must_redraw.get();
    must_redraw.set(0 as ::core::ffi::c_int);
    updating_screen.set(true_0 != 0);
    display_tick.set((*display_tick.ptr()).wrapping_add(1));
    if schar_cache_clear_if_full() {
        type_0 = if type_0 > UPD_CLEAR as ::core::ffi::c_int {
            type_0
        } else {
            UPD_CLEAR as ::core::ffi::c_int
        };
    }
    if msg_did_scroll.get() {
        msg_did_scroll.set(false_0 != 0);
        msg_scrolled_at_flush.set(0 as ::core::ffi::c_int);
    }
    if type_0 >= UPD_CLEAR as ::core::ffi::c_int || !(*default_grid.ptr()).valid {
        ui_comp_set_screen_valid(false_0 != 0);
    }
    if msg_scrolled.get() != 0 || msg_grid_invalid.get() as ::core::ffi::c_int != 0 {
        clear_cmdline.set(true_0 != 0);
        let mut valid: ::core::ffi::c_int =
            if Rows.get() - msg_scrollsize() > 0 as ::core::ffi::c_int {
                Rows.get() - msg_scrollsize()
            } else {
                0 as ::core::ffi::c_int
            };
        if !(*msg_grid.ptr()).chars.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i
                < (if msg_scrollsize() < (*msg_grid.ptr()).rows {
                    msg_scrollsize()
                } else {
                    (*msg_grid.ptr()).rows
                })
            {
                grid_clear_line(
                    msg_grid.ptr(),
                    *(*msg_grid.ptr()).line_offset.offset(i as isize),
                    (*msg_grid.ptr()).cols,
                    (i as OptInt) < p_ch.get(),
                );
                i += 1;
            }
        }
        (*msg_grid.ptr()).throttled = false_0 != 0;
        let mut was_invalidated: bool = false_0 != 0;
        if type_0 == UPD_NOT_VALID as ::core::ffi::c_int
            && !ui_has(kUIMultigrid)
            && msg_scrolled.get() != 0
        {
            was_invalidated = ui_comp_set_screen_valid(false_0 != 0);
            let mut i_0: ::core::ffi::c_int = valid;
            while (i_0 as OptInt) < Rows.get() as OptInt - p_ch.get() {
                grid_clear_line(
                    default_grid.ptr(),
                    *(*default_grid.ptr()).line_offset.offset(i_0 as isize),
                    Columns.get(),
                    false_0 != 0,
                );
                i_0 += 1;
            }
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if !(*wp).w_floating {
                    if (*wp).w_winrow + (*wp).w_height > valid {
                        (*wp).w_redr_type =
                            if (*wp).w_redr_type > UPD_NOT_VALID as ::core::ffi::c_int {
                                (*wp).w_redr_type
                            } else {
                                UPD_NOT_VALID as ::core::ffi::c_int
                            };
                    }
                    if !is_stl_global
                        && (*wp).w_winrow + (*wp).w_height + (*wp).w_status_height > valid
                    {
                        (*wp).w_redr_status = true_0 != 0;
                    }
                }
                wp = (*wp).w_next;
            }
            if is_stl_global as ::core::ffi::c_int != 0
                && Rows.get() as OptInt - p_ch.get() - 1 as OptInt > valid as OptInt
            {
                (*curwin.get()).w_redr_status = true_0 != 0;
            }
        }
        msg_grid_set_pos(Rows.get() - p_ch.get() as ::core::ffi::c_int, false_0 != 0);
        msg_grid_invalid.set(false_0 != 0);
        if was_invalidated {
            ui_comp_set_screen_valid(true_0 != 0);
        }
        msg_scrolled.set(0 as ::core::ffi::c_int);
        msg_scrolled_at_flush.set(0 as ::core::ffi::c_int);
        msg_grid_scroll_discount.set(0 as ::core::ffi::c_int);
        need_wait_return.set(false_0 != 0);
    }
    win_ui_flush(true_0 != 0);
    compute_cmdrow();
    let mut hl_changed: bool = false_0 != 0;
    if need_highlight_changed.get() {
        highlight_changed();
        hl_changed = true_0 != 0;
    }
    if type_0 == UPD_CLEAR as ::core::ffi::c_int {
        screenclear();
        cmdline_screen_cleared();
        if ui_has(kUIMessages) {
            ui_call_msg_clear();
        }
        type_0 = UPD_NOT_VALID as ::core::ffi::c_int;
        must_redraw.set(0 as ::core::ffi::c_int);
    } else if !(*default_grid.ptr()).valid {
        grid_invalidate(default_grid.ptr());
        (*default_grid.ptr()).valid = true_0 != 0;
    }
    if type_0 == UPD_NOT_VALID as ::core::ffi::c_int
        && clear_cmdline.get() as ::core::ffi::c_int != 0
        && !ui_has(kUIMessages)
    {
        grid_clear(
            default_gridview.ptr(),
            Rows.get() - p_ch.get() as ::core::ffi::c_int,
            Rows.get(),
            0 as ::core::ffi::c_int,
            Columns.get(),
            0 as ::core::ffi::c_int,
        );
    }
    ui_comp_set_screen_valid(true_0 != 0);
    decor_providers_start();
    if win_check_ns_hl(::core::ptr::null_mut::<win_T>()) {
        redraw_cmdline.set(true_0 != 0);
        redraw_tabline.set(true_0 != 0);
    }
    if clear_cmdline.get() {
        msg_check_for_delay(false_0 != 0);
    }
    if (*curwin.get()).w_redr_type < UPD_NOT_VALID as ::core::ffi::c_int
        && (*curwin.get()).w_nrwidth
            != (if (*curwin.get()).w_onebuf_opt.wo_nu != 0
                || (*curwin.get()).w_onebuf_opt.wo_rnu != 0
                || *(*curwin.get()).w_onebuf_opt.wo_stc as ::core::ffi::c_int != 0
            {
                number_width(curwin.get())
            } else {
                0 as ::core::ffi::c_int
            })
    {
        (*curwin.get()).w_redr_type = UPD_NOT_VALID as ::core::ffi::c_int;
    }
    if (*curwin.get()).w_redr_type == UPD_INVERTED as ::core::ffi::c_int {
        update_curswant();
    }
    if redraw_tabline.get() as ::core::ffi::c_int != 0
        || type_0 >= UPD_NOT_VALID as ::core::ffi::c_int
    {
        update_window_hl(curwin.get(), type_0 >= UPD_NOT_VALID as ::core::ffi::c_int);
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if tp != curtab.get() {
                update_window_hl(
                    (*tp).tp_curwin,
                    type_0 >= UPD_NOT_VALID as ::core::ffi::c_int,
                );
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        draw_tabline();
    }
    let mut wp_0: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_0.is_null() {
        update_window_hl(
            wp_0,
            type_0 >= UPD_NOT_VALID as ::core::ffi::c_int || hl_changed as ::core::ffi::c_int != 0,
        );
        let mut buf: *mut buf_T = (*wp_0).w_buffer;
        if (*buf).b_mod_set {
            if (*buf).b_mod_tick_syn < display_tick.get()
                && syntax_present(wp_0) as ::core::ffi::c_int != 0
            {
                syn_stack_apply_changes(buf);
                (*buf).b_mod_tick_syn = display_tick.get();
            }
            if (*buf).b_mod_tick_decor < display_tick.get() {
                decor_providers_invoke_buf(buf);
                (*buf).b_mod_tick_decor = display_tick.get();
            }
        }
        wp_0 = (*wp_0).w_next;
    }
    let mut did_one: bool = false_0 != 0;
    (*screen_search_hl.ptr()).rm.regprog = ::core::ptr::null_mut::<regprog_T>();
    let mut wp_1: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_1.is_null() {
        if (*wp_1).w_redr_type == UPD_CLEAR as ::core::ffi::c_int
            && (*wp_1).w_floating as ::core::ffi::c_int != 0
            && !(*wp_1).w_grid_alloc.chars.is_null()
        {
            grid_invalidate(&raw mut (*wp_1).w_grid_alloc);
            (*wp_1).w_redr_type = UPD_NOT_VALID as ::core::ffi::c_int;
        }
        win_check_ns_hl(wp_1);
        win_grid_alloc(wp_1);
        if (*wp_1).w_redr_border as ::core::ffi::c_int != 0
            || (*wp_1).w_redr_type >= UPD_NOT_VALID as ::core::ffi::c_int
        {
            grid_draw_border(
                &raw mut (*wp_1).w_grid_alloc,
                &raw mut (*wp_1).w_config,
                &raw mut (*wp_1).w_border_adj as *mut ::core::ffi::c_int,
                (*wp_1).w_onebuf_opt.wo_winbl as ::core::ffi::c_int,
                (*wp_1).w_ns_hl_attr,
            );
        }
        if (*wp_1).w_redr_type != 0 as ::core::ffi::c_int {
            if !did_one {
                did_one = true_0 != 0;
                start_search_hl();
            }
            win_update(wp_1);
        }
        if (*wp_1).w_redr_status {
            win_redr_winbar(wp_1);
            win_redr_status(wp_1);
        }
        wp_1 = (*wp_1).w_next;
    }
    if did_one {
        let mut wp_2: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp_2.is_null() {
            draw_sep_connectors_win(wp_2);
            wp_2 = (*wp_2).w_next;
        }
    }
    end_search_hl();
    if pum_drawn() as ::core::ffi::c_int != 0 && must_redraw_pum.get() as ::core::ffi::c_int != 0 {
        win_check_ns_hl(curwin.get());
        pum_redraw();
    } else if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        pum_check_clear();
    }
    win_check_ns_hl(::core::ptr::null_mut::<win_T>());
    let mut wp_3: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp_3.is_null() {
        (*(*wp_3).w_buffer).b_mod_set = false_0 != 0;
        wp_3 = (*wp_3).w_next;
    }
    updating_screen.set(false_0 != 0);
    if need_maketitle.get() {
        maketitle();
    }
    if clear_cmdline.get() as ::core::ffi::c_int != 0
        || redraw_cmdline.get() as ::core::ffi::c_int != 0
        || redraw_mode.get() as ::core::ffi::c_int != 0
    {
        showmode();
    }
    if still_may_intro.get() {
        intro_message(false_0 != 0);
    }
    repeat_message();
    decor_providers_invoke_end();
    if !ui_has(kUICmdline) {
        cmdline_was_last_drawn.set(false_0 != 0);
    }
    return OK;
}
pub unsafe extern "C" fn start_search_hl() {
    if p_hls.get() == 0 || no_hlsearch.get() as ::core::ffi::c_int != 0 {
        return;
    }
    end_search_hl();
    last_pat_prog(&raw mut (*screen_search_hl.ptr()).rm);
    (*screen_search_hl.ptr()).tm = profile_setlimit(p_rdt.get() as int64_t);
}
pub unsafe extern "C" fn end_search_hl() {
    if (*screen_search_hl.ptr()).rm.regprog.is_null() {
        return;
    }
    vim_regfree((*screen_search_hl.ptr()).rm.regprog);
    (*screen_search_hl.ptr()).rm.regprog = ::core::ptr::null_mut::<regprog_T>();
}
pub unsafe extern "C" fn setcursor() {
    setcursor_mayforce(curwin.get(), false_0 != 0);
}
pub unsafe extern "C" fn setcursor_mayforce(mut wp: *mut win_T, mut force: bool) {
    if force as ::core::ffi::c_int != 0 || redrawing() as ::core::ffi::c_int != 0 {
        validate_cursor(wp);
        let mut row: ::core::ffi::c_int = (*wp).w_wrow;
        let mut col: ::core::ffi::c_int = (*wp).w_wcol;
        if (*wp).w_onebuf_opt.wo_rl != 0 {
            let mut cursor: *mut ::core::ffi::c_char =
                ml_get_buf((*wp).w_buffer, (*wp).w_cursor.lnum).offset((*wp).w_cursor.col as isize);
            col = (*wp).w_view_width
                - (*wp).w_wcol
                - (if utf_ptr2cells(cursor) == 2 as ::core::ffi::c_int
                    && vim_isprintc(utf_ptr2char(cursor)) as ::core::ffi::c_int != 0
                {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                });
        }
        let mut grid: *mut ScreenGrid =
            grid_adjust(&raw mut (*wp).w_grid, &raw mut row, &raw mut col);
        if !grid.is_null() {
            ui_grid_cursor_goto((*grid).handle, row, col);
        }
    }
}
pub unsafe extern "C" fn redraw_custom_title_later() -> bool {
    if p_icon.get() != 0 && stl_syntax.get() & STL_IN_ICON != 0
        || p_title.get() != 0 && stl_syntax.get() & STL_IN_TITLE != 0
    {
        need_maketitle.set(true_0 != 0);
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn show_cursor_info_later(mut force: bool) {
    let mut state: ::core::ffi::c_int = get_real_state();
    let mut empty_line: ::core::ffi::c_int = (State.get() & MODE_INSERT as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
        && *ml_get_buf((*curwin.get()).w_buffer, (*curwin.get()).w_cursor.lnum)
            as ::core::ffi::c_int
            == NUL) as ::core::ffi::c_int;
    validate_virtcol(curwin.get());
    if force as ::core::ffi::c_int != 0
        || (*curwin.get()).w_cursor.lnum != (*curwin.get()).w_stl_cursor.lnum
        || (*curwin.get()).w_cursor.col != (*curwin.get()).w_stl_cursor.col
        || (*curwin.get()).w_virtcol != (*curwin.get()).w_stl_virtcol
        || (*curwin.get()).w_cursor.coladd != (*curwin.get()).w_stl_cursor.coladd
        || (*curwin.get()).w_topline != (*curwin.get()).w_stl_topline
        || (*(*curwin.get()).w_buffer).b_ml.ml_line_count != (*curwin.get()).w_stl_line_count
        || (*curwin.get()).w_topfill != (*curwin.get()).w_stl_topfill
        || empty_line != (*curwin.get()).w_stl_empty as ::core::ffi::c_int
        || reg_recording.get() != (*curwin.get()).w_stl_recording
        || state != (*curwin.get()).w_stl_state
        || VIsual_active.get() as ::core::ffi::c_int != 0
            && (VIsual_mode.get() != (*curwin.get()).w_stl_visual_mode
                || (*VIsual.ptr()).lnum != (*curwin.get()).w_stl_visual_pos.lnum
                || (*VIsual.ptr()).col != (*curwin.get()).w_stl_visual_pos.col
                || (*VIsual.ptr()).coladd != (*curwin.get()).w_stl_visual_pos.coladd)
    {
        if (*curwin.get()).w_status_height != 0 || global_stl_height() != 0 {
            (*curwin.get()).w_redr_status = true_0 != 0;
        } else {
            redraw_cmdline.set(true_0 != 0);
        }
        if *p_wbr.get() as ::core::ffi::c_int != NUL
            || *(*curwin.get()).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL
        {
            (*curwin.get()).w_redr_status = true_0 != 0;
        }
        redraw_custom_title_later();
    }
    (*curwin.get()).w_stl_cursor = (*curwin.get()).w_cursor;
    (*curwin.get()).w_stl_virtcol = (*curwin.get()).w_virtcol;
    (*curwin.get()).w_stl_empty = empty_line as ::core::ffi::c_char;
    (*curwin.get()).w_stl_topline = (*curwin.get()).w_topline;
    (*curwin.get()).w_stl_line_count = (*(*curwin.get()).w_buffer).b_ml.ml_line_count;
    (*curwin.get()).w_stl_topfill = (*curwin.get()).w_topfill;
    (*curwin.get()).w_stl_recording = reg_recording.get();
    (*curwin.get()).w_stl_state = state;
    if VIsual_active.get() {
        (*curwin.get()).w_stl_visual_mode = VIsual_mode.get();
        (*curwin.get()).w_stl_visual_pos = VIsual.get();
    }
}
pub unsafe extern "C" fn skip_showmode() -> bool {
    if global_busy.get() != 0
        || msg_silent.get() != 0 as ::core::ffi::c_int
        || !redrawing()
        || char_avail() as ::core::ffi::c_int != 0 && !KeyTyped.get()
    {
        redraw_mode.set(true_0 != 0);
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn showmode() -> ::core::ffi::c_int {
    let mut length: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    msg_ext_ui_flush();
    msg_grid_validate();
    let mut do_mode: bool = p_smd.get() != 0
        && msg_silent.get() == 0 as ::core::ffi::c_int
        && (State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0
            || State.get() & MODE_INSERT as ::core::ffi::c_int != 0
            || restart_edit.get() != NUL
            || VIsual_active.get() as ::core::ffi::c_int != 0);
    let mut can_show_mode: bool =
        p_ch.get() != 0 as OptInt || ui_has(kUIMessages) as ::core::ffi::c_int != 0;
    if (do_mode as ::core::ffi::c_int != 0 || reg_recording.get() != 0 as ::core::ffi::c_int)
        && can_show_mode as ::core::ffi::c_int != 0
    {
        if skip_showmode() {
            return 0 as ::core::ffi::c_int;
        }
        let mut nwr_save: bool = need_wait_return.get();
        msg_check_for_delay(false_0 != 0);
        let mut need_clear: bool = clear_cmdline.get();
        if clear_cmdline.get() as ::core::ffi::c_int != 0
            && cmdline_row.get() < Rows.get() - 1 as ::core::ffi::c_int
        {
            msg_clr_cmdline();
        }
        msg_pos_mode();
        let mut hl_id: ::core::ffi::c_int = HLF_CM as ::core::ffi::c_int;
        msg_no_more.set(true_0 != 0);
        let mut save_lines_left: ::core::ffi::c_int = lines_left.get();
        lines_left.set(0 as ::core::ffi::c_int);
        if do_mode {
            msg_puts_hl(
                b"--\0".as_ptr() as *const ::core::ffi::c_char,
                hl_id,
                false_0 != 0,
            );
            if !(*edit_submode.ptr()).is_null()
                && !shortmess(SHM_COMPLETIONMENU as ::core::ffi::c_int)
            {
                if ui_has(kUIMessages) {
                    length = INT_MAX;
                } else {
                    length = (Rows.get() - msg_row.get()) * Columns.get() - 3 as ::core::ffi::c_int;
                }
                if !(*edit_submode_extra.ptr()).is_null() {
                    length -= vim_strsize(edit_submode_extra.get());
                }
                if length > 0 as ::core::ffi::c_int {
                    if !(*edit_submode_pre.ptr()).is_null() {
                        length -= vim_strsize(edit_submode_pre.get());
                    }
                    if length - vim_strsize(edit_submode.get()) > 0 as ::core::ffi::c_int {
                        if !(*edit_submode_pre.ptr()).is_null() {
                            msg_puts_hl(edit_submode_pre.get(), hl_id, false_0 != 0);
                        }
                        msg_puts_hl(edit_submode.get(), hl_id, false_0 != 0);
                    }
                    if !(*edit_submode_extra.ptr()).is_null() {
                        msg_puts_hl(
                            b" \0".as_ptr() as *const ::core::ffi::c_char,
                            hl_id,
                            false_0 != 0,
                        );
                        let mut sub_id: ::core::ffi::c_int = if (edit_submode_highl.get()
                            as ::core::ffi::c_uint)
                            < HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            edit_submode_highl.get() as ::core::ffi::c_int
                        } else {
                            hl_id
                        };
                        msg_puts_hl(edit_submode_extra.get(), sub_id, false_0 != 0);
                    }
                }
            } else {
                if State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0 {
                    msg_puts_hl(
                        gettext(b" TERMINAL\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                } else if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                    msg_puts_hl(
                        gettext(b" VREPLACE\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                } else if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
                    msg_puts_hl(
                        gettext(b" REPLACE\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                } else if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
                    if p_ri.get() != 0 {
                        msg_puts_hl(
                            gettext(b" REVERSE\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    }
                    msg_puts_hl(
                        gettext(b" INSERT\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                } else if restart_edit.get() == 'I' as ::core::ffi::c_int
                    || restart_edit.get() == 'i' as ::core::ffi::c_int
                    || restart_edit.get() == 'a' as ::core::ffi::c_int
                    || restart_edit.get() == 'A' as ::core::ffi::c_int
                {
                    if !(*curbuf.get()).terminal.is_null() {
                        msg_puts_hl(
                            gettext(b" (terminal)\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    } else {
                        msg_puts_hl(
                            gettext(b" (insert)\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    }
                } else if restart_edit.get() == 'R' as ::core::ffi::c_int {
                    msg_puts_hl(
                        gettext(b" (replace)\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                } else if restart_edit.get() == 'V' as ::core::ffi::c_int {
                    msg_puts_hl(
                        gettext(b" (vreplace)\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                }
                if State.get() & MODE_LANGMAP as ::core::ffi::c_int != 0 {
                    if (*curwin.get()).w_onebuf_opt.wo_arab != 0 {
                        msg_puts_hl(
                            gettext(b" Arabic\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    } else if let Some(keymap_name) = keymap_str(curwin.get()) {
                        let buf = NameBuff.ptr() as *mut ::core::ffi::c_char;
                        let plen = vim_snprintf(
                            buf,
                            MAXPATHL as size_t,
                            b" (%s)\0".as_ptr() as *const ::core::ffi::c_char,
                            keymap_name.as_ptr(),
                        );
                        if plen > 0 && plen <= MAXPATHL - 1 {
                            msg_puts_hl(buf, hl_id, false_0 != 0);
                        }
                    }
                }
                if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 && p_paste.get() != 0 {
                    msg_puts_hl(
                        gettext(b" (paste)\0".as_ptr() as *const ::core::ffi::c_char),
                        hl_id,
                        false_0 != 0,
                    );
                }
                if VIsual_active.get() {
                    let mut p: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    match (if VIsual_select.get() as ::core::ffi::c_int != 0 {
                        4 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) + (VIsual_mode.get() == Ctrl_V) as ::core::ffi::c_int
                        * 2 as ::core::ffi::c_int
                        + (VIsual_mode.get() == 'V' as ::core::ffi::c_int) as ::core::ffi::c_int
                    {
                        0 => {
                            p = b" VISUAL\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        1 => {
                            p = b" VISUAL LINE\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        2 => {
                            p = b" VISUAL BLOCK\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        4 => {
                            p = b" SELECT\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        5 => {
                            p = b" SELECT LINE\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        _ => {
                            p = b" SELECT BLOCK\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                    }
                    msg_puts_hl(gettext(p), hl_id, false_0 != 0);
                }
                msg_puts_hl(
                    b" --\0".as_ptr() as *const ::core::ffi::c_char,
                    hl_id,
                    false_0 != 0,
                );
            }
            need_clear = true_0 != 0;
        }
        if reg_recording.get() != 0 as ::core::ffi::c_int && (*edit_submode.ptr()).is_null() {
            recording_mode(hl_id);
            need_clear = true_0 != 0;
        }
        mode_displayed.set(true_0 != 0);
        if need_clear as ::core::ffi::c_int != 0
            || clear_cmdline.get() as ::core::ffi::c_int != 0
            || redraw_mode.get() as ::core::ffi::c_int != 0
        {
            msg_clr_eos();
        }
        msg_didout.set(false_0 != 0);
        length = msg_col.get();
        msg_col.set(0 as ::core::ffi::c_int);
        msg_no_more.set(false_0 != 0);
        lines_left.set(save_lines_left);
        need_wait_return.set(nwr_save);
    } else if clear_cmdline.get() as ::core::ffi::c_int != 0
        && msg_silent.get() == 0 as ::core::ffi::c_int
    {
        msg_clr_cmdline();
    } else if redraw_mode.get() {
        msg_pos_mode();
        msg_clr_eos();
    }
    msg_ext_flush_showmode();
    if VIsual_active.get() {
        clear_showcmd();
    }
    redraw_ruler();
    redraw_cmdline.set(false_0 != 0);
    redraw_mode.set(false_0 != 0);
    clear_cmdline.set(false_0 != 0);
    return length;
}
unsafe extern "C" fn msg_pos_mode() {
    msg_col.set(0 as ::core::ffi::c_int);
    msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn unshowmode(mut force: bool) {
    if !redrawing() || !force && char_avail() as ::core::ffi::c_int != 0 && !KeyTyped.get() {
        redraw_cmdline.set(true_0 != 0);
    } else {
        clearmode();
    };
}
pub unsafe extern "C" fn clearmode() {
    let save_msg_row: ::core::ffi::c_int = msg_row.get();
    let save_msg_col: ::core::ffi::c_int = msg_col.get();
    msg_ext_ui_flush();
    msg_pos_mode();
    if reg_recording.get() != 0 as ::core::ffi::c_int {
        recording_mode(HLF_CM as ::core::ffi::c_int);
    }
    msg_clr_eos();
    msg_ext_flush_showmode();
    msg_col.set(save_msg_col);
    msg_row.set(save_msg_row);
}
unsafe extern "C" fn recording_mode(mut hl_id: ::core::ffi::c_int) {
    if shortmess(SHM_RECORDING as ::core::ffi::c_int) {
        return;
    }
    msg_puts_hl(
        gettext(b"recording\0".as_ptr() as *const ::core::ffi::c_char),
        hl_id,
        false_0 != 0,
    );
    let mut s: [::core::ffi::c_char; 4] = [0; 4];
    snprintf(
        &raw mut s as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
        b" @%c\0".as_ptr() as *const ::core::ffi::c_char,
        reg_recording.get(),
    );
    msg_puts_hl(&raw mut s as *mut ::core::ffi::c_char, hl_id, false_0 != 0);
}
pub const COL_RULER: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub unsafe extern "C" fn comp_col() {
    let mut last_has_status: bool = last_stl_height(false_0 != 0) > 0 as ::core::ffi::c_int;
    sc_col.set(0 as ::core::ffi::c_int);
    ru_col.set(0 as ::core::ffi::c_int);
    if p_ru.get() != 0 {
        ru_col.set(
            (if ru_wid.get() != 0 {
                ru_wid.get()
            } else {
                COL_RULER
            }) + 1 as ::core::ffi::c_int,
        );
        if !last_has_status {
            sc_col.set(ru_col.get());
        }
    }
    if p_sc.get() != 0 && *p_sloc.get() as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
        (*sc_col.ptr()) += SHOWCMD_COLS as ::core::ffi::c_int;
        if p_ru.get() == 0 || last_has_status as ::core::ffi::c_int != 0 {
            (*sc_col.ptr()) += 1;
        }
    }
    '_c2rust_label: {
        if sc_col.get() >= 0 as ::core::ffi::c_int
            && -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int + sc_col.get()
                <= Columns.get()
        {
        } else {
            __assert_fail(
                b"sc_col >= 0 && INT_MIN + sc_col <= Columns\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1128 as ::core::ffi::c_uint,
                b"void comp_col(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    sc_col.set(Columns.get() - sc_col.get());
    '_c2rust_label_0: {
        if ru_col.get() >= 0 as ::core::ffi::c_int
            && -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int + ru_col.get()
                <= Columns.get()
        {
        } else {
            __assert_fail(
                b"ru_col >= 0 && INT_MIN + ru_col <= Columns\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1131 as ::core::ffi::c_uint,
                b"void comp_col(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    ru_col.set(Columns.get() - ru_col.get());
    if sc_col.get() <= 0 as ::core::ffi::c_int {
        sc_col.set(1 as ::core::ffi::c_int);
    }
    if ru_col.get() <= 0 as ::core::ffi::c_int {
        ru_col.set(1 as ::core::ffi::c_int);
    }
    set_vim_var_nr(
        VV_ECHOSPACE,
        (sc_col.get() - 1 as ::core::ffi::c_int) as varnumber_T,
    );
}
unsafe extern "C" fn win_redraw_signcols(mut wp: *mut win_T) -> bool {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    if !(*buf).b_signcols.autom
        && (*(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
            || (*wp).w_maxscwidth > 1 as ::core::ffi::c_int
                && (*wp).w_minscwidth != (*wp).w_maxscwidth)
    {
        (*buf).b_signcols.autom = true_0 != 0;
        buf_signcols_count_range(
            buf,
            0 as ::core::ffi::c_int,
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            MAXLNUM as ::core::ffi::c_int,
            kFalse,
        );
    }
    while (*buf).b_signcols.max > 0 as ::core::ffi::c_int
        && (*buf).b_signcols.count[((*buf).b_signcols.max - 1 as ::core::ffi::c_int) as usize]
            == 0 as ::core::ffi::c_int
    {
        (*buf).b_signcols.max -= 1;
    }
    let mut width: ::core::ffi::c_int = if (*wp).w_maxscwidth < (*buf).b_signcols.max {
        (*wp).w_maxscwidth
    } else {
        (*buf).b_signcols.max
    };
    let mut rebuild_stc: bool = (*buf).b_signcols.max != (*buf).b_signcols.last_max
        && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL;
    if rebuild_stc {
        (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
    } else if (*wp).w_minscwidth == 0 as ::core::ffi::c_int
        && (*wp).w_maxscwidth == 1 as ::core::ffi::c_int
    {
        width = (buf_meta_total(buf, kMTMetaSignText) > 0 as uint32_t) as ::core::ffi::c_int;
    }
    let mut scwidth: ::core::ffi::c_int = (*wp).w_scwidth;
    (*wp).w_scwidth = if (if 0 as ::core::ffi::c_int > (*wp).w_minscwidth {
        0 as ::core::ffi::c_int
    } else {
        (*wp).w_minscwidth
    }) > width
    {
        if 0 as ::core::ffi::c_int > (*wp).w_minscwidth {
            0 as ::core::ffi::c_int
        } else {
            (*wp).w_minscwidth
        }
    } else {
        width
    };
    return (*wp).w_scwidth != scwidth || rebuild_stc as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn hsep_connected(mut wp: *mut win_T, mut corner: WindowCorner) -> bool {
    let mut before: bool = corner as ::core::ffi::c_uint
        == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        || corner as ::core::ffi::c_uint
            == WC_BOTTOM_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint;
    let mut sep_row: ::core::ffi::c_int = if corner as ::core::ffi::c_uint
        == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        || corner as ::core::ffi::c_uint
            == WC_TOP_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*wp).w_winrow - 1 as ::core::ffi::c_int
    } else {
        (*wp).w_winrow + (*wp).w_height
    };
    let mut fr: *mut frame_T = (*wp).w_frame;
    while !(*fr).fr_parent.is_null() {
        if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
            && !(if before as ::core::ffi::c_int != 0 {
                (*fr).fr_prev
            } else {
                (*fr).fr_next
            })
            .is_null()
        {
            fr = if before as ::core::ffi::c_int != 0 {
                (*fr).fr_prev
            } else {
                (*fr).fr_next
            };
            break;
        } else {
            fr = (*fr).fr_parent;
        }
    }
    if (*fr).fr_parent.is_null() {
        return false_0 != 0;
    }
    while (*fr).fr_layout as ::core::ffi::c_int != FR_LEAF {
        fr = (*fr).fr_child;
        if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
            && before as ::core::ffi::c_int != 0
        {
            while !(*fr).fr_next.is_null() {
                fr = (*fr).fr_next;
            }
        } else {
            while !(*fr).fr_next.is_null() && (*frame2win(fr)).w_winrow + (*fr).fr_height < sep_row
            {
                fr = (*fr).fr_next;
            }
        }
    }
    return sep_row == (*(*fr).fr_win).w_winrow - 1 as ::core::ffi::c_int
        || sep_row == (*(*fr).fr_win).w_winrow + (*(*fr).fr_win).w_height;
}
unsafe extern "C" fn vsep_connected(mut wp: *mut win_T, mut corner: WindowCorner) -> bool {
    let mut before: bool = corner as ::core::ffi::c_uint
        == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        || corner as ::core::ffi::c_uint
            == WC_TOP_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint;
    let mut sep_col: ::core::ffi::c_int = if corner as ::core::ffi::c_uint
        == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        || corner as ::core::ffi::c_uint
            == WC_BOTTOM_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*wp).w_wincol - 1 as ::core::ffi::c_int
    } else {
        (*wp).w_wincol + (*wp).w_width
    };
    let mut fr: *mut frame_T = (*wp).w_frame;
    while !(*fr).fr_parent.is_null() {
        if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
            && !(if before as ::core::ffi::c_int != 0 {
                (*fr).fr_prev
            } else {
                (*fr).fr_next
            })
            .is_null()
        {
            fr = if before as ::core::ffi::c_int != 0 {
                (*fr).fr_prev
            } else {
                (*fr).fr_next
            };
            break;
        } else {
            fr = (*fr).fr_parent;
        }
    }
    if (*fr).fr_parent.is_null() {
        return false_0 != 0;
    }
    while (*fr).fr_layout as ::core::ffi::c_int != FR_LEAF {
        fr = (*fr).fr_child;
        if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
            && before as ::core::ffi::c_int != 0
        {
            while !(*fr).fr_next.is_null() {
                fr = (*fr).fr_next;
            }
        } else {
            while !(*fr).fr_next.is_null() && (*frame2win(fr)).w_wincol + (*fr).fr_width < sep_col {
                fr = (*fr).fr_next;
            }
        }
    }
    return sep_col == (*(*fr).fr_win).w_wincol - 1 as ::core::ffi::c_int
        || sep_col == (*(*fr).fr_win).w_wincol + (*(*fr).fr_win).w_width;
}
unsafe extern "C" fn draw_vsep_win(mut wp: *mut win_T) {
    if (*wp).w_vsep_width == 0 {
        return;
    }
    let mut row: ::core::ffi::c_int = (*wp).w_winrow;
    while row < (*wp).w_winrow + (*wp).w_height {
        grid_line_start(default_gridview.ptr(), row);
        grid_line_put_schar(
            (*wp).w_wincol + (*wp).w_width,
            (*wp).w_p_fcs_chars.vert,
            win_hl_attr(wp, HLF_C as ::core::ffi::c_int),
        );
        grid_line_flush();
        row += 1;
    }
}
unsafe extern "C" fn draw_hsep_win(mut wp: *mut win_T) {
    if (*wp).w_hsep_height == 0 {
        return;
    }
    grid_line_start(default_gridview.ptr(), (*wp).w_winrow + (*wp).w_height);
    grid_line_fill(
        (*wp).w_wincol,
        (*wp).w_wincol + (*wp).w_width,
        (*wp).w_p_fcs_chars.horiz,
        win_hl_attr(wp, HLF_C as ::core::ffi::c_int),
    );
    grid_line_flush();
}
unsafe extern "C" fn get_corner_sep_connector(
    mut wp: *mut win_T,
    mut corner: WindowCorner,
) -> schar_T {
    if vsep_connected(wp, corner) {
        if hsep_connected(wp, corner) {
            return (*wp).w_p_fcs_chars.verthoriz;
        } else if corner as ::core::ffi::c_uint
            == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
            || corner as ::core::ffi::c_uint
                == WC_BOTTOM_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return (*wp).w_p_fcs_chars.vertright;
        } else {
            return (*wp).w_p_fcs_chars.vertleft;
        }
    } else if corner as ::core::ffi::c_uint
        == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        || corner as ::core::ffi::c_uint
            == WC_TOP_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (*wp).w_p_fcs_chars.horizdown;
    } else {
        return (*wp).w_p_fcs_chars.horizup;
    };
}
unsafe extern "C" fn draw_sep_connectors_win(mut wp: *mut win_T) {
    if global_stl_height() == 0 as ::core::ffi::c_int
        || !((*wp).w_hsep_height == 1 as ::core::ffi::c_int
            || (*wp).w_vsep_width == 1 as ::core::ffi::c_int)
    {
        return;
    }
    let mut hl: ::core::ffi::c_int = win_hl_attr(wp, HLF_C as ::core::ffi::c_int);
    let mut win_at_top: bool = false;
    let mut win_at_bottom: bool = (*wp).w_hsep_height == 0 as ::core::ffi::c_int;
    let mut win_at_left: bool = false;
    let mut win_at_right: bool = (*wp).w_vsep_width == 0 as ::core::ffi::c_int;
    let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    frp = (*wp).w_frame;
    while !(*frp).fr_parent.is_null() {
        if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
            && !(*frp).fr_prev.is_null()
        {
            break;
        }
        frp = (*frp).fr_parent;
    }
    win_at_top = (*frp).fr_parent.is_null();
    frp = (*wp).w_frame;
    while !(*frp).fr_parent.is_null() {
        if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
            && !(*frp).fr_prev.is_null()
        {
            break;
        }
        frp = (*frp).fr_parent;
    }
    win_at_left = (*frp).fr_parent.is_null();
    let mut top_left: bool =
        !(win_at_top as ::core::ffi::c_int != 0 || win_at_left as ::core::ffi::c_int != 0);
    let mut top_right: bool =
        !(win_at_top as ::core::ffi::c_int != 0 || win_at_right as ::core::ffi::c_int != 0);
    let mut bot_left: bool =
        !(win_at_bottom as ::core::ffi::c_int != 0 || win_at_left as ::core::ffi::c_int != 0);
    let mut bot_right: bool =
        !(win_at_bottom as ::core::ffi::c_int != 0 || win_at_right as ::core::ffi::c_int != 0);
    if top_left {
        grid_line_start(
            default_gridview.ptr(),
            (*wp).w_winrow - 1 as ::core::ffi::c_int,
        );
        grid_line_put_schar(
            (*wp).w_wincol - 1 as ::core::ffi::c_int,
            get_corner_sep_connector(wp, WC_TOP_LEFT),
            hl,
        );
        grid_line_flush();
    }
    if top_right {
        grid_line_start(
            default_gridview.ptr(),
            (*wp).w_winrow - 1 as ::core::ffi::c_int,
        );
        grid_line_put_schar(
            (*wp).w_wincol + (*wp).w_width,
            get_corner_sep_connector(wp, WC_TOP_RIGHT),
            hl,
        );
        grid_line_flush();
    }
    if bot_left {
        grid_line_start(default_gridview.ptr(), (*wp).w_winrow + (*wp).w_height);
        grid_line_put_schar(
            (*wp).w_wincol - 1 as ::core::ffi::c_int,
            get_corner_sep_connector(wp, WC_BOTTOM_LEFT),
            hl,
        );
        grid_line_flush();
    }
    if bot_right {
        grid_line_start(default_gridview.ptr(), (*wp).w_winrow + (*wp).w_height);
        grid_line_put_schar(
            (*wp).w_wincol + (*wp).w_width,
            get_corner_sep_connector(wp, WC_BOTTOM_RIGHT),
            hl,
        );
        grid_line_flush();
    }
}
unsafe extern "C" fn win_update(mut wp: *mut win_T) {
    let mut old_botline: linenr_T = 0;
    if (*wp).w_grid.target == default_grid.ptr() && (*wp).w_wincol >= Columns.get() {
        return;
    }
    let mut top_end: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut mid_start: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
    let mut mid_end: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut bot_start: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
    let mut scrolled_down: bool = false_0 != 0;
    let mut scrolled_for_mod: bool = false_0 != 0;
    let mut top_to_mod: bool = false_0 != 0;
    let mut bot_scroll_start: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut did_update: C2Rust_Unnamed_25 = DID_NONE;
    let mut syntax_last_parsed: linenr_T = 0 as linenr_T;
    let mut mod_top: linenr_T = 0 as linenr_T;
    let mut mod_bot: linenr_T = 0 as linenr_T;
    let mut type_0: ::core::ffi::c_int = (*wp).w_redr_type;
    if type_0 >= UPD_NOT_VALID as ::core::ffi::c_int {
        (*wp).w_redr_status = true_0 != 0;
        (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
    }
    if (*wp).w_view_height == 0 as ::core::ffi::c_int {
        draw_hsep_win(wp);
        (*wp).w_redr_type = 0 as ::core::ffi::c_int;
        return;
    }
    if (*wp).w_view_width == 0 as ::core::ffi::c_int {
        draw_vsep_win(wp);
        (*wp).w_redr_type = 0 as ::core::ffi::c_int;
        return;
    }
    let mut buf: *mut buf_T = (*wp).w_buffer;
    let mut save_got_int: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
    got_int.set(false);
    let mut syntax_tm: proftime_T = profile_setlimit(p_rdt.get() as int64_t);
    syn_set_timeout(&raw mut syntax_tm);
    (*win_extmark_arr.ptr()).size = 0 as size_t;
    decor_redraw_reset(wp, decor_state.ptr());
    decor_providers_invoke_win(wp);
    if !(*buf).terminal.is_null() && terminal_suspended((*buf).terminal) as ::core::ffi::c_int != 0
    {
        static chunk: GlobalCell<VirtTextChunk> = GlobalCell::new(VirtTextChunk {
            text: b"[Process suspended]\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            hl_id: -1 as ::core::ffi::c_int,
        });
        static virt_text: GlobalCell<DecorVirtText> = GlobalCell::new(DecorVirtText {
            flags: 0,
            hl_mode: 0,
            priority: DECOR_PRIORITY_BASE as DecorPriority,
            width: 0,
            col: 0,
            pos: kVPosWinCol,
            data: C2Rust_Unnamed_2 {
                virt_text: VirtText {
                    size: 1 as size_t,
                    capacity: 0,
                    items: (chunk.as_raw() as *const _) as *mut VirtTextChunk,
                },
            },
            next: ::core::ptr::null_mut::<DecorVirtText>(),
        });
        decor_range_add_virt(
            decor_state.ptr(),
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            virt_text.ptr(),
            false_0 != 0,
        );
    }
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        if (*win).w_buffer == (*wp).w_buffer && win_redraw_signcols(win) as ::core::ffi::c_int != 0
        {
            changed_line_abv_curs_win(win);
            redraw_later(win, UPD_NOT_VALID as ::core::ffi::c_int);
        }
        win = (*win).w_next;
    }
    (*buf).b_signcols.last_max = (*buf).b_signcols.max;
    validate_virtcol(wp);
    type_0 = (*wp).w_redr_type;
    init_search_hl(wp, screen_search_hl.ptr());
    if (*wp).w_skipcol > 0 as ::core::ffi::c_int && (*wp).w_view_width > win_col_off(wp) {
        let mut w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut width1: ::core::ffi::c_int = (*wp).w_view_width - win_col_off(wp);
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
        let mut add: ::core::ffi::c_int = width1;
        while w < (*wp).w_skipcol {
            if w > 0 as ::core::ffi::c_int {
                add = width2;
            }
            w += add;
        }
        if w != (*wp).w_skipcol {
            (*wp).w_skipcol = (w - add) as colnr_T;
        }
    }
    let nrwidth_before: ::core::ffi::c_int = (*wp).w_nrwidth;
    let mut nrwidth_new: ::core::ffi::c_int = if (*wp).w_onebuf_opt.wo_nu != 0
        || (*wp).w_onebuf_opt.wo_rnu != 0
        || *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != 0
    {
        number_width(wp)
    } else {
        0 as ::core::ffi::c_int
    };
    if (*wp).w_nrwidth != nrwidth_new {
        type_0 = UPD_NOT_VALID as ::core::ffi::c_int;
        changed_line_abv_curs_win(wp);
        (*wp).w_nrwidth = nrwidth_new;
    } else {
        mod_top = (*wp).w_redraw_top;
        if (*wp).w_redraw_bot != 0 as linenr_T {
            mod_bot = (*wp).w_redraw_bot + 1 as linenr_T;
        } else {
            mod_bot = 0 as ::core::ffi::c_int as linenr_T;
        }
        if (*buf).b_mod_set {
            if mod_top == 0 as linenr_T || mod_top > (*buf).b_mod_top {
                mod_top = (*buf).b_mod_top;
                if syntax_present(wp) {
                    mod_top -= (*buf).b_s.b_syn_sync_linebreaks;
                    mod_top = if mod_top > 1 as linenr_T {
                        mod_top
                    } else {
                        1 as linenr_T
                    };
                }
            }
            if mod_bot == 0 as linenr_T || mod_bot < (*buf).b_mod_bot {
                mod_bot = (*buf).b_mod_bot;
            }
            if !(*screen_search_hl.ptr()).rm.regprog.is_null()
                && re_multiline((*screen_search_hl.ptr()).rm.regprog) != 0
            {
                top_to_mod = true_0 != 0;
            } else {
                let mut cur: *const matchitem_T = (*wp).w_match_head;
                while !cur.is_null() {
                    if !(*cur).mit_match.regprog.is_null()
                        && re_multiline((*cur).mit_match.regprog) != 0
                    {
                        top_to_mod = true_0 != 0;
                        break;
                    } else {
                        cur = (*cur).mit_next;
                    }
                }
            }
        }
        if search_hl_has_cursor_lnum.get() > 0 as linenr_T {
            if mod_top == 0 as linenr_T || mod_top > search_hl_has_cursor_lnum.get() {
                mod_top = search_hl_has_cursor_lnum.get();
            }
            if mod_bot == 0 as linenr_T || mod_bot < search_hl_has_cursor_lnum.get() + 1 as linenr_T
            {
                mod_bot = search_hl_has_cursor_lnum.get() + 1 as linenr_T;
            }
        }
        if mod_top != 0 as linenr_T && win_lines_concealed(wp) as ::core::ffi::c_int != 0 {
            let mut lnumt: linenr_T = (*wp).w_topline;
            let mut lnumb: linenr_T = MAXLNUM as ::core::ffi::c_int as linenr_T;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(i as isize)).wl_valid {
                    if (*(*wp).w_lines.offset(i as isize)).wl_lastlnum < mod_top {
                        lnumt = (*(*wp).w_lines.offset(i as isize)).wl_lastlnum + 1 as linenr_T;
                    }
                    if lnumb == MAXLNUM as ::core::ffi::c_int as linenr_T
                        && (*(*wp).w_lines.offset(i as isize)).wl_lnum >= mod_bot
                    {
                        lnumb = (*(*wp).w_lines.offset(i as isize)).wl_lnum;
                        if compute_foldcolumn(wp, 0 as ::core::ffi::c_int) > 0 as ::core::ffi::c_int
                        {
                            lnumb += 1;
                        }
                    }
                }
                i += 1;
            }
            hasFolding(
                wp,
                mod_top,
                &raw mut mod_top,
                ::core::ptr::null_mut::<linenr_T>(),
            );
            mod_top = if mod_top < lnumt { mod_top } else { lnumt };
            mod_bot -= 1;
            hasFolding(
                wp,
                mod_bot,
                ::core::ptr::null_mut::<linenr_T>(),
                &raw mut mod_bot,
            );
            mod_bot += 1;
            mod_bot = if mod_bot > lnumb { mod_bot } else { lnumb };
        }
        if mod_top != 0 as linenr_T && mod_top < (*wp).w_topline {
            if mod_bot > (*wp).w_topline {
                mod_top = (*wp).w_topline;
            } else if syntax_present(wp) {
                top_end = 1 as ::core::ffi::c_int;
            }
        }
    }
    (*wp).w_redraw_top = 0 as ::core::ffi::c_int as linenr_T;
    (*wp).w_redraw_bot = 0 as ::core::ffi::c_int as linenr_T;
    search_hl_has_cursor_lnum.set(0 as ::core::ffi::c_int as linenr_T);
    if type_0 == UPD_REDRAW_TOP as ::core::ffi::c_int {
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*wp).w_lines_valid {
            j += (*(*wp).w_lines.offset(i_0 as isize)).wl_size as ::core::ffi::c_int;
            if j >= (*wp).w_upd_rows {
                top_end = j;
                break;
            } else {
                i_0 += 1;
            }
        }
        if top_end == 0 as ::core::ffi::c_int {
            type_0 = UPD_NOT_VALID as ::core::ffi::c_int;
        } else {
            type_0 = UPD_VALID as ::core::ffi::c_int;
        }
    }
    let mut topline_conceal: linenr_T = (*wp).w_topline;
    while topline_conceal < (*buf).b_ml.ml_line_count
        && decor_conceal_line(
            wp,
            topline_conceal as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            false_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        topline_conceal += 1;
        hasFolding(
            wp,
            topline_conceal,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut topline_conceal,
        );
    }
    if (type_0 == UPD_VALID as ::core::ffi::c_int
        || type_0 == UPD_SOME_VALID as ::core::ffi::c_int
        || type_0 == UPD_INVERTED as ::core::ffi::c_int
        || type_0 == UPD_INVERTED_ALL as ::core::ffi::c_int)
        && !(*wp).w_botfill
        && !(*wp).w_old_botfill
    {
        if !(mod_top != 0 as linenr_T
            && (*wp).w_topline == mod_top
            && (!(*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_valid
                || topline_conceal
                    == (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum))
        {
            if (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_valid
                as ::core::ffi::c_int
                != 0
                && (topline_conceal
                    < (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                    || topline_conceal
                        == (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        && (*wp).w_topfill > (*wp).w_old_topfill)
            {
                let mut j_0: ::core::ffi::c_int = 0;
                if win_lines_concealed(wp) {
                    j_0 = 0 as ::core::ffi::c_int;
                    let mut ln: linenr_T = (*wp).w_topline;
                    while ln < (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum {
                        j_0 += !decor_conceal_line(
                            wp,
                            ln as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                            false_0 != 0,
                        ) as ::core::ffi::c_int;
                        if j_0 >= (*wp).w_view_height - 2 as ::core::ffi::c_int {
                            break;
                        }
                        hasFolding(wp, ln, ::core::ptr::null_mut::<linenr_T>(), &raw mut ln);
                        ln += 1;
                    }
                } else {
                    j_0 = ((*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        - (*wp).w_topline) as ::core::ffi::c_int;
                }
                if j_0 < (*wp).w_view_height - 2 as ::core::ffi::c_int {
                    let mut i_1: ::core::ffi::c_int = plines_m_win(
                        wp,
                        (*wp).w_topline,
                        (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                            - 1 as linenr_T,
                        (*wp).w_view_height,
                    );
                    if (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        != (*wp).w_topline
                    {
                        i_1 += win_get_fill(
                            wp,
                            (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum,
                        ) - (*wp).w_old_topfill;
                    }
                    if i_1 != 0 as ::core::ffi::c_int
                        && i_1 < (*wp).w_view_height - 2 as ::core::ffi::c_int
                    {
                        win_scroll_lines(wp, 0 as ::core::ffi::c_int, i_1);
                        bot_scroll_start = 0 as ::core::ffi::c_int;
                        if (*wp).w_lines_valid != 0 as ::core::ffi::c_int {
                            top_end = i_1;
                            scrolled_down = true_0 != 0;
                            (*wp).w_lines_valid += j_0 as linenr_T as ::core::ffi::c_int;
                            if (*wp).w_lines_valid > (*wp).w_view_height {
                                (*wp).w_lines_valid = (*wp).w_view_height;
                            }
                            let mut idx: ::core::ffi::c_int = 0;
                            idx = (*wp).w_lines_valid;
                            while idx - j_0 >= 0 as ::core::ffi::c_int {
                                *(*wp).w_lines.offset(idx as isize) =
                                    *(*wp).w_lines.offset((idx - j_0) as isize);
                                idx -= 1;
                            }
                            while idx >= 0 as ::core::ffi::c_int {
                                let c2rust_fresh0 = idx;
                                idx = idx - 1;
                                (*(*wp).w_lines.offset(c2rust_fresh0 as isize)).wl_valid =
                                    false_0 != 0;
                            }
                        }
                    } else {
                        mid_start = 0 as ::core::ffi::c_int;
                    }
                } else {
                    mid_start = 0 as ::core::ffi::c_int;
                }
            } else {
                let mut j_1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_2 < (*wp).w_lines_valid {
                    if (*(*wp).w_lines.offset(i_2 as isize)).wl_valid as ::core::ffi::c_int != 0
                        && (*(*wp).w_lines.offset(i_2 as isize)).wl_lnum == (*wp).w_topline
                    {
                        j_1 = i_2;
                        break;
                    } else {
                        row += (*(*wp).w_lines.offset(i_2 as isize)).wl_size as ::core::ffi::c_int;
                        i_2 += 1;
                    }
                }
                if j_1 == -1 as ::core::ffi::c_int {
                    mid_start = 0 as ::core::ffi::c_int;
                } else {
                    if (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_lnum
                        == (*wp).w_topline
                    {
                        row += (*wp).w_old_topfill;
                    } else {
                        row += win_get_fill(wp, (*wp).w_topline);
                    }
                    row -= (*wp).w_topfill;
                    if row > 0 as ::core::ffi::c_int {
                        win_scroll_lines(wp, 0 as ::core::ffi::c_int, -row);
                        bot_start = (*wp).w_view_height - row;
                        bot_scroll_start = bot_start;
                    }
                    if (row == 0 as ::core::ffi::c_int || bot_start < 999 as ::core::ffi::c_int)
                        && (*wp).w_lines_valid != 0 as ::core::ffi::c_int
                    {
                        bot_start = 0 as ::core::ffi::c_int;
                        let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        loop {
                            *(*wp).w_lines.offset(idx_0 as isize) =
                                *(*wp).w_lines.offset(j_1 as isize);
                            if row > 0 as ::core::ffi::c_int
                                && bot_start
                                    + row
                                    + (*(*wp).w_lines.offset(j_1 as isize)).wl_size
                                        as ::core::ffi::c_int
                                    > (*wp).w_view_height
                            {
                                (*wp).w_lines_valid = idx_0 + 1 as ::core::ffi::c_int;
                                break;
                            } else {
                                let c2rust_fresh1 = idx_0;
                                idx_0 = idx_0 + 1;
                                bot_start += (*(*wp).w_lines.offset(c2rust_fresh1 as isize)).wl_size
                                    as ::core::ffi::c_int;
                                j_1 += 1;
                                if j_1 < (*wp).w_lines_valid {
                                    continue;
                                }
                                (*wp).w_lines_valid = idx_0;
                                break;
                            }
                        }
                        if win_may_fill(wp) as ::core::ffi::c_int != 0
                            && bot_start > 0 as ::core::ffi::c_int
                        {
                            (*(*wp).w_lines.offset(0 as ::core::ffi::c_int as isize)).wl_size =
                                plines_correct_topline(
                                    wp,
                                    (*wp).w_topline,
                                    ::core::ptr::null_mut::<linenr_T>(),
                                    true_0 != 0,
                                    ::core::ptr::null_mut::<bool>(),
                                ) as uint16_t;
                        }
                    }
                }
            }
        }
        if mid_start == 0 as ::core::ffi::c_int {
            mid_end = (*wp).w_view_height;
        }
    } else {
        mid_start = 0 as ::core::ffi::c_int;
        mid_end = (*wp).w_view_height;
    }
    if type_0 == UPD_SOME_VALID as ::core::ffi::c_int {
        mid_start = 0 as ::core::ffi::c_int;
        mid_end = (*wp).w_view_height;
        type_0 = UPD_NOT_VALID as ::core::ffi::c_int;
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0 && buf == (*curwin.get()).w_buffer
        || (*wp).w_old_cursor_lnum != 0 as linenr_T && type_0 != UPD_NOT_VALID as ::core::ffi::c_int
    {
        let mut from: linenr_T = 0;
        let mut to: linenr_T = 0;
        if VIsual_active.get() {
            if VIsual_mode.get() != (*wp).w_old_visual_mode as ::core::ffi::c_int
                || type_0 == UPD_INVERTED_ALL as ::core::ffi::c_int
            {
                if (*curwin.get()).w_cursor.lnum < (*VIsual.ptr()).lnum {
                    from = (*curwin.get()).w_cursor.lnum;
                    to = (*VIsual.ptr()).lnum;
                } else {
                    from = (*VIsual.ptr()).lnum;
                    to = (*curwin.get()).w_cursor.lnum;
                }
                from = if (if from < (*wp).w_old_cursor_lnum {
                    from
                } else {
                    (*wp).w_old_cursor_lnum
                }) < (*wp).w_old_visual_lnum
                {
                    if from < (*wp).w_old_cursor_lnum {
                        from
                    } else {
                        (*wp).w_old_cursor_lnum
                    }
                } else {
                    (*wp).w_old_visual_lnum
                };
                to = if (if to > (*wp).w_old_cursor_lnum {
                    to
                } else {
                    (*wp).w_old_cursor_lnum
                }) > (*wp).w_old_visual_lnum
                {
                    if to > (*wp).w_old_cursor_lnum {
                        to
                    } else {
                        (*wp).w_old_cursor_lnum
                    }
                } else {
                    (*wp).w_old_visual_lnum
                };
            } else {
                if (*curwin.get()).w_cursor.lnum < (*wp).w_old_cursor_lnum {
                    from = (*curwin.get()).w_cursor.lnum;
                    to = (*wp).w_old_cursor_lnum;
                } else {
                    from = (*wp).w_old_cursor_lnum;
                    to = (*curwin.get()).w_cursor.lnum;
                    if from == 0 as linenr_T {
                        from = to;
                    }
                }
                if (*VIsual.ptr()).lnum != (*wp).w_old_visual_lnum
                    || (*VIsual.ptr()).col != (*wp).w_old_visual_col
                {
                    if (*wp).w_old_visual_lnum < from && (*wp).w_old_visual_lnum != 0 as linenr_T {
                        from = (*wp).w_old_visual_lnum;
                    }
                    to = if (if to > (*wp).w_old_visual_lnum {
                        to
                    } else {
                        (*wp).w_old_visual_lnum
                    }) > (*VIsual.ptr()).lnum
                    {
                        if to > (*wp).w_old_visual_lnum {
                            to
                        } else {
                            (*wp).w_old_visual_lnum
                        }
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                    from = if from < (*VIsual.ptr()).lnum {
                        from
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                }
            }
            if VIsual_mode.get() == Ctrl_V {
                let mut fromc: colnr_T = 0;
                let mut toc: colnr_T = 0;
                let mut save_ve_flags: ::core::ffi::c_uint =
                    (*curwin.get()).w_onebuf_opt.wo_ve_flags;
                if (*curwin.get()).w_onebuf_opt.wo_lbr != 0 {
                    (*curwin.get()).w_onebuf_opt.wo_ve_flags =
                        kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint;
                }
                getvcols(
                    wp,
                    VIsual.ptr(),
                    &raw mut (*curwin.get()).w_cursor,
                    &raw mut fromc,
                    &raw mut toc,
                );
                toc += 1;
                (*curwin.get()).w_onebuf_opt.wo_ve_flags = save_ve_flags;
                if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int {
                    if get_ve_flags(curwin.get())
                        & kOptVeFlagBlock as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        let mut pos: pos_T = pos_T {
                            lnum: 0,
                            col: 0,
                            coladd: 0,
                        };
                        let mut cursor_above: ::core::ffi::c_int = ((*curwin.get()).w_cursor.lnum
                            < (*VIsual.ptr()).lnum)
                            as ::core::ffi::c_int;
                        toc = 0 as ::core::ffi::c_int as colnr_T;
                        pos.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        pos.lnum = (*curwin.get()).w_cursor.lnum;
                        while if cursor_above != 0 {
                            (pos.lnum <= (*VIsual.ptr()).lnum) as ::core::ffi::c_int
                        } else {
                            (pos.lnum >= (*VIsual.ptr()).lnum) as ::core::ffi::c_int
                        } != 0
                        {
                            let mut t: colnr_T = 0;
                            pos.col = ml_get_buf_len((*wp).w_buffer, pos.lnum);
                            getvvcol(
                                wp,
                                &raw mut pos,
                                ::core::ptr::null_mut::<colnr_T>(),
                                ::core::ptr::null_mut::<colnr_T>(),
                                &raw mut t,
                            );
                            toc = if toc > t { toc } else { t };
                            pos.lnum = (pos.lnum as ::core::ffi::c_int
                                + if cursor_above != 0 {
                                    1 as ::core::ffi::c_int
                                } else {
                                    -1 as ::core::ffi::c_int
                                }) as linenr_T;
                        }
                        toc += 1;
                    } else {
                        toc = MAXCOL as ::core::ffi::c_int as colnr_T;
                    }
                }
                if fromc != (*wp).w_old_cursor_fcol || toc != (*wp).w_old_cursor_lcol {
                    from = if from < (*VIsual.ptr()).lnum {
                        from
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                    to = if to > (*VIsual.ptr()).lnum {
                        to
                    } else {
                        (*VIsual.ptr()).lnum
                    };
                }
                (*wp).w_old_cursor_fcol = fromc;
                (*wp).w_old_cursor_lcol = toc;
            }
        } else if (*wp).w_old_cursor_lnum < (*wp).w_old_visual_lnum {
            from = (*wp).w_old_cursor_lnum;
            to = (*wp).w_old_visual_lnum;
        } else {
            from = (*wp).w_old_visual_lnum;
            to = (*wp).w_old_cursor_lnum;
        }
        from = if from > (*wp).w_topline {
            from
        } else {
            (*wp).w_topline
        };
        if (*wp).w_valid & VALID_BOTLINE != 0 {
            from = if from < (*wp).w_botline - 1 as linenr_T {
                from
            } else {
                (*wp).w_botline - 1 as linenr_T
            };
            to = if to < (*wp).w_botline - 1 as linenr_T {
                to
            } else {
                (*wp).w_botline - 1 as linenr_T
            };
        }
        if mid_start > 0 as ::core::ffi::c_int {
            let mut lnum: linenr_T = (*wp).w_topline;
            let mut idx_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut srow: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if scrolled_down {
                mid_start = top_end;
            } else {
                mid_start = 0 as ::core::ffi::c_int;
            }
            while lnum < from && idx_1 < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(idx_1 as isize)).wl_valid {
                    mid_start +=
                        (*(*wp).w_lines.offset(idx_1 as isize)).wl_size as ::core::ffi::c_int;
                } else if !scrolled_down {
                    srow += (*(*wp).w_lines.offset(idx_1 as isize)).wl_size as ::core::ffi::c_int;
                }
                idx_1 += 1;
                if idx_1 < (*wp).w_lines_valid
                    && (*(*wp).w_lines.offset(idx_1 as isize)).wl_valid as ::core::ffi::c_int != 0
                {
                    lnum = (*(*wp).w_lines.offset(idx_1 as isize)).wl_lnum;
                } else {
                    lnum += 1;
                }
            }
            srow += mid_start;
            mid_end = (*wp).w_view_height;
            while idx_1 < (*wp).w_lines_valid {
                if (*(*wp).w_lines.offset(idx_1 as isize)).wl_valid as ::core::ffi::c_int != 0
                    && (*(*wp).w_lines.offset(idx_1 as isize)).wl_lnum >= to + 1 as linenr_T
                {
                    mid_end = srow;
                    break;
                } else {
                    srow += (*(*wp).w_lines.offset(idx_1 as isize)).wl_size as ::core::ffi::c_int;
                    idx_1 += 1;
                }
            }
        }
    }
    if VIsual_active.get() as ::core::ffi::c_int != 0 && buf == (*curwin.get()).w_buffer {
        (*wp).w_old_visual_mode = VIsual_mode.get() as ::core::ffi::c_char;
        (*wp).w_old_cursor_lnum = (*curwin.get()).w_cursor.lnum;
        (*wp).w_old_visual_lnum = (*VIsual.ptr()).lnum;
        (*wp).w_old_visual_col = (*VIsual.ptr()).col;
        (*wp).w_old_curswant = (*curwin.get()).w_curswant;
    } else {
        (*wp).w_old_visual_mode = 0 as ::core::ffi::c_char;
        (*wp).w_old_cursor_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*wp).w_old_visual_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*wp).w_old_visual_col = 0 as ::core::ffi::c_int as colnr_T;
    }
    let mut cursorline_fi: foldinfo_T = foldinfo_T {
        fi_lnum: 0 as linenr_T,
        fi_level: 0,
        fi_low_level: 0,
        fi_lines: 0,
    };
    win_update_cursorline(wp, &raw mut cursorline_fi);
    if wp == curwin.get() {
        conceal_cursor_used.set(conceal_cursor_line(curwin.get()));
    }
    win_check_ns_hl(wp);
    let mut spv: spellvars_T = spellvars_T {
        spv_has_spell: false,
        spv_unchanged: false,
        spv_checked_col: 0,
        spv_checked_lnum: 0,
        spv_cap_col: 0,
        spv_capcol_lnum: 0,
    };
    let mut lnum_0: linenr_T = (*wp).w_topline;
    if spell_check_window(wp) {
        spv.spv_has_spell = true_0 != 0;
        spv.spv_unchanged = mod_top == 0 as linenr_T;
    }
    let mut idx_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut row_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut srow_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut eof: bool = false_0 != 0;
    let mut didline: bool = false_0 != 0;
    's_2363: {
        's_2327: loop {
            '_redr_statuscol: {
                's_2139: {
                    if row_0 == (*wp).w_view_height {
                        didline = true_0 != 0;
                    } else if lnum_0 > (*buf).b_ml.ml_line_count {
                        eof = true_0 != 0;
                    } else {
                        srow_0 = row_0;
                        if row_0 < top_end
                            || row_0 >= mid_start && row_0 < mid_end
                            || top_to_mod as ::core::ffi::c_int != 0
                            || idx_2 >= (*wp).w_lines_valid
                            || row_0
                                + (*(*wp).w_lines.offset(idx_2 as isize)).wl_size
                                    as ::core::ffi::c_int
                                > bot_start
                            || mod_top != 0 as linenr_T
                                && (lnum_0 == mod_top
                                    || lnum_0 >= mod_top
                                        && (lnum_0 < mod_bot
                                            || did_update as ::core::ffi::c_uint
                                                == DID_FOLD as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || did_update as ::core::ffi::c_uint
                                                == DID_LINE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                && syntax_present(wp) as ::core::ffi::c_int != 0
                                                && (foldmethodIsSyntax(wp) as ::core::ffi::c_int
                                                    != 0
                                                    && hasAnyFolding(wp) != 0
                                                    || syntax_check_changed(lnum_0)
                                                        as ::core::ffi::c_int
                                                        != 0)
                                            || !(*wp).w_match_head.is_null()
                                                && (*buf).b_mod_set as ::core::ffi::c_int != 0
                                                && (*buf).b_mod_xlines != 0 as linenr_T))
                            || lnum_0 == (*wp).w_cursorline
                            || lnum_0 == (*wp).w_last_cursorline
                        {
                            if lnum_0 == mod_top {
                                top_to_mod = false_0 != 0;
                            }
                            let mut foldinfo: foldinfo_T = if (*wp).w_onebuf_opt.wo_cul != 0
                                && lnum_0 == (*wp).w_cursor.lnum
                            {
                                cursorline_fi
                            } else {
                                fold_info(wp, lnum_0)
                            };
                            let mut concealed: bool = decor_conceal_line(
                                wp,
                                lnum_0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                false_0 != 0,
                            );
                            if concealed as ::core::ffi::c_int != 0
                                && win_get_fill(wp, lnum_0) == 0 as ::core::ffi::c_int
                            {
                                if lnum_0 == mod_top && lnum_0 < mod_bot {
                                    mod_top = (mod_top as ::core::ffi::c_int
                                        + (if foldinfo.fi_lines != 0 {
                                            foldinfo.fi_lines
                                        } else {
                                            1 as linenr_T
                                        })
                                            as ::core::ffi::c_int)
                                        as linenr_T;
                                }
                                lnum_0 = (lnum_0 as ::core::ffi::c_int
                                    + (if foldinfo.fi_lines != 0 {
                                        foldinfo.fi_lines
                                    } else {
                                        1 as linenr_T
                                    }) as ::core::ffi::c_int)
                                    as linenr_T;
                                spv.spv_capcol_lnum = 0 as ::core::ffi::c_int as linenr_T;
                                continue 's_2327;
                            } else {
                                if !scrolled_for_mod
                                    && mod_bot != MAXLNUM as ::core::ffi::c_int as linenr_T
                                    && lnum_0 >= mod_top
                                    && lnum_0
                                        < (if mod_bot > mod_top + 1 as linenr_T {
                                            mod_bot
                                        } else {
                                            mod_top + 1 as linenr_T
                                        })
                                    && (!scrolled_down || row_0 >= top_end)
                                {
                                    scrolled_for_mod = true_0 != 0;
                                    let mut old_cline_height: ::core::ffi::c_int =
                                        0 as ::core::ffi::c_int;
                                    let mut old_rows: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    let mut l: linenr_T = 0;
                                    let mut i_3: ::core::ffi::c_int = 0;
                                    i_3 = idx_2;
                                    while i_3 < (*wp).w_lines_valid {
                                        if (*(*wp).w_lines.offset(i_3 as isize)).wl_valid
                                            as ::core::ffi::c_int
                                            != 0
                                            && (*(*wp).w_lines.offset(i_3 as isize)).wl_lnum
                                                == mod_bot
                                        {
                                            break;
                                        }
                                        if (*(*wp).w_lines.offset(i_3 as isize)).wl_lnum
                                            == (*wp).w_cursor.lnum
                                        {
                                            old_cline_height = (*(*wp).w_lines.offset(i_3 as isize))
                                                .wl_size
                                                as ::core::ffi::c_int;
                                        }
                                        old_rows += (*(*wp).w_lines.offset(i_3 as isize)).wl_size
                                            as ::core::ffi::c_int;
                                        if (*(*wp).w_lines.offset(i_3 as isize)).wl_valid
                                            as ::core::ffi::c_int
                                            != 0
                                            && (*(*wp).w_lines.offset(i_3 as isize)).wl_lastlnum
                                                + 1 as linenr_T
                                                == mod_bot
                                        {
                                            i_3 += 1;
                                            while i_3 < (*wp).w_lines_valid
                                                && !(*(*wp).w_lines.offset(i_3 as isize)).wl_valid
                                            {
                                                let c2rust_fresh2 = i_3;
                                                i_3 = i_3 + 1;
                                                old_rows +=
                                                    (*(*wp).w_lines.offset(c2rust_fresh2 as isize))
                                                        .wl_size
                                                        as ::core::ffi::c_int;
                                            }
                                            break;
                                        } else {
                                            i_3 += 1;
                                        }
                                    }
                                    if i_3 >= (*wp).w_lines_valid {
                                        bot_start = 0 as ::core::ffi::c_int;
                                        bot_scroll_start = 0 as ::core::ffi::c_int;
                                    } else {
                                        let mut new_rows: ::core::ffi::c_int =
                                            0 as ::core::ffi::c_int;
                                        let mut j_2: ::core::ffi::c_int = idx_2;
                                        l = lnum_0;
                                        while l < mod_bot {
                                            if dollar_vcol.get() >= 0 as ::core::ffi::c_int
                                                && wp == curwin.get()
                                                && old_cline_height > 0 as ::core::ffi::c_int
                                                && l == (*wp).w_cursor.lnum
                                            {
                                                new_rows += old_cline_height;
                                                j_2 += 1;
                                            } else {
                                                let mut n: ::core::ffi::c_int =
                                                    plines_correct_topline(
                                                        wp,
                                                        l,
                                                        &raw mut l,
                                                        true_0 != 0,
                                                        ::core::ptr::null_mut::<bool>(),
                                                    );
                                                new_rows += n;
                                                j_2 += (n > 0 as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int;
                                            }
                                            if new_rows
                                                > (*wp).w_view_height
                                                    - row_0
                                                    - 2 as ::core::ffi::c_int
                                            {
                                                new_rows = 9999 as ::core::ffi::c_int;
                                                break;
                                            } else {
                                                l += 1;
                                            }
                                        }
                                        let mut xtra_rows: ::core::ffi::c_int = new_rows - old_rows;
                                        if xtra_rows < 0 as ::core::ffi::c_int {
                                            if row_0 - xtra_rows
                                                >= (*wp).w_view_height - 2 as ::core::ffi::c_int
                                            {
                                                mod_bot = MAXLNUM as ::core::ffi::c_int as linenr_T;
                                            } else {
                                                win_scroll_lines(wp, row_0, xtra_rows);
                                                bot_start = (*wp).w_view_height + xtra_rows;
                                                bot_scroll_start = bot_start;
                                            }
                                        } else if xtra_rows > 0 as ::core::ffi::c_int {
                                            if row_0 + xtra_rows
                                                >= (*wp).w_view_height - 2 as ::core::ffi::c_int
                                            {
                                                mod_bot = MAXLNUM as ::core::ffi::c_int as linenr_T;
                                            } else {
                                                win_scroll_lines(wp, row_0 + old_rows, xtra_rows);
                                                bot_scroll_start = 0 as ::core::ffi::c_int;
                                                if top_end > row_0 + old_rows {
                                                    top_end += xtra_rows;
                                                }
                                            }
                                        }
                                        if mod_bot != MAXLNUM as ::core::ffi::c_int as linenr_T
                                            && i_3 != j_2
                                        {
                                            if j_2 < i_3 {
                                                let mut x: ::core::ffi::c_int = row_0 + new_rows;
                                                loop {
                                                    if i_3 >= (*wp).w_lines_valid {
                                                        (*wp).w_lines_valid = j_2;
                                                        break;
                                                    } else {
                                                        *(*wp).w_lines.offset(j_2 as isize) =
                                                            *(*wp).w_lines.offset(i_3 as isize);
                                                        if x + (*(*wp).w_lines.offset(j_2 as isize))
                                                            .wl_size
                                                            as ::core::ffi::c_int
                                                            > (*wp).w_view_height
                                                        {
                                                            (*wp).w_lines_valid =
                                                                j_2 + 1 as ::core::ffi::c_int;
                                                            break;
                                                        } else {
                                                            let c2rust_fresh3 = j_2;
                                                            j_2 = j_2 + 1;
                                                            x += (*(*wp)
                                                                .w_lines
                                                                .offset(c2rust_fresh3 as isize))
                                                            .wl_size
                                                                as ::core::ffi::c_int;
                                                            i_3 += 1;
                                                        }
                                                    }
                                                }
                                                bot_start =
                                                    if bot_start < x { bot_start } else { x };
                                            } else {
                                                j_2 -= i_3;
                                                (*wp).w_lines_valid +=
                                                    j_2 as linenr_T as ::core::ffi::c_int;
                                                (*wp).w_lines_valid =
                                                    if (*wp).w_lines_valid < (*wp).w_view_height {
                                                        (*wp).w_lines_valid
                                                    } else {
                                                        (*wp).w_view_height
                                                    };
                                                i_3 = (*wp).w_lines_valid;
                                                while i_3 - j_2 >= idx_2 {
                                                    *(*wp).w_lines.offset(i_3 as isize) =
                                                        *(*wp).w_lines.offset((i_3 - j_2) as isize);
                                                    i_3 -= 1;
                                                }
                                                while i_3 >= idx_2 {
                                                    (*(*wp).w_lines.offset(i_3 as isize)).wl_size =
                                                        0 as uint16_t;
                                                    let c2rust_fresh4 = i_3;
                                                    i_3 = i_3 - 1;
                                                    (*(*wp)
                                                        .w_lines
                                                        .offset(c2rust_fresh4 as isize))
                                                    .wl_valid = false_0 != 0;
                                                }
                                            }
                                        }
                                    }
                                }
                                if foldinfo.fi_lines == 0 as linenr_T
                                    && idx_2 < (*wp).w_lines_valid
                                    && (*(*wp).w_lines.offset(idx_2 as isize)).wl_valid
                                        as ::core::ffi::c_int
                                        != 0
                                    && (*(*wp).w_lines.offset(idx_2 as isize)).wl_lnum == lnum_0
                                    && lnum_0 > (*wp).w_topline
                                    && dy_flags.get()
                                        & (kOptDyFlagLastline as ::core::ffi::c_int
                                            | kOptDyFlagTruncate as ::core::ffi::c_int)
                                            as ::core::ffi::c_uint
                                        == 0
                                    && srow_0
                                        + (*(*wp).w_lines.offset(idx_2 as isize)).wl_size
                                            as ::core::ffi::c_int
                                        > (*wp).w_view_height
                                    && win_get_fill(wp, lnum_0) == 0 as ::core::ffi::c_int
                                {
                                    row_0 = (*wp).w_view_height + 1 as ::core::ffi::c_int;
                                } else {
                                    prepare_search_hl(wp, screen_search_hl.ptr(), lnum_0);
                                    if syntax_last_parsed != 0 as linenr_T
                                        && (syntax_last_parsed + 1 as linenr_T) < lnum_0
                                        && syntax_present(wp) as ::core::ffi::c_int != 0
                                    {
                                        syntax_end_parsing(wp, syntax_last_parsed + 1 as linenr_T);
                                    }
                                    let mut display_buf_line: bool = !concealed
                                        && (foldinfo.fi_lines == 0 as linenr_T
                                            || *(*wp).w_onebuf_opt.wo_fdt as ::core::ffi::c_int
                                                == NUL);
                                    let mut zero_spv: spellvars_T = spellvars_T {
                                        spv_has_spell: false,
                                        spv_unchanged: false,
                                        spv_checked_col: 0,
                                        spv_checked_lnum: 0,
                                        spv_cap_col: 0,
                                        spv_capcol_lnum: 0,
                                    };
                                    row_0 = win_line(
                                        wp,
                                        lnum_0,
                                        srow_0,
                                        (*wp).w_view_height,
                                        0 as ::core::ffi::c_int,
                                        concealed,
                                        if display_buf_line as ::core::ffi::c_int != 0 {
                                            &raw mut spv
                                        } else {
                                            &raw mut zero_spv
                                        },
                                        foldinfo,
                                    );
                                    if display_buf_line {
                                        syntax_last_parsed = lnum_0;
                                    } else {
                                        spv.spv_capcol_lnum = 0 as ::core::ffi::c_int as linenr_T;
                                    }
                                    let mut lastlnum: linenr_T = lnum_0 + foldinfo.fi_lines
                                        - (foldinfo.fi_lines > 0 as linenr_T) as ::core::ffi::c_int;
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_folded =
                                        foldinfo.fi_lines > 0 as linenr_T;
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_foldend = lastlnum;
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum = lastlnum;
                                    did_update = (if foldinfo.fi_lines > 0 as linenr_T {
                                        DID_FOLD as ::core::ffi::c_int
                                    } else {
                                        DID_LINE as ::core::ffi::c_int
                                    })
                                        as C2Rust_Unnamed_25;
                                    let mut virt_below: bool = decor_virt_lines(
                                        wp,
                                        lastlnum as ::core::ffi::c_int,
                                        lastlnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                        ::core::ptr::null_mut::<VirtLines>(),
                                        true_0 != 0,
                                    ) > 0 as ::core::ffi::c_int;
                                    while !virt_below
                                        && (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum
                                            < (*buf).b_ml.ml_line_count
                                        && decor_conceal_line(
                                            wp,
                                            (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum
                                                as ::core::ffi::c_int,
                                            false_0 != 0,
                                        )
                                            as ::core::ffi::c_int
                                            != 0
                                    {
                                        virt_below = false_0 != 0;
                                        (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum += 1;
                                        hasFolding(
                                            wp,
                                            (*(*wp).w_lines.offset(idx_2 as isize)).wl_lastlnum,
                                            ::core::ptr::null_mut::<linenr_T>(),
                                            &raw mut (*(*wp).w_lines.offset(idx_2 as isize))
                                                .wl_lastlnum,
                                        );
                                    }
                                }
                                (*(*wp).w_lines.offset(idx_2 as isize)).wl_lnum = lnum_0;
                                (*(*wp).w_lines.offset(idx_2 as isize)).wl_valid = true_0 != 0;
                                let mut is_curline: bool =
                                    wp == curwin.get() && lnum_0 == (*wp).w_cursor.lnum;
                                if row_0 > (*wp).w_view_height {
                                    if dollar_vcol.get() == -1 as ::core::ffi::c_int || !is_curline
                                    {
                                        (*(*wp).w_lines.offset(idx_2 as isize)).wl_size =
                                            plines_win(wp, lnum_0, true_0 != 0) as uint16_t;
                                    }
                                    idx_2 += 1;
                                    break 's_2139;
                                } else {
                                    if dollar_vcol.get() == -1 as ::core::ffi::c_int || !is_curline
                                    {
                                        (*(*wp).w_lines.offset(idx_2 as isize)).wl_size =
                                            (row_0 - srow_0) as uint16_t;
                                    }
                                    let c2rust_fresh5 = idx_2;
                                    idx_2 = idx_2 + 1;
                                    lnum_0 = (*(*wp).w_lines.offset(c2rust_fresh5 as isize))
                                        .wl_lastlnum
                                        + 1 as linenr_T;
                                }
                            }
                        } else {
                            if (*wp).w_onebuf_opt.wo_nu != 0
                                && mod_top != 0 as linenr_T
                                && lnum_0 >= mod_bot
                                && (*buf).b_mod_set as ::core::ffi::c_int != 0
                                && (*buf).b_mod_xlines != 0 as linenr_T
                                || (*wp).w_onebuf_opt.wo_rnu != 0
                                    && (*wp).w_last_cursor_lnum_rnu != (*wp).w_cursor.lnum
                            {
                                let mut info: foldinfo_T = if (*wp).w_onebuf_opt.wo_cul != 0
                                    && lnum_0 == (*wp).w_cursor.lnum
                                {
                                    cursorline_fi
                                } else {
                                    fold_info(wp, lnum_0)
                                };
                                win_line(
                                    wp,
                                    lnum_0,
                                    srow_0,
                                    (*wp).w_view_height,
                                    (*(*wp).w_lines.offset(idx_2 as isize)).wl_size
                                        as ::core::ffi::c_int,
                                    false_0 != 0,
                                    &raw mut spv,
                                    info,
                                );
                            }
                            let c2rust_fresh6 = idx_2;
                            idx_2 = idx_2 + 1;
                            row_0 += (*(*wp).w_lines.offset(c2rust_fresh6 as isize)).wl_size
                                as ::core::ffi::c_int;
                            if row_0 > (*wp).w_view_height {
                                break 's_2139;
                            } else {
                                lnum_0 = (*(*wp)
                                    .w_lines
                                    .offset((idx_2 - 1 as ::core::ffi::c_int) as isize))
                                .wl_lastlnum
                                    + 1 as linenr_T;
                                did_update = DID_NONE;
                                spv.spv_capcol_lnum = 0 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                        if (*wp).w_redr_statuscol {
                            break '_redr_statuscol;
                        } else {
                            if lnum_0 <= (*buf).b_ml.ml_line_count {
                                continue 's_2327;
                            }
                            eof = true_0 != 0;
                        }
                    }
                }
                (*wp).w_last_cursorline = (*wp).w_cursorline;
                (*wp).w_last_cursor_lnum_rnu = if (*wp).w_onebuf_opt.wo_rnu != 0 {
                    (*wp).w_cursor.lnum
                } else {
                    0 as linenr_T
                };
                (*wp).w_lines_valid = if (*wp).w_lines_valid > idx_2 {
                    (*wp).w_lines_valid
                } else {
                    idx_2
                };
                (*wp).w_display_tick = display_tick.get();
                if syntax_last_parsed != 0 as linenr_T
                    && syntax_present(wp) as ::core::ffi::c_int != 0
                {
                    syntax_end_parsing(wp, syntax_last_parsed + 1 as linenr_T);
                }
                old_botline = (*wp).w_botline;
                (*wp).w_empty_rows = 0 as ::core::ffi::c_int;
                (*wp).w_filler_rows = 0 as ::core::ffi::c_int;
                if !eof && !didline {
                    let mut at_attr: ::core::ffi::c_int = hl_combine_attr(
                        win_bg_attr(wp),
                        win_hl_attr(wp, HLF_AT as ::core::ffi::c_int),
                    );
                    if lnum_0 == (*wp).w_topline {
                        (*wp).w_botline = lnum_0 + 1 as linenr_T;
                    } else if win_get_fill(wp, lnum_0) >= (*wp).w_view_height - srow_0 {
                        (*wp).w_botline = lnum_0;
                        (*wp).w_filler_rows = (*wp).w_view_height - srow_0;
                    } else if dy_flags.get()
                        & kOptDyFlagTruncate as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        grid_line_start(
                            &raw mut (*wp).w_grid,
                            (*wp).w_view_height - 1 as ::core::ffi::c_int,
                        );
                        grid_line_fill(
                            0 as ::core::ffi::c_int,
                            if (*wp).w_view_width < 3 as ::core::ffi::c_int {
                                (*wp).w_view_width
                            } else {
                                3 as ::core::ffi::c_int
                            },
                            (*wp).w_p_fcs_chars.lastline,
                            at_attr,
                        );
                        grid_line_fill(
                            3 as ::core::ffi::c_int,
                            (*wp).w_view_width,
                            ' ' as ::core::ffi::c_int as schar_T,
                            at_attr,
                        );
                        grid_line_flush();
                        set_empty_rows(wp, srow_0);
                        (*wp).w_botline = lnum_0;
                    } else if dy_flags.get()
                        & kOptDyFlagLastline as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        grid_line_start(
                            &raw mut (*wp).w_grid,
                            (*wp).w_view_height - 1 as ::core::ffi::c_int,
                        );
                        let mut width: ::core::ffi::c_int = if grid_line_getchar(
                            if (*wp).w_view_width - 3 as ::core::ffi::c_int
                                > 0 as ::core::ffi::c_int
                            {
                                (*wp).w_view_width - 3 as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            },
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        ) == NUL as schar_T
                        {
                            4 as ::core::ffi::c_int
                        } else {
                            3 as ::core::ffi::c_int
                        };
                        grid_line_fill(
                            if (*wp).w_view_width - width > 0 as ::core::ffi::c_int {
                                (*wp).w_view_width - width
                            } else {
                                0 as ::core::ffi::c_int
                            },
                            (*wp).w_view_width,
                            (*wp).w_p_fcs_chars.lastline,
                            at_attr,
                        );
                        grid_line_flush();
                        set_empty_rows(wp, srow_0);
                        (*wp).w_botline = lnum_0;
                    } else {
                        win_draw_end(
                            wp,
                            (*wp).w_p_fcs_chars.lastline,
                            true_0 != 0,
                            srow_0,
                            (*wp).w_view_height,
                            HLF_AT,
                        );
                        set_empty_rows(wp, srow_0);
                        (*wp).w_botline = lnum_0;
                    }
                    break 's_2363;
                } else if eof {
                    (*wp).w_botline = (*buf).b_ml.ml_line_count + 1 as linenr_T;
                    let mut j_3: ::core::ffi::c_int = win_get_fill(wp, (*wp).w_botline);
                    if !(j_3 > 0 as ::core::ffi::c_int
                        && !(*wp).w_botfill
                        && row_0 < (*wp).w_view_height)
                    {
                        break 's_2327;
                    }
                    let mut zero_spv_0: spellvars_T = spellvars_T {
                        spv_has_spell: false,
                        spv_unchanged: false,
                        spv_checked_col: 0,
                        spv_checked_lnum: 0,
                        spv_cap_col: 0,
                        spv_capcol_lnum: 0,
                    };
                    let mut zero_foldinfo: foldinfo_T = foldinfo_T {
                        fi_lnum: 0 as linenr_T,
                        fi_level: 0,
                        fi_low_level: 0,
                        fi_lines: 0,
                    };
                    row_0 = win_line(
                        wp,
                        (*wp).w_botline,
                        row_0,
                        (*wp).w_view_height,
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                        &raw mut zero_spv_0,
                        zero_foldinfo,
                    );
                    if !(*wp).w_redr_statuscol {
                        break 's_2327;
                    }
                    eof = false_0 != 0;
                } else {
                    if dollar_vcol.get() == -1 as ::core::ffi::c_int || wp != curwin.get() {
                        (*wp).w_botline = lnum_0;
                    }
                    break 's_2327;
                }
            }
            (*wp).w_redr_statuscol = false_0 != 0;
            idx_2 = 0 as ::core::ffi::c_int;
            row_0 = 0 as ::core::ffi::c_int;
            lnum_0 = (*wp).w_topline;
            (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
            (*wp).w_valid &= !VALID_WCOL;
            decor_redraw_reset(wp, decor_state.ptr());
            decor_providers_invoke_win(wp);
        }
        let mut lastline: ::core::ffi::c_int = bot_scroll_start;
        if mid_end >= row_0 {
            lastline = if lastline < mid_start {
                lastline
            } else {
                mid_start
            };
        }
        if mod_bot > (*buf).b_ml.ml_line_count {
            lastline = 0 as ::core::ffi::c_int;
        }
        win_draw_end(
            wp,
            (*wp).w_p_fcs_chars.eob,
            false_0 != 0,
            if lastline > row_0 { lastline } else { row_0 },
            (*wp).w_view_height,
            HLF_EOB,
        );
        set_empty_rows(wp, row_0);
    }
    if (*wp).w_redr_type >= UPD_REDRAW_TOP as ::core::ffi::c_int {
        draw_vsep_win(wp);
        draw_hsep_win(wp);
    }
    syn_set_timeout(::core::ptr::null_mut::<proftime_T>());
    (*wp).w_redr_type = 0 as ::core::ffi::c_int;
    (*wp).w_old_topfill = (*wp).w_topfill;
    (*wp).w_old_botfill = (*wp).w_botfill;
    let mut n_0: size_t = 0 as size_t;
    while n_0 < (*win_extmark_arr.ptr()).size {
        ui_call_win_extmark(
            (*wp).w_grid_alloc.handle as Integer,
            (*wp).handle as Window,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).ns_id as Integer,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).mark_id as Integer,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).win_row as Integer,
            (*(*win_extmark_arr.ptr()).items.offset(n_0 as isize)).win_col as Integer,
        );
        n_0 = n_0.wrapping_add(1);
    }
    if dollar_vcol.get() == -1 as ::core::ffi::c_int || wp != curwin.get() {
        (*wp).w_valid |= VALID_BOTLINE;
        (*wp).w_viewport_invalid = true_0 != 0;
        if wp == curwin.get() && (*wp).w_botline != old_botline && !recursive.get() {
            recursive.set(true_0 != 0);
            (*curwin.get()).w_valid &= !VALID_TOPLINE;
            update_topline(curwin.get());
            if must_redraw.get() != 0 as ::core::ffi::c_int {
                let mut mod_set: ::core::ffi::c_int =
                    (*curbuf.get()).b_mod_set as ::core::ffi::c_int;
                (*curbuf.get()).b_mod_set = false_0 != 0;
                curs_columns(curwin.get(), true_0);
                win_update(curwin.get());
                must_redraw.set(0 as ::core::ffi::c_int);
                (*curbuf.get()).b_mod_set = mod_set != 0;
            }
            recursive.set(false_0 != 0);
        }
    }
    if nrwidth_before != (*wp).w_nrwidth && !(*buf).terminal.is_null() {
        terminal_check_size((*buf).terminal);
    }
    if !got_int.get() {
        got_int.set(save_got_int != 0);
    }
}
pub unsafe extern "C" fn win_scroll_lines(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
    mut line_count: ::core::ffi::c_int,
) {
    if !redrawing() || line_count == 0 as ::core::ffi::c_int {
        return;
    }
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut row_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut grid: *mut ScreenGrid =
        grid_adjust(&raw mut (*wp).w_grid, &raw mut row_off, &raw mut col);
    let mut checked_width: ::core::ffi::c_int = if (*grid).cols - col < (*wp).w_view_width {
        (*grid).cols - col
    } else {
        (*wp).w_view_width
    };
    let mut checked_height: ::core::ffi::c_int = if (*grid).rows - row_off < (*wp).w_view_height {
        (*grid).rows - row_off
    } else {
        (*wp).w_view_height
    };
    if row + abs(line_count) >= checked_height {
        return;
    }
    if line_count < 0 as ::core::ffi::c_int {
        grid_del_lines(
            grid,
            row + row_off,
            -line_count,
            checked_height + row_off,
            col,
            checked_width,
        );
    } else {
        grid_ins_lines(
            grid,
            row + row_off,
            line_count,
            checked_height + row_off,
            col,
            checked_width,
        );
    };
}
pub unsafe extern "C" fn win_draw_end(
    mut wp: *mut win_T,
    mut c1: schar_T,
    mut draw_margin: bool,
    mut startrow: ::core::ffi::c_int,
    mut endrow: ::core::ffi::c_int,
    mut hl: hlf_T,
) {
    '_c2rust_label: {
        if hl as ::core::ffi::c_uint >= 0 as ::core::ffi::c_uint
            && (hl as ::core::ffi::c_uint) < HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"hl >= 0 && hl < HLF_COUNT\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2513 as ::core::ffi::c_uint,
                b"void win_draw_end(win_T *, schar_T, _Bool, int, int, hlf_T)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let view_width: ::core::ffi::c_int = (*wp).w_view_width;
    let fdc: ::core::ffi::c_int = compute_foldcolumn(wp, 0 as ::core::ffi::c_int);
    let scwidth: ::core::ffi::c_int = (*wp).w_scwidth;
    let mut row: ::core::ffi::c_int = startrow;
    while row < endrow {
        grid_line_start(&raw mut (*wp).w_grid, row);
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if draw_margin {
            if fdc > 0 as ::core::ffi::c_int {
                n = grid_line_fill(
                    n,
                    if view_width < n + fdc {
                        view_width
                    } else {
                        n + fdc
                    },
                    ' ' as ::core::ffi::c_int as schar_T,
                    win_hl_attr(wp, HLF_FC as ::core::ffi::c_int),
                );
            }
            if scwidth > 0 as ::core::ffi::c_int {
                n = grid_line_fill(
                    n,
                    if view_width < n + scwidth * SIGN_WIDTH as ::core::ffi::c_int {
                        view_width
                    } else {
                        n + scwidth * SIGN_WIDTH as ::core::ffi::c_int
                    },
                    ' ' as ::core::ffi::c_int as schar_T,
                    win_hl_attr(wp, HLF_SC as ::core::ffi::c_int),
                );
            }
            if ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                && vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null()
            {
                let mut width: ::core::ffi::c_int = number_width(wp) + 1 as ::core::ffi::c_int;
                n = grid_line_fill(
                    n,
                    if view_width < n + width {
                        view_width
                    } else {
                        n + width
                    },
                    ' ' as ::core::ffi::c_int as schar_T,
                    win_hl_attr(wp, HLF_N as ::core::ffi::c_int),
                );
            }
        }
        let mut attr: ::core::ffi::c_int = win_hl_attr(wp, hl as ::core::ffi::c_int);
        if n < view_width {
            grid_line_put_schar(n, c1, attr);
            n += 1;
        }
        grid_line_clear_end(n, view_width, win_bg_attr(wp), attr);
        if (*wp).w_onebuf_opt.wo_rl != 0 {
            grid_line_mirror(view_width);
        }
        grid_line_flush();
        row += 1;
    }
}
pub unsafe extern "C" fn compute_foldcolumn(
    mut wp: *mut win_T,
    mut col: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut fdc: ::core::ffi::c_int = win_fdccol_count(wp);
    let mut wmw: ::core::ffi::c_int = if wp == curwin.get() && p_wmw.get() == 0 as OptInt {
        1 as ::core::ffi::c_int
    } else {
        p_wmw.get() as ::core::ffi::c_int
    };
    let mut n: ::core::ffi::c_int = (*wp).w_view_width - (col + wmw);
    return if fdc < n { fdc } else { n };
}
pub unsafe extern "C" fn number_width(mut wp: *mut win_T) -> ::core::ffi::c_int {
    let mut lnum: linenr_T = 0;
    if (*wp).w_onebuf_opt.wo_rnu != 0 && (*wp).w_onebuf_opt.wo_nu == 0 {
        lnum = (*wp).w_view_height as linenr_T;
    } else {
        lnum = (*(*wp).w_buffer).b_ml.ml_line_count;
    }
    if lnum == (*wp).w_nrwidth_line_count {
        return (*wp).w_nrwidth_width;
    }
    (*wp).w_nrwidth_line_count = lnum;
    if *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL {
        (*wp).w_statuscol_line_count = 0 as ::core::ffi::c_int as linenr_T;
        (*wp).w_nrwidth_width = ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
            as ::core::ffi::c_int
            * (*wp).w_onebuf_opt.wo_nuw as ::core::ffi::c_int;
        return (*wp).w_nrwidth_width;
    }
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        lnum = (lnum as ::core::ffi::c_int / 10 as ::core::ffi::c_int) as linenr_T;
        n += 1;
        if lnum <= 0 as linenr_T {
            break;
        }
    }
    n = if n > (*wp).w_onebuf_opt.wo_nuw as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        n
    } else {
        (*wp).w_onebuf_opt.wo_nuw as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    };
    if n < 2 as ::core::ffi::c_int
        && buf_meta_total((*wp).w_buffer, kMTMetaSignText) != 0
        && (*wp).w_minscwidth == SCL_NUM
    {
        n = 2 as ::core::ffi::c_int;
    }
    (*wp).w_nrwidth_width = n;
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn redraw_later(mut wp: *mut win_T, mut type_0: ::core::ffi::c_int) {
    '_c2rust_label: {
        if !wp.is_null() || exiting.get() as ::core::ffi::c_int != 0 {
        } else {
            __assert_fail(
                b"wp != NULL || exiting\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2623 as ::core::ffi::c_uint,
                b"void redraw_later(win_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if !exiting.get() && !redraw_not_allowed.get() && (*wp).w_redr_type < type_0 {
        (*wp).w_redr_type = type_0;
        if type_0 >= UPD_NOT_VALID as ::core::ffi::c_int {
            (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
        }
        must_redraw.set(if must_redraw.get() > type_0 {
            must_redraw.get()
        } else {
            type_0
        });
    }
}
pub unsafe extern "C" fn redraw_all_later(mut type_0: ::core::ffi::c_int) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        redraw_later(wp, type_0);
        wp = (*wp).w_next;
    }
    set_must_redraw(type_0);
}
pub unsafe extern "C" fn set_must_redraw(mut type_0: ::core::ffi::c_int) {
    if !redraw_not_allowed.get() {
        must_redraw.set(if must_redraw.get() > type_0 {
            must_redraw.get()
        } else {
            type_0
        });
    }
}
pub unsafe extern "C" fn screen_invalidate_highlights() {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
        (*wp).w_grid_alloc.valid = false_0 != 0;
        wp = (*wp).w_next;
    }
}
pub unsafe extern "C" fn redraw_curbuf_later(mut type_0: ::core::ffi::c_int) {
    redraw_buf_later(curbuf.get(), type_0);
}
#[no_mangle]
pub unsafe extern "C" fn redraw_buf_later(mut buf: *mut buf_T, mut type_0: ::core::ffi::c_int) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf {
            redraw_later(wp, type_0);
        }
        wp = (*wp).w_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn redraw_buf_line_later(
    mut buf: *mut buf_T,
    mut line: linenr_T,
    mut force: bool,
) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf {
            redrawWinline(
                wp,
                if line < (*buf).b_ml.ml_line_count {
                    line
                } else {
                    (*buf).b_ml.ml_line_count
                },
            );
            if force as ::core::ffi::c_int != 0 && line > (*buf).b_ml.ml_line_count {
                (*wp).w_redraw_bot = line;
            }
        }
        wp = (*wp).w_next;
    }
}
pub unsafe extern "C" fn redraw_win_range_later(
    mut wp: *mut win_T,
    mut first: linenr_T,
    mut last: linenr_T,
) {
    if last >= (*wp).w_topline && first < (*wp).w_botline {
        if (*wp).w_redraw_top == 0 as linenr_T || (*wp).w_redraw_top > first {
            (*wp).w_redraw_top = first;
        }
        if (*wp).w_redraw_bot == 0 as linenr_T || (*wp).w_redraw_bot < last {
            (*wp).w_redraw_bot = last;
        }
        redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
    }
}
pub unsafe extern "C" fn redrawWinline(mut wp: *mut win_T, mut lnum: linenr_T) {
    redraw_win_range_later(wp, lnum, lnum);
}
pub unsafe extern "C" fn redraw_buf_range_later(
    mut buf: *mut buf_T,
    mut first: linenr_T,
    mut last: linenr_T,
) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf {
            redraw_win_range_later(wp, first, last);
        }
        wp = (*wp).w_next;
    }
}
pub unsafe extern "C" fn redraw_buf_status_later(mut buf: *mut buf_T) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf
            && ((*wp).w_status_height != 0
                || wp == curwin.get() && global_stl_height() != 0
                || (*wp).w_winbar_height != 0)
        {
            (*wp).w_redr_status = true_0 != 0;
            set_must_redraw(UPD_VALID as ::core::ffi::c_int);
        }
        wp = (*wp).w_next;
    }
}
pub unsafe extern "C" fn status_redraw_all() {
    let mut is_stl_global: bool = global_stl_height() != 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if !is_stl_global && (*wp).w_status_height != 0
            || wp == curwin.get()
            || (*wp).w_winbar_height != 0
        {
            (*wp).w_redr_status = true_0 != 0;
            redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
        }
        wp = (*wp).w_next;
    }
}
pub unsafe extern "C" fn status_redraw_curbuf() {
    status_redraw_buf(curbuf.get());
}
#[no_mangle]
pub unsafe extern "C" fn status_redraw_buf(mut buf: *mut buf_T) {
    let mut is_stl_global: bool = global_stl_height() != 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == buf
            && (!is_stl_global && (*wp).w_status_height != 0
                || is_stl_global as ::core::ffi::c_int != 0 && wp == curwin.get()
                || (*wp).w_winbar_height != 0)
        {
            (*wp).w_redr_status = true_0 != 0;
            redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
        }
        wp = (*wp).w_next;
    }
    if p_ru.get() != 0 && (*curwin.get()).w_status_height == 0 && !(*curwin.get()).w_redr_status {
        redraw_cmdline.set(true_0 != 0);
        redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
    }
}
pub unsafe extern "C" fn redraw_statuslines() {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_redr_status {
            win_check_ns_hl(wp);
            win_redr_winbar(wp);
            win_redr_status(wp);
        }
        wp = (*wp).w_next;
    }
    win_check_ns_hl(::core::ptr::null_mut::<win_T>());
    if redraw_tabline.get() {
        draw_tabline();
    }
    if need_maketitle.get() {
        maketitle();
    }
}
pub unsafe extern "C" fn win_redraw_last_status(mut frp: *const frame_T) {
    if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
        (*(*frp).fr_win).w_redr_status = true_0 != 0;
    } else if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
        frp = (*frp).fr_child;
        while !frp.is_null() {
            win_redraw_last_status(frp);
            frp = (*frp).fr_next;
        }
    } else {
        '_c2rust_label: {
            if (*frp).fr_layout as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"frp->fr_layout == FR_COL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2806 as ::core::ffi::c_uint,
                    b"void win_redraw_last_status(const frame_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        frp = (*frp).fr_child;
        while !(*frp).fr_next.is_null() {
            frp = (*frp).fr_next;
        }
        win_redraw_last_status(frp);
    };
}
pub unsafe extern "C" fn conceal_cursor_line(mut wp: *const win_T) -> bool {
    let mut c: ::core::ffi::c_int = 0;
    if *(*wp).w_onebuf_opt.wo_cocu as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    if get_real_state() & MODE_VISUAL as ::core::ffi::c_int != 0 {
        c = 'v' as ::core::ffi::c_int;
    } else if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
        c = 'i' as ::core::ffi::c_int;
    } else if State.get() & MODE_NORMAL as ::core::ffi::c_int != 0 {
        c = 'n' as ::core::ffi::c_int;
    } else if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
        c = 'c' as ::core::ffi::c_int;
    } else {
        return false_0 != 0;
    }
    return !vim_strchr((*wp).w_onebuf_opt.wo_cocu, c).is_null();
}
pub unsafe extern "C" fn win_cursorline_standout(mut wp: *const win_T) -> bool {
    return (*wp).w_onebuf_opt.wo_cul != 0
        || wp == curwin.get() as *const win_T
            && (*wp).w_onebuf_opt.wo_cole > 0 as OptInt
            && !conceal_cursor_line(wp);
}
pub unsafe extern "C" fn win_update_cursorline(mut wp: *mut win_T, mut foldinfo: *mut foldinfo_T) {
    (*wp).w_cursorline = if win_cursorline_standout(wp) as ::core::ffi::c_int != 0 {
        (*wp).w_cursor.lnum
    } else {
        0 as linenr_T
    };
    if (*wp).w_onebuf_opt.wo_cul != 0 {
        *foldinfo = fold_info(wp, (*wp).w_cursor.lnum);
        if (*foldinfo).fi_level != 0 as ::core::ffi::c_int && (*foldinfo).fi_lines > 0 as linenr_T {
            (*wp).w_cursorline = (*foldinfo).fi_lnum;
        }
    }
}
pub const NO_SCREEN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const STL_IN_ICON: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STL_IN_TITLE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn win_hl_attr(
    mut wp: *mut win_T,
    mut hlf: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return *if !(*wp).w_ns_hl_attr.is_null() && ns_hl_fast.get() < 0 as ::core::ffi::c_int {
        (*wp).w_ns_hl_attr
    } else {
        hl_attr_active.get()
    }
    .offset(hlf as isize);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
