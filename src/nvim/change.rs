use crate::src::nvim::autocmd::apply_autocmds;
use crate::src::nvim::buffer::{bt_dontwrite, bt_prompt, buf_inc_changedtick};
use crate::src::nvim::buffer_updates::buf_updates_send_changes;
use crate::src::nvim::charset::{
    getdigits_int, getwhitecols_curline, ptr2cells, skipwhite, vim_strnsize,
};
use crate::src::nvim::cursor::{
    check_cursor_lnum, check_visual_pos, coladvance_force, get_cursor_line_len,
    get_cursor_line_ptr, get_cursor_pos_ptr, getviscol,
};
use crate::src::nvim::diff::{diff_internal, diff_lnum_win, diff_update_line};
use crate::src::nvim::drawscreen::{
    redrawWinline, redraw_buf_status_later, redraw_later, set_must_redraw, showmode,
};
use crate::src::nvim::edit::{prompt_text, replace_push, replace_push_nul, truncate_spaces};
use crate::src::nvim::eval::vars::set_vim_var_string;
use crate::src::nvim::extmark::{extmark_adjust, extmark_splice, extmark_splice_cols};
use crate::src::nvim::fold::{find_wl_entry, foldUpdate, hasAnyFolding, hasFoldingWin};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::{
    copy_indent, fixthisline, get_indent, get_lisp_indent, get_sw_value, indent_size_ts, may_do_si,
    set_indent, use_indentexpr_for_lisp,
};
use crate::src::nvim::indent_c::{cin_is_cinword, do_c_expr_indent, in_cinkeys};
use crate::src::nvim::insexpand::ins_compl_active;
use crate::src::nvim::main::{
    ai_col, autocmd_busy, can_si, can_si_back, cmdmod, curbuf, curbuf_splice_pending, curtab,
    curwin, did_ai, did_si, emsg_silent, end_comment_pending, first_tabpage, firstwin,
    highlight_match, in_assert_fails, inhibit_delete_count, last_cursormoved, last_cursormoved_win,
    msg_col, msg_row, msg_scroll, msg_silent, need_maketitle, need_wait_return, orig_line_count,
    p_cpo, p_deco, p_paste, p_ri, p_sm, p_sr, redraw_cmdline, redraw_not_allowed, redraw_tabline,
    restart_edit, search_hl_has_cursor_lnum, silent_mode, vr_lines_changed, Insstart, Rows, State,
    VIsual_active,
};
use crate::src::nvim::mark::{free_fmark, mark_adjust, mark_col_adjust, mark_view_make};
use crate::src::nvim::mbyte::{
    mb_adjust_cursor, utf_char2bytes, utf_composinglike, utf_head_off, utf_iscomposing_first,
    utf_ptr2char, utf_ptr2len, utfc_ptr2len, utfc_ptr2len_len,
};
use crate::src::nvim::memline::{
    ml_add_deleted_len, ml_append, ml_delete_flags, ml_get, ml_get_buf, ml_get_len,
    ml_line_alloced, ml_open_file, ml_replace, ml_setflags,
};
use crate::src::nvim::memory::{xfree, xmalloc, xmallocz, xmemcpyz, xstrdup};
use crate::src::nvim::message::{
    msg_clr_eos, msg_delay, msg_end, msg_ext_set_kind, msg_puts_hl, msg_source, msg_start, siemsg,
    wait_return,
};
use crate::src::nvim::option::{copy_option_part, get_ve_flags};
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, memmove, strcat, strcmp, strlen, strncmp,
};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::plines::{getvcol, linetabsize_eol, win_chartabsize};
use crate::src::nvim::r#move::{
    approximate_botline_win, changed_cline_bef_curs, changed_line_abv_curs_win,
    invalidate_botline_win, set_topline, sms_marker_overlap,
};
use crate::src::nvim::search::{check_linecomment, findmatch, linewhite, showmatch};
use crate::src::nvim::spell::spell_check_window;
use crate::src::nvim::state::virtual_active;
use crate::src::nvim::strings::{concat_str, vim_strchr, xstrnsave};
use crate::src::nvim::textformat::{comp_textwidth, has_format_option};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1,
    ExtmarkMove, ExtmarkOp, ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject, FileID, FloatAnchor,
    FloatRelative, GraphemeState, GridView, IndentGetter, Intersection, LuaRef, MTKey, MTNode,
    MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MetaIndex, MotionType, OptInt, ScopeDictDictItem, ScopeType,
    ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal, Timestamp, UIExtension,
    UndoObjectType, VarLockStatus, VarType, VimVarIndex, VirtLines, VirtText, VirtTextChunk,
    VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, auto_event,
    bcount_t, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, cmdmod_T,
    colnr_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T, event_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, foldinfo_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, intptr_t,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, oparg_T, partial_S, partial_T, pos_T, pos_save_T,
    proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, ssize_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_6, utf8proc_int32_t, varnumber_T, virt_line,
    visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
