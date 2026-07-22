use crate::src::nvim::api::extmark::{parse_virt_text, virt_text_to_array};
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_array, api_free_object, api_set_error, api_typename, arena_array,
    cstr_as_string, cstr_to_string, cstrn_as_string, find_buffer_by_handle, find_window_by_handle,
    object_to_hl_id, try_enter, try_leave,
};
use crate::src::nvim::api::private::validate::{
    api_err_conflict, api_err_exp, api_err_invalid, api_err_required,
};
use crate::src::nvim::autocmd::{apply_autocmds, block_autocmds, is_aucmd_win, unblock_autocmds};
use crate::src::nvim::buffer::{bufref_valid, set_bufref};
use crate::src::nvim::drawscreen::{redraw_later, set_must_redraw};
use crate::src::nvim::eval::window::{
    restore_win, restore_win_noblock, switch_win, switch_win_noblock,
};
use crate::src::nvim::ex_docmd::expr_map_locked;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{syn_check_group, syn_id2name};
use crate::src::nvim::main::{
    autocmd_no_enter, autocmd_no_leave, cmdline_win, cmdwin_buf, cmdwin_old_curwin, cmdwin_type,
    cmdwin_win, curbuf, curtab, curwin, e_cmdwin, e_textlock, float_anchor_str,
    opt_winborder_values, p_sb, p_spr, p_winborder, textlock,
};
use crate::src::nvim::mbyte::{mb_string2cells, mb_string2cells_len};
use crate::src::nvim::memory::{strequal, xrealloc, xstrdup};
use crate::src::nvim::option::{copy_option_part, didset_window_options};
use crate::src::nvim::os::libc::{__assert_fail, memcpy, memset, strchr};
use crate::src::nvim::r#move::changed_window_setting;
use crate::src::nvim::strings::striequal;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, Array, BoolVarValue, Boolean, BorderTextType,
    BufUpdateCallbacks, Buffer, CMD_index, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict,
    Error, ErrorType, ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridView,
    Integer, Intersection, KeyDict_win_config, KeyValuePair, LuaRef, MTKey, MTNode, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    Object, ObjectType, OptInt, OptionalKeys, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0, Terminal, Timestamp, TryState,
    UIExtension, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, auto_event, bhdr_T, blob_T,
    blobvar_S, blocknr_T, buf_T, bufref_T, bufstate_T, chunksize_T, cmdidx_T, colnr_T, dict_T,
    dictvar_S, diff_T, diffblock_S, disptick_T, event_T, except_T, except_type_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
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
    undo_object, varnumber_T, vim_exception, virt_line, visualinfo_T, win_T, window_S, wininfo_S,
    winopt_T, wline_T, xfmark_T, QUEUE,
};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::ui_compositor::ui_comp_remove_grid;
use crate::src::nvim::window::{
    check_split_disallowed_err, clear_float_config, goto_tabpage_win, last_status,
    lastwin_nofloating, merge_win_config, one_window, win_append, win_comp_pos, win_find_tabpage,
    win_goto, win_locked, win_remove, win_set_buf, win_setheight_win, win_setwidth_win,
    win_split_ins, win_valid, win_valid_any_tab, window_layout_locked_err, winframe_find_altwin,
    winframe_remove, winframe_restore,
};
use crate::src::nvim::winfloat::{
    win_config_float, win_float_find_altwin, win_new_float, win_set_minimal_style,
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
pub const kZIndexCmdlinePopupMenu: C2Rust_Unnamed_13 = 250;
pub const kZIndexMessages: C2Rust_Unnamed_13 = 200;
pub const kZIndexPopupMenu: C2Rust_Unnamed_13 = 100;
pub const kZIndexFloatDefault: C2Rust_Unnamed_13 = 50;
pub const kZIndexDefaultGrid: C2Rust_Unnamed_13 = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kFloatAnchorSouth: C2Rust_Unnamed_14 = 2;
pub const kFloatAnchorEast: C2Rust_Unnamed_14 = 1;
pub const kBorderTextFooter: BorderTextType = 1;
pub const kBorderTextTitle: BorderTextType = 0;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
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
pub const WSP_VERT: C2Rust_Unnamed_17 = 2;
pub const WSP_NOENTER: C2Rust_Unnamed_17 = 512;
pub const WSP_BELOW: C2Rust_Unnamed_17 = 64;
pub const WSP_BOT: C2Rust_Unnamed_17 = 16;
pub const WSP_ABOVE: C2Rust_Unnamed_17 = 128;
pub const WSP_TOP: C2Rust_Unnamed_17 = 8;
pub const WSP_HOR: C2Rust_Unnamed_17 = 4;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
    pub name: *const ::core::ffi::c_char,
    pub chars: [[::core::ffi::c_char; 32]; 8],
    pub shadow_color: bool,
}
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
pub const UPD_NOT_VALID: C2Rust_Unnamed_16 = 40;
pub const CMD_USER_BUF: CMD_index = -2;
pub const CMD_USER: CMD_index = -1;
pub const CMD_SIZE: CMD_index = 557;
pub const CMD_Next: CMD_index = 556;
pub const CMD_tilde: CMD_index = 555;
pub const CMD_at: CMD_index = 554;
pub const CMD_rshift: CMD_index = 553;
pub const CMD_equal: CMD_index = 552;
pub const CMD_lshift: CMD_index = 551;
pub const CMD_and: CMD_index = 550;
pub const CMD_pound: CMD_index = 549;
pub const CMD_bang: CMD_index = 548;
pub const CMD_z: CMD_index = 547;
pub const CMD_yank: CMD_index = 546;
pub const CMD_xunmenu: CMD_index = 545;
pub const CMD_xunmap: CMD_index = 544;
pub const CMD_xnoremenu: CMD_index = 543;
pub const CMD_xnoremap: CMD_index = 542;
pub const CMD_xmenu: CMD_index = 541;
pub const CMD_xmapclear: CMD_index = 540;
pub const CMD_xmap: CMD_index = 539;
pub const CMD_xall: CMD_index = 538;
pub const CMD_xit: CMD_index = 537;
pub const CMD_wviminfo: CMD_index = 536;
pub const CMD_wundo: CMD_index = 535;
pub const CMD_wshada: CMD_index = 534;
pub const CMD_wqall: CMD_index = 533;
pub const CMD_wq: CMD_index = 532;
pub const CMD_wprevious: CMD_index = 531;
pub const CMD_wnext: CMD_index = 530;
pub const CMD_winpos: CMD_index = 529;
pub const CMD_windo: CMD_index = 528;
pub const CMD_wincmd: CMD_index = 527;
pub const CMD_winsize: CMD_index = 526;
pub const CMD_while: CMD_index = 525;
pub const CMD_wall: CMD_index = 524;
pub const CMD_wNext: CMD_index = 523;
pub const CMD_write: CMD_index = 522;
pub const CMD_vunmenu: CMD_index = 521;
pub const CMD_vunmap: CMD_index = 520;
pub const CMD_vsplit: CMD_index = 519;
pub const CMD_vnoremenu: CMD_index = 518;
pub const CMD_vnew: CMD_index = 517;
pub const CMD_vnoremap: CMD_index = 516;
pub const CMD_vmenu: CMD_index = 515;
pub const CMD_vmapclear: CMD_index = 514;
pub const CMD_vmap: CMD_index = 513;
pub const CMD_viusage: CMD_index = 512;
pub const CMD_vimgrepadd: CMD_index = 511;
pub const CMD_vimgrep: CMD_index = 510;
pub const CMD_view: CMD_index = 509;
pub const CMD_visual: CMD_index = 508;
pub const CMD_vertical: CMD_index = 507;
pub const CMD_verbose: CMD_index = 506;
pub const CMD_version: CMD_index = 505;
pub const CMD_vglobal: CMD_index = 504;
pub const CMD_update: CMD_index = 503;
pub const CMD_unsilent: CMD_index = 502;
pub const CMD_unmenu: CMD_index = 501;
pub const CMD_unmap: CMD_index = 500;
pub const CMD_unlockvar: CMD_index = 499;
pub const CMD_unlet: CMD_index = 498;
pub const CMD_uniq: CMD_index = 497;
pub const CMD_unhide: CMD_index = 496;
pub const CMD_unabbreviate: CMD_index = 495;
pub const CMD_undolist: CMD_index = 494;
pub const CMD_undojoin: CMD_index = 493;
pub const CMD_undo: CMD_index = 492;
pub const CMD_tunmap: CMD_index = 491;
pub const CMD_tunmenu: CMD_index = 490;
pub const CMD_tselect: CMD_index = 489;
pub const CMD_try: CMD_index = 488;
pub const CMD_trust: CMD_index = 487;
pub const CMD_trewind: CMD_index = 486;
pub const CMD_tprevious: CMD_index = 485;
pub const CMD_topleft: CMD_index = 484;
pub const CMD_tnoremap: CMD_index = 483;
pub const CMD_tnext: CMD_index = 482;
pub const CMD_tmapclear: CMD_index = 481;
pub const CMD_tmap: CMD_index = 480;
pub const CMD_tmenu: CMD_index = 479;
pub const CMD_tlunmenu: CMD_index = 478;
pub const CMD_tlnoremenu: CMD_index = 477;
pub const CMD_tlmenu: CMD_index = 476;
pub const CMD_tlast: CMD_index = 475;
pub const CMD_tjump: CMD_index = 474;
pub const CMD_throw: CMD_index = 473;
pub const CMD_tfirst: CMD_index = 472;
pub const CMD_terminal: CMD_index = 471;
pub const CMD_tclfile: CMD_index = 470;
pub const CMD_tcldo: CMD_index = 469;
pub const CMD_tcl: CMD_index = 468;
pub const CMD_tabs: CMD_index = 467;
pub const CMD_tabrewind: CMD_index = 466;
pub const CMD_tabNext: CMD_index = 465;
pub const CMD_tabprevious: CMD_index = 464;
pub const CMD_tabonly: CMD_index = 463;
pub const CMD_tabnew: CMD_index = 462;
pub const CMD_tabnext: CMD_index = 461;
pub const CMD_tablast: CMD_index = 460;
pub const CMD_tabmove: CMD_index = 459;
pub const CMD_tabfirst: CMD_index = 458;
pub const CMD_tabfind: CMD_index = 457;
pub const CMD_tabedit: CMD_index = 456;
pub const CMD_tabdo: CMD_index = 455;
pub const CMD_tabclose: CMD_index = 454;
pub const CMD_tab: CMD_index = 453;
pub const CMD_tags: CMD_index = 452;
pub const CMD_tag: CMD_index = 451;
pub const CMD_tNext: CMD_index = 450;
pub const CMD_tchdir: CMD_index = 449;
pub const CMD_tcd: CMD_index = 448;
pub const CMD_t: CMD_index = 447;
pub const CMD_syncbind: CMD_index = 446;
pub const CMD_syntime: CMD_index = 445;
pub const CMD_syntax: CMD_index = 444;
pub const CMD_swapname: CMD_index = 443;
pub const CMD_sview: CMD_index = 442;
pub const CMD_suspend: CMD_index = 441;
pub const CMD_sunmenu: CMD_index = 440;
pub const CMD_sunmap: CMD_index = 439;
pub const CMD_sunhide: CMD_index = 438;
pub const CMD_stselect: CMD_index = 437;
pub const CMD_stjump: CMD_index = 436;
pub const CMD_stopinsert: CMD_index = 435;
pub const CMD_startreplace: CMD_index = 434;
pub const CMD_startgreplace: CMD_index = 433;
pub const CMD_startinsert: CMD_index = 432;
pub const CMD_stag: CMD_index = 431;
pub const CMD_stop: CMD_index = 430;
pub const CMD_srewind: CMD_index = 429;
pub const CMD_sprevious: CMD_index = 428;
pub const CMD_spellwrong: CMD_index = 427;
pub const CMD_spellundo: CMD_index = 426;
pub const CMD_spellrare: CMD_index = 425;
pub const CMD_spellrepall: CMD_index = 424;
pub const CMD_spellinfo: CMD_index = 423;
pub const CMD_spelldump: CMD_index = 422;
pub const CMD_spellgood: CMD_index = 421;
pub const CMD_split: CMD_index = 420;
pub const CMD_sort: CMD_index = 419;
pub const CMD_source: CMD_index = 418;
pub const CMD_snoremenu: CMD_index = 417;
pub const CMD_snoremap: CMD_index = 416;
pub const CMD_snomagic: CMD_index = 415;
pub const CMD_snext: CMD_index = 414;
pub const CMD_smenu: CMD_index = 413;
pub const CMD_smapclear: CMD_index = 412;
pub const CMD_smap: CMD_index = 411;
pub const CMD_smagic: CMD_index = 410;
pub const CMD_slast: CMD_index = 409;
pub const CMD_sleep: CMD_index = 408;
pub const CMD_silent: CMD_index = 407;
pub const CMD_sign: CMD_index = 406;
pub const CMD_simalt: CMD_index = 405;
pub const CMD_sfirst: CMD_index = 404;
pub const CMD_sfind: CMD_index = 403;
pub const CMD_setlocal: CMD_index = 402;
pub const CMD_setglobal: CMD_index = 401;
pub const CMD_setfiletype: CMD_index = 400;
pub const CMD_set: CMD_index = 399;
pub const CMD_scriptencoding: CMD_index = 398;
pub const CMD_scriptnames: CMD_index = 397;
pub const CMD_sbrewind: CMD_index = 396;
pub const CMD_sbprevious: CMD_index = 395;
pub const CMD_sbnext: CMD_index = 394;
pub const CMD_sbmodified: CMD_index = 393;
pub const CMD_sblast: CMD_index = 392;
pub const CMD_sbfirst: CMD_index = 391;
pub const CMD_sball: CMD_index = 390;
pub const CMD_sbNext: CMD_index = 389;
pub const CMD_sbuffer: CMD_index = 388;
pub const CMD_saveas: CMD_index = 387;
pub const CMD_sandbox: CMD_index = 386;
pub const CMD_sall: CMD_index = 385;
pub const CMD_sargument: CMD_index = 384;
pub const CMD_sNext: CMD_index = 383;
pub const CMD_substitute: CMD_index = 382;
pub const CMD_rviminfo: CMD_index = 381;
pub const CMD_rubyfile: CMD_index = 380;
pub const CMD_rubydo: CMD_index = 379;
pub const CMD_ruby: CMD_index = 378;
pub const CMD_rundo: CMD_index = 377;
pub const CMD_runtime: CMD_index = 376;
pub const CMD_rshada: CMD_index = 375;
pub const CMD_rightbelow: CMD_index = 374;
pub const CMD_right: CMD_index = 373;
pub const CMD_rewind: CMD_index = 372;
pub const CMD_return: CMD_index = 371;
pub const CMD_retab: CMD_index = 370;
pub const CMD_restart: CMD_index = 369;
pub const CMD_resize: CMD_index = 368;
pub const CMD_registers: CMD_index = 367;
pub const CMD_redrawtabline: CMD_index = 366;
pub const CMD_redrawstatus: CMD_index = 365;
pub const CMD_redraw: CMD_index = 364;
pub const CMD_redir: CMD_index = 363;
pub const CMD_redo: CMD_index = 362;
pub const CMD_recover: CMD_index = 361;
pub const CMD_read: CMD_index = 360;
pub const CMD_qall: CMD_index = 359;
pub const CMD_quitall: CMD_index = 358;
pub const CMD_quit: CMD_index = 357;
pub const CMD_pyxfile: CMD_index = 356;
pub const CMD_pythonx: CMD_index = 355;
pub const CMD_pyxdo: CMD_index = 354;
pub const CMD_pyx: CMD_index = 353;
pub const CMD_py3file: CMD_index = 352;
pub const CMD_python3: CMD_index = 351;
pub const CMD_py3do: CMD_index = 350;
pub const CMD_py3: CMD_index = 349;
pub const CMD_pyfile: CMD_index = 348;
pub const CMD_pydo: CMD_index = 347;
pub const CMD_python: CMD_index = 346;
pub const CMD_pwd: CMD_index = 345;
pub const CMD_put: CMD_index = 344;
pub const CMD_ptselect: CMD_index = 343;
pub const CMD_ptrewind: CMD_index = 342;
pub const CMD_ptprevious: CMD_index = 341;
pub const CMD_ptnext: CMD_index = 340;
pub const CMD_ptlast: CMD_index = 339;
pub const CMD_ptjump: CMD_index = 338;
pub const CMD_ptfirst: CMD_index = 337;
pub const CMD_ptNext: CMD_index = 336;
pub const CMD_ptag: CMD_index = 335;
pub const CMD_psearch: CMD_index = 334;
pub const CMD_profdel: CMD_index = 333;
pub const CMD_profile: CMD_index = 332;
pub const CMD_previous: CMD_index = 331;
pub const CMD_preserve: CMD_index = 330;
pub const CMD_ppop: CMD_index = 329;
pub const CMD_popup: CMD_index = 328;
pub const CMD_pop: CMD_index = 327;
pub const CMD_pedit: CMD_index = 326;
pub const CMD_perlfile: CMD_index = 325;
pub const CMD_perldo: CMD_index = 324;
pub const CMD_perl: CMD_index = 323;
pub const CMD_pclose: CMD_index = 322;
pub const CMD_pbuffer: CMD_index = 321;
pub const CMD_packloadall: CMD_index = 320;
pub const CMD_packadd: CMD_index = 319;
pub const CMD_print: CMD_index = 318;
pub const CMD_ownsyntax: CMD_index = 317;
pub const CMD_ounmenu: CMD_index = 316;
pub const CMD_ounmap: CMD_index = 315;
pub const CMD_options: CMD_index = 314;
pub const CMD_onoremenu: CMD_index = 313;
pub const CMD_onoremap: CMD_index = 312;
pub const CMD_only: CMD_index = 311;
pub const CMD_omenu: CMD_index = 310;
pub const CMD_omapclear: CMD_index = 309;
pub const CMD_omap: CMD_index = 308;
pub const CMD_oldfiles: CMD_index = 307;
pub const CMD_nunmenu: CMD_index = 306;
pub const CMD_nunmap: CMD_index = 305;
pub const CMD_number: CMD_index = 304;
pub const CMD_normal: CMD_index = 303;
pub const CMD_noswapfile: CMD_index = 302;
pub const CMD_noremenu: CMD_index = 301;
pub const CMD_noreabbrev: CMD_index = 300;
pub const CMD_nohlsearch: CMD_index = 299;
pub const CMD_noautocmd: CMD_index = 298;
pub const CMD_noremap: CMD_index = 297;
pub const CMD_nnoremenu: CMD_index = 296;
pub const CMD_nnoremap: CMD_index = 295;
pub const CMD_nmenu: CMD_index = 294;
pub const CMD_nmapclear: CMD_index = 293;
pub const CMD_nmap: CMD_index = 292;
pub const CMD_new: CMD_index = 291;
pub const CMD_next: CMD_index = 290;
pub const CMD_mzfile: CMD_index = 289;
pub const CMD_mzscheme: CMD_index = 288;
pub const CMD_mode: CMD_index = 287;
pub const CMD_mkview: CMD_index = 286;
pub const CMD_mkvimrc: CMD_index = 285;
pub const CMD_mkspell: CMD_index = 284;
pub const CMD_mksession: CMD_index = 283;
pub const CMD_mkexrc: CMD_index = 282;
pub const CMD_messages: CMD_index = 281;
pub const CMD_menutranslate: CMD_index = 280;
pub const CMD_menu: CMD_index = 279;
pub const CMD_match: CMD_index = 278;
pub const CMD_marks: CMD_index = 277;
pub const CMD_mapclear: CMD_index = 276;
pub const CMD_map: CMD_index = 275;
pub const CMD_make: CMD_index = 274;
pub const CMD_mark: CMD_index = 273;
pub const CMD_move: CMD_index = 272;
pub const CMD_lsp: CMD_index = 271;
pub const CMD_ls: CMD_index = 270;
pub const CMD_lwindow: CMD_index = 269;
pub const CMD_lvimgrepadd: CMD_index = 268;
pub const CMD_lvimgrep: CMD_index = 267;
pub const CMD_luafile: CMD_index = 266;
pub const CMD_luado: CMD_index = 265;
pub const CMD_lua: CMD_index = 264;
pub const CMD_lunmap: CMD_index = 263;
pub const CMD_ltag: CMD_index = 262;
pub const CMD_lrewind: CMD_index = 261;
pub const CMD_lpfile: CMD_index = 260;
pub const CMD_lprevious: CMD_index = 259;
pub const CMD_lopen: CMD_index = 258;
pub const CMD_lolder: CMD_index = 257;
pub const CMD_lockvar: CMD_index = 256;
pub const CMD_lockmarks: CMD_index = 255;
pub const CMD_loadkeymap: CMD_index = 254;
pub const CMD_loadview: CMD_index = 253;
pub const CMD_lnfile: CMD_index = 252;
pub const CMD_lnewer: CMD_index = 251;
pub const CMD_lnext: CMD_index = 250;
pub const CMD_lnoremap: CMD_index = 249;
pub const CMD_lmake: CMD_index = 248;
pub const CMD_lmapclear: CMD_index = 247;
pub const CMD_lmap: CMD_index = 246;
pub const CMD_llist: CMD_index = 245;
pub const CMD_llast: CMD_index = 244;
pub const CMD_ll: CMD_index = 243;
pub const CMD_lhistory: CMD_index = 242;
pub const CMD_lhelpgrep: CMD_index = 241;
pub const CMD_lgrepadd: CMD_index = 240;
pub const CMD_lgrep: CMD_index = 239;
pub const CMD_lgetexpr: CMD_index = 238;
pub const CMD_lgetbuffer: CMD_index = 237;
pub const CMD_lgetfile: CMD_index = 236;
pub const CMD_lfirst: CMD_index = 235;
pub const CMD_lfdo: CMD_index = 234;
pub const CMD_lfile: CMD_index = 233;
pub const CMD_lexpr: CMD_index = 232;
pub const CMD_let: CMD_index = 231;
pub const CMD_leftabove: CMD_index = 230;
pub const CMD_left: CMD_index = 229;
pub const CMD_ldo: CMD_index = 228;
pub const CMD_lclose: CMD_index = 227;
pub const CMD_lchdir: CMD_index = 226;
pub const CMD_lcd: CMD_index = 225;
pub const CMD_lbottom: CMD_index = 224;
pub const CMD_lbelow: CMD_index = 223;
pub const CMD_lbefore: CMD_index = 222;
pub const CMD_lbuffer: CMD_index = 221;
pub const CMD_later: CMD_index = 220;
pub const CMD_lafter: CMD_index = 219;
pub const CMD_laddfile: CMD_index = 218;
pub const CMD_laddbuffer: CMD_index = 217;
pub const CMD_laddexpr: CMD_index = 216;
pub const CMD_language: CMD_index = 215;
pub const CMD_labove: CMD_index = 214;
pub const CMD_last: CMD_index = 213;
pub const CMD_lNfile: CMD_index = 212;
pub const CMD_lNext: CMD_index = 211;
pub const CMD_list: CMD_index = 210;
pub const CMD_keepalt: CMD_index = 209;
pub const CMD_keeppatterns: CMD_index = 208;
pub const CMD_keepjumps: CMD_index = 207;
pub const CMD_keepmarks: CMD_index = 206;
pub const CMD_k: CMD_index = 205;
pub const CMD_jumps: CMD_index = 204;
pub const CMD_join: CMD_index = 203;
pub const CMD_iunmenu: CMD_index = 202;
pub const CMD_iunabbrev: CMD_index = 201;
pub const CMD_iunmap: CMD_index = 200;
pub const CMD_isplit: CMD_index = 199;
pub const CMD_isearch: CMD_index = 198;
pub const CMD_iput: CMD_index = 197;
pub const CMD_intro: CMD_index = 196;
pub const CMD_inoremenu: CMD_index = 195;
pub const CMD_inoreabbrev: CMD_index = 194;
pub const CMD_inoremap: CMD_index = 193;
pub const CMD_imenu: CMD_index = 192;
pub const CMD_imapclear: CMD_index = 191;
pub const CMD_imap: CMD_index = 190;
pub const CMD_ilist: CMD_index = 189;
pub const CMD_ijump: CMD_index = 188;
pub const CMD_if: CMD_index = 187;
pub const CMD_iabclear: CMD_index = 186;
pub const CMD_iabbrev: CMD_index = 185;
pub const CMD_insert: CMD_index = 184;
pub const CMD_horizontal: CMD_index = 183;
pub const CMD_history: CMD_index = 182;
pub const CMD_hide: CMD_index = 181;
pub const CMD_highlight: CMD_index = 180;
pub const CMD_helptags: CMD_index = 179;
pub const CMD_helpgrep: CMD_index = 178;
pub const CMD_helpclose: CMD_index = 177;
pub const CMD_help: CMD_index = 176;
pub const CMD_gvim: CMD_index = 175;
pub const CMD_gui: CMD_index = 174;
pub const CMD_grepadd: CMD_index = 173;
pub const CMD_grep: CMD_index = 172;
pub const CMD_goto: CMD_index = 171;
pub const CMD_global: CMD_index = 170;
pub const CMD_fclose: CMD_index = 169;
pub const CMD_function: CMD_index = 168;
pub const CMD_for: CMD_index = 167;
pub const CMD_foldopen: CMD_index = 166;
pub const CMD_folddoclosed: CMD_index = 165;
pub const CMD_folddoopen: CMD_index = 164;
pub const CMD_foldclose: CMD_index = 163;
pub const CMD_fold: CMD_index = 162;
pub const CMD_first: CMD_index = 161;
pub const CMD_finish: CMD_index = 160;
pub const CMD_finally: CMD_index = 159;
pub const CMD_find: CMD_index = 158;
pub const CMD_filter: CMD_index = 157;
pub const CMD_filetype: CMD_index = 156;
pub const CMD_files: CMD_index = 155;
pub const CMD_file: CMD_index = 154;
pub const CMD_exusage: CMD_index = 153;
pub const CMD_exit: CMD_index = 152;
pub const CMD_execute: CMD_index = 151;
pub const CMD_ex: CMD_index = 150;
pub const CMD_eval: CMD_index = 149;
pub const CMD_enew: CMD_index = 148;
pub const CMD_endwhile: CMD_index = 147;
pub const CMD_endtry: CMD_index = 146;
pub const CMD_endfor: CMD_index = 145;
pub const CMD_endfunction: CMD_index = 144;
pub const CMD_endif: CMD_index = 143;
pub const CMD_emenu: CMD_index = 142;
pub const CMD_elseif: CMD_index = 141;
pub const CMD_else: CMD_index = 140;
pub const CMD_echon: CMD_index = 139;
pub const CMD_echomsg: CMD_index = 138;
pub const CMD_echohl: CMD_index = 137;
pub const CMD_echoerr: CMD_index = 136;
pub const CMD_echo: CMD_index = 135;
pub const CMD_earlier: CMD_index = 134;
pub const CMD_edit: CMD_index = 133;
pub const CMD_dsplit: CMD_index = 132;
pub const CMD_dsearch: CMD_index = 131;
pub const CMD_drop: CMD_index = 130;
pub const CMD_doautoall: CMD_index = 129;
pub const CMD_doautocmd: CMD_index = 128;
pub const CMD_dlist: CMD_index = 127;
pub const CMD_djump: CMD_index = 126;
pub const CMD_digraphs: CMD_index = 125;
pub const CMD_diffthis: CMD_index = 124;
pub const CMD_diffsplit: CMD_index = 123;
pub const CMD_diffput: CMD_index = 122;
pub const CMD_diffpatch: CMD_index = 121;
pub const CMD_diffoff: CMD_index = 120;
pub const CMD_diffget: CMD_index = 119;
pub const CMD_diffupdate: CMD_index = 118;
pub const CMD_display: CMD_index = 117;
pub const CMD_detach: CMD_index = 116;
pub const CMD_delfunction: CMD_index = 115;
pub const CMD_delcommand: CMD_index = 114;
pub const CMD_defer: CMD_index = 113;
pub const CMD_debuggreedy: CMD_index = 112;
pub const CMD_debug: CMD_index = 111;
pub const CMD_delmarks: CMD_index = 110;
pub const CMD_delete: CMD_index = 109;
pub const CMD_cwindow: CMD_index = 108;
pub const CMD_cunmenu: CMD_index = 107;
pub const CMD_cunabbrev: CMD_index = 106;
pub const CMD_cunmap: CMD_index = 105;
pub const CMD_crewind: CMD_index = 104;
pub const CMD_cquit: CMD_index = 103;
pub const CMD_cpfile: CMD_index = 102;
pub const CMD_cprevious: CMD_index = 101;
pub const CMD_copen: CMD_index = 100;
pub const CMD_const: CMD_index = 99;
pub const CMD_connect: CMD_index = 98;
pub const CMD_confirm: CMD_index = 97;
pub const CMD_continue: CMD_index = 96;
pub const CMD_compiler: CMD_index = 95;
pub const CMD_comclear: CMD_index = 94;
pub const CMD_command: CMD_index = 93;
pub const CMD_colorscheme: CMD_index = 92;
pub const CMD_colder: CMD_index = 91;
pub const CMD_copy: CMD_index = 90;
pub const CMD_cnoremenu: CMD_index = 89;
pub const CMD_cnoreabbrev: CMD_index = 88;
pub const CMD_cnoremap: CMD_index = 87;
pub const CMD_cnfile: CMD_index = 86;
pub const CMD_cnewer: CMD_index = 85;
pub const CMD_cnext: CMD_index = 84;
pub const CMD_cmenu: CMD_index = 83;
pub const CMD_cmapclear: CMD_index = 82;
pub const CMD_cmap: CMD_index = 81;
pub const CMD_clearjumps: CMD_index = 80;
pub const CMD_close: CMD_index = 79;
pub const CMD_clast: CMD_index = 78;
pub const CMD_clist: CMD_index = 77;
pub const CMD_chistory: CMD_index = 76;
pub const CMD_checktime: CMD_index = 75;
pub const CMD_checkpath: CMD_index = 74;
pub const CMD_checkhealth: CMD_index = 73;
pub const CMD_changes: CMD_index = 72;
pub const CMD_chdir: CMD_index = 71;
pub const CMD_cgetexpr: CMD_index = 70;
pub const CMD_cgetbuffer: CMD_index = 69;
pub const CMD_cgetfile: CMD_index = 68;
pub const CMD_cfirst: CMD_index = 67;
pub const CMD_cfdo: CMD_index = 66;
pub const CMD_cfile: CMD_index = 65;
pub const CMD_cexpr: CMD_index = 64;
pub const CMD_center: CMD_index = 63;
pub const CMD_cdo: CMD_index = 62;
pub const CMD_cd: CMD_index = 61;
pub const CMD_cclose: CMD_index = 60;
pub const CMD_cc: CMD_index = 59;
pub const CMD_cbottom: CMD_index = 58;
pub const CMD_cbelow: CMD_index = 57;
pub const CMD_cbefore: CMD_index = 56;
pub const CMD_cbuffer: CMD_index = 55;
pub const CMD_catch: CMD_index = 54;
pub const CMD_call: CMD_index = 53;
pub const CMD_cafter: CMD_index = 52;
pub const CMD_caddfile: CMD_index = 51;
pub const CMD_caddexpr: CMD_index = 50;
pub const CMD_caddbuffer: CMD_index = 49;
pub const CMD_cabove: CMD_index = 48;
pub const CMD_cabclear: CMD_index = 47;
pub const CMD_cabbrev: CMD_index = 46;
pub const CMD_cNfile: CMD_index = 45;
pub const CMD_cNext: CMD_index = 44;
pub const CMD_change: CMD_index = 43;
pub const CMD_bwipeout: CMD_index = 42;
pub const CMD_bunload: CMD_index = 41;
pub const CMD_bufdo: CMD_index = 40;
pub const CMD_buffers: CMD_index = 39;
pub const CMD_browse: CMD_index = 38;
pub const CMD_breaklist: CMD_index = 37;
pub const CMD_breakdel: CMD_index = 36;
pub const CMD_breakadd: CMD_index = 35;
pub const CMD_break: CMD_index = 34;
pub const CMD_brewind: CMD_index = 33;
pub const CMD_bprevious: CMD_index = 32;
pub const CMD_botright: CMD_index = 31;
pub const CMD_bnext: CMD_index = 30;
pub const CMD_bmodified: CMD_index = 29;
pub const CMD_blast: CMD_index = 28;
pub const CMD_bfirst: CMD_index = 27;
pub const CMD_belowright: CMD_index = 26;
pub const CMD_bdelete: CMD_index = 25;
pub const CMD_balt: CMD_index = 24;
pub const CMD_badd: CMD_index = 23;
pub const CMD_ball: CMD_index = 22;
pub const CMD_bNext: CMD_index = 21;
pub const CMD_buffer: CMD_index = 20;
pub const CMD_aunmenu: CMD_index = 19;
pub const CMD_augroup: CMD_index = 18;
pub const CMD_autocmd: CMD_index = 17;
pub const CMD_ascii: CMD_index = 16;
pub const CMD_argument: CMD_index = 15;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argedit: CMD_index = 12;
pub const CMD_argdedupe: CMD_index = 11;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_argdelete: CMD_index = 9;
pub const CMD_argadd: CMD_index = 8;
pub const CMD_args: CMD_index = 7;
pub const CMD_anoremenu: CMD_index = 6;
pub const CMD_amenu: CMD_index = 5;
pub const CMD_all: CMD_index = 4;
pub const CMD_aboveleft: CMD_index = 3;
pub const CMD_abclear: CMD_index = 2;
pub const CMD_abbreviate: CMD_index = 1;
pub const CMD_append: CMD_index = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_16 = 50;
pub const UPD_SOME_VALID: C2Rust_Unnamed_16 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_16 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_16 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_16 = 20;
pub const UPD_VALID: C2Rust_Unnamed_16 = 10;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_17 = 1024;
pub const WSP_NEWLOC: C2Rust_Unnamed_17 = 256;
pub const WSP_HELP: C2Rust_Unnamed_17 = 32;
pub const WSP_ROOM: C2Rust_Unnamed_17 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const KEYSET_OPTIDX_win_config__col: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__row: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__win: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__hide: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__width: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__split: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__title: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__mouse: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__fixed: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__style: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__anchor: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__bufpos: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__height: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__zindex: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__footer: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__border: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__external: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__relative: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__vertical: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__focusable: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__noautocmd: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__title_pos: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__footer_pos: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config___cmdline_offset: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const KEYDICT_INIT: KeyDict_win_config = KeyDict_win_config {
    is_set__win_config_: 0 as OptionalKeys,
    external: false,
    fixed: false,
    focusable: false,
    footer: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    footer_pos: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    hide: false,
    height: 0,
    mouse: false,
    relative: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    row: 0.,
    style: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    noautocmd: false,
    vertical: false,
    win: 0,
    width: 0,
    zindex: 0,
    anchor: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    border: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    bufpos: Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    },
    col: 0.,
    split: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    title: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    title_pos: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    _cmdline_offset: 0,
};
#[no_mangle]
pub unsafe extern "C" fn nvim_open_win(
    mut buf: Buffer,
    mut enter: Boolean,
    mut config: *mut KeyDict_win_config,
    mut err: *mut Error,
) -> Window {
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return 0 as Window;
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
        return 0 as Window;
    }
    let mut fconfig: WinConfig = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0 as ::core::ffi::c_int,
        width: 0 as ::core::ffi::c_int,
        row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false_0 != 0,
        focusable: true_0 != 0,
        mouse: true_0 != 0,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as ::core::ffi::c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false_0 != 0,
        fixed: false_0 != 0,
        hide: false_0 != 0,
        _cmdline_offset: INT_MAX,
    };
    if !parse_win_config(
        ::core::ptr::null_mut::<win_T>(),
        config,
        &raw mut fconfig,
        false_0 != 0,
        err,
    ) {
        return 0 as Window;
    }
    let mut is_split: bool = (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
        != 0 as ::core::ffi::c_ulonglong
        || (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__vertical
            != 0 as ::core::ffi::c_ulonglong;
    let mut rv: Window = 0 as Window;
    if fconfig.noautocmd {
        block_autocmds();
    }
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tp: *mut tabpage_T = curtab.get();
    '_c2rust_label: {
        if !(*curwin.ptr()).is_null() {
        } else {
            __assert_fail(
                b"curwin != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/win_config.rs\0".as_ptr() as *const ::core::ffi::c_char,
                229 as ::core::ffi::c_uint,
                b"Window nvim_open_win(Buffer, Boolean, KeyDict_win_config *, Error *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut parent: *mut win_T = if (*config).win == 0 as ::core::ffi::c_int {
        curwin.get()
    } else {
        ::core::ptr::null_mut::<win_T>()
    };
    '_cleanup: {
        if (*config).win > 0 as ::core::ffi::c_int {
            parent = find_window_by_handle(fconfig.window, err);
            if parent.is_null() {
                break '_cleanup;
            } else if is_split as ::core::ffi::c_int != 0
                && (*parent).w_floating as ::core::ffi::c_int != 0
            {
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Cannot split a floating window\0".as_ptr() as *const ::core::ffi::c_char,
                );
                break '_cleanup;
            } else {
                tp = win_find_tabpage(parent);
            }
        }
        if is_split {
            if !check_split_disallowed_err(
                if !parent.is_null() {
                    parent
                } else {
                    curwin.get()
                },
                err,
            ) {
                break '_cleanup;
            } else {
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__vertical
                    != 0 as ::core::ffi::c_ulonglong
                    && !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
                        != 0 as ::core::ffi::c_ulonglong)
                {
                    if (*config).vertical {
                        fconfig.split = (if p_spr.get() != 0 {
                            kWinSplitRight as ::core::ffi::c_int
                        } else {
                            kWinSplitLeft as ::core::ffi::c_int
                        }) as WinSplit;
                    } else {
                        fconfig.split = (if p_sb.get() != 0 {
                            kWinSplitBelow as ::core::ffi::c_int
                        } else {
                            kWinSplitAbove as ::core::ffi::c_int
                        }) as WinSplit;
                    }
                }
                let mut flags: ::core::ffi::c_int =
                    win_split_flags(fconfig.split, parent.is_null())
                        | WSP_NOENTER as ::core::ffi::c_int;
                let mut size: ::core::ffi::c_int = if flags & WSP_VERT as ::core::ffi::c_int != 0 {
                    fconfig.width
                } else {
                    fconfig.height
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
                if parent.is_null() || parent == curwin.get() {
                    wp = win_split_ins(
                        size,
                        flags,
                        ::core::ptr::null_mut::<win_T>(),
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<frame_T>(),
                    );
                } else {
                    let mut switchwin: switchwin_T = switchwin_T {
                        sw_curwin: ::core::ptr::null_mut::<win_T>(),
                        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                        sw_same_win: false,
                        sw_visual_active: false,
                    };
                    let result: ::core::ffi::c_int =
                        switch_win(&raw mut switchwin, parent, tp, true);
                    '_c2rust_label_0: {
                        if result == 1 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"result == OK\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/api/win_config.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                264 as ::core::ffi::c_uint,
                                b"Window nvim_open_win(Buffer, Boolean, KeyDict_win_config *, Error *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    wp = win_split_ins(
                        size,
                        flags,
                        ::core::ptr::null_mut::<win_T>(),
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<frame_T>(),
                    );
                    restore_win(&raw mut switchwin, true);
                }
                try_leave(&raw mut tstate, err);
                if !wp.is_null() {
                    (*wp).w_config = fconfig;
                    if size > 0 as ::core::ffi::c_int {
                        if flags & WSP_VERT as ::core::ffi::c_int != 0 && (*wp).w_width != size {
                            win_setwidth_win(size, wp);
                        } else if flags & WSP_VERT as ::core::ffi::c_int == 0
                            && (*wp).w_height != size
                        {
                            win_setheight_win(size, wp);
                        }
                    }
                }
            }
        } else if (*(*curwin.get()).w_buffer).b_locked_split != 0 {
            api_set_error(
                err,
                kErrorTypeException,
                b"E1159: Cannot open a float when closing the buffer\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            break '_cleanup;
        } else {
            wp = win_new_float(::core::ptr::null_mut::<win_T>(), false_0 != 0, fconfig, err);
        }
        if wp.is_null() {
            if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Failed to create window\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        } else {
            if fconfig._cmdline_offset < INT_MAX {
                cmdline_win.set(wp);
            }
            bufref = bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            };
            set_bufref(&raw mut bufref, b);
            if !fconfig.noautocmd {
                let mut switchwin_0: switchwin_T = switchwin_T {
                    sw_curwin: ::core::ptr::null_mut::<win_T>(),
                    sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                    sw_same_win: false,
                    sw_visual_active: false,
                };
                let result_0: ::core::ffi::c_int =
                    switch_win_noblock(&raw mut switchwin_0, wp, tp, true_0 != 0);
                '_c2rust_label_1: {
                    if result_0 == 1 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"result == OK\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/api/win_config.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            311 as ::core::ffi::c_uint,
                            b"Window nvim_open_win(Buffer, Boolean, KeyDict_win_config *, Error *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if apply_autocmds(
                    EVENT_WINNEW,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                ) {
                    tp = win_find_tabpage(wp);
                }
                restore_win_noblock(&raw mut switchwin_0, true_0 != 0);
            }
            if !tp.is_null() && enter as ::core::ffi::c_int != 0 {
                goto_tabpage_win(tp, wp);
                tp = win_find_tabpage(wp);
            }
            if !tp.is_null()
                && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                && b != (*wp).w_buffer
            {
                let au_no_enter_leave: bool = curwin.get() != wp && !fconfig.noautocmd;
                if au_no_enter_leave {
                    (*autocmd_no_enter.ptr()) += 1;
                    (*autocmd_no_leave.ptr()) += 1;
                }
                win_set_buf(wp, b, err);
                if !fconfig.noautocmd {
                    tp = win_find_tabpage(wp);
                }
                if au_no_enter_leave {
                    (*autocmd_no_enter.ptr()) -= 1;
                    (*autocmd_no_leave.ptr()) -= 1;
                }
            }
            if tp.is_null() {
                api_clear_error(err);
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Window was closed immediately\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                if fconfig.style as ::core::ffi::c_uint
                    == kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    win_set_minimal_style(wp);
                    didset_window_options(wp, true_0 != 0);
                    changed_window_setting(wp);
                }
                rv = (*wp).handle as Window;
            }
        }
    }
    if fconfig.noautocmd {
        unblock_autocmds();
    }
    return rv;
}
unsafe extern "C" fn win_split_dir(mut win: *mut win_T) -> WinSplit {
    if (*win).w_frame.is_null() || (*(*win).w_frame).fr_parent.is_null() {
        return kWinSplitLeft;
    }
    let mut layout: ::core::ffi::c_char = (*(*(*win).w_frame).fr_parent).fr_layout;
    if layout as ::core::ffi::c_int == FR_COL {
        return (if !(*(*win).w_frame).fr_next.is_null() {
            kWinSplitAbove as ::core::ffi::c_int
        } else {
            kWinSplitBelow as ::core::ffi::c_int
        }) as WinSplit;
    } else {
        return (if !(*(*win).w_frame).fr_next.is_null() {
            kWinSplitLeft as ::core::ffi::c_int
        } else {
            kWinSplitRight as ::core::ffi::c_int
        }) as WinSplit;
    };
}
unsafe extern "C" fn win_split_flags(
    mut split: WinSplit,
    mut toplevel: bool,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if split as ::core::ffi::c_uint == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
        || split as ::core::ffi::c_uint
            == kWinSplitBelow as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= WSP_HOR as ::core::ffi::c_int;
    } else {
        flags |= WSP_VERT as ::core::ffi::c_int;
    }
    if split as ::core::ffi::c_uint == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
        || split as ::core::ffi::c_uint
            == kWinSplitLeft as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= if toplevel as ::core::ffi::c_int != 0 {
            WSP_TOP as ::core::ffi::c_int
        } else {
            WSP_ABOVE as ::core::ffi::c_int
        };
    } else {
        flags |= if toplevel as ::core::ffi::c_int != 0 {
            WSP_BOT as ::core::ffi::c_int
        } else {
            WSP_BELOW as ::core::ffi::c_int
        };
    }
    return flags;
}
unsafe extern "C" fn win_can_move_tp(
    mut wp: *mut win_T,
    mut tp: *mut tabpage_T,
    mut err: *mut Error,
) -> bool {
    if one_window(
        wp,
        if tp == curtab.get() {
            ::core::ptr::null_mut::<tabpage_T>()
        } else {
            tp
        },
    ) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Cannot move last non-floating window\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    if win_locked(wp) != 0 {
        api_set_error(
            err,
            kErrorTypeException,
            b"Cannot move window to another tabpage whilst in use\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    if window_layout_locked_err(CMD_SIZE, err) {
        return false_0 != 0;
    }
    if textlock.get() != 0 || expr_map_locked() as ::core::ffi::c_int != 0 {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_textlock as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    if is_aucmd_win(wp) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Cannot move autocmd window to another tabpage\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    if wp == cmdwin_win.get() || wp == cmdwin_old_curwin.get() {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn win_find_altwin(mut win: *mut win_T, mut tp: *mut tabpage_T) -> *mut win_T {
    if (*win).w_floating {
        return win_float_find_altwin(
            win,
            if tp == curtab.get() {
                ::core::ptr::null_mut::<tabpage_T>()
            } else {
                tp
            },
        );
    } else {
        let mut dir: ::core::ffi::c_int = 0;
        return winframe_find_altwin(
            win,
            &raw mut dir,
            if tp == curtab.get() {
                ::core::ptr::null_mut::<tabpage_T>()
            } else {
                tp
            },
            ::core::ptr::null_mut::<*mut frame_T>(),
        );
    };
}
unsafe extern "C" fn win_config_split(
    mut win: *mut win_T,
    mut config: *const KeyDict_win_config,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) -> bool {
    let mut dir: ::core::ffi::c_int = 0;
    let mut unflat_altfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    let mut altwin_0: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut flags: ::core::ffi::c_int = 0;
    let mut parent: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut parent_tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut win_tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut to_split_ok: bool = false;
    let mut curwin_moving_tp: bool = false;
    let mut was_split: bool = !(*win).w_floating;
    let mut has_split: bool = (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
        != 0 as ::core::ffi::c_ulonglong;
    let mut has_vertical: bool = (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__vertical
        != 0 as ::core::ffi::c_ulonglong;
    let mut old_split: WinSplit = win_split_dir(win);
    if has_vertical as ::core::ffi::c_int != 0 && !has_split {
        if (*config).vertical {
            (*fconfig).split = (if old_split as ::core::ffi::c_uint
                == kWinSplitRight as ::core::ffi::c_int as ::core::ffi::c_uint
                || p_spr.get() != 0
            {
                kWinSplitRight as ::core::ffi::c_int
            } else {
                kWinSplitLeft as ::core::ffi::c_int
            }) as WinSplit;
        } else {
            (*fconfig).split = (if old_split as ::core::ffi::c_uint
                == kWinSplitBelow as ::core::ffi::c_int as ::core::ffi::c_uint
                || p_sb.get() != 0
            {
                kWinSplitBelow as ::core::ffi::c_int
            } else {
                kWinSplitAbove as ::core::ffi::c_int
            }) as WinSplit;
        }
    }
    '_resize: {
        if !(!has_vertical && !has_split
            || was_split as ::core::ffi::c_int != 0
                && !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win
                    != 0 as ::core::ffi::c_ulonglong)
                && old_split as ::core::ffi::c_uint == (*fconfig).split as ::core::ffi::c_uint)
        {
            parent = ::core::ptr::null_mut::<win_T>();
            parent_tp = ::core::ptr::null_mut::<tabpage_T>();
            if (*config).win == 0 as ::core::ffi::c_int {
                parent = curwin.get();
                parent_tp = curtab.get();
            } else if (*config).win > 0 as ::core::ffi::c_int {
                parent = find_window_by_handle((*fconfig).window, err);
                if parent.is_null() {
                    return false_0 != 0;
                }
                parent_tp = win_find_tabpage(parent);
            }
            win_tp = win_find_tabpage(win);
            if !parent.is_null() {
                if (*parent).w_floating {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Cannot split a floating window\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    return false_0 != 0;
                }
                if win_tp != parent_tp && !win_can_move_tp(win, win_tp, err) {
                    return false_0 != 0;
                }
            }
            if !check_split_disallowed_err(win, err) {
                return false_0 != 0;
            }
            to_split_ok = false_0 != 0;
            curwin_moving_tp = win == curwin.get() && !parent.is_null() && win_tp != parent_tp;
            '_restore_curwin: {
                if curwin_moving_tp {
                    let mut altwin: *mut win_T = win_find_altwin(win, win_tp);
                    '_c2rust_label: {
                        if !altwin.is_null() {
                        } else {
                            __assert_fail(
                                b"altwin\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/api/win_config.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                492 as ::core::ffi::c_uint,
                                b"_Bool win_config_split(win_T *, const KeyDict_win_config *, WinConfig *, Error *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    win_goto(altwin);
                    if curwin.get() == win {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Failed to switch away from window %d\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            (*win).handle,
                        );
                        return false_0 != 0;
                    }
                    win_tp = win_find_tabpage(win);
                    if win_tp.is_null() || !win_valid_any_tab(parent) {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Windows to split were closed\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_restore_curwin;
                    } else if was_split as ::core::ffi::c_int
                        == (*win).w_floating as ::core::ffi::c_int
                        || (*parent).w_floating as ::core::ffi::c_int != 0
                    {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Floating state of windows to split changed\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_restore_curwin;
                    }
                }
                dir = 0 as ::core::ffi::c_int;
                unflat_altfr = ::core::ptr::null_mut::<frame_T>();
                altwin_0 = ::core::ptr::null_mut::<win_T>();
                if was_split {
                    if (*(*win).w_frame).fr_parent.is_null() {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Cannot move last non-floating window\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_restore_curwin;
                    } else if !parent.is_null() && (*parent).handle == (*win).handle {
                        let mut n_frames: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut fr: *mut frame_T = (*(*(*win).w_frame).fr_parent).fr_child;
                        while !fr.is_null() {
                            n_frames += 1;
                            fr = (*fr).fr_next;
                        }
                        let mut neighbor: *mut win_T = ::core::ptr::null_mut::<win_T>();
                        if n_frames > 2 as ::core::ffi::c_int {
                            let mut frame: *mut frame_T = (*(*win).w_frame).fr_parent;
                            if !(*frame).fr_parent.is_null() {
                                if (*fconfig).split as ::core::ffi::c_uint
                                    == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
                                    || (*fconfig).split as ::core::ffi::c_uint
                                        == kWinSplitLeft as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                {
                                    neighbor = (*win).w_next;
                                } else {
                                    neighbor = (*win).w_prev;
                                }
                            }
                            altwin_0 = winframe_remove(
                                win,
                                &raw mut dir,
                                if win_tp == curtab.get() {
                                    ::core::ptr::null_mut::<tabpage_T>()
                                } else {
                                    win_tp
                                },
                                &raw mut unflat_altfr,
                            );
                        } else if n_frames == 2 as ::core::ffi::c_int {
                            altwin_0 = winframe_remove(
                                win,
                                &raw mut dir,
                                if win_tp == curtab.get() {
                                    ::core::ptr::null_mut::<tabpage_T>()
                                } else {
                                    win_tp
                                },
                                &raw mut unflat_altfr,
                            );
                            neighbor = altwin_0;
                        } else {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                b"Cannot split window into itself\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_restore_curwin;
                        }
                        parent = neighbor;
                    } else {
                        altwin_0 = winframe_remove(
                            win,
                            &raw mut dir,
                            if win_tp == curtab.get() {
                                ::core::ptr::null_mut::<tabpage_T>()
                            } else {
                                win_tp
                            },
                            &raw mut unflat_altfr,
                        );
                    }
                } else {
                    altwin_0 = win_float_find_altwin(
                        win,
                        if win_tp == curtab.get() {
                            ::core::ptr::null_mut::<tabpage_T>()
                        } else {
                            win_tp
                        },
                    );
                }
                win_remove(
                    win,
                    if win_tp == curtab.get() {
                        ::core::ptr::null_mut::<tabpage_T>()
                    } else {
                        win_tp
                    },
                );
                if win_tp == curtab.get() {
                    last_status(false_0 != 0);
                    win_comp_pos();
                }
                flags = win_split_flags((*fconfig).split, parent.is_null())
                    | WSP_NOENTER as ::core::ffi::c_int;
                parent_tp = if !parent.is_null() {
                    win_find_tabpage(parent)
                } else {
                    curtab.get()
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
                let need_switch: bool = !parent.is_null() && parent != curwin.get();
                let mut switchwin: switchwin_T = switchwin_T {
                    sw_curwin: ::core::ptr::null_mut::<win_T>(),
                    sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                    sw_same_win: false,
                    sw_visual_active: false,
                };
                if need_switch {
                    let result: ::core::ffi::c_int =
                        switch_win(&raw mut switchwin, parent, parent_tp, true);
                    '_c2rust_label_0: {
                        if result == 1 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"result == OK\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/api/win_config.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                594 as ::core::ffi::c_uint,
                                b"_Bool win_config_split(win_T *, const KeyDict_win_config *, WinConfig *, Error *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
                to_split_ok = !win_split_ins(
                    0 as ::core::ffi::c_int,
                    flags,
                    win,
                    0 as ::core::ffi::c_int,
                    unflat_altfr,
                )
                .is_null();
                if !to_split_ok {
                    win_append(
                        (*win).w_prev,
                        win,
                        if win_tp == curtab.get() {
                            ::core::ptr::null_mut::<tabpage_T>()
                        } else {
                            win_tp
                        },
                    );
                }
                if need_switch {
                    restore_win(&raw mut switchwin, true);
                }
                try_leave(&raw mut tstate, err);
                if !to_split_ok {
                    if was_split {
                        winframe_restore(win, dir, unflat_altfr);
                    }
                    if !((*err).type_0 as ::core::ffi::c_int
                        != kErrorTypeNone as ::core::ffi::c_int)
                    {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Failed to move window %d into split\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            (*win).handle,
                        );
                    }
                } else {
                    if win_tp != parent_tp && (*win_tp).tp_curwin == win {
                        (*win_tp).tp_curwin = altwin_0;
                    }
                    break '_resize;
                }
            }
            if curwin_moving_tp as ::core::ffi::c_int != 0
                && win_valid(win) as ::core::ffi::c_int != 0
            {
                win_goto(win);
            }
            return false_0 != 0;
        }
    }
    if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__width
        != 0 as ::core::ffi::c_ulonglong
    {
        win_setwidth_win((*fconfig).width, win);
    }
    if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__height
        != 0 as ::core::ffi::c_ulonglong
    {
        win_setheight_win((*fconfig).height, win);
    }
    if !was_split {
        clear_float_config(fconfig, false_0 != 0);
    }
    merge_win_config(&raw mut (*win).w_config, *fconfig);
    return true_0 != 0;
}
unsafe extern "C" fn win_config_float_tp(
    mut win: *mut win_T,
    mut config: *const KeyDict_win_config,
    mut fconfig: *const WinConfig,
    mut err: *mut Error,
) -> bool {
    let mut win_tp: *mut tabpage_T = win_find_tabpage(win);
    let mut parent: *mut win_T = win;
    let mut parent_tp: *mut tabpage_T = win_tp;
    if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win
        != 0 as ::core::ffi::c_ulonglong
    {
        parent = find_window_by_handle((*fconfig).window, err);
        if parent.is_null() {
            return false_0 != 0;
        }
        parent_tp = win_find_tabpage(parent);
    }
    let mut curwin_moving_tp: bool = false_0 != 0;
    let mut altwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    '_restore_curwin: {
        if win_tp != parent_tp {
            if !win_can_move_tp(win, win_tp, err) {
                return false_0 != 0;
            }
            altwin = win_find_altwin(win, win_tp);
            '_c2rust_label: {
                if !altwin.is_null() {
                } else {
                    __assert_fail(
                        b"altwin\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/win_config.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        671 as ::core::ffi::c_uint,
                        b"_Bool win_config_float_tp(win_T *, const KeyDict_win_config *, const WinConfig *, Error *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if curwin.get() == win {
                curwin_moving_tp = true_0 != 0;
                win_goto(altwin);
                if curwin.get() == win {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Failed to switch away from window %d\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        (*win).handle,
                    );
                    return false_0 != 0;
                }
                win_tp = win_find_tabpage(win);
                parent_tp = win_find_tabpage(parent);
                if win_tp.is_null() || parent_tp.is_null() {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Target windows were closed\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_restore_curwin;
                } else if win_tp != parent_tp && !win_can_move_tp(win, win_tp, err) {
                    break '_restore_curwin;
                } else {
                    altwin = win_find_altwin(win, win_tp);
                    '_c2rust_label_0: {
                        if !altwin.is_null() {
                        } else {
                            __assert_fail(
                                b"altwin\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/api/win_config.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                696 as ::core::ffi::c_uint,
                                b"_Bool win_config_float_tp(win_T *, const KeyDict_win_config *, const WinConfig *, Error *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
            }
        }
        if !(*win).w_floating {
            if win_new_float(win, false_0 != 0, *fconfig, err).is_null() {
                break '_restore_curwin;
            } else {
                redraw_later(win, UPD_NOT_VALID as ::core::ffi::c_int);
            }
        }
        if win_tp != parent_tp {
            win_remove(
                win,
                if win_tp == curtab.get() {
                    ::core::ptr::null_mut::<tabpage_T>()
                } else {
                    win_tp
                },
            );
            let mut append_tp: *mut tabpage_T = if parent_tp == curtab.get() {
                ::core::ptr::null_mut::<tabpage_T>()
            } else {
                parent_tp
            };
            win_append(lastwin_nofloating(append_tp), win, append_tp);
            if win_tp != curtab.get() && (*win_tp).tp_curwin == win {
                (*win_tp).tp_curwin = altwin;
            }
            ui_comp_remove_grid(&raw mut (*win).w_grid_alloc);
            redraw_later(win, UPD_NOT_VALID as ::core::ffi::c_int);
            set_must_redraw(UPD_NOT_VALID as ::core::ffi::c_int);
        }
        win_config_float(win, *fconfig);
        return true_0 != 0;
    }
    if curwin_moving_tp as ::core::ffi::c_int != 0 && win_valid(win) as ::core::ffi::c_int != 0 {
        win_goto(win);
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_set_config(
    mut win: Window,
    mut config: *mut KeyDict_win_config,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    let mut was_split: bool = !(*w).w_floating;
    let mut has_split: bool = (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
        != 0 as ::core::ffi::c_ulonglong;
    let mut has_vertical: bool = (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__vertical
        != 0 as ::core::ffi::c_ulonglong;
    let mut old_style: WinStyle = (*w).w_config.style;
    let mut fconfig: WinConfig = (*w).w_config;
    let mut to_split: bool = (*config).relative.size == 0 as size_t
        && !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__external
            != 0 as ::core::ffi::c_ulonglong
            && (*config).external as ::core::ffi::c_int != 0)
        && (has_split as ::core::ffi::c_int != 0
            || has_vertical as ::core::ffi::c_int != 0
            || was_split as ::core::ffi::c_int != 0);
    if !parse_win_config(
        w,
        config,
        &raw mut fconfig,
        !was_split || to_split as ::core::ffi::c_int != 0,
        err,
    ) {
        return;
    }
    if to_split {
        if !win_config_split(w, config, &raw mut fconfig, err) {
            return;
        }
    } else if !win_config_float_tp(w, config, &raw mut fconfig, err) {
        return;
    }
    if fconfig.style as ::core::ffi::c_uint
        == kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
        && old_style as ::core::ffi::c_uint != fconfig.style as ::core::ffi::c_uint
    {
        win_set_minimal_style(w);
        didset_window_options(w, true_0 != 0);
        changed_window_setting(w);
    }
    if fconfig._cmdline_offset < INT_MAX {
        cmdline_win.set(w);
    } else if w == cmdline_win.get() && fconfig._cmdline_offset == INT_MAX {
        cmdline_win.set(::core::ptr::null_mut::<win_T>());
    }
}
unsafe extern "C" fn config_put_bordertext(
    mut config: *mut KeyDict_win_config,
    mut fconfig: *mut WinConfig,
    mut bordertext_type: BorderTextType,
    mut arena: *mut Arena,
) {
    let mut vt: VirtText = VirtText {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    };
    let mut align: AlignTextPos = kAlignLeft;
    match bordertext_type as ::core::ffi::c_uint {
        0 => {
            vt = (*fconfig).title_chunks;
            align = (*fconfig).title_pos;
        }
        1 => {
            vt = (*fconfig).footer_chunks;
            align = (*fconfig).footer_pos;
        }
        _ => {}
    }
    let mut bordertext: Array = virt_text_to_array(vt, true_0 != 0, arena);
    let mut pos: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match align as ::core::ffi::c_uint {
        0 => {
            pos = b"left\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        1 => {
            pos = b"center\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        2 => {
            pos = b"right\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        _ => {}
    }
    match bordertext_type as ::core::ffi::c_uint {
        0 => {
            (*config).is_set__win_config_ = ((*config).is_set__win_config_
                as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__title)
                as OptionalKeys;
            (*config).title = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: bordertext },
            };
            (*config).is_set__win_config_ = ((*config).is_set__win_config_
                as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__title_pos)
                as OptionalKeys;
            (*config).title_pos = cstr_as_string(pos);
        }
        1 => {
            (*config).is_set__win_config_ = ((*config).is_set__win_config_
                as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__footer)
                as OptionalKeys;
            (*config).footer = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: bordertext },
            };
            (*config).is_set__win_config_ = ((*config).is_set__win_config_
                as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__footer_pos)
                as OptionalKeys;
            (*config).footer_pos = cstr_as_string(pos);
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn nvim_win_get_config(
    mut win: Window,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> KeyDict_win_config {
    static float_relative_str: GlobalCell<[*const ::core::ffi::c_char; 6]> = GlobalCell::new([
        b"editor\0".as_ptr() as *const ::core::ffi::c_char,
        b"win\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
        b"mouse\0".as_ptr() as *const ::core::ffi::c_char,
        b"tabline\0".as_ptr() as *const ::core::ffi::c_char,
        b"laststatus\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    static win_split_str: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
        b"left\0".as_ptr() as *const ::core::ffi::c_char,
        b"right\0".as_ptr() as *const ::core::ffi::c_char,
        b"above\0".as_ptr() as *const ::core::ffi::c_char,
        b"below\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    static win_style_str: GlobalCell<[*const ::core::ffi::c_char; 2]> = GlobalCell::new([
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        b"minimal\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut rv: KeyDict_win_config = KEYDICT_INIT;
    let mut wp: *mut win_T = find_window_by_handle(win, err);
    if wp.is_null() {
        return rv;
    }
    let mut config: *mut WinConfig = &raw mut (*wp).w_config;
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__focusable)
        as OptionalKeys;
    rv.focusable = (*config).focusable as Boolean;
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__external)
        as OptionalKeys;
    rv.external = (*config).external as Boolean;
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__hide)
        as OptionalKeys;
    rv.hide = (*config).hide as Boolean;
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__mouse)
        as OptionalKeys;
    rv.mouse = (*config).mouse as Boolean;
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__style)
        as OptionalKeys;
    rv.style = cstr_as_string((*win_style_str.ptr())[(*config).style as usize]);
    if (*wp).w_floating {
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__width)
            as OptionalKeys;
        rv.width = (*config).width as Integer;
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__height)
            as OptionalKeys;
        rv.height = (*config).height as Integer;
        if !(*config).external {
            if (*config).relative as ::core::ffi::c_uint
                == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win)
                    as OptionalKeys;
                rv.win = (*config).window;
                if (*config).bufpos.lnum >= 0 as linenr_T {
                    let mut pos: Array = arena_array(arena, 2 as size_t);
                    let c2rust_fresh2 = pos.size;
                    pos.size = pos.size.wrapping_add(1);
                    *pos.items.offset(c2rust_fresh2 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*config).bufpos.lnum as Integer,
                        },
                    };
                    let c2rust_fresh3 = pos.size;
                    pos.size = pos.size.wrapping_add(1);
                    *pos.items.offset(c2rust_fresh3 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*config).bufpos.col as Integer,
                        },
                    };
                    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__bufpos)
                        as OptionalKeys;
                    rv.bufpos = pos;
                }
            }
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__anchor)
                as OptionalKeys;
            rv.anchor = cstr_as_string(
                *(&raw const float_anchor_str as *const *const ::core::ffi::c_char)
                    .offset((*config).anchor as isize),
            );
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__row)
                as OptionalKeys;
            rv.row = (*config).row as Float;
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__col)
                as OptionalKeys;
            rv.col = (*config).col as Float;
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__zindex)
                as OptionalKeys;
            rv.zindex = (*config).zindex as Integer;
        }
        if (*config).border {
            let mut border: Array = arena_array(arena, 8 as size_t);
            let mut i: size_t = 0 as size_t;
            while i < 8 as size_t {
                let mut s: String_0 = cstrn_as_string(
                    &raw mut *(&raw mut (*config).border_chars as *mut [::core::ffi::c_char; 32])
                        .offset(i as isize) as *mut ::core::ffi::c_char,
                    MAX_SCHAR_SIZE as size_t,
                );
                let mut hi_id: ::core::ffi::c_int = (*config).border_hl_ids[i as usize];
                let mut hi_name: *mut ::core::ffi::c_char = syn_id2name(hi_id);
                if *hi_name.offset(0 as ::core::ffi::c_int as isize) != 0 {
                    let mut tuple: Array = arena_array(arena, 2 as size_t);
                    let c2rust_fresh4 = tuple.size;
                    tuple.size = tuple.size.wrapping_add(1);
                    *tuple.items.offset(c2rust_fresh4 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed { string: s },
                    };
                    let c2rust_fresh5 = tuple.size;
                    tuple.size = tuple.size.wrapping_add(1);
                    *tuple.items.offset(c2rust_fresh5 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: cstr_as_string(hi_name),
                        },
                    };
                    let c2rust_fresh6 = border.size;
                    border.size = border.size.wrapping_add(1);
                    *border.items.offset(c2rust_fresh6 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed { array: tuple },
                    };
                } else {
                    let c2rust_fresh7 = border.size;
                    border.size = border.size.wrapping_add(1);
                    *border.items.offset(c2rust_fresh7 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed { string: s },
                    };
                }
                i = i.wrapping_add(1);
            }
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__border)
                as OptionalKeys;
            rv.border = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: border },
            };
            if (*config).title {
                config_put_bordertext(&raw mut rv, config, kBorderTextTitle, arena);
            }
            if (*config).footer {
                config_put_bordertext(&raw mut rv, config, kBorderTextFooter, arena);
            }
        } else {
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__border)
                as OptionalKeys;
            rv.border = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(b"none\0".as_ptr() as *const ::core::ffi::c_char),
                },
            };
        }
    } else if !(*config).external {
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__width)
            as OptionalKeys;
        rv.width = (*wp).w_width as Integer;
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__height)
            as OptionalKeys;
        rv.height = (*wp).w_height as Integer;
        let mut split: WinSplit = win_split_dir(wp);
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split)
            as OptionalKeys;
        rv.split = cstr_as_string((*win_split_str.ptr())[split as usize]);
    }
    let mut rel: *const ::core::ffi::c_char =
        if (*wp).w_floating as ::core::ffi::c_int != 0 && !(*config).external {
            (*float_relative_str.ptr())[(*config).relative as usize]
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        };
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__relative)
        as OptionalKeys;
    rv.relative = cstr_as_string(rel);
    if (*config)._cmdline_offset < INT_MAX {
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config___cmdline_offset)
            as OptionalKeys;
        rv._cmdline_offset = (*config)._cmdline_offset as Integer;
    }
    return rv;
}
unsafe extern "C" fn parse_float_anchor(mut anchor: String_0, mut out: *mut FloatAnchor) -> bool {
    if anchor.size == 0 as size_t {
        *out = 0 as ::core::ffi::c_int;
    }
    let mut str: *mut ::core::ffi::c_char = anchor.data;
    if striequal(str, b"NW\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = 0 as ::core::ffi::c_int as FloatAnchor;
    } else if striequal(str, b"NE\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatAnchorEast as ::core::ffi::c_int as FloatAnchor;
    } else if striequal(str, b"SW\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatAnchorSouth as ::core::ffi::c_int as FloatAnchor;
    } else if striequal(str, b"SE\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = (kFloatAnchorSouth as ::core::ffi::c_int | kFloatAnchorEast as ::core::ffi::c_int)
            as FloatAnchor;
    } else {
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn parse_float_relative(
    mut relative: String_0,
    mut out: *mut FloatRelative,
) -> bool {
    let mut str: *mut ::core::ffi::c_char = relative.data;
    if striequal(str, b"editor\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeEditor;
    } else if striequal(str, b"win\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeWindow;
    } else if striequal(str, b"cursor\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeCursor;
    } else if striequal(str, b"mouse\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeMouse;
    } else if striequal(str, b"tabline\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeTabline;
    } else if striequal(str, b"laststatus\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeLaststatus;
    } else {
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn parse_config_split(mut split: String_0, mut out: *mut WinSplit) -> bool {
    let mut str: *mut ::core::ffi::c_char = split.data;
    if striequal(str, b"left\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kWinSplitLeft;
    } else if striequal(str, b"right\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kWinSplitRight;
    } else if striequal(str, b"above\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kWinSplitAbove;
    } else if striequal(str, b"below\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kWinSplitBelow;
    } else {
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn parse_float_bufpos(mut bufpos: Array, mut out: *mut lpos_T) -> bool {
    if bufpos.size != 2 as size_t
        || (*bufpos.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*bufpos.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return false_0 != 0;
    }
    (*out).lnum = (*bufpos.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer as linenr_T;
    (*out).col = (*bufpos.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer as colnr_T;
    return true_0 != 0;
}
unsafe extern "C" fn parse_bordertext(
    mut bordertext: Object,
    mut bordertext_type: BorderTextType,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) {
    if bordertext.type_0 as ::core::ffi::c_uint
        != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        && bordertext.type_0 as ::core::ffi::c_uint
            != kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        api_err_exp(
            err,
            b"title/footer\0".as_ptr() as *const ::core::ffi::c_char,
            b"String or Array\0".as_ptr() as *const ::core::ffi::c_char,
            api_typename(bordertext.type_0),
        );
        return;
    }
    if bordertext.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        && bordertext.data.array.size == 0 as size_t
    {
        api_err_exp(
            err,
            b"title/footer\0".as_ptr() as *const ::core::ffi::c_char,
            b"non-empty Array\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        return;
    }
    let mut is_present: *mut bool = ::core::ptr::null_mut::<bool>();
    let mut chunks: *mut VirtText = ::core::ptr::null_mut::<VirtText>();
    let mut width: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    match bordertext_type as ::core::ffi::c_uint {
        0 => {
            is_present = &raw mut (*fconfig).title;
            chunks = &raw mut (*fconfig).title_chunks;
            width = &raw mut (*fconfig).title_width;
        }
        1 => {
            is_present = &raw mut (*fconfig).footer;
            chunks = &raw mut (*fconfig).footer_chunks;
            width = &raw mut (*fconfig).footer_width;
        }
        _ => {}
    }
    if bordertext.type_0 as ::core::ffi::c_uint
        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if bordertext.data.string.size == 0 as size_t {
            *is_present = false_0 != 0;
            return;
        }
        (*chunks).capacity = 0 as size_t;
        (*chunks).size = (*chunks).capacity;
        (*chunks).items = ::core::ptr::null_mut::<VirtTextChunk>();
        if (*chunks).size == (*chunks).capacity {
            (*chunks).capacity = if (*chunks).capacity != 0 {
                (*chunks).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*chunks).items = xrealloc(
                (*chunks).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<VirtTextChunk>().wrapping_mul((*chunks).capacity),
            ) as *mut VirtTextChunk;
        } else {
        };
        let c2rust_fresh1 = (*chunks).size;
        (*chunks).size = (*chunks).size.wrapping_add(1);
        *(*chunks).items.offset(c2rust_fresh1 as isize) = VirtTextChunk {
            text: xstrdup(bordertext.data.string.data),
            hl_id: -1 as ::core::ffi::c_int,
        };
        *width = mb_string2cells(bordertext.data.string.data) as ::core::ffi::c_int;
        *is_present = true_0 != 0;
        return;
    }
    *width = 0 as ::core::ffi::c_int;
    *chunks = parse_virt_text(bordertext.data.array, err, width);
    *is_present = true_0 != 0;
}
unsafe extern "C" fn parse_bordertext_pos(
    mut wp: *mut win_T,
    mut bordertext_pos: String_0,
    mut bordertext_type: BorderTextType,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) -> bool {
    let mut align: *mut AlignTextPos = ::core::ptr::null_mut::<AlignTextPos>();
    match bordertext_type as ::core::ffi::c_uint {
        0 => {
            align = &raw mut (*fconfig).title_pos;
        }
        1 => {
            align = &raw mut (*fconfig).footer_pos;
        }
        _ => {}
    }
    if bordertext_pos.size == 0 as size_t {
        if wp.is_null() {
            *align = kAlignLeft;
        }
        return true_0 != 0;
    }
    let mut pos: *mut ::core::ffi::c_char = bordertext_pos.data;
    if strequal(pos, b"left\0".as_ptr() as *const ::core::ffi::c_char) {
        *align = kAlignLeft;
    } else if strequal(pos, b"center\0".as_ptr() as *const ::core::ffi::c_char) {
        *align = kAlignCenter;
    } else if strequal(pos, b"right\0".as_ptr() as *const ::core::ffi::c_char) {
        *align = kAlignRight;
    } else if true {
        api_err_invalid(
            err,
            if bordertext_type as ::core::ffi::c_uint
                == kBorderTextTitle as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                b"title_pos\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"footer_pos\0".as_ptr() as *const ::core::ffi::c_char
            },
            pos,
            0 as int64_t,
            true_0 != 0,
        );
        return false;
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn parse_border_style(
    mut style: Object,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) {
    let mut defaults: [C2Rust_Unnamed_15; 7] = [
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[1 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x94\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x91\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x9D\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x9A\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x91\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: false_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[2 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x8C\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x98\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x94\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: false_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[3 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: true_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[4 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\xAD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\xAE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\xAF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\xB0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: false_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[5 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: false_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[6 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x8F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x81\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x93\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x83\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x9B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x81\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x83\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: false_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            chars: [
                [
                    NUL as ::core::ffi::c_char,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                [0; 32],
                [0; 32],
                [0; 32],
                [0; 32],
                [0; 32],
                [0; 32],
                [0; 32],
            ],
            shadow_color: false_0 != 0,
        },
    ];
    let mut chars: *mut [::core::ffi::c_char; 32] =
        &raw mut (*fconfig).border_chars as *mut [::core::ffi::c_char; 32];
    let mut hl_ids: *mut ::core::ffi::c_int =
        &raw mut (*fconfig).border_hl_ids as *mut ::core::ffi::c_int;
    (*fconfig).border = true_0 != 0;
    if style.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut arr: Array = style.data.array;
        let mut size: size_t = arr.size;
        if size == 0 || size > 8 as size_t || size & size.wrapping_sub(1 as size_t) != 0 {
            api_err_exp(
                err,
                b"border\0".as_ptr() as *const ::core::ffi::c_char,
                b"1, 2, 4, or 8 chars\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return;
        }
        let mut i: size_t = 0 as size_t;
        while i < size {
            let mut iytem: Object = *arr.items.offset(i as isize);
            let mut string: String_0 = String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            };
            let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if iytem.type_0 as ::core::ffi::c_uint
                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut iarr: Array = iytem.data.array;
                if iarr.size == 0 || iarr.size > 2 as size_t {
                    api_err_exp(
                        err,
                        b"border\0".as_ptr() as *const ::core::ffi::c_char,
                        b"1 or 2-item Array\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    return;
                }
                if !((*iarr.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                    as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    api_err_exp(
                        err,
                        b"border\0".as_ptr() as *const ::core::ffi::c_char,
                        b"Array of Strings\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    return;
                }
                string = (*iarr.items.offset(0 as ::core::ffi::c_int as isize))
                    .data
                    .string;
                if iarr.size == 2 as size_t {
                    hl_id = object_to_hl_id(
                        *iarr.items.offset(1 as ::core::ffi::c_int as isize),
                        b"border char highlight\0".as_ptr() as *const ::core::ffi::c_char,
                        err,
                    );
                    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        return;
                    }
                }
            } else if iytem.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                string = iytem.data.string;
            } else if true {
                api_err_exp(
                    err,
                    b"border\0".as_ptr() as *const ::core::ffi::c_char,
                    b"String or Array\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(iytem.type_0),
                );
                return;
            }
            if string.size != 0 && mb_string2cells_len(string.data, string.size) > 1 as size_t {
                api_err_exp(
                    err,
                    b"border\0".as_ptr() as *const ::core::ffi::c_char,
                    b"only one-cell chars\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
                return;
            }
            let mut len: size_t = if string.size
                < ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_sub(1 as usize)
            {
                string.size
            } else {
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_sub(1 as size_t)
            };
            if len != 0 {
                memcpy(
                    &raw mut *chars.offset(i as isize) as *mut ::core::ffi::c_char
                        as *mut ::core::ffi::c_void,
                    string.data as *const ::core::ffi::c_void,
                    len,
                );
            }
            (*chars.offset(i as isize))[len as usize] = NUL as ::core::ffi::c_char;
            *hl_ids.offset(i as isize) = hl_id;
            i = i.wrapping_add(1);
        }
        while size < 8 as size_t {
            memcpy(
                chars.offset(size as isize) as *mut ::core::ffi::c_void,
                chars as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_mul(size),
            );
            memcpy(
                hl_ids.offset(size as isize) as *mut ::core::ffi::c_void,
                hl_ids as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(size),
            );
            size <<= 1 as ::core::ffi::c_int;
        }
        if (*chars.offset(7 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            != 0
            && (*chars.offset(1 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                != 0
            && (*chars.offset(0 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                == 0
            || (*chars.offset(1 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                != 0
                && (*chars.offset(3 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    != 0
                && (*chars.offset(2 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize]
                    == 0
            || (*chars.offset(3 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                != 0
                && (*chars.offset(5 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    != 0
                && (*chars.offset(4 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize]
                    == 0
            || (*chars.offset(5 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                != 0
                && (*chars.offset(7 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    != 0
                && (*chars.offset(6 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize]
                    == 0
        {
            api_err_exp(
                err,
                b"border\0".as_ptr() as *const ::core::ffi::c_char,
                b"corner char between edge chars\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return;
        }
    } else if style.type_0 as ::core::ffi::c_uint
        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut str: String_0 = style.data.string;
        if str.size == 0 as size_t
            || strequal(str.data, b"none\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
        {
            (*fconfig).border = false_0 != 0;
            (*fconfig).title = false_0 != 0;
            (*fconfig).footer = false_0 != 0;
            return;
        }
        let mut i_0: size_t = 0 as size_t;
        while !defaults[i_0 as usize].name.is_null() {
            if strequal(str.data, defaults[i_0 as usize].name) {
                memcpy(
                    chars as *mut ::core::ffi::c_void,
                    &raw mut (*(&raw mut defaults as *mut C2Rust_Unnamed_15).offset(i_0 as isize))
                        .chars as *mut [::core::ffi::c_char; 32]
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[[::core::ffi::c_char; 32]; 8]>(),
                );
                memset(
                    hl_ids as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    (8 as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
                );
                if defaults[i_0 as usize].shadow_color {
                    let mut hl_blend: ::core::ffi::c_int = syn_check_group(
                        b"FloatShadow\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 12]>()
                            .wrapping_sub(1 as size_t),
                    );
                    let mut hl_through: ::core::ffi::c_int = syn_check_group(
                        b"FloatShadowThrough\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 19]>()
                            .wrapping_sub(1 as size_t),
                    );
                    *hl_ids.offset(2 as ::core::ffi::c_int as isize) = hl_through;
                    *hl_ids.offset(3 as ::core::ffi::c_int as isize) = hl_blend;
                    *hl_ids.offset(4 as ::core::ffi::c_int as isize) = hl_blend;
                    *hl_ids.offset(5 as ::core::ffi::c_int as isize) = hl_blend;
                    *hl_ids.offset(6 as ::core::ffi::c_int as isize) = hl_through;
                }
                return;
            }
            i_0 = i_0.wrapping_add(1);
        }
        if true {
            api_err_invalid(
                err,
                b"border\0".as_ptr() as *const ::core::ffi::c_char,
                str.data,
                0 as int64_t,
                true_0 != 0,
            );
            return;
        }
    }
}
unsafe extern "C" fn generate_api_error(
    mut wp: *mut win_T,
    mut attribute: *const ::core::ffi::c_char,
    mut err: *mut Error,
) {
    if !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Required: 'relative' when reconfiguring floating window %d\0".as_ptr()
                as *const ::core::ffi::c_char,
            (*wp).handle,
        );
    } else if true {
        api_err_conflict(
            err,
            attribute,
            b"non-float window\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn parse_winborder(
    mut fconfig: *mut WinConfig,
    mut border_opt: *mut ::core::ffi::c_char,
    mut err: *mut Error,
) -> bool {
    if fconfig.is_null() {
        return false_0 != 0;
    }
    let mut style: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if !strchr(border_opt, ',' as ::core::ffi::c_int).is_null() {
        let mut border_chars: Array = ARRAY_DICT_INIT;
        let mut p: *mut ::core::ffi::c_char = border_opt;
        let mut part: [::core::ffi::c_char; 32] = [
            0 as ::core::ffi::c_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while *p as ::core::ffi::c_int != NUL {
            if count >= 8 as ::core::ffi::c_int {
                api_free_array(border_chars);
                return false_0 != 0;
            }
            let mut part_len: size_t = copy_option_part(
                &raw mut p,
                &raw mut part as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if part_len == 0 as size_t
                || part[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL
            {
                api_free_array(border_chars);
                return false_0 != 0;
            }
            let mut str: String_0 = cstr_to_string(&raw mut part as *mut ::core::ffi::c_char);
            if border_chars.size == border_chars.capacity {
                border_chars.capacity = if border_chars.capacity != 0 {
                    border_chars.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                border_chars.items = xrealloc(
                    border_chars.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<Object>().wrapping_mul(border_chars.capacity),
                ) as *mut Object;
            } else {
            };
            let c2rust_fresh0 = border_chars.size;
            border_chars.size = border_chars.size.wrapping_add(1);
            *border_chars.items.offset(c2rust_fresh0 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed { string: str },
            };
            count += 1;
        }
        if count != 8 as ::core::ffi::c_int {
            api_free_array(border_chars);
            return false_0 != 0;
        }
        style = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed {
                array: border_chars,
            },
        };
    } else {
        style = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_to_string(border_opt),
            },
        };
    }
    parse_border_style(style, fconfig, err);
    api_free_object(style);
    return !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int);
}
unsafe extern "C" fn parse_win_config(
    mut wp: *mut win_T,
    mut config: *mut KeyDict_win_config,
    mut fconfig: *mut WinConfig,
    mut reconf: bool,
    mut err: *mut Error,
) -> bool {
    let mut border_style: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut has_relative: bool = false_0 != 0;
    let mut relative_is_win: bool = false_0 != 0;
    let mut is_split: bool = false_0 != 0;
    '_fail: {
        if (*config).relative.size > 0 as size_t {
            if !parse_float_relative((*config).relative, &raw mut (*fconfig).relative) {
                api_err_invalid(
                    err,
                    b"relative\0".as_ptr() as *const ::core::ffi::c_char,
                    (*config).relative.data,
                    0 as int64_t,
                    true_0 != 0,
                );
                break '_fail;
            } else if (*config).relative.size > 0 as size_t
                && !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong
                    && (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong)
                && !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 12 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong)
            {
                api_err_required(
                    err,
                    b"'relative' requires 'row'/'col' or 'bufpos'\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                break '_fail;
            } else {
                has_relative = true_0 != 0;
                (*fconfig).external = false_0 != 0;
                if (*fconfig).relative as ::core::ffi::c_uint
                    == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    relative_is_win = true_0 != 0;
                    (*fconfig).bufpos.lnum = -1 as ::core::ffi::c_int as linenr_T;
                }
            }
        } else if !(*config).external {
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__vertical
                != 0 as ::core::ffi::c_ulonglong
                || (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
                    != 0 as ::core::ffi::c_ulonglong
            {
                is_split = true_0 != 0;
                (*fconfig).external = false_0 != 0;
            } else if wp.is_null() {
                if true {
                    api_err_required(
                        err,
                        b"'relative' or 'external' when creating a float\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                }
            }
        }
        if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 19 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong
            && !is_split
        {
            api_err_conflict(
                err,
                b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
                b"floating windows\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong
            && !is_split
        {
            api_err_conflict(
                err,
                b"split\0".as_ptr() as *const ::core::ffi::c_char,
                b"floating windows\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
                != 0 as ::core::ffi::c_ulonglong
            {
                if !is_split {
                    api_err_conflict(
                        err,
                        b"split\0".as_ptr() as *const ::core::ffi::c_char,
                        b"floating windows\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                } else if !parse_config_split((*config).split, &raw mut (*fconfig).split) {
                    api_err_invalid(
                        err,
                        b"split\0".as_ptr() as *const ::core::ffi::c_char,
                        (*config).split.data,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_fail;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__anchor
                != 0 as ::core::ffi::c_ulonglong
            {
                if !parse_float_anchor((*config).anchor, &raw mut (*fconfig).anchor) {
                    api_err_invalid(
                        err,
                        b"anchor\0".as_ptr() as *const ::core::ffi::c_char,
                        (*config).anchor.data,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_fail;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__row
                != 0 as ::core::ffi::c_ulonglong
            {
                if !has_relative || is_split as ::core::ffi::c_int != 0 {
                    generate_api_error(wp, b"row\0".as_ptr() as *const ::core::ffi::c_char, err);
                    break '_fail;
                } else {
                    (*fconfig).row = (*config).row as ::core::ffi::c_double;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__col
                != 0 as ::core::ffi::c_ulonglong
            {
                if !has_relative || is_split as ::core::ffi::c_int != 0 {
                    generate_api_error(wp, b"col\0".as_ptr() as *const ::core::ffi::c_char, err);
                    break '_fail;
                } else {
                    (*fconfig).col = (*config).col as ::core::ffi::c_double;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__bufpos
                != 0 as ::core::ffi::c_ulonglong
            {
                if !has_relative || is_split as ::core::ffi::c_int != 0 {
                    generate_api_error(wp, b"bufpos\0".as_ptr() as *const ::core::ffi::c_char, err);
                    break '_fail;
                } else if !parse_float_bufpos((*config).bufpos, &raw mut (*fconfig).bufpos) {
                    api_err_exp(
                        err,
                        b"bufpos\0".as_ptr() as *const ::core::ffi::c_char,
                        b"[row, col] array\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_fail;
                } else {
                    if !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__row
                        != 0 as ::core::ffi::c_ulonglong)
                    {
                        (*fconfig).row = (if (*fconfig).anchor as ::core::ffi::c_int
                            & kFloatAnchorSouth as ::core::ffi::c_int
                            != 0
                        {
                            0 as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                        }) as ::core::ffi::c_double;
                    }
                    if !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__col
                        != 0 as ::core::ffi::c_ulonglong)
                    {
                        (*fconfig).col = 0 as ::core::ffi::c_int as ::core::ffi::c_double;
                    }
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__width
                != 0 as ::core::ffi::c_ulonglong
            {
                if !((*config).width > 0 as Integer) {
                    api_err_exp(
                        err,
                        b"width\0".as_ptr() as *const ::core::ffi::c_char,
                        b"positive Integer\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_fail;
                } else {
                    (*fconfig).width = (*config).width as ::core::ffi::c_int;
                }
            } else if !reconf && !is_split {
                if true {
                    api_err_required(err, b"width\0".as_ptr() as *const ::core::ffi::c_char);
                    break '_fail;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__height
                != 0 as ::core::ffi::c_ulonglong
            {
                if !((*config).height > 0 as Integer) {
                    api_err_exp(
                        err,
                        b"height\0".as_ptr() as *const ::core::ffi::c_char,
                        b"positive Integer\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_fail;
                } else {
                    (*fconfig).height = (*config).height as ::core::ffi::c_int;
                }
            } else if !reconf && !is_split {
                if true {
                    api_err_required(err, b"height\0".as_ptr() as *const ::core::ffi::c_char);
                    break '_fail;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__external
                != 0 as ::core::ffi::c_ulonglong
            {
                (*fconfig).external = (*config).external as bool;
                if has_relative as ::core::ffi::c_int != 0
                    && (*fconfig).external as ::core::ffi::c_int != 0
                {
                    api_err_conflict(
                        err,
                        b"relative\0".as_ptr() as *const ::core::ffi::c_char,
                        b"external\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                } else if (*fconfig).external as ::core::ffi::c_int != 0 && !ui_has(kUIMultigrid) {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"UI doesn't support external windows\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong
                && (*fconfig).external as ::core::ffi::c_int != 0
            {
                api_err_conflict(
                    err,
                    b"win\0".as_ptr() as *const ::core::ffi::c_char,
                    b"external window\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                if relative_is_win as ::core::ffi::c_int != 0
                    || (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win
                        != 0 as ::core::ffi::c_ulonglong
                        && !is_split
                        && !wp.is_null()
                        && (*wp).w_floating as ::core::ffi::c_int != 0
                        && (*fconfig).relative as ::core::ffi::c_uint
                            == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut target_win: *mut win_T = find_window_by_handle((*config).win, err);
                    if target_win.is_null() {
                        break '_fail;
                    } else if target_win == wp {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"floating window cannot be relative to itself\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else {
                        (*fconfig).window = (*target_win).handle as Window;
                    }
                } else {
                    if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if !is_split && !has_relative && (wp.is_null() || !(*wp).w_floating) {
                            api_err_required(
                                err,
                                b"non-float with 'win' requires 'split' or 'vertical'\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_fail;
                        } else {
                            (*fconfig).window = (*config).win;
                        }
                    }
                    if (*fconfig).window == 0 as ::core::ffi::c_int {
                        (*fconfig).window = (*curwin.get()).handle as Window;
                    }
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__focusable
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*fconfig).focusable = (*config).focusable as bool;
                    (*fconfig).mouse = (*config).focusable as bool;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__mouse
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*fconfig).mouse = (*config).mouse as bool;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__zindex
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if is_split {
                        api_err_conflict(
                            err,
                            b"zindex\0".as_ptr() as *const ::core::ffi::c_char,
                            b"non-float window\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else if !((*config).zindex > 0 as Integer) {
                        api_err_exp(
                            err,
                            b"zindex\0".as_ptr() as *const ::core::ffi::c_char,
                            b"positive Integer\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        );
                        break '_fail;
                    } else {
                        (*fconfig).zindex = (*config).zindex as ::core::ffi::c_int;
                    }
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__title
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if is_split {
                        api_err_conflict(
                            err,
                            b"title\0".as_ptr() as *const ::core::ffi::c_char,
                            b"non-float window\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else {
                        parse_bordertext((*config).title, kBorderTextTitle, fconfig, err);
                        if (*err).type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break '_fail;
                        } else if !parse_bordertext_pos(
                            wp,
                            (*config).title_pos,
                            kBorderTextTitle,
                            fconfig,
                            err,
                        ) {
                            break '_fail;
                        }
                    }
                } else if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 22 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong
                {
                    api_err_required(
                        err,
                        b"'title' requires 'title_pos'\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__footer
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if is_split {
                        api_err_conflict(
                            err,
                            b"footer\0".as_ptr() as *const ::core::ffi::c_char,
                            b"non-float window\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else {
                        parse_bordertext((*config).footer, kBorderTextFooter, fconfig, err);
                        if (*err).type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break '_fail;
                        } else if !parse_bordertext_pos(
                            wp,
                            (*config).footer_pos,
                            kBorderTextFooter,
                            fconfig,
                            err,
                        ) {
                            break '_fail;
                        }
                    }
                } else if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 23 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong
                {
                    api_err_required(
                        err,
                        b"'footer' requires 'footer_pos'\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                }
                border_style = object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                };
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__border
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if is_split {
                        api_err_conflict(
                            err,
                            b"border\0".as_ptr() as *const ::core::ffi::c_char,
                            b"non-float window\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else {
                        border_style = (*config).border;
                        if border_style.type_0 as ::core::ffi::c_uint
                            != kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            parse_border_style(border_style, fconfig, err);
                            if (*err).type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                break '_fail;
                            }
                        }
                    }
                } else if *p_winborder.get() as ::core::ffi::c_int != NUL
                    && (wp.is_null() || !(*wp).w_floating)
                    && !parse_winborder(fconfig, p_winborder.get(), err)
                {
                    break '_fail;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__style
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if *(*config)
                        .style
                        .data
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == NUL
                    {
                        (*fconfig).style = kWinStyleUnused;
                    } else if striequal(
                        (*config).style.data,
                        b"minimal\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        (*fconfig).style = kWinStyleMinimal;
                    } else if true {
                        api_err_invalid(
                            err,
                            b"style\0".as_ptr() as *const ::core::ffi::c_char,
                            (*config).style.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_fail;
                    }
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__noautocmd
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if !wp.is_null()
                        && (*config).noautocmd as ::core::ffi::c_int
                            != (*fconfig).noautocmd as ::core::ffi::c_int
                    {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"'noautocmd' cannot be changed on existing window\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else {
                        (*fconfig).noautocmd = (*config).noautocmd as bool;
                    }
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__fixed
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*fconfig).fixed = (*config).fixed as bool;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__hide
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*fconfig).hide = (*config).hide as bool;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config___cmdline_offset
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*fconfig)._cmdline_offset = (*config)._cmdline_offset as ::core::ffi::c_int;
                }
                return true_0 != 0;
            }
        }
    }
    merge_win_config(
        fconfig,
        if !wp.is_null() {
            (*wp).w_config
        } else {
            WinConfig {
                window: 0,
                bufpos: lpos_T {
                    lnum: -1 as linenr_T,
                    col: 0 as colnr_T,
                },
                height: 0 as ::core::ffi::c_int,
                width: 0 as ::core::ffi::c_int,
                row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                anchor: 0 as FloatAnchor,
                relative: kFloatRelativeEditor,
                external: false_0 != 0,
                focusable: true_0 != 0,
                mouse: true_0 != 0,
                split: kWinSplitLeft,
                zindex: kZIndexFloatDefault as ::core::ffi::c_int,
                style: kWinStyleUnused,
                border: false,
                shadow: false,
                border_chars: [[0; 32]; 8],
                border_hl_ids: [0; 8],
                border_attr: [0; 8],
                title: false,
                title_pos: kAlignLeft,
                title_chunks: VirtText {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                },
                title_width: 0,
                footer: false,
                footer_pos: kAlignLeft,
                footer_chunks: VirtText {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                },
                footer_width: 0,
                noautocmd: false_0 != 0,
                fixed: false_0 != 0,
                hide: false_0 != 0,
                _cmdline_offset: INT_MAX,
            }
        },
    );
    return false_0 != 0;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