use crate::src::nvim::ui::{ui_active, ui_has};
use crate::src::nvim::undo::{curbufIsChanged, u_clearline, u_save_cursor, u_savedel};
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_15 = 76;
pub const HLF_PRE: C2Rust_Unnamed_15 = 75;
pub const HLF_OK: C2Rust_Unnamed_15 = 74;
pub const HLF_SO: C2Rust_Unnamed_15 = 73;
pub const HLF_SE: C2Rust_Unnamed_15 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_15 = 71;
pub const HLF_TS: C2Rust_Unnamed_15 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_15 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_15 = 68;
pub const HLF_CU: C2Rust_Unnamed_15 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_15 = 66;
pub const HLF_WBR: C2Rust_Unnamed_15 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_15 = 64;
pub const HLF_MSG: C2Rust_Unnamed_15 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_15 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_15 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_15 = 60;
pub const HLF_0: C2Rust_Unnamed_15 = 59;
pub const HLF_QFL: C2Rust_Unnamed_15 = 58;
pub const HLF_MC: C2Rust_Unnamed_15 = 57;
pub const HLF_CUL: C2Rust_Unnamed_15 = 56;
pub const HLF_CUC: C2Rust_Unnamed_15 = 55;
pub const HLF_TPF: C2Rust_Unnamed_15 = 54;
pub const HLF_TPS: C2Rust_Unnamed_15 = 53;
pub const HLF_TP: C2Rust_Unnamed_15 = 52;
pub const HLF_PBR: C2Rust_Unnamed_15 = 51;
pub const HLF_PST: C2Rust_Unnamed_15 = 50;
pub const HLF_PSB: C2Rust_Unnamed_15 = 49;
pub const HLF_PSX: C2Rust_Unnamed_15 = 48;
pub const HLF_PNX: C2Rust_Unnamed_15 = 47;
pub const HLF_PSK: C2Rust_Unnamed_15 = 46;
pub const HLF_PNK: C2Rust_Unnamed_15 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_15 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_15 = 43;
pub const HLF_PSI: C2Rust_Unnamed_15 = 42;
pub const HLF_PNI: C2Rust_Unnamed_15 = 41;
pub const HLF_SPL: C2Rust_Unnamed_15 = 40;
pub const HLF_SPR: C2Rust_Unnamed_15 = 39;
pub const HLF_SPC: C2Rust_Unnamed_15 = 38;
pub const HLF_SPB: C2Rust_Unnamed_15 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_15 = 36;
pub const HLF_SC: C2Rust_Unnamed_15 = 35;
pub const HLF_TXA: C2Rust_Unnamed_15 = 34;
pub const HLF_TXD: C2Rust_Unnamed_15 = 33;
pub const HLF_DED: C2Rust_Unnamed_15 = 32;
pub const HLF_CHD: C2Rust_Unnamed_15 = 31;
pub const HLF_ADD: C2Rust_Unnamed_15 = 30;
pub const HLF_FC: C2Rust_Unnamed_15 = 29;
pub const HLF_FL: C2Rust_Unnamed_15 = 28;
pub const HLF_WM: C2Rust_Unnamed_15 = 27;
pub const HLF_W: C2Rust_Unnamed_15 = 26;
pub const HLF_VNC: C2Rust_Unnamed_15 = 25;
pub const HLF_V: C2Rust_Unnamed_15 = 24;
pub const HLF_T: C2Rust_Unnamed_15 = 23;
pub const HLF_VSP: C2Rust_Unnamed_15 = 22;
pub const HLF_C: C2Rust_Unnamed_15 = 21;
pub const HLF_SNC: C2Rust_Unnamed_15 = 20;
pub const HLF_S: C2Rust_Unnamed_15 = 19;
pub const HLF_R: C2Rust_Unnamed_15 = 18;
pub const HLF_CLF: C2Rust_Unnamed_15 = 17;
pub const HLF_CLS: C2Rust_Unnamed_15 = 16;
pub const HLF_CLN: C2Rust_Unnamed_15 = 15;
pub const HLF_LNB: C2Rust_Unnamed_15 = 14;
pub const HLF_LNA: C2Rust_Unnamed_15 = 13;
pub const HLF_N: C2Rust_Unnamed_15 = 12;
pub const HLF_CM: C2Rust_Unnamed_15 = 11;
pub const HLF_M: C2Rust_Unnamed_15 = 10;
pub const HLF_LC: C2Rust_Unnamed_15 = 9;
pub const HLF_L: C2Rust_Unnamed_15 = 8;
pub const HLF_I: C2Rust_Unnamed_15 = 7;
pub const HLF_E: C2Rust_Unnamed_15 = 6;
pub const HLF_D: C2Rust_Unnamed_15 = 5;
pub const HLF_AT: C2Rust_Unnamed_15 = 4;
pub const HLF_TERM: C2Rust_Unnamed_15 = 3;
pub const HLF_EOB: C2Rust_Unnamed_15 = 2;
pub const HLF_8: C2Rust_Unnamed_15 = 1;
pub const HLF_NONE: C2Rust_Unnamed_15 = 0;
pub const kMTMetaCount: MetaIndex = 5;
pub const kMTMetaConcealLines: MetaIndex = 4;
pub const kMTMetaSignText: MetaIndex = 3;
pub const kMTMetaSignHL: MetaIndex = 2;
pub const kMTMetaLines: MetaIndex = 1;
pub const kMTMetaInline: MetaIndex = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_16 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_16 = 3;
pub const BACKWARD: C2Rust_Unnamed_16 = -1;
pub const FORWARD: C2Rust_Unnamed_16 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_16 = 0;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_17 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_17 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_17 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_17 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_17 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_17 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_17 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_17 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_17 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_17 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_17 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_17 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_17 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_17 = 1;
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const OPENLINE_FORCE_INDENT: C2Rust_Unnamed_18 = 64;
pub const OPENLINE_FORMAT: C2Rust_Unnamed_18 = 32;
pub const OPENLINE_COM_LIST: C2Rust_Unnamed_18 = 16;
pub const OPENLINE_MARKFIX: C2Rust_Unnamed_18 = 8;
pub const OPENLINE_KEEPTRAIL: C2Rust_Unnamed_18 = 4;
pub const OPENLINE_DO_COM: C2Rust_Unnamed_18 = 2;
pub const OPENLINE_DELSPACES: C2Rust_Unnamed_18 = 1;
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
pub const UPD_VALID: C2Rust_Unnamed_20 = 10;
pub const UPD_NOT_VALID: C2Rust_Unnamed_20 = 40;
pub const REPLACE_FLAG: C2Rust_Unnamed_22 = 256;
pub const MODE_INSERT: C2Rust_Unnamed_22 = 16;
pub const VREPLACE_FLAG: C2Rust_Unnamed_22 = 512;
pub const kOptVeFlagOnemore: C2Rust_Unnamed_19 = 8;
pub const KEY_OPEN_BACK: C2Rust_Unnamed_21 = 258;
pub const KEY_OPEN_FORW: C2Rust_Unnamed_21 = 257;
pub const SIN_NOMARK: C2Rust_Unnamed_23 = 8;
pub const SIN_INSERT: C2Rust_Unnamed_23 = 2;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const ML_DEL_MESSAGE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kOptVeFlagNoneU: C2Rust_Unnamed_19 = 32;
pub const kOptVeFlagNone: C2Rust_Unnamed_19 = 16;
pub const kOptVeFlagInsert: C2Rust_Unnamed_19 = 6;
pub const kOptVeFlagBlock: C2Rust_Unnamed_19 = 5;
pub const kOptVeFlagAll: C2Rust_Unnamed_19 = 4;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_20 = 50;
pub const UPD_SOME_VALID: C2Rust_Unnamed_20 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_20 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_20 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_20 = 20;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const KEY_COMPLETE: C2Rust_Unnamed_21 = 259;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_22 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_22 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_22 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_22 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_22 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_22 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_22 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_22 = 784;
pub const MODE_REPLACE: C2Rust_Unnamed_22 = 272;
pub const MAP_ALL_MODES: C2Rust_Unnamed_22 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_22 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_22 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_22 = 32;
pub const MODE_CMDLINE: C2Rust_Unnamed_22 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_22 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_22 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const SIN_UNDO: C2Rust_Unnamed_23 = 4;
pub const SIN_CHANGED: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const BF_NEVERLOADED: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const JUMPLISTSIZE: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
#[no_mangle]
pub unsafe extern "C" fn change_warning(mut buf: *mut buf_T, mut col: ::core::ffi::c_int) {
    static w_readonly: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
        b"W10: Warning: Changing a readonly file\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if (*buf).b_did_warn as ::core::ffi::c_int == false_0
        && curbufIsChanged() as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && !autocmd_busy.get()
        && (*buf).b_p_ro != 0
    {
        (*buf).b_ro_locked += 1;
        apply_autocmds(
            EVENT_FILECHANGEDRO,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            buf,
        );
        (*buf).b_ro_locked -= 1;
        if (*buf).b_p_ro == 0 {
            return;
        }
        msg_start();
        if msg_row.get() == Rows.get() - 1 as ::core::ffi::c_int {
            msg_col.set(col);
        }
        msg_source(HLF_W as ::core::ffi::c_int);
        msg_ext_set_kind(b"wmsg\0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts_hl(
            gettext(w_readonly.get()),
            HLF_W as ::core::ffi::c_int,
            true_0 != 0,
        );
        set_vim_var_string(VV_WARNINGMSG, gettext(w_readonly.get()), -1 as ptrdiff_t);
        msg_clr_eos();
        msg_end();
        if msg_silent.get() == 0 as ::core::ffi::c_int && !silent_mode.get() && ui_active() != 0 {
            msg_delay(1002 as uint64_t, true_0 != 0);
        }
        (*buf).b_did_warn = true_0 != 0;
        redraw_cmdline.set(false_0 != 0);
        if msg_row.get() < Rows.get() - 1 as ::core::ffi::c_int {
            showmode();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn changed(mut buf: *mut buf_T) {
    if (*buf).b_changed == 0 {
        let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
        change_warning(buf, 0 as ::core::ffi::c_int);
        if (*buf).b_may_swap as ::core::ffi::c_int != 0 && !bt_dontwrite(buf) {
            let mut save_need_wait_return: bool = need_wait_return.get();
            need_wait_return.set(false_0 != 0);
            ml_open_file(buf);
            if need_wait_return.get() as ::core::ffi::c_int != 0
                && emsg_silent.get() == 0 as ::core::ffi::c_int
                && !in_assert_fails.get()
                && !ui_has(kUIMessages)
            {
                msg_delay(2002 as uint64_t, true_0 != 0);
                wait_return(true_0);
                msg_scroll.set(save_msg_scroll);
            } else {
                need_wait_return.set(save_need_wait_return);
            }
        }
        changed_internal(buf);
    }
    buf_inc_changedtick(buf);
    highlight_match.set(false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn changed_internal(mut buf: *mut buf_T) {
    (*buf).b_changed = true_0;
    (*buf).b_changed_invalid = true_0 != 0;
    ml_setflags(buf);
    redraw_buf_status_later(buf);
    redraw_tabline.set(true_0 != 0);
    need_maketitle.set(true_0 != 0);
}
unsafe extern "C" fn changed_lines_invalidate_win(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut lnume: linenr_T,
    mut xtra: linenr_T,
) {
    if (*wp).w_cursor.lnum <= lnum {
        let mut i: ::core::ffi::c_int = find_wl_entry(wp, lnum);
        if i >= 0 as ::core::ffi::c_int
            && (*wp).w_cursor.lnum > (*(*wp).w_lines.offset(i as isize)).wl_lnum
        {
            changed_line_abv_curs_win(wp);
        }
    }
    if (*wp).w_cursor.lnum > lnum {
        changed_line_abv_curs_win(wp);
    } else if (*wp).w_cursor.lnum == lnum && (*wp).w_cursor.col >= col {
        changed_cline_bef_curs(wp);
    }
    if (*wp).w_botline >= lnum {
        if xtra < 0 as linenr_T {
            invalidate_botline_win(wp);
        } else {
            approximate_botline_win(wp);
        }
    }
    if xtra < 0 as linenr_T
        && (*wp).w_onebuf_opt.wo_wrap != 0
        && buf_meta_total((*wp).w_buffer, kMTMetaInline) != 0
        || xtra != 0 as linenr_T && buf_meta_total((*wp).w_buffer, kMTMetaLines) != 0
    {
        lnume += 1;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*wp).w_lines_valid {
        if (*(*wp).w_lines.offset(i_0 as isize)).wl_valid {
            if (*(*wp).w_lines.offset(i_0 as isize)).wl_lnum >= lnum {
                if i_0 == 0 as ::core::ffi::c_int
                    || (*(*wp).w_lines.offset(i_0 as isize)).wl_lnum < lnume
                {
                    (*(*wp).w_lines.offset(i_0 as isize)).wl_valid = false_0 != 0;
                } else if xtra != 0 as linenr_T {
                    (*(*wp).w_lines.offset(i_0 as isize)).wl_lnum += xtra;
                    (*(*wp).w_lines.offset(i_0 as isize)).wl_foldend += xtra;
                    (*(*wp).w_lines.offset(i_0 as isize)).wl_lastlnum += xtra;
                }
            } else if (*(*wp).w_lines.offset(i_0 as isize)).wl_lastlnum >= lnum {
                (*(*wp).w_lines.offset(i_0 as isize)).wl_valid = false_0 != 0;
            }
        }
        i_0 += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn changed_lines_invalidate_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut lnume: linenr_T,
    mut xtra: linenr_T,
) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                changed_lines_invalidate_win(wp, lnum, col, lnume, xtra);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
unsafe extern "C" fn changed_common(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut lnume: linenr_T,
    mut xtra: linenr_T,
) {
    changed(buf);
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        if (*win).w_buffer == buf && (*win).w_onebuf_opt.wo_diff != 0 && diff_internal() != 0 {
            (*curtab.get()).tp_diff_update = true_0;
            diff_update_line(lnum);
        }
        win = (*win).w_next;
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        let mut view: fmarkv_T = fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        };
        if (*curwin.get()).w_buffer == buf {
            if lnum >= (*curwin.get()).w_topline && lnum <= (*curwin.get()).w_botline {
                view = mark_view_make(curwin.get(), (*curwin.get()).w_cursor);
            }
        }
        let fmarkp___: *mut fmark_T = &raw mut (*buf).b_last_change;
        free_fmark(*fmarkp___);
        let fmarkp__: *mut fmark_T = fmarkp___;
        (*fmarkp__).mark = pos_T {
            lnum: lnum,
            col: col,
            coladd: 0 as colnr_T,
        };
        (*fmarkp__).fnum = (*buf).handle as ::core::ffi::c_int;
        (*fmarkp__).timestamp = os_time();
        (*fmarkp__).view = view;
        (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        if (*buf).b_new_change as ::core::ffi::c_int != 0
            || (*buf).b_changelistlen == 0 as ::core::ffi::c_int
        {
            let mut add: bool = false;
            if (*buf).b_changelistlen == 0 as ::core::ffi::c_int {
                add = true_0 != 0;
            } else {
                let mut p: *mut pos_T = &raw mut (*(&raw mut (*buf).b_changelist as *mut fmark_T)
                    .offset(((*buf).b_changelistlen - 1 as ::core::ffi::c_int) as isize))
                .mark;
                if (*p).lnum != lnum {
                    add = true_0 != 0;
                } else {
                    let mut cols: ::core::ffi::c_int = comp_textwidth(false_0 != 0);
                    if cols == 0 as ::core::ffi::c_int {
                        cols = 79 as ::core::ffi::c_int;
                    }
                    add = (*p).col as ::core::ffi::c_int + cols < col
                        || col as ::core::ffi::c_int + cols < (*p).col;
                }
            }
            if add {
                (*buf).b_new_change = false_0 != 0;
                if (*buf).b_changelistlen == JUMPLISTSIZE {
                    (*buf).b_changelistlen = JUMPLISTSIZE - 1 as ::core::ffi::c_int;
                    memmove(
                        &raw mut (*buf).b_changelist as *mut fmark_T as *mut ::core::ffi::c_void,
                        (&raw mut (*buf).b_changelist as *mut fmark_T)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<fmark_T>()
                            .wrapping_mul((JUMPLISTSIZE - 1 as ::core::ffi::c_int) as size_t),
                    );
                    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                    while !tp.is_null() {
                        let mut wp: *mut win_T = if tp == curtab.get() {
                            firstwin.get()
                        } else {
                            (*tp).tp_firstwin
                        };
                        while !wp.is_null() {
                            if (*wp).w_buffer == buf
                                && (*wp).w_changelistidx > 0 as ::core::ffi::c_int
                            {
                                (*wp).w_changelistidx -= 1;
                            }
                            wp = (*wp).w_next;
                        }
                        tp = (*tp).tp_next as *mut tabpage_T;
                    }
                }
                let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                while !tp_0.is_null() {
                    let mut wp_0: *mut win_T = if tp_0 == curtab.get() {
                        firstwin.get()
                    } else {
                        (*tp_0).tp_firstwin
                    };
                    while !wp_0.is_null() {
                        if (*wp_0).w_buffer == buf
                            && (*wp_0).w_changelistidx == (*buf).b_changelistlen
                        {
                            (*wp_0).w_changelistidx += 1;
                        }
                        wp_0 = (*wp_0).w_next;
                    }
                    tp_0 = (*tp_0).tp_next as *mut tabpage_T;
                }
                (*buf).b_changelistlen += 1;
            }
        }
        (*buf).b_changelist[((*buf).b_changelistlen - 1 as ::core::ffi::c_int) as usize] =
            (*buf).b_last_change;
        if (*curwin.get()).w_buffer == buf {
            (*curwin.get()).w_changelistidx = (*buf).b_changelistlen;
        }
    }
    if (*curwin.get()).w_buffer == buf && VIsual_active.get() as ::core::ffi::c_int != 0 {
        check_visual_pos();
    }
    let mut tp_1: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp_1.is_null() {
        let mut wp_1: *mut win_T = if tp_1 == curtab.get() {
            firstwin.get()
        } else {
            (*tp_1).tp_firstwin
        };
        while !wp_1.is_null() {
            if (*wp_1).w_buffer == buf {
                if !redraw_not_allowed.get()
                    && (*wp_1).w_redr_type < UPD_VALID as ::core::ffi::c_int
                {
                    (*wp_1).w_redr_type = UPD_VALID as ::core::ffi::c_int;
                }
                if xtra != 0 as linenr_T && (*wp_1).w_redraw_top != 0 as linenr_T {
                    redraw_later(wp_1, UPD_NOT_VALID as ::core::ffi::c_int);
                }
                let mut last: linenr_T = lnume + xtra - 1 as linenr_T;
                if (*wp_1).w_skipcol > 0 as ::core::ffi::c_int
                    && (last < (*wp_1).w_topline
                        || (*wp_1).w_topline >= lnum
                            && (*wp_1).w_topline < lnume
                            && linetabsize_eol(wp_1, (*wp_1).w_topline)
                                <= (*wp_1).w_skipcol as ::core::ffi::c_int
                                    + sms_marker_overlap(wp_1, -1 as ::core::ffi::c_int))
                {
                    (*wp_1).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
                }
                foldUpdate(wp_1, lnum, last);
                let mut folded: bool = hasFoldingWin(
                    wp_1,
                    lnum,
                    &raw mut lnum,
                    ::core::ptr::null_mut::<linenr_T>(),
                    false_0 != 0,
                    ::core::ptr::null_mut::<foldinfo_T>(),
                );
                if (*wp_1).w_cursor.lnum == lnum {
                    (*wp_1).w_cline_folded = folded;
                }
                folded = hasFoldingWin(
                    wp_1,
                    last,
                    ::core::ptr::null_mut::<linenr_T>(),
                    &raw mut last,
                    false_0 != 0,
                    ::core::ptr::null_mut::<foldinfo_T>(),
                );
                if (*wp_1).w_cursor.lnum == last {
                    (*wp_1).w_cline_folded = folded;
                }
                changed_lines_invalidate_win(wp_1, lnum, col, lnume, xtra);
                if hasAnyFolding(wp_1) != 0 {
                    set_topline(wp_1, (*wp_1).w_topline);
                }
                if (*wp_1).w_onebuf_opt.wo_rnu != 0 && xtra != 0 as linenr_T {
                    (*wp_1).w_last_cursor_lnum_rnu = 0 as ::core::ffi::c_int as linenr_T;
                }
                if (*wp_1).w_onebuf_opt.wo_cul != 0 && (*wp_1).w_last_cursorline >= lnum {
                    if (*wp_1).w_last_cursorline < lnume {
                        (*wp_1).w_last_cursorline = 0 as ::core::ffi::c_int as linenr_T;
                    } else {
                        (*wp_1).w_last_cursorline += xtra;
                    }
                }
            }
            if wp_1 == curwin.get()
                && xtra != 0 as linenr_T
                && search_hl_has_cursor_lnum.get() >= lnum
            {
                (*search_hl_has_cursor_lnum.ptr()) += xtra;
            }
            wp_1 = (*wp_1).w_next;
        }
        tp_1 = (*tp_1).tp_next as *mut tabpage_T;
    }
    set_must_redraw(UPD_VALID as ::core::ffi::c_int);
    if last_cursormoved_win.get() == curwin.get()
        && (*curwin.get()).w_buffer == buf
        && lnum <= (*curwin.get()).w_cursor.lnum
        && lnume + (if xtra < 0 as linenr_T { -xtra } else { xtra }) > (*curwin.get()).w_cursor.lnum
    {
        (*last_cursormoved.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn changed_bytes(mut lnum: linenr_T, mut col: colnr_T) {
    changed_lines_redraw_buf(curbuf.get(), lnum, lnum + 1 as linenr_T, 0 as linenr_T);
    changed_common(curbuf.get(), lnum, col, lnum + 1 as linenr_T, 0 as linenr_T);
    if spell_check_window(curwin.get()) as ::core::ffi::c_int != 0
        && lnum < (*curbuf.get()).b_ml.ml_line_count
        && vim_strchr(p_cpo.get(), CPO_DOLLAR).is_null()
    {
        redrawWinline(curwin.get(), lnum + 1 as linenr_T);
    }
    buf_updates_send_changes(curbuf.get(), lnum, 1 as int64_t, 1 as int64_t);
    if (*curwin.get()).w_onebuf_opt.wo_diff != 0 {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_diff != 0 && wp != curwin.get() {
                redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
                let mut wlnum: linenr_T = diff_lnum_win(lnum, wp);
                if wlnum > 0 as linenr_T {
                    changed_lines_redraw_buf(
                        (*wp).w_buffer,
                        wlnum,
                        wlnum + 1 as linenr_T,
                        0 as linenr_T,
                    );
                }
            }
            wp = (*wp).w_next;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn inserted_bytes(
    mut lnum: linenr_T,
    mut start_col: colnr_T,
    mut old_col: ::core::ffi::c_int,
    mut new_col: ::core::ffi::c_int,
) {
    if curbuf_splice_pending.get() == 0 as ::core::ffi::c_int {
        extmark_splice_cols(
            curbuf.get(),
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            start_col,
            old_col as colnr_T,
            new_col as colnr_T,
            kExtmarkUndo,
        );
    }
    changed_bytes(lnum, start_col);
}
#[no_mangle]
pub unsafe extern "C" fn appended_lines_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut count: linenr_T,
) {
    changed_lines(
        buf,
        lnum + 1 as linenr_T,
        0 as colnr_T,
        lnum + 1 as linenr_T,
        count,
        true_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn appended_lines(mut lnum: linenr_T, mut count: linenr_T) {
    appended_lines_buf(curbuf.get(), lnum, count);
}
#[no_mangle]
pub unsafe extern "C" fn appended_lines_mark(mut lnum: linenr_T, mut count: ::core::ffi::c_int) {
    mark_adjust(
        lnum + 1 as linenr_T,
        MAXLNUM as ::core::ffi::c_int as linenr_T,
        count as linenr_T,
        0 as linenr_T,
        kExtmarkUndo,
    );
    changed_lines(
        curbuf.get(),
        lnum + 1 as linenr_T,
        0 as colnr_T,
        lnum + 1 as linenr_T,
        count as linenr_T,
        true_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn deleted_lines_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut count: linenr_T,
) {
    changed_lines(buf, lnum, 0 as colnr_T, lnum + count, -count, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn deleted_lines(mut lnum: linenr_T, mut count: linenr_T) {
    deleted_lines_buf(curbuf.get(), lnum, count);
}
#[no_mangle]
pub unsafe extern "C" fn deleted_lines_mark(mut lnum: linenr_T, mut count: ::core::ffi::c_int) {
    let mut made_empty: bool =
        count > 0 as ::core::ffi::c_int && (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0;
    mark_adjust(
        lnum,
        lnum + count as linenr_T - 1 as linenr_T,
        MAXLNUM as ::core::ffi::c_int as linenr_T,
        -(count as linenr_T),
        kExtmarkNOOP,
    );
    extmark_adjust(
        curbuf.get(),
        lnum,
        lnum + count as linenr_T - 1 as linenr_T,
        MAXLNUM as ::core::ffi::c_int as linenr_T,
        -(count as linenr_T)
            + (if made_empty as ::core::ffi::c_int != 0 {
                1 as linenr_T
            } else {
                0 as linenr_T
            }),
        kExtmarkUndo,
    );
    changed_lines(
        curbuf.get(),
        lnum,
        0 as colnr_T,
        lnum + count as linenr_T,
        -count as linenr_T,
        true_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn changed_lines_redraw_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut lnume: linenr_T,
    mut xtra: linenr_T,
) {
    if xtra != 0 as linenr_T
        && (*(&raw mut (*buf).b_marktree as *mut MarkTree)).n_keys > 0 as size_t
    {
        lnume = (lnume as ::core::ffi::c_int
            + (1 as ::core::ffi::c_int
                + (xtra < 0 as linenr_T && buf_meta_total(buf, kMTMetaLines) != 0)
                    as ::core::ffi::c_int)) as linenr_T;
    }
    if (*buf).b_mod_set {
        (*buf).b_mod_top = if (*buf).b_mod_top < lnum {
            (*buf).b_mod_top
        } else {
            lnum
        };
        if lnum < (*buf).b_mod_bot {
            (*buf).b_mod_bot += xtra;
            (*buf).b_mod_bot = if (*buf).b_mod_bot > lnum {
                (*buf).b_mod_bot
            } else {
                lnum
            };
        }
        (*buf).b_mod_bot = if (*buf).b_mod_bot > lnume + xtra {
            (*buf).b_mod_bot
        } else {
            lnume + xtra
        };
        (*buf).b_mod_xlines += xtra;
    } else {
        (*buf).b_mod_set = true_0 != 0;
        (*buf).b_mod_top = lnum;
        (*buf).b_mod_bot = lnume + xtra;
        (*buf).b_mod_xlines = xtra;
    };
}
#[no_mangle]
pub unsafe extern "C" fn changed_lines(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut lnume: linenr_T,
    mut xtra: linenr_T,
    mut do_buf_event: bool,
) {
    changed_lines_redraw_buf(buf, lnum, lnume, xtra);
    if xtra == 0 as linenr_T
        && (*curwin.get()).w_onebuf_opt.wo_diff != 0
        && (*curwin.get()).w_buffer == buf
        && diff_internal() == 0
    {
        let mut wlnum: linenr_T = 0;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_diff != 0 && wp != curwin.get() {
                redraw_later(wp, UPD_VALID as ::core::ffi::c_int);
                wlnum = diff_lnum_win(lnum, wp);
                if wlnum > 0 as linenr_T {
                    changed_lines_redraw_buf(
                        (*wp).w_buffer,
                        wlnum,
                        lnume - lnum + wlnum,
                        0 as linenr_T,
                    );
                }
            }
            wp = (*wp).w_next;
        }
    }
    changed_common(buf, lnum, col, lnume, xtra);
    if do_buf_event {
        let mut num_added: int64_t = (lnume + xtra - lnum) as int64_t;
        let mut num_removed: int64_t = (lnume - lnum) as int64_t;
        buf_updates_send_changes(buf, lnum, num_added, num_removed);
    }
}
#[no_mangle]
pub unsafe extern "C" fn unchanged(
    mut buf: *mut buf_T,
    mut ff: bool,
    mut always_inc_changedtick: bool,
) {
    if (*buf).b_changed != 0
        || ff as ::core::ffi::c_int != 0
            && file_ff_differs(buf, false_0 != 0) as ::core::ffi::c_int != 0
    {
        (*buf).b_changed = false_0;
        (*buf).b_changed_invalid = true_0 != 0;
        ml_setflags(buf);
        if ff {
            save_file_ff(buf);
        }
        redraw_buf_status_later(buf);
        redraw_tabline.set(true_0 != 0);
        need_maketitle.set(true_0 != 0);
        buf_inc_changedtick(buf);
    } else if always_inc_changedtick {
        buf_inc_changedtick(buf);
    }
}
#[no_mangle]
pub unsafe extern "C" fn save_file_ff(mut buf: *mut buf_T) {
    (*buf).b_start_ffc = *(*buf).b_p_ff as ::core::ffi::c_uchar as ::core::ffi::c_int;
    (*buf).b_start_eof = (*buf).b_p_eof;
    (*buf).b_start_eol = (*buf).b_p_eol;
    (*buf).b_start_bomb = (*buf).b_p_bomb;
    if (*buf).b_start_fenc.is_null()
        || strcmp((*buf).b_start_fenc, (*buf).b_p_fenc) != 0 as ::core::ffi::c_int
    {
        xfree((*buf).b_start_fenc as *mut ::core::ffi::c_void);
        (*buf).b_start_fenc = xstrdup((*buf).b_p_fenc);
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_ff_differs(mut buf: *mut buf_T, mut ignore_empty: bool) -> bool {
    if (*buf).b_flags & BF_NEVERLOADED != 0 {
        return false_0 != 0;
    }
    if ignore_empty as ::core::ffi::c_int != 0
        && (*buf).b_flags & BF_NEW != 0
        && (*buf).b_ml.ml_line_count == 1 as linenr_T
        && *ml_get_buf(buf, 1 as linenr_T) as ::core::ffi::c_int == NUL
    {
        return false_0 != 0;
    }
    if (*buf).b_start_ffc != *(*buf).b_p_ff as ::core::ffi::c_int {
        return true_0 != 0;
    }
    if ((*buf).b_p_bin != 0 || (*buf).b_p_fixeol == 0)
        && ((*buf).b_start_eof != (*buf).b_p_eof || (*buf).b_start_eol != (*buf).b_p_eol)
    {
        return true_0 != 0;
    }
    if (*buf).b_p_bin == 0 && (*buf).b_start_bomb != (*buf).b_p_bomb {
        return true_0 != 0;
    }
    if (*buf).b_start_fenc.is_null() {
        return *(*buf).b_p_fenc as ::core::ffi::c_int != NUL;
    }
    return strcmp((*buf).b_start_fenc, (*buf).b_p_fenc) != 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ins_bytes(mut p: *mut ::core::ffi::c_char) {
    ins_bytes_len(p, strlen(p));
}
#[no_mangle]
pub unsafe extern "C" fn ins_bytes_len(mut p: *mut ::core::ffi::c_char, mut len: size_t) {
    let mut n: size_t = 0;
    let mut i: size_t = 0 as size_t;
    while i < len {
        n = utfc_ptr2len_len(
            p.offset(i as isize),
            len.wrapping_sub(i) as ::core::ffi::c_int,
        ) as size_t;
        ins_char_bytes(p.offset(i as isize), n);
        i = i.wrapping_add(n);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ins_char(mut c: ::core::ffi::c_int) {
    let mut buf: [::core::ffi::c_char; 7] = [0; 7];
    let mut n: size_t = utf_char2bytes(c, &raw mut buf as *mut ::core::ffi::c_char) as size_t;
    if buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        buf[0 as ::core::ffi::c_int as usize] = '\n' as ::core::ffi::c_char;
    }
    ins_char_bytes(&raw mut buf as *mut ::core::ffi::c_char, n);
}
#[no_mangle]
pub unsafe extern "C" fn ins_char_bytes(mut buf: *mut ::core::ffi::c_char, mut charlen: size_t) {
    if virtual_active(curwin.get()) as ::core::ffi::c_int != 0
        && (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
    {
        coladvance_force(getviscol());
    }
    let mut col: size_t = (*curwin.get()).w_cursor.col as size_t;
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut oldp: *mut ::core::ffi::c_char = ml_get(lnum);
    let mut linelen: size_t = (ml_get_len(lnum) as size_t).wrapping_add(1 as size_t);
    let mut oldlen: size_t = 0 as size_t;
    let mut newlen: size_t = charlen;
    if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
        if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            let mut old_list: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_list;
            if old_list != 0 && vim_strchr(p_cpo.get(), CPO_LISTWM).is_null() {
                (*curwin.get()).w_onebuf_opt.wo_list = false_0;
            }
            let mut vcol: colnr_T = 0;
            getvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut vcol,
                ::core::ptr::null_mut::<colnr_T>(),
            );
            let mut new_vcol: colnr_T = vcol + win_chartabsize(curwin.get(), buf, vcol);
            while *oldp.offset(col.wrapping_add(oldlen) as isize) as ::core::ffi::c_int != NUL
                && vcol < new_vcol
            {
                vcol += win_chartabsize(
                    curwin.get(),
                    oldp.offset(col as isize).offset(oldlen as isize),
                    vcol,
                );
                if vcol > new_vcol
                    && *oldp.offset(col.wrapping_add(oldlen) as isize) as ::core::ffi::c_int == TAB
                {
                    break;
                }
                oldlen = oldlen.wrapping_add(utfc_ptr2len(
                    oldp.offset(col as isize).offset(oldlen as isize),
                ) as size_t);
                if vcol > new_vcol {
                    newlen = newlen.wrapping_add((vcol - new_vcol) as size_t);
                }
            }
            (*curwin.get()).w_onebuf_opt.wo_list = old_list;
        } else if *oldp.offset(col as isize) as ::core::ffi::c_int != NUL {
            oldlen = utfc_ptr2len(oldp.offset(col as isize)) as size_t;
        }
        replace_push_nul();
        replace_push(oldp.offset(col as isize), oldlen);
    }
    let mut newp: *mut ::core::ffi::c_char =
        xmalloc(linelen.wrapping_add(newlen).wrapping_sub(oldlen)) as *mut ::core::ffi::c_char;
    if col > 0 as size_t {
        memmove(
            newp as *mut ::core::ffi::c_void,
            oldp as *const ::core::ffi::c_void,
            col,
        );
    }
    let mut p: *mut ::core::ffi::c_char = newp.offset(col as isize);
    if linelen > col.wrapping_add(oldlen) {
        memmove(
            p.offset(newlen as isize) as *mut ::core::ffi::c_void,
            oldp.offset(col as isize).offset(oldlen as isize) as *const ::core::ffi::c_void,
            linelen.wrapping_sub(col).wrapping_sub(oldlen),
        );
    }
    memmove(
        p as *mut ::core::ffi::c_void,
        buf as *const ::core::ffi::c_void,
        charlen,
    );
    let mut i: size_t = charlen;
    while i < newlen {
        *p.offset(i as isize) = ' ' as ::core::ffi::c_char;
        i = i.wrapping_add(1);
    }
    ml_replace(lnum, newp, false_0 != 0);
    inserted_bytes(
        lnum,
        col as colnr_T,
        oldlen as ::core::ffi::c_int,
        newlen as ::core::ffi::c_int,
    );
    if p_sm.get() != 0
        && State.get() & MODE_INSERT as ::core::ffi::c_int != 0
        && msg_silent.get() == 0 as ::core::ffi::c_int
        && !ins_compl_active()
    {
        showmatch(utf_ptr2char(buf));
    }
    if p_ri.get() == 0 || State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0 {
        (*curwin.get()).w_cursor.col += charlen as colnr_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ins_str(mut s: *mut ::core::ffi::c_char, mut slen: size_t) {
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    if virtual_active(curwin.get()) as ::core::ffi::c_int != 0
        && (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
    {
        coladvance_force(getviscol());
    }
    let mut col: colnr_T = (*curwin.get()).w_cursor.col;
    let mut oldp: *mut ::core::ffi::c_char = ml_get(lnum);
    let mut oldlen: ::core::ffi::c_int = ml_get_len(lnum);
    let mut newp: *mut ::core::ffi::c_char = xmalloc(
        (oldlen as size_t)
            .wrapping_add(slen)
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    if col > 0 as ::core::ffi::c_int {
        memmove(
            newp as *mut ::core::ffi::c_void,
            oldp as *const ::core::ffi::c_void,
            col as size_t,
        );
    }
    memmove(
        newp.offset(col as isize) as *mut ::core::ffi::c_void,
        s as *const ::core::ffi::c_void,
        slen,
    );
    let mut bytes: ::core::ffi::c_int =
        oldlen - col as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    '_c2rust_label: {
        if bytes >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"bytes >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/change.rs\0".as_ptr() as *const ::core::ffi::c_char,
                836 as ::core::ffi::c_uint,
                b"void ins_str(char *, size_t)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    memmove(
        newp.offset(col as isize).offset(slen as isize) as *mut ::core::ffi::c_void,
        oldp.offset(col as isize) as *const ::core::ffi::c_void,
        bytes as size_t,
    );
    ml_replace(lnum, newp, false_0 != 0);
    inserted_bytes(
        lnum,
        col,
        0 as ::core::ffi::c_int,
        slen as ::core::ffi::c_int,
    );
    (*curwin.get()).w_cursor.col += slen as colnr_T;
}
#[no_mangle]
pub unsafe extern "C" fn del_char(mut fixpos: bool) -> ::core::ffi::c_int {
    mb_adjust_cursor();
    if *get_cursor_pos_ptr() as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    return del_chars(1 as ::core::ffi::c_int, fixpos as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn del_chars(
    mut count: ::core::ffi::c_int,
    mut fixpos: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut bytes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count && *p as ::core::ffi::c_int != NUL {
        let mut l: ::core::ffi::c_int = utfc_ptr2len(p);
        bytes += l;
        p = p.offset(l as isize);
        i += 1;
    }
    return del_bytes(bytes as colnr_T, fixpos != 0, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn del_bytes(
    mut count: colnr_T,
    mut fixpos_arg: bool,
    mut use_delcombine: bool,
) -> ::core::ffi::c_int {
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut col: colnr_T = (*curwin.get()).w_cursor.col;
    let mut fixpos: bool = fixpos_arg;
    let mut oldp: *mut ::core::ffi::c_char = ml_get(lnum);
    let mut oldlen: colnr_T = ml_get_len(lnum);
    if col >= oldlen {
        return FAIL;
    }
    if count == 0 as ::core::ffi::c_int {
        return OK;
    }
    if count < 1 as ::core::ffi::c_int {
        siemsg(
            b"E292: Invalid count for del_bytes(): %ld\0".as_ptr() as *const ::core::ffi::c_char,
            count as int64_t,
        );
        return FAIL;
    }
    if p_deco.get() != 0
        && use_delcombine as ::core::ffi::c_int != 0
        && utfc_ptr2len(oldp.offset(col as isize)) >= count
    {
        let mut p0: *mut ::core::ffi::c_char = oldp.offset(col as isize);
        let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
        if utf_composinglike(p0, p0.offset(utf_ptr2len(p0) as isize), &raw mut state) {
            let mut n: ::core::ffi::c_int = col as ::core::ffi::c_int;
            loop {
                col = n as colnr_T;
                count = utf_ptr2len(oldp.offset(n as isize)) as colnr_T;
                n += count as ::core::ffi::c_int;
                if !utf_composinglike(
                    oldp.offset(col as isize),
                    oldp.offset(n as isize),
                    &raw mut state,
                ) {
                    break;
                }
            }
            fixpos = false_0 != 0;
        }
    }
    let mut movelen: ::core::ffi::c_int =
        oldlen as ::core::ffi::c_int - col as ::core::ffi::c_int - count as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int;
    if movelen <= 1 as ::core::ffi::c_int {
        if col > 0 as ::core::ffi::c_int
            && fixpos as ::core::ffi::c_int != 0
            && restart_edit.get() == 0 as ::core::ffi::c_int
            && get_ve_flags(curwin.get())
                & kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint
                == 0 as ::core::ffi::c_uint
        {
            (*curwin.get()).w_cursor.col -= 1;
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            (*curwin.get()).w_cursor.col -=
                utf_head_off(oldp, oldp.offset((*curwin.get()).w_cursor.col as isize));
        }
        count = oldlen - col;
        movelen = 1 as ::core::ffi::c_int;
    }
    let mut newlen: colnr_T = oldlen - count;
    let mut alloc_newp: bool = ml_line_alloced() == 0;
    let mut newp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !alloc_newp {
        ml_add_deleted_len((*curbuf.get()).b_ml.ml_line_ptr, oldlen as ssize_t);
        newp = oldp;
    } else {
        newp = xmallocz(newlen as size_t) as *mut ::core::ffi::c_char;
        memmove(
            newp as *mut ::core::ffi::c_void,
            oldp as *const ::core::ffi::c_void,
            col as size_t,
        );
    }
    memmove(
        newp.offset(col as isize) as *mut ::core::ffi::c_void,
        oldp.offset(col as isize).offset(count as isize) as *const ::core::ffi::c_void,
        movelen as size_t,
    );
    if alloc_newp {
        ml_replace(lnum, newp, false_0 != 0);
    } else {
        (*curbuf.get()).b_ml.ml_line_textlen =
            (newlen as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
    }
    inserted_bytes(
        lnum,
        col,
        count as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn open_line(
    mut dir: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut second_line_indent: ::core::ffi::c_int,
    mut did_do_comment: *mut bool,
) -> bool {
    let mut next_line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p_extra: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut less_cols: colnr_T = 0 as colnr_T;
    let mut less_cols_off: colnr_T = 0 as colnr_T;
    let mut old_cursor: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut newcol: colnr_T = 0 as colnr_T;
    let mut newindent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut trunc_line: bool = false_0 != 0;
    let mut retval: bool = false_0 != 0;
    let mut extra_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lead_len: ::core::ffi::c_int = 0;
    let mut comment_start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lead_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut leader: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut allocated: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut saved_char: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut do_si: bool = may_do_si();
    let mut no_si: bool = false_0 != 0;
    let mut first_char: ::core::ffi::c_int = NUL;
    let mut vreplace_mode: ::core::ffi::c_int = 0;
    let mut did_append: bool = false;
    let mut saved_pi: ::core::ffi::c_int = (*curbuf.get()).b_p_pi;
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut mincol: colnr_T = (*curwin.get()).w_cursor.col + 1 as colnr_T;
    let mut saved_line: *mut ::core::ffi::c_char =
        xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
    if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
        if (*curwin.get()).w_cursor.lnum < orig_line_count.get() {
            next_line = xstrnsave(
                ml_get((*curwin.get()).w_cursor.lnum + 1 as linenr_T),
                ml_get_len((*curwin.get()).w_cursor.lnum + 1 as linenr_T) as size_t,
            );
        } else {
            next_line = xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
        }
        replace_push_nul();
        replace_push_nul();
        p = saved_line.offset((*curwin.get()).w_cursor.col as isize);
        replace_push(p, strlen(p));
        *saved_line.offset((*curwin.get()).w_cursor.col as isize) = NUL as ::core::ffi::c_char;
    }
    if State.get() & MODE_INSERT as ::core::ffi::c_int != 0
        && State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        p_extra = saved_line.offset((*curwin.get()).w_cursor.col as isize);
        if do_si {
            p = skipwhite(p_extra);
            first_char = *p as ::core::ffi::c_uchar as ::core::ffi::c_int;
        }
        extra_len = strlen(p_extra) as ::core::ffi::c_int;
        saved_char = *p_extra;
        *p_extra = NUL as ::core::ffi::c_char;
    }
    u_clearline(curbuf.get());
    did_si.set(false_0 != 0);
    ai_col.set(0 as ::core::ffi::c_int as colnr_T);
    if dir == FORWARD as ::core::ffi::c_int && did_ai.get() as ::core::ffi::c_int != 0 {
        trunc_line = true_0 != 0;
    }
    if flags & OPENLINE_FORCE_INDENT as ::core::ffi::c_int != 0 {
        newindent = second_line_indent;
    } else if (*curbuf.get()).b_p_ai != 0 || do_si as ::core::ffi::c_int != 0 {
        newindent = indent_size_ts(
            saved_line,
            (*curbuf.get()).b_p_ts,
            (*curbuf.get()).b_p_vts_array,
        );
        if newindent == 0 as ::core::ffi::c_int
            && flags & OPENLINE_COM_LIST as ::core::ffi::c_int == 0
        {
            newindent = second_line_indent;
        }
        if !trunc_line
            && do_si as ::core::ffi::c_int != 0
            && *saved_line as ::core::ffi::c_int != NUL
            && (p_extra.is_null() || first_char != '{' as ::core::ffi::c_int)
        {
            old_cursor = (*curwin.get()).w_cursor;
            let mut ptr: *mut ::core::ffi::c_char = saved_line;
            if flags & OPENLINE_DO_COM as ::core::ffi::c_int != 0 {
                lead_len = get_leader_len(
                    ptr,
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                    false_0 != 0,
                    true_0 != 0,
                );
            } else {
                lead_len = 0 as ::core::ffi::c_int;
            }
            if dir == FORWARD as ::core::ffi::c_int {
                if lead_len == 0 as ::core::ffi::c_int
                    && *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '#' as ::core::ffi::c_int
                {
                    while *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '#' as ::core::ffi::c_int
                        && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
                    {
                        (*curwin.get()).w_cursor.lnum -= 1;
                        ptr = ml_get((*curwin.get()).w_cursor.lnum);
                    }
                    newindent = get_indent();
                }
                if flags & OPENLINE_DO_COM as ::core::ffi::c_int != 0 {
                    lead_len = get_leader_len(
                        ptr,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        false_0 != 0,
                        true_0 != 0,
                    );
                } else {
                    lead_len = 0 as ::core::ffi::c_int;
                }
                if lead_len > 0 as ::core::ffi::c_int {
                    p = skipwhite(ptr);
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '*' as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                    }
                    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '*' as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                        while *p != 0 {
                            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '/' as ::core::ffi::c_int
                                && *p.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == '*' as ::core::ffi::c_int
                            {
                                (*curwin.get()).w_cursor.col = p.offset_from(ptr) as colnr_T;
                                pos = findmatch(::core::ptr::null_mut::<oparg_T>(), NUL);
                                if !pos.is_null() {
                                    (*curwin.get()).w_cursor.lnum = (*pos).lnum;
                                    newindent = get_indent();
                                    break;
                                } else {
                                    ptr = ml_get((*curwin.get()).w_cursor.lnum);
                                    p = ptr.offset((*curwin.get()).w_cursor.col as isize);
                                }
                            }
                            p = p.offset(1);
                        }
                    }
                } else {
                    p = ptr
                        .offset(strlen(ptr) as isize)
                        .offset(-(1 as ::core::ffi::c_int as isize));
                    while p > ptr
                        && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    {
                        p = p.offset(-1);
                    }
                    let mut last_char: ::core::ffi::c_char = *p;
                    if last_char as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                        || last_char as ::core::ffi::c_int == ';' as ::core::ffi::c_int
                    {
                        if p > ptr {
                            p = p.offset(-1);
                        }
                        while p > ptr
                            && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        {
                            p = p.offset(-1);
                        }
                    }
                    if *p as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
                        (*curwin.get()).w_cursor.col = p.offset_from(ptr) as colnr_T;
                        pos = findmatch(
                            ::core::ptr::null_mut::<oparg_T>(),
                            '(' as ::core::ffi::c_int,
                        );
                        if !pos.is_null() {
                            (*curwin.get()).w_cursor.lnum = (*pos).lnum;
                            newindent = get_indent();
                            ptr = get_cursor_line_ptr();
                        }
                    }
                    if last_char as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                        did_si.set(true_0 != 0);
                        no_si = true_0 != 0;
                    } else if last_char as ::core::ffi::c_int != ';' as ::core::ffi::c_int
                        && last_char as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                        && cin_is_cinword(ptr) as ::core::ffi::c_int != 0
                    {
                        did_si.set(true_0 != 0);
                    }
                }
            } else {
                if lead_len == 0 as ::core::ffi::c_int
                    && *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '#' as ::core::ffi::c_int
                {
                    let mut was_backslashed: bool = false_0 != 0;
                    while (*ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '#' as ::core::ffi::c_int
                        || was_backslashed as ::core::ffi::c_int != 0)
                        && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
                    {
                        if *ptr as ::core::ffi::c_int != 0
                            && *ptr.offset(strlen(ptr).wrapping_sub(1 as size_t) as isize)
                                as ::core::ffi::c_int
                                == '\\' as ::core::ffi::c_int
                        {
                            was_backslashed = true_0 != 0;
                        } else {
                            was_backslashed = false_0 != 0;
                        }
                        (*curwin.get()).w_cursor.lnum += 1;
                        ptr = ml_get((*curwin.get()).w_cursor.lnum);
                    }
                    if was_backslashed {
                        newindent = 0 as ::core::ffi::c_int;
                    } else {
                        newindent = get_indent();
                    }
                }
                p = skipwhite(ptr);
                if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                    did_si.set(true_0 != 0);
                } else {
                    can_si_back.set(true_0 != 0);
                }
            }
            (*curwin.get()).w_cursor = old_cursor;
        }
        if do_si {
            can_si.set(true_0 != 0);
        }
        did_ai.set(true_0 != 0);
    }
    let mut do_cindent: bool = p_paste.get() == 0
        && ((*curbuf.get()).b_p_cin != 0 || *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL)
        && in_cinkeys(
            if dir == FORWARD as ::core::ffi::c_int {
                KEY_OPEN_FORW as ::core::ffi::c_int
            } else {
                KEY_OPEN_BACK as ::core::ffi::c_int
            },
            ' ' as ::core::ffi::c_int,
            linewhite((*curwin.get()).w_cursor.lnum),
        ) as ::core::ffi::c_int
            != 0
        && flags & OPENLINE_FORCE_INDENT as ::core::ffi::c_int == 0;
    end_comment_pending.set(NUL);
    if flags & OPENLINE_DO_COM as ::core::ffi::c_int != 0 {
        lead_len = get_leader_len(
            saved_line,
            &raw mut lead_flags,
            dir == BACKWARD as ::core::ffi::c_int,
            true_0 != 0,
        );
        if lead_len == 0 as ::core::ffi::c_int
            && (*curbuf.get()).b_p_cin != 0
            && do_cindent as ::core::ffi::c_int != 0
            && dir == FORWARD as ::core::ffi::c_int
            && (!has_format_option(FO_NO_OPEN_COMS)
                || flags & OPENLINE_FORMAT as ::core::ffi::c_int != 0)
        {
            comment_start = check_linecomment(saved_line);
            if comment_start != MAXCOL as ::core::ffi::c_int {
                lead_len = get_leader_len(
                    saved_line.offset(comment_start as isize),
                    &raw mut lead_flags,
                    false_0 != 0,
                    true_0 != 0,
                );
                if lead_len != 0 as ::core::ffi::c_int {
                    lead_len += comment_start;
                    if !did_do_comment.is_null() {
                        *did_do_comment = true_0 != 0;
                    }
                }
            }
        }
    } else {
        lead_len = 0 as ::core::ffi::c_int;
    }
    if lead_len > 0 as ::core::ffi::c_int {
        let mut lead_repl: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut lead_repl_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut lead_middle: [::core::ffi::c_char; 50] = [0; 50];
        let mut lead_middle_len: ::core::ffi::c_int = 0;
        let mut lead_end: [::core::ffi::c_char; 50] = [0; 50];
        let mut comment_end: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut extra_space: ::core::ffi::c_int = false_0;
        let mut require_blank: bool = false_0 != 0;
        let mut p2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p = lead_flags;
        while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
        {
            if *p as ::core::ffi::c_int == COM_BLANK {
                require_blank = true_0 != 0;
            } else if *p as ::core::ffi::c_int == COM_START
                || *p as ::core::ffi::c_int == COM_MIDDLE
            {
                let mut current_flag: ::core::ffi::c_int =
                    *p as ::core::ffi::c_uchar as ::core::ffi::c_int;
                if *p as ::core::ffi::c_int == COM_START {
                    if dir == BACKWARD as ::core::ffi::c_int {
                        lead_len = 0 as ::core::ffi::c_int;
                        break;
                    } else {
                        copy_option_part(
                            &raw mut p,
                            &raw mut lead_middle as *mut ::core::ffi::c_char,
                            COM_MAX_LEN as size_t,
                            b",\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                        require_blank = false_0 != 0;
                    }
                }
                while *p as ::core::ffi::c_int != 0
                    && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ':' as ::core::ffi::c_int
                {
                    if *p as ::core::ffi::c_int == COM_BLANK {
                        require_blank = true_0 != 0;
                    }
                    p = p.offset(1);
                }
                lead_middle_len = copy_option_part(
                    &raw mut p,
                    &raw mut lead_middle as *mut ::core::ffi::c_char,
                    COM_MAX_LEN as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ) as ::core::ffi::c_int;
                while *p as ::core::ffi::c_int != 0
                    && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ':' as ::core::ffi::c_int
                {
                    if *p as ::core::ffi::c_int == COM_AUTO_END {
                        end_comment_pending.set(-1 as ::core::ffi::c_int);
                    }
                    p = p.offset(1);
                }
                let mut n: size_t = copy_option_part(
                    &raw mut p,
                    &raw mut lead_end as *mut ::core::ffi::c_char,
                    COM_MAX_LEN as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                if end_comment_pending.get() == -1 as ::core::ffi::c_int {
                    end_comment_pending.set(
                        lead_end[n.wrapping_sub(1 as size_t) as usize] as ::core::ffi::c_uchar
                            as ::core::ffi::c_int,
                    );
                }
                if dir == FORWARD as ::core::ffi::c_int {
                    p = saved_line.offset(lead_len as isize);
                    while *p != 0 {
                        if strncmp(p, &raw mut lead_end as *mut ::core::ffi::c_char, n)
                            == 0 as ::core::ffi::c_int
                        {
                            comment_end = p;
                            lead_len = 0 as ::core::ffi::c_int;
                            break;
                        } else {
                            p = p.offset(1);
                        }
                    }
                }
                if lead_len > 0 as ::core::ffi::c_int {
                    if current_flag == COM_START {
                        lead_repl = &raw mut lead_middle as *mut ::core::ffi::c_char;
                        lead_repl_len = lead_middle_len;
                    }
                    if !ascii_iswhite(
                        *saved_line.offset((lead_len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int,
                    ) && (!p_extra.is_null() && (*curwin.get()).w_cursor.col == lead_len
                        || p_extra.is_null()
                            && *saved_line.offset(lead_len as isize) as ::core::ffi::c_int == NUL
                        || require_blank as ::core::ffi::c_int != 0)
                    {
                        extra_space = true_0;
                    }
                }
                break;
            } else if *p as ::core::ffi::c_int == COM_END {
                if dir == FORWARD as ::core::ffi::c_int {
                    comment_end = skipwhite(saved_line);
                    lead_len = 0 as ::core::ffi::c_int;
                    break;
                } else {
                    while p > (*curbuf.get()).b_p_com
                        && *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                    {
                        p = p.offset(-1);
                    }
                    lead_repl = p;
                    while lead_repl > (*curbuf.get()).b_p_com
                        && *lead_repl.offset(-1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            != ':' as ::core::ffi::c_int
                    {
                        lead_repl = lead_repl.offset(-1);
                    }
                    lead_repl_len = p.offset_from(lead_repl) as ::core::ffi::c_int;
                    extra_space = true_0;
                    p2 = p;
                    while *p2 as ::core::ffi::c_int != 0
                        && *p2 as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                    {
                        if *p2 as ::core::ffi::c_int == COM_AUTO_END {
                            end_comment_pending.set(-1 as ::core::ffi::c_int);
                        }
                        p2 = p2.offset(1);
                    }
                    if end_comment_pending.get() == -1 as ::core::ffi::c_int {
                        while *p2 as ::core::ffi::c_int != 0
                            && *p2 as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                        {
                            p2 = p2.offset(1);
                        }
                        end_comment_pending.set(*p2.offset(-1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_uchar
                            as ::core::ffi::c_int);
                    }
                    break;
                }
            } else if *p as ::core::ffi::c_int == COM_FIRST {
                if dir == BACKWARD as ::core::ffi::c_int {
                    lead_len = 0 as ::core::ffi::c_int;
                } else {
                    lead_repl =
                        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                    lead_repl_len = 0 as ::core::ffi::c_int;
                }
                break;
            }
            p = p.offset(1);
        }
        if lead_len > 0 as ::core::ffi::c_int {
            let mut bytes: ::core::ffi::c_int = lead_len
                + lead_repl_len
                + extra_space
                + extra_len
                + (if second_line_indent > 0 as ::core::ffi::c_int {
                    second_line_indent
                } else {
                    0 as ::core::ffi::c_int
                })
                + 1 as ::core::ffi::c_int;
            '_c2rust_label: {
                if bytes >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"bytes >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/change.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1386 as ::core::ffi::c_uint,
                        b"_Bool open_line(int, int, int, _Bool *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            leader = xmalloc(bytes as size_t) as *mut ::core::ffi::c_char;
            allocated = leader;
            xmemcpyz(
                leader as *mut ::core::ffi::c_void,
                saved_line as *const ::core::ffi::c_void,
                lead_len as size_t,
            );
            let mut li: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while li < comment_start {
                if !ascii_iswhite(*leader.offset(li as isize) as ::core::ffi::c_int) {
                    *leader.offset(li as isize) = ' ' as ::core::ffi::c_char;
                }
                li += 1;
            }
            if !lead_repl.is_null() {
                let mut c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                p = lead_flags;
                while *p as ::core::ffi::c_int != NUL
                    && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                {
                    if *p as ::core::ffi::c_int == COM_RIGHT || *p as ::core::ffi::c_int == COM_LEFT
                    {
                        let c2rust_fresh0 = p;
                        p = p.offset(1);
                        c = *c2rust_fresh0 as ::core::ffi::c_uchar as ::core::ffi::c_int;
                    } else if ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                    {
                        off = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
                    } else {
                        p = p.offset(1);
                    }
                }
                if c == COM_RIGHT {
                    p = leader
                        .offset(lead_len as isize)
                        .offset(-(1 as ::core::ffi::c_int as isize));
                    while p > leader
                        && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    {
                        p = p.offset(-1);
                    }
                    p = p.offset(1);
                    let mut repl_size: ::core::ffi::c_int = vim_strnsize(lead_repl, lead_repl_len);
                    let mut old_size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut endp: *mut ::core::ffi::c_char = p;
                    while old_size < repl_size && p > leader {
                        p = p.offset(
                            -((utf_head_off(leader, p.offset(-(1 as ::core::ffi::c_int as isize)))
                                + 1 as ::core::ffi::c_int) as isize),
                        );
                        old_size += ptr2cells(p);
                    }
                    let mut l: ::core::ffi::c_int =
                        lead_repl_len - endp.offset_from(p) as ::core::ffi::c_int;
                    if l != 0 as ::core::ffi::c_int {
                        memmove(
                            endp.offset(l as isize) as *mut ::core::ffi::c_void,
                            endp as *const ::core::ffi::c_void,
                            leader.offset(lead_len as isize).offset_from(endp) as size_t,
                        );
                    }
                    lead_len += l;
                    memmove(
                        p as *mut ::core::ffi::c_void,
                        lead_repl as *const ::core::ffi::c_void,
                        lead_repl_len as size_t,
                    );
                    if p.offset(lead_repl_len as isize) > leader.offset(lead_len as isize) {
                        *p.offset(lead_repl_len as isize) = NUL as ::core::ffi::c_char;
                    }
                    loop {
                        p = p.offset(-1);
                        if p < leader {
                            break;
                        }
                        let mut l_0: ::core::ffi::c_int = utf_head_off(leader, p);
                        if l_0 > 1 as ::core::ffi::c_int {
                            p = p.offset(-(l_0 as isize));
                            if ptr2cells(p) > 1 as ::core::ffi::c_int {
                                *p.offset(1 as ::core::ffi::c_int as isize) =
                                    ' ' as ::core::ffi::c_char;
                                l_0 -= 1;
                            }
                            memmove(
                                p.offset(1 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_void,
                                p.offset(l_0 as isize)
                                    .offset(1 as ::core::ffi::c_int as isize)
                                    as *const ::core::ffi::c_void,
                                leader.offset(lead_len as isize).offset_from(
                                    p.offset(l_0 as isize)
                                        .offset(1 as ::core::ffi::c_int as isize),
                                ) as size_t,
                            );
                            lead_len -= l_0;
                            *p = ' ' as ::core::ffi::c_char;
                        } else if !ascii_iswhite(*p as ::core::ffi::c_int) {
                            *p = ' ' as ::core::ffi::c_char;
                        }
                    }
                } else {
                    p = skipwhite(leader);
                    let mut repl_size_0: ::core::ffi::c_int =
                        vim_strnsize(lead_repl, lead_repl_len);
                    let mut i: ::core::ffi::c_int = 0;
                    let mut l_1: ::core::ffi::c_int = 0;
                    i = 0 as ::core::ffi::c_int;
                    while i < lead_len && *p.offset(i as isize) as ::core::ffi::c_int != NUL {
                        l_1 = utfc_ptr2len(p.offset(i as isize));
                        if vim_strnsize(p, i + l_1) > repl_size_0 {
                            break;
                        }
                        i += l_1;
                    }
                    if i != lead_repl_len {
                        memmove(
                            p.offset(lead_repl_len as isize) as *mut ::core::ffi::c_void,
                            p.offset(i as isize) as *const ::core::ffi::c_void,
                            ((lead_len - i) as isize - p.offset_from(leader)) as size_t,
                        );
                        lead_len += lead_repl_len - i;
                    }
                    memmove(
                        p as *mut ::core::ffi::c_void,
                        lead_repl as *const ::core::ffi::c_void,
                        lead_repl_len as size_t,
                    );
                    p = p.offset(lead_repl_len as isize);
                    while p < leader.offset(lead_len as isize) {
                        if !ascii_iswhite(*p as ::core::ffi::c_int) {
                            if p.offset(1 as ::core::ffi::c_int as isize)
                                < leader.offset(lead_len as isize)
                                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                    == TAB
                            {
                                lead_len -= 1;
                                memmove(
                                    p as *mut ::core::ffi::c_void,
                                    p.offset(1 as ::core::ffi::c_int as isize)
                                        as *const ::core::ffi::c_void,
                                    leader.offset(lead_len as isize).offset_from(p) as size_t,
                                );
                            } else {
                                let mut l_2: ::core::ffi::c_int = utfc_ptr2len(p);
                                if l_2 > 1 as ::core::ffi::c_int {
                                    if ptr2cells(p) > 1 as ::core::ffi::c_int {
                                        l_2 -= 1;
                                        let c2rust_fresh1 = p;
                                        p = p.offset(1);
                                        *c2rust_fresh1 = ' ' as ::core::ffi::c_char;
                                    }
                                    memmove(
                                        p.offset(1 as ::core::ffi::c_int as isize)
                                            as *mut ::core::ffi::c_void,
                                        p.offset(l_2 as isize) as *const ::core::ffi::c_void,
                                        leader.offset(lead_len as isize).offset_from(p) as size_t,
                                    );
                                    lead_len -= l_2 - 1 as ::core::ffi::c_int;
                                }
                                *p = ' ' as ::core::ffi::c_char;
                            }
                        }
                        p = p.offset(1);
                    }
                    *p = NUL as ::core::ffi::c_char;
                }
                if (*curbuf.get()).b_p_ai != 0 || do_si as ::core::ffi::c_int != 0 {
                    newindent = indent_size_ts(
                        leader,
                        (*curbuf.get()).b_p_ts,
                        (*curbuf.get()).b_p_vts_array,
                    );
                }
                if newindent + off < 0 as ::core::ffi::c_int {
                    off = -newindent;
                    newindent = 0 as ::core::ffi::c_int;
                } else {
                    newindent += off;
                }
                while off > 0 as ::core::ffi::c_int
                    && lead_len > 0 as ::core::ffi::c_int
                    && *leader.offset((lead_len - 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int
                {
                    if !vim_strchr(skipwhite(leader), '\t' as ::core::ffi::c_int).is_null() {
                        break;
                    }
                    lead_len -= 1;
                    off -= 1;
                }
                if lead_len > 0 as ::core::ffi::c_int
                    && ascii_iswhite(
                        *leader.offset((lead_len - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                        != 0
                {
                    extra_space = false_0;
                }
                *leader.offset(lead_len as isize) = NUL as ::core::ffi::c_char;
            }
            if extra_space != 0 {
                let c2rust_fresh2 = lead_len;
                lead_len = lead_len + 1;
                *leader.offset(c2rust_fresh2 as isize) = ' ' as ::core::ffi::c_char;
                *leader.offset(lead_len as isize) = NUL as ::core::ffi::c_char;
            }
            newcol = lead_len as colnr_T;
            if newindent != 0 || did_si.get() as ::core::ffi::c_int != 0 {
                while lead_len != 0
                    && ascii_iswhite(*leader as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                {
                    lead_len -= 1;
                    newcol -= 1;
                    leader = leader.offset(1);
                }
            }
            can_si.set(false_0 != 0);
            did_si.set(can_si.get());
        } else if !comment_end.is_null() {
            if *comment_end.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
                && *comment_end.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int
                && ((*curbuf.get()).b_p_ai != 0 || do_si as ::core::ffi::c_int != 0)
            {
                old_cursor = (*curwin.get()).w_cursor;
                (*curwin.get()).w_cursor.col = comment_end.offset_from(saved_line) as colnr_T;
                pos = findmatch(::core::ptr::null_mut::<oparg_T>(), NUL);
                if !pos.is_null() {
                    (*curwin.get()).w_cursor.lnum = (*pos).lnum;
                    newindent = get_indent();
                }
                (*curwin.get()).w_cursor = old_cursor;
            }
        }
    }
    if !p_extra.is_null() {
        *p_extra = saved_char;
        if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0
            && State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0
        {
            replace_push_nul();
        }
        if (*curbuf.get()).b_p_ai != 0 || flags & OPENLINE_DELSPACES as ::core::ffi::c_int != 0 {
            while (*p_extra as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                || *p_extra as ::core::ffi::c_int == '\t' as ::core::ffi::c_int)
                && !utf_iscomposing_first(utf_ptr2char(
                    p_extra.offset(1 as ::core::ffi::c_int as isize),
                ))
            {
                if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0
                    && State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0
                {
                    replace_push(p_extra, 1 as size_t);
                }
                p_extra = p_extra.offset(1);
                less_cols_off += 1;
            }
        }
        less_cols = p_extra.offset_from(saved_line) as ::core::ffi::c_int as colnr_T;
    }
    if p_extra.is_null() {
        p_extra = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if lead_len > 0 as ::core::ffi::c_int {
        if flags & OPENLINE_COM_LIST as ::core::ffi::c_int != 0
            && second_line_indent > 0 as ::core::ffi::c_int
        {
            let mut padding: ::core::ffi::c_int =
                second_line_indent - (newindent + strlen(leader) as ::core::ffi::c_int);
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < padding {
                strcat(leader, b" \0".as_ptr() as *const ::core::ffi::c_char);
                less_cols -= 1;
                newcol += 1;
                i_0 += 1;
            }
        }
        strcat(leader, p_extra);
        p_extra = leader;
        did_ai.set(true_0 != 0);
        less_cols -= lead_len;
    } else {
        end_comment_pending.set(NUL);
    }
    (*curbuf_splice_pending.ptr()) += 1;
    old_cursor = (*curwin.get()).w_cursor;
    let mut old_cmod_flags: ::core::ffi::c_int = (*cmdmod.ptr()).cmod_flags;
    let mut prompt_moved: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if dir == BACKWARD as ::core::ffi::c_int {
        if bt_prompt(curbuf.get()) as ::core::ffi::c_int != 0
            && (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_prompt_start.mark.lnum
        {
            let mut prompt_line: *mut ::core::ffi::c_char = ml_get((*curwin.get()).w_cursor.lnum);
            let mut prompt: *mut ::core::ffi::c_char = prompt_text();
            let mut prompt_len: size_t = strlen(prompt);
            if strncmp(prompt_line, prompt, prompt_len) == 0 as ::core::ffi::c_int {
                memmove(
                    prompt_line as *mut ::core::ffi::c_void,
                    prompt_line.offset(prompt_len as isize) as *const ::core::ffi::c_void,
                    strlen(prompt_line.offset(prompt_len as isize)).wrapping_add(1 as size_t),
                );
                (*cmdmod.ptr()).cmod_flags =
                    (*cmdmod.ptr()).cmod_flags | CMOD_LOCKMARKS as ::core::ffi::c_int;
                ml_replace((*curwin.get()).w_cursor.lnum, prompt_line, true_0 != 0);
                prompt_moved = concat_str(prompt, p_extra);
                p_extra = prompt_moved;
            }
        }
        (*curwin.get()).w_cursor.lnum -= 1;
    }
    '_theend: {
        if State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            || old_cursor.lnum >= orig_line_count.get()
        {
            if ml_append(
                (*curwin.get()).w_cursor.lnum,
                p_extra,
                0 as colnr_T,
                false_0 != 0,
            ) == FAIL
            {
                break '_theend;
            } else {
                mark_adjust(
                    (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                    MAXLNUM as ::core::ffi::c_int as linenr_T,
                    1 as linenr_T,
                    0 as linenr_T,
                    kExtmarkNOOP,
                );
                did_append = true_0 != 0;
            }
        } else {
            (*curwin.get()).w_cursor.lnum += 1;
            if (*curwin.get()).w_cursor.lnum
                >= (*Insstart.ptr()).lnum + vr_lines_changed.get() as linenr_T
            {
                u_save_cursor();
                (*vr_lines_changed.ptr()) += 1;
            }
            ml_replace((*curwin.get()).w_cursor.lnum, p_extra, true_0 != 0);
            changed_bytes((*curwin.get()).w_cursor.lnum, 0 as colnr_T);
            (*curwin.get()).w_cursor.lnum -= 1;
            did_append = false_0 != 0;
        }
        (*inhibit_delete_count.ptr()) += 1;
        if newindent != 0 || did_si.get() as ::core::ffi::c_int != 0 {
            (*curwin.get()).w_cursor.lnum += 1;
            if did_si.get() {
                let mut sw: ::core::ffi::c_int = get_sw_value(curbuf.get());
                if p_sr.get() != 0 {
                    newindent -= newindent % sw;
                }
                newindent += sw;
            }
            if (*curbuf.get()).b_p_ci != 0 {
                copy_indent(newindent, saved_line);
                (*curbuf.get()).b_p_pi = true_0;
            } else {
                set_indent(
                    newindent,
                    SIN_INSERT as ::core::ffi::c_int | SIN_NOMARK as ::core::ffi::c_int,
                );
            }
            less_cols -= (*curwin.get()).w_cursor.col;
            ai_col.set((*curwin.get()).w_cursor.col);
            if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0
                && State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0
            {
                let mut n_0: colnr_T = 0 as colnr_T;
                while n_0 < (*curwin.get()).w_cursor.col {
                    replace_push_nul();
                    n_0 += 1;
                }
            }
            newcol += (*curwin.get()).w_cursor.col;
            if no_si {
                did_si.set(false_0 != 0);
            }
        }
        (*inhibit_delete_count.ptr()) -= 1;
        if State.get() & REPLACE_FLAG as ::core::ffi::c_int != 0
            && State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0
        {
            loop {
                let c2rust_fresh3 = lead_len;
                lead_len = lead_len - 1;
                if c2rust_fresh3 <= 0 as ::core::ffi::c_int {
                    break;
                }
                replace_push_nul();
            }
        }
        (*curwin.get()).w_cursor = old_cursor;
        if dir == FORWARD as ::core::ffi::c_int {
            if trunc_line as ::core::ffi::c_int != 0
                || State.get() & MODE_INSERT as ::core::ffi::c_int != 0
            {
                *saved_line.offset((*curwin.get()).w_cursor.col as isize) =
                    NUL as ::core::ffi::c_char;
                if trunc_line as ::core::ffi::c_int != 0
                    && flags & OPENLINE_KEEPTRAIL as ::core::ffi::c_int == 0
                {
                    truncate_spaces(saved_line, (*curwin.get()).w_cursor.col as size_t);
                }
                ml_replace((*curwin.get()).w_cursor.lnum, saved_line, false_0 != 0);
                let mut new_len: ::core::ffi::c_int = strlen(saved_line) as ::core::ffi::c_int;
                let mut cols_spliced: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if new_len < (*curwin.get()).w_cursor.col {
                    extmark_splice_cols(
                        curbuf.get(),
                        (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int,
                        new_len as colnr_T,
                        (*curwin.get()).w_cursor.col - new_len as colnr_T,
                        0 as colnr_T,
                        kExtmarkUndo,
                    );
                    cols_spliced = (*curwin.get()).w_cursor.col as ::core::ffi::c_int - new_len;
                }
                saved_line = ::core::ptr::null_mut::<::core::ffi::c_char>();
                if did_append {
                    let mut cols_added: ::core::ffi::c_int = mincol as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int
                        + less_cols_off as ::core::ffi::c_int
                        - less_cols as ::core::ffi::c_int;
                    extmark_splice(
                        curbuf.get(),
                        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                        mincol - 1 as colnr_T - cols_spliced as colnr_T,
                        0 as ::core::ffi::c_int,
                        less_cols_off,
                        less_cols_off as bcount_t,
                        1 as ::core::ffi::c_int,
                        cols_added as colnr_T,
                        (1 as ::core::ffi::c_int + cols_added) as bcount_t,
                        kExtmarkUndo,
                    );
                    changed_lines(
                        curbuf.get(),
                        (*curwin.get()).w_cursor.lnum,
                        (*curwin.get()).w_cursor.col,
                        (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                        1 as linenr_T,
                        true_0 != 0,
                    );
                    did_append = false_0 != 0;
                    if flags & OPENLINE_MARKFIX as ::core::ffi::c_int != 0 {
                        mark_col_adjust(
                            (*curwin.get()).w_cursor.lnum,
                            (*curwin.get()).w_cursor.col + less_cols_off,
                            1 as linenr_T,
                            -less_cols,
                            0 as ::core::ffi::c_int,
                        );
                    }
                } else {
                    changed_bytes((*curwin.get()).w_cursor.lnum, (*curwin.get()).w_cursor.col);
                }
            }
            (*curwin.get()).w_cursor.lnum = old_cursor.lnum + 1 as linenr_T;
        }
        if did_append {
            let mut extra: bcount_t = ml_get_len((*curwin.get()).w_cursor.lnum) as bcount_t;
            extmark_splice(
                curbuf.get(),
                (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                0 as colnr_T,
                0 as ::core::ffi::c_int,
                0 as colnr_T,
                0 as bcount_t,
                1 as ::core::ffi::c_int,
                0 as colnr_T,
                1 as bcount_t + extra,
                kExtmarkUndo,
            );
            changed_lines(
                curbuf.get(),
                (*curwin.get()).w_cursor.lnum,
                0 as colnr_T,
                (*curwin.get()).w_cursor.lnum,
                1 as linenr_T,
                true_0 != 0,
            );
        }
        (*curbuf_splice_pending.ptr()) -= 1;
        (*curwin.get()).w_cursor.col = newcol;
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            vreplace_mode = State.get();
            State.set(MODE_INSERT as ::core::ffi::c_int);
        } else {
            vreplace_mode = 0 as ::core::ffi::c_int;
        }
        if p_paste.get() == 0 {
            if leader.is_null()
                && !use_indentexpr_for_lisp()
                && (*curbuf.get()).b_p_lisp != 0
                && (*curbuf.get()).b_p_ai != 0
            {
                fixthisline(Some(
                    get_lisp_indent as unsafe extern "C" fn() -> ::core::ffi::c_int,
                ));
                ai_col.set(getwhitecols_curline() as colnr_T);
            } else if do_cindent as ::core::ffi::c_int != 0
                || (*curbuf.get()).b_p_ai != 0
                    && use_indentexpr_for_lisp() as ::core::ffi::c_int != 0
            {
                do_c_expr_indent();
                ai_col.set(getwhitecols_curline() as colnr_T);
            }
        }
        if vreplace_mode != 0 as ::core::ffi::c_int {
            State.set(vreplace_mode);
        }
        if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
            p_extra = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
            ml_replace((*curwin.get()).w_cursor.lnum, next_line, false_0 != 0);
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            ins_bytes(p_extra);
            xfree(p_extra as *mut ::core::ffi::c_void);
            next_line = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        retval = true_0 != 0;
    }
    (*curbuf.get()).b_p_pi = saved_pi;
    xfree(saved_line as *mut ::core::ffi::c_void);
    xfree(next_line as *mut ::core::ffi::c_void);
    xfree(allocated as *mut ::core::ffi::c_void);
    xfree(prompt_moved as *mut ::core::ffi::c_void);
    (*cmdmod.ptr()).cmod_flags = old_cmod_flags;
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn truncate_line(mut fixpos: ::core::ffi::c_int) {
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut col: colnr_T = (*curwin.get()).w_cursor.col;
    let mut old_line: *mut ::core::ffi::c_char = ml_get(lnum);
    let mut newp: *mut ::core::ffi::c_char = if col == 0 as ::core::ffi::c_int {
        xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char)
    } else {
        xstrnsave(old_line, col as size_t)
    };
    let mut deleted: ::core::ffi::c_int = ml_get_len(lnum) - col as ::core::ffi::c_int;
    ml_replace(lnum, newp, false_0 != 0);
    inserted_bytes(
        lnum,
        (*curwin.get()).w_cursor.col,
        deleted,
        0 as ::core::ffi::c_int,
    );
    if fixpos != 0 && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int {
        (*curwin.get()).w_cursor.col -= 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn del_lines(mut nlines: linenr_T, mut undo: bool) {
    let mut n: ::core::ffi::c_int = 0;
    let mut first: linenr_T = (*curwin.get()).w_cursor.lnum;
    if nlines <= 0 as linenr_T {
        return;
    }
    if undo as ::core::ffi::c_int != 0 && u_savedel(first, nlines) == FAIL {
        return;
    }
    n = 0 as ::core::ffi::c_int;
    while (n as linenr_T) < nlines {
        if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
            break;
        }
        ml_delete_flags(first, ML_DEL_MESSAGE as ::core::ffi::c_int);
        n += 1;
        if first > (*curbuf.get()).b_ml.ml_line_count {
            break;
        }
    }
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    check_cursor_lnum(curwin.get());
    deleted_lines_mark(first, n);
}
#[no_mangle]
pub unsafe extern "C" fn get_leader_len(
    mut line: *mut ::core::ffi::c_char,
    mut flags: *mut *mut ::core::ffi::c_char,
    mut backward: bool,
    mut include_space: bool,
) -> ::core::ffi::c_int {
    let mut j: ::core::ffi::c_int = 0;
    let mut got_com: bool = false_0 != 0;
    let mut part_buf: [::core::ffi::c_char; 50] = [0; 50];
    let mut string: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut middle_match_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut saved_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while ascii_iswhite(*line.offset(i as isize) as ::core::ffi::c_int) {
        i += 1;
    }
    while *line.offset(i as isize) as ::core::ffi::c_int != NUL {
        let mut found_one: bool = false_0 != 0;
        let mut list: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_com;
        while *list != 0 {
            if !got_com && !flags.is_null() {
                *flags = list;
            }
            let mut prev_list: *mut ::core::ffi::c_char = list;
            copy_option_part(
                &raw mut list,
                &raw mut part_buf as *mut ::core::ffi::c_char,
                COM_MAX_LEN as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            string = vim_strchr(
                &raw mut part_buf as *mut ::core::ffi::c_char,
                ':' as ::core::ffi::c_int,
            );
            if !string.is_null() {
                let c2rust_fresh4 = string;
                string = string.offset(1);
                *c2rust_fresh4 = NUL as ::core::ffi::c_char;
                if middle_match_len != 0 as ::core::ffi::c_int
                    && vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_MIDDLE)
                        .is_null()
                    && vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_END).is_null()
                {
                    break;
                }
                if got_com as ::core::ffi::c_int != 0
                    && vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_NEST).is_null()
                {
                    continue;
                }
                if backward as ::core::ffi::c_int != 0
                    && !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_NOBACK)
                        .is_null()
                {
                    continue;
                }
                if ascii_iswhite(
                    *string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) {
                    if i == 0 as ::core::ffi::c_int
                        || !ascii_iswhite(*line.offset((i - 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int)
                    {
                        continue;
                    } else {
                        while ascii_iswhite(
                            *string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) {
                            string = string.offset(1);
                        }
                    }
                }
                j = 0 as ::core::ffi::c_int;
                while *string.offset(j as isize) as ::core::ffi::c_int != NUL
                    && *string.offset(j as isize) as ::core::ffi::c_int
                        == *line.offset((i + j) as isize) as ::core::ffi::c_int
                {
                    j += 1;
                }
                if *string.offset(j as isize) as ::core::ffi::c_int != NUL {
                    continue;
                }
                if !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_BLANK).is_null()
                    && !ascii_iswhite(*line.offset((i + j) as isize) as ::core::ffi::c_int)
                    && *line.offset((i + j) as isize) as ::core::ffi::c_int != NUL
                {
                    continue;
                }
                if !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_MIDDLE).is_null()
                {
                    if middle_match_len == 0 as ::core::ffi::c_int {
                        middle_match_len = j;
                        saved_flags = prev_list;
                    }
                } else {
                    if middle_match_len != 0 as ::core::ffi::c_int && j > middle_match_len {
                        middle_match_len = 0 as ::core::ffi::c_int;
                    }
                    if middle_match_len == 0 as ::core::ffi::c_int {
                        i += j;
                    }
                    found_one = true_0 != 0;
                    break;
                }
            }
        }
        if middle_match_len != 0 as ::core::ffi::c_int {
            if !got_com && !flags.is_null() {
                *flags = saved_flags;
            }
            i += middle_match_len;
            found_one = true_0 != 0;
        }
        if !found_one {
            break;
        }
        result = i;
        while ascii_iswhite(*line.offset(i as isize) as ::core::ffi::c_int) {
            i += 1;
        }
        if include_space {
            result = i;
        }
        got_com = true_0 != 0;
        if vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_NEST).is_null() {
            break;
        }
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn get_last_leader_offset(
    mut line: *mut ::core::ffi::c_char,
    mut flags: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut j: ::core::ffi::c_int = 0;
    let mut lower_check_bound: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut com_leader: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut com_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut part_buf: [::core::ffi::c_char; 50] = [0; 50];
    let mut i: ::core::ffi::c_int = strlen(line) as ::core::ffi::c_int;
    loop {
        i -= 1;
        if i < lower_check_bound {
            break;
        }
        let mut found_one: bool = false_0 != 0;
        let mut list: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_com;
        while *list != 0 {
            let mut flags_save: *mut ::core::ffi::c_char = list;
            copy_option_part(
                &raw mut list,
                &raw mut part_buf as *mut ::core::ffi::c_char,
                COM_MAX_LEN as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            let mut string: *mut ::core::ffi::c_char = vim_strchr(
                &raw mut part_buf as *mut ::core::ffi::c_char,
                ':' as ::core::ffi::c_int,
            );
            if string.is_null() {
                continue;
            }
            let c2rust_fresh5 = string;
            string = string.offset(1);
            *c2rust_fresh5 = NUL as ::core::ffi::c_char;
            com_leader = string;
            if ascii_iswhite(*string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            {
                if i == 0 as ::core::ffi::c_int
                    || !ascii_iswhite(
                        *line.offset((i - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    )
                {
                    continue;
                }
                while ascii_iswhite(*string as ::core::ffi::c_int) {
                    string = string.offset(1);
                }
            }
            j = 0 as ::core::ffi::c_int;
            while *string.offset(j as isize) as ::core::ffi::c_int != NUL
                && *string.offset(j as isize) as ::core::ffi::c_int
                    == *line.offset((i + j) as isize) as ::core::ffi::c_int
            {
                j += 1;
            }
            if *string.offset(j as isize) as ::core::ffi::c_int != NUL {
                continue;
            }
            if !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_BLANK).is_null()
                && !ascii_iswhite(*line.offset((i + j) as isize) as ::core::ffi::c_int)
                && *line.offset((i + j) as isize) as ::core::ffi::c_int != NUL
            {
                continue;
            }
            if !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_MIDDLE).is_null() {
                j = 0 as ::core::ffi::c_int;
                while j <= i
                    && ascii_iswhite(*line.offset(j as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
                {
                    j += 1;
                }
                if j < i {
                    continue;
                }
            }
            found_one = true_0 != 0;
            if !flags.is_null() {
                *flags = flags_save;
            }
            com_flags = flags_save;
            break;
        }
        if !found_one {
            continue;
        }
        let mut part_buf2: [::core::ffi::c_char; 50] = [0; 50];
        result = i;
        if !vim_strchr(&raw mut part_buf as *mut ::core::ffi::c_char, COM_NEST).is_null() {
            continue;
        }
        lower_check_bound = i;
        while ascii_iswhite(*com_leader as ::core::ffi::c_int) {
            com_leader = com_leader.offset(1);
        }
        let mut len1: ::core::ffi::c_int = strlen(com_leader) as ::core::ffi::c_int;
        let mut list_0: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_com;
        while *list_0 != 0 {
            let mut flags_save_0: *mut ::core::ffi::c_char = list_0;
            copy_option_part(
                &raw mut list_0,
                &raw mut part_buf2 as *mut ::core::ffi::c_char,
                COM_MAX_LEN as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if flags_save_0 == com_flags {
                continue;
            }
            let mut string_0: *mut ::core::ffi::c_char = vim_strchr(
                &raw mut part_buf2 as *mut ::core::ffi::c_char,
                ':' as ::core::ffi::c_int,
            );
            string_0 = string_0.offset(1);
            while ascii_iswhite(*string_0 as ::core::ffi::c_int) {
                string_0 = string_0.offset(1);
            }
            let mut len2: ::core::ffi::c_int = strlen(string_0) as ::core::ffi::c_int;
            if len2 == 0 as ::core::ffi::c_int {
                continue;
            }
            let mut off: ::core::ffi::c_int = if len2 > i { i } else { len2 };
            while off > 0 as ::core::ffi::c_int && off + len1 > len2 {
                off -= 1;
                if strncmp(
                    string_0.offset(off as isize),
                    com_leader,
                    (len2 - off) as size_t,
                ) == 0
                {
                    lower_check_bound = if lower_check_bound < i - off {
                        lower_check_bound
                    } else {
                        i - off
                    };
                }
            }
        }
    }
    return result;
}
pub const FO_NO_OPEN_COMS: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const CPO_LISTWM: ::core::ffi::c_int = 'L' as ::core::ffi::c_int;
pub const CPO_DOLLAR: ::core::ffi::c_int = '$' as ::core::ffi::c_int;
pub const COM_NEST: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const COM_BLANK: ::core::ffi::c_int = 'b' as ::core::ffi::c_int;
pub const COM_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const COM_AUTO_END: ::core::ffi::c_int = 'x' as ::core::ffi::c_int;
pub const COM_FIRST: ::core::ffi::c_int = 'f' as ::core::ffi::c_int;
pub const COM_LEFT: ::core::ffi::c_int = 'l' as ::core::ffi::c_int;
pub const COM_RIGHT: ::core::ffi::c_int = 'r' as ::core::ffi::c_int;
pub const COM_NOBACK: ::core::ffi::c_int = 'O' as ::core::ffi::c_int;
pub const COM_MAX_LEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const GRAPHEME_STATE_INIT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
