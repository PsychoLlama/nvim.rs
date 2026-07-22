use crate::src::nvim::api::private::helpers::{
    api_set_error, api_set_sctx, api_typename, arena_array, arena_dict, arena_string,
    arena_take_arraybuilder, cstr_as_string, find_buffer_by_handle, string_to_cstr, try_enter,
    try_leave,
};
use crate::src::nvim::api::private::validate::{
    api_err_conflict, api_err_exp, api_err_invalid, api_err_required, check_string_array,
};
use crate::src::nvim::autocmd::{
    apply_autocmds_group, au_get_autocmds_for_event, aucmd_del_for_event_and_group,
    aucmd_span_pattern, augroup_add, augroup_del, augroup_exists, augroup_find, augroup_name,
    aupat_get_buflocal_nr, aupat_is_buflocal, aupat_normalize_buflocal_pat, autocmd_delete_id,
    autocmd_register, do_autocmd_event, event_name2nr_str, event_nr2name,
};
use crate::src::nvim::buffer::do_modelines;
use crate::src::nvim::eval::typval::{callback_free, callback_to_string};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::lua::executor::{api_new_luaref, nlua_ref_is_function};
use crate::src::nvim::main::{curbuf, current_sctx};
use crate::src::nvim::memory::{strequal, xfree, xmalloc, xrealloc};
use crate::src::nvim::os::libc::{__assert_fail, abort, memcpy, strlen};
use crate::src::nvim::strings::arena_printf;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Arena, Array, ArrayBuilder, AutoCmd, AutoCmdVec, AutoPat,
    BoolVarValue, Boolean, BufUpdateCallbacks, Buffer, CMD_index, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict,
    Error, ErrorType, ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridView,
    Integer, Intersection, KeyDict_clear_autocmds, KeyDict_create_augroup, KeyDict_create_autocmd,
    KeyDict_exec_autocmds, KeyDict_get_autocmds, KeyValuePair, LineGetter, LuaRef, MTKey, MTNode,
    MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, Object, ObjectType, OptInt, OptionalKeys, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0, Terminal,
    Timestamp, TryState, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos,
    WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, auto_event, bhdr_T, blob_T,
    blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_13, dict_T, dictvar_S, disptick_T, eslist_T, eslist_elem,
    event_T, exarg, exarg_T, except_T, except_type_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    msglist, msglist_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, taggy_T, terminal,
    time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, vim_exception, virt_line,
    visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
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
pub const AUGROUP_DEFAULT: C2Rust_Unnamed_14 = -1;
pub const AUGROUP_ERROR: C2Rust_Unnamed_14 = -2;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_UNSIGNED: cmd_addr_T = 9;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_QUICKFIX_VALID: cmd_addr_T = 7;
pub const ADDR_TABS_RELATIVE: cmd_addr_T = 6;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
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
pub const AUGROUP_ALL: C2Rust_Unnamed_14 = -3;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const AUGROUP_DELETED: C2Rust_Unnamed_14 = -4;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return dest;
}
pub const KEYSET_OPTIDX_clear_autocmds__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_clear_autocmds__buffer: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__desc: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__buffer: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__command: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__callback: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__data: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__buffer: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__modeline: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__buf: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__event: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__buffer: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__pattern: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_augroup__clear: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static next_autocmd_id: GlobalCell<int64_t> = GlobalCell::new(1 as int64_t);
pub unsafe extern "C" fn nvim_get_autocmds(
    mut opts: *mut KeyDict_get_autocmds,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = 0;
    let mut has_buf: bool = false;
    let mut buf: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut pattern_filter_count: ::core::ffi::c_int = 0;
    let mut autocmd_list: ArrayBuilder = ArrayBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
        init_array: [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 16],
    };
    autocmd_list.capacity = ::core::mem::size_of::<[Object; 16]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 16]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    autocmd_list.size = 0 as size_t;
    autocmd_list.items = &raw mut autocmd_list.init_array as *mut Object;
    let mut pattern_filters: [*mut ::core::ffi::c_char; 256] =
        [::core::ptr::null_mut::<::core::ffi::c_char>(); 256];
    let mut buffers: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut event_set: [bool; 145] = [
        false_0 != 0,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
    ];
    let mut check_event: bool = false_0 != 0;
    let mut group: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_cleanup: {
        match (*opts).group.type_0 as ::core::ffi::c_uint {
            0 => {}
            4 => {
                group = augroup_find((*opts).group.data.string.data);
                if !(group >= 0 as ::core::ffi::c_int) {
                    api_err_invalid(
                        err,
                        b"group\0".as_ptr() as *const ::core::ffi::c_char,
                        (*opts).group.data.string.data,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_cleanup;
                }
            }
            2 => {
                group = (*opts).group.data.integer as ::core::ffi::c_int;
                name = if group == 0 as ::core::ffi::c_int {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    augroup_name(group)
                };
                if !augroup_exists(name) {
                    api_err_invalid(
                        err,
                        b"group\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        (*opts).group.data.integer as int64_t,
                        false_0 != 0,
                    );
                    break '_cleanup;
                }
            }
            _ => {
                if true {
                    api_err_exp(
                        err,
                        b"group\0".as_ptr() as *const ::core::ffi::c_char,
                        b"String or Integer\0".as_ptr() as *const ::core::ffi::c_char,
                        api_typename((*opts).group.type_0),
                    );
                    break '_cleanup;
                }
            }
        }
        id = if (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__id
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).id as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
        's_299: {
            if (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__event
                != 0 as ::core::ffi::c_ulonglong
            {
                check_event = true_0 != 0;
                let mut v: Object = (*opts).event;
                if v.type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut event_nr: event_T = event_name2nr_str(v.data.string);
                    if !((event_nr as ::core::ffi::c_uint)
                        < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        api_err_invalid(
                            err,
                            b"event\0".as_ptr() as *const ::core::ffi::c_char,
                            v.data.string.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_cleanup;
                    } else {
                        event_set[event_nr as usize] = true_0 != 0;
                    }
                } else if v.type_0 as ::core::ffi::c_uint
                    == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut event_v_index: size_t = 0 as size_t;
                    loop {
                        if event_v_index >= v.data.array.size {
                            break 's_299;
                        }
                        let mut event_v: Object =
                            *v.data.array.items.offset(event_v_index as isize);
                        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                            != event_v.type_0 as ::core::ffi::c_uint
                        {
                            api_err_exp(
                                err,
                                b"event item\0".as_ptr() as *const ::core::ffi::c_char,
                                api_typename(kObjectTypeString),
                                api_typename(event_v.type_0),
                            );
                            break '_cleanup;
                        } else {
                            let mut event_nr_0: event_T = event_name2nr_str(event_v.data.string);
                            if !((event_nr_0 as ::core::ffi::c_uint)
                                < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                            {
                                api_err_invalid(
                                    err,
                                    b"event\0".as_ptr() as *const ::core::ffi::c_char,
                                    event_v.data.string.data,
                                    0 as int64_t,
                                    true,
                                );
                                break '_cleanup;
                            } else {
                                event_set[event_nr_0 as usize] = true;
                                event_v_index = event_v_index.wrapping_add(1);
                            }
                        }
                    }
                } else if true {
                    api_err_exp(
                        err,
                        b"event\0".as_ptr() as *const ::core::ffi::c_char,
                        b"String or Array\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_cleanup;
                }
            }
        }
        has_buf = (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__buf
            != 0 as ::core::ffi::c_ulonglong
            || (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__buffer
                != 0 as ::core::ffi::c_ulonglong;
        buf = if (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__buf
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).buf
        } else {
            (*opts).buffer
        };
        if !(!((*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
            || !((*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong))
        {
            api_err_conflict(
                err,
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if !(!((*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
            || !has_buf)
        {
            api_err_conflict(
                err,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            pattern_filter_count = 0 as ::core::ffi::c_int;
            's_506: {
                if (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__pattern
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut v_0: Object = (*opts).pattern;
                    if v_0.type_0 as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        pattern_filters[pattern_filter_count as usize] = v_0.data.string.data;
                        pattern_filter_count += 1 as ::core::ffi::c_int;
                    } else if v_0.type_0 as ::core::ffi::c_uint
                        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if !(v_0.data.array.size <= 256 as size_t) {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Too many patterns (maximum of %d)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                256 as ::core::ffi::c_int,
                            );
                            break '_cleanup;
                        } else {
                            let mut item_index: size_t = 0 as size_t;
                            loop {
                                if item_index >= v_0.data.array.size {
                                    break 's_506;
                                }
                                let mut item: Object =
                                    *v_0.data.array.items.offset(item_index as isize);
                                if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                                    != item.type_0 as ::core::ffi::c_uint
                                {
                                    api_err_exp(
                                        err,
                                        b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                                        api_typename(kObjectTypeString),
                                        api_typename(item.type_0),
                                    );
                                    break '_cleanup;
                                } else {
                                    pattern_filters[pattern_filter_count as usize] =
                                        item.data.string.data;
                                    pattern_filter_count += 1 as ::core::ffi::c_int;
                                    item_index = item_index.wrapping_add(1);
                                }
                            }
                        }
                    } else if true {
                        api_err_exp(
                            err,
                            b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                            b"String or Array\0".as_ptr() as *const ::core::ffi::c_char,
                            api_typename(v_0.type_0),
                        );
                        break '_cleanup;
                    }
                }
            }
            's_659: {
                if buf.type_0 as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                    || buf.type_0 as ::core::ffi::c_uint
                        == kObjectTypeBuffer as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut b: *mut buf_T = find_buffer_by_handle(buf.data.integer as Buffer, err);
                    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        break '_cleanup;
                    } else {
                        let mut pat: String_0 = arena_printf(
                            arena,
                            b"<buffer=%d>\0".as_ptr() as *const ::core::ffi::c_char,
                            (*b).handle,
                        );
                        buffers = arena_array(arena, 1 as size_t);
                        let c2rust_fresh0 = buffers.size;
                        buffers.size = buffers.size.wrapping_add(1);
                        *buffers.items.offset(c2rust_fresh0 as isize) = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed { string: pat },
                        };
                    }
                } else if buf.type_0 as ::core::ffi::c_uint
                    == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if !(buf.data.array.size <= 256 as size_t) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Too many buffers (maximum of %d)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            256 as ::core::ffi::c_int,
                        );
                        break '_cleanup;
                    } else {
                        buffers = arena_array(arena, buf.data.array.size);
                        let mut bufnr_index: size_t = 0 as size_t;
                        loop {
                            if bufnr_index >= buf.data.array.size {
                                break 's_659;
                            }
                            let mut bufnr: Object =
                                *buf.data.array.items.offset(bufnr_index as isize);
                            if !(bufnr.type_0 as ::core::ffi::c_uint
                                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                                || bufnr.type_0 as ::core::ffi::c_uint
                                    == kObjectTypeBuffer as ::core::ffi::c_int
                                        as ::core::ffi::c_uint)
                            {
                                api_err_exp(
                                    err,
                                    b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Integer\0".as_ptr() as *const ::core::ffi::c_char,
                                    api_typename(bufnr.type_0),
                                );
                                break '_cleanup;
                            } else {
                                let mut b_0: *mut buf_T =
                                    find_buffer_by_handle(bufnr.data.integer as Buffer, err);
                                if (*err).type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    break '_cleanup;
                                }
                                let c2rust_fresh1 = buffers.size;
                                buffers.size = buffers.size.wrapping_add(1);
                                *buffers.items.offset(c2rust_fresh1 as isize) = object {
                                    type_0: kObjectTypeString,
                                    data: C2Rust_Unnamed {
                                        string: arena_printf(
                                            arena,
                                            b"<buffer=%d>\0".as_ptr() as *const ::core::ffi::c_char,
                                            (*b_0).handle,
                                        ),
                                    },
                                };
                                bufnr_index = bufnr_index.wrapping_add(1);
                            }
                        }
                    }
                } else if has_buf {
                    if true {
                        api_err_exp(
                            err,
                            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
                            b"Integer or Array\0".as_ptr() as *const ::core::ffi::c_char,
                            api_typename(buf.type_0),
                        );
                        break '_cleanup;
                    }
                }
            }
            let mut bufnr_index_0: size_t = 0 as size_t;
            while bufnr_index_0 < buffers.size {
                let mut bufnr_0: Object = *buffers.items.offset(bufnr_index_0 as isize);
                pattern_filters[pattern_filter_count as usize] = bufnr_0.data.string.data;
                pattern_filter_count += 1 as ::core::ffi::c_int;
                bufnr_index_0 = bufnr_index_0.wrapping_add(1);
            }
            let mut event: event_T = EVENT_BUFADD;
            while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
                if !(check_event as ::core::ffi::c_int != 0 && !event_set[event as usize]) {
                    let mut acs: *mut AutoCmdVec = au_get_autocmds_for_event(event);
                    let mut i: size_t = 0 as size_t;
                    while i < (*acs).size {
                        let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
                        let ap: *mut AutoPat = (*ac).pat;
                        's_712: {
                            if !ap.is_null() {
                                if !(id != -1 as ::core::ffi::c_int && (*ac).id != id as int64_t) {
                                    if !(group != 0 as ::core::ffi::c_int && (*ap).group != group) {
                                        if pattern_filter_count > 0 as ::core::ffi::c_int {
                                            let mut passed: bool = false_0 != 0;
                                            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                            while j < pattern_filter_count {
                                                '_c2rust_label: {
                                                    if j < 256 as ::core::ffi::c_int {
                                                    } else {
                                                        __assert_fail(
                                                            b"j < AUCMD_MAX_PATTERNS\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                            b"src/nvim/api/autocmd.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            256 as ::core::ffi::c_uint,
                                                            b"Array nvim_get_autocmds(KeyDict_get_autocmds *, Arena *, Error *)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                                '_c2rust_label_0: {
                                                    if !pattern_filters[j as usize].is_null() {
                                                    } else {
                                                        __assert_fail(
                                                            b"pattern_filters[j]\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                            b"src/nvim/api/autocmd.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            257 as ::core::ffi::c_uint,
                                                            b"Array nvim_get_autocmds(KeyDict_get_autocmds *, Arena *, Error *)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                                let mut pat_0: *mut ::core::ffi::c_char =
                                                    pattern_filters[j as usize];
                                                let mut patlen: ::core::ffi::c_int =
                                                    strlen(pat_0) as ::core::ffi::c_int;
                                                let mut pattern_buflocal: [::core::ffi::c_char;
                                                    25] = [0; 25];
                                                if aupat_is_buflocal(pat_0, patlen) {
                                                    aupat_normalize_buflocal_pat(
                                                        &raw mut pattern_buflocal
                                                            as *mut ::core::ffi::c_char,
                                                        pat_0,
                                                        patlen,
                                                        aupat_get_buflocal_nr(pat_0, patlen),
                                                    );
                                                    pat_0 = &raw mut pattern_buflocal
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                if strequal((*ap).pat, pat_0) {
                                                    passed = true_0 != 0;
                                                    break;
                                                } else {
                                                    j += 1;
                                                }
                                            }
                                            if !passed {
                                                break 's_712;
                                            }
                                        }
                                        let mut autocmd_info: Dict =
                                            arena_dict(arena, 12 as size_t);
                                        if (*ap).group != AUGROUP_DEFAULT as ::core::ffi::c_int {
                                            let c2rust_fresh2 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh2 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"group\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeInteger,
                                                        data: C2Rust_Unnamed {
                                                            integer: (*ap).group as Integer,
                                                        },
                                                    },
                                                };
                                            let c2rust_fresh3 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh3 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"group_name\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeString,
                                                        data: C2Rust_Unnamed {
                                                            string: cstr_as_string(augroup_name(
                                                                (*ap).group,
                                                            )),
                                                        },
                                                    },
                                                };
                                        }
                                        if (*ac).id > 0 as int64_t {
                                            let c2rust_fresh4 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh4 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"id\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeInteger,
                                                        data: C2Rust_Unnamed { integer: (*ac).id },
                                                    },
                                                };
                                        }
                                        if !(*ac).desc.is_null() {
                                            let c2rust_fresh5 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh5 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"desc\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeString,
                                                        data: C2Rust_Unnamed {
                                                            string: cstr_as_string((*ac).desc),
                                                        },
                                                    },
                                                };
                                        }
                                        if !(*ac).handler_cmd.is_null() {
                                            let c2rust_fresh6 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh6 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"command\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeString,
                                                        data: C2Rust_Unnamed {
                                                            string: cstr_as_string(
                                                                (*ac).handler_cmd,
                                                            ),
                                                        },
                                                    },
                                                };
                                        } else {
                                            let c2rust_fresh7 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh7 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"command\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeString,
                                                        data: C2Rust_Unnamed {
                                                            string: String_0 {
                                                                data: ::core::ptr::null_mut::<
                                                                    ::core::ffi::c_char,
                                                                >(
                                                                ),
                                                                size: 0 as size_t,
                                                            },
                                                        },
                                                    },
                                                };
                                            let mut cb: *mut Callback = &raw mut (*ac).handler_fn;
                                            match (*cb).type_0 as ::core::ffi::c_uint {
                                                3 => {
                                                    if nlua_ref_is_function((*cb).data.luaref) {
                                                        let c2rust_fresh8 = autocmd_info.size;
                                                        autocmd_info.size =
                                                            autocmd_info.size.wrapping_add(1);
                                                        *autocmd_info.items.offset(c2rust_fresh8 as isize) = key_value_pair {
                                                            key: cstr_as_string(
                                                                b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                                            ),
                                                            value: object {
                                                                type_0: kObjectTypeLuaRef,
                                                                data: C2Rust_Unnamed {
                                                                    luaref: api_new_luaref((*cb).data.luaref),
                                                                },
                                                            },
                                                        };
                                                    }
                                                }
                                                1 | 2 => {
                                                    let c2rust_fresh9 = autocmd_info.size;
                                                    autocmd_info.size =
                                                        autocmd_info.size.wrapping_add(1);
                                                    *autocmd_info
                                                        .items
                                                        .offset(c2rust_fresh9 as isize) =
                                                        key_value_pair {
                                                            key: cstr_as_string(
                                                                b"callback\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                            ),
                                                            value: object {
                                                                type_0: kObjectTypeString,
                                                                data: C2Rust_Unnamed {
                                                                    string: cstr_as_string(
                                                                        callback_to_string(
                                                                            cb, arena,
                                                                        ),
                                                                    ),
                                                                },
                                                            },
                                                        };
                                                }
                                                0 => {
                                                    abort();
                                                }
                                                _ => {}
                                            }
                                        }
                                        let c2rust_fresh10 = autocmd_info.size;
                                        autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                        *autocmd_info.items.offset(c2rust_fresh10 as isize) =
                                            key_value_pair {
                                                key: cstr_as_string(b"pattern\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                value: object {
                                                    type_0: kObjectTypeString,
                                                    data: C2Rust_Unnamed {
                                                        string: cstr_as_string((*ap).pat),
                                                    },
                                                },
                                            };
                                        let c2rust_fresh11 = autocmd_info.size;
                                        autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                        *autocmd_info.items.offset(c2rust_fresh11 as isize) =
                                            key_value_pair {
                                                key: cstr_as_string(b"event\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                value: object {
                                                    type_0: kObjectTypeString,
                                                    data: C2Rust_Unnamed {
                                                        string: cstr_as_string(event_nr2name(
                                                            event,
                                                        )),
                                                    },
                                                },
                                            };
                                        let c2rust_fresh12 = autocmd_info.size;
                                        autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                        *autocmd_info.items.offset(c2rust_fresh12 as isize) =
                                            key_value_pair {
                                                key: cstr_as_string(b"once\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                value: object {
                                                    type_0: kObjectTypeBoolean,
                                                    data: C2Rust_Unnamed {
                                                        boolean: (*ac).once,
                                                    },
                                                },
                                            };
                                        if (*ap).buflocal_nr != 0 {
                                            let c2rust_fresh13 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh13 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"buflocal\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeBoolean,
                                                        data: C2Rust_Unnamed { boolean: true },
                                                    },
                                                };
                                            let c2rust_fresh14 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh14 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"buf\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeInteger,
                                                        data: C2Rust_Unnamed {
                                                            integer: (*ap).buflocal_nr as Integer,
                                                        },
                                                    },
                                                };
                                            let c2rust_fresh15 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh15 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"buffer\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeInteger,
                                                        data: C2Rust_Unnamed {
                                                            integer: (*ap).buflocal_nr as Integer,
                                                        },
                                                    },
                                                };
                                        } else {
                                            let c2rust_fresh16 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh16 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"buflocal\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeBoolean,
                                                        data: C2Rust_Unnamed { boolean: false },
                                                    },
                                                };
                                        }
                                        if autocmd_list.size == autocmd_list.capacity {
                                            autocmd_list.capacity = if autocmd_list.capacity
                                                << 1 as ::core::ffi::c_int
                                                > ::core::mem::size_of::<[Object; 16]>()
                                                    .wrapping_div(::core::mem::size_of::<Object>())
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<[Object; 16]>()
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                Object,
                                                            >(
                                                            ))
                                                            == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                autocmd_list.capacity << 1 as ::core::ffi::c_int
                                            } else {
                                                ::core::mem::size_of::<[Object; 16]>()
                                                    .wrapping_div(::core::mem::size_of::<Object>())
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<[Object; 16]>()
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                Object,
                                                            >(
                                                            ))
                                                            == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                            };
                                            autocmd_list.items = (if autocmd_list.capacity
                                                == ::core::mem::size_of::<[Object; 16]>()
                                                    .wrapping_div(::core::mem::size_of::<Object>())
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<[Object; 16]>()
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                Object,
                                                            >(
                                                            ))
                                                            == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                if autocmd_list.items
                                                    == &raw mut autocmd_list.init_array
                                                        as *mut Object
                                                {
                                                    autocmd_list.items as *mut ::core::ffi::c_void
                                                } else {
                                                    _memcpy_free(
                                                        &raw mut autocmd_list.init_array
                                                            as *mut Object
                                                            as *mut ::core::ffi::c_void,
                                                        autocmd_list.items
                                                            as *mut ::core::ffi::c_void,
                                                        autocmd_list.size.wrapping_mul(
                                                            ::core::mem::size_of::<Object>(),
                                                        ),
                                                    )
                                                }
                                            } else {
                                                if autocmd_list.items
                                                    == &raw mut autocmd_list.init_array
                                                        as *mut Object
                                                {
                                                    memcpy(
                                                        xmalloc(
                                                            autocmd_list.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<Object>(),
                                                            ),
                                                        ),
                                                        autocmd_list.items
                                                            as *const ::core::ffi::c_void,
                                                        autocmd_list.size.wrapping_mul(
                                                            ::core::mem::size_of::<Object>(),
                                                        ),
                                                    )
                                                } else {
                                                    xrealloc(
                                                        autocmd_list.items
                                                            as *mut ::core::ffi::c_void,
                                                        autocmd_list.capacity.wrapping_mul(
                                                            ::core::mem::size_of::<Object>(),
                                                        ),
                                                    )
                                                }
                                            })
                                                as *mut Object;
                                        } else {
                                        };
                                        let c2rust_fresh17 = autocmd_list.size;
                                        autocmd_list.size = autocmd_list.size.wrapping_add(1);
                                        *autocmd_list.items.offset(c2rust_fresh17 as isize) =
                                            object {
                                                type_0: kObjectTypeDict,
                                                data: C2Rust_Unnamed { dict: autocmd_info },
                                            };
                                    }
                                }
                            }
                        }
                        i = i.wrapping_add(1);
                    }
                }
                event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
            }
        }
    }
    return arena_take_arraybuilder(arena, &raw mut autocmd_list);
}
pub unsafe extern "C" fn nvim_create_autocmd(
    mut channel_id: uint64_t,
    mut event: Object,
    mut opts: *mut KeyDict_create_autocmd,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Integer {
    let mut au_group: ::core::ffi::c_int = 0;
    let mut has_buf: bool = false;
    let mut buf: Buffer = 0;
    let mut patterns: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut autocmd_id: int64_t = -1 as int64_t;
    let mut desc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut handler_cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut handler_fn: Callback = Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    let mut event_array: Array = unpack_string_or_array(
        event,
        b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        true_0 != 0,
        arena,
        err,
    );
    '_cleanup: {
        if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            if !(!((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 9 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong)
                || !((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong))
            {
                api_err_conflict(
                    err,
                    b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                    b"command\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__callback
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut callback: *mut Object = &raw mut (*opts).callback;
                    match (*callback).type_0 as ::core::ffi::c_uint {
                        7 => {
                            if !((*callback).data.luaref != -2 as ::core::ffi::c_int) {
                                api_err_invalid(
                                    err,
                                    b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"<no value>\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_cleanup;
                            } else if !nlua_ref_is_function((*callback).data.luaref) {
                                api_err_invalid(
                                    err,
                                    b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"<not a function>\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_cleanup;
                            } else {
                                handler_fn.type_0 = kCallbackLua;
                                handler_fn.data.luaref = (*callback).data.luaref;
                                (*callback).data.luaref = LUA_NOREF as LuaRef;
                            }
                        }
                        4 => {
                            handler_fn.type_0 = kCallbackFuncref;
                            handler_fn.data.funcref = string_to_cstr((*callback).data.string);
                        }
                        _ => {
                            if true {
                                api_err_exp(
                                    err,
                                    b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Lua function or Vim function name\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    api_typename((*callback).type_0),
                                );
                                break '_cleanup;
                            }
                        }
                    }
                } else if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__command
                    != 0 as ::core::ffi::c_ulonglong
                {
                    handler_cmd = string_to_cstr((*opts).command);
                } else if true {
                    api_err_required(
                        err,
                        b"'command' or 'callback'\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_cleanup;
                }
                au_group = get_augroup_from_object((*opts).group, err);
                if au_group != AUGROUP_ERROR as ::core::ffi::c_int {
                    has_buf = (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__buf
                        != 0 as ::core::ffi::c_ulonglong
                        || (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong)
                                << KEYSET_OPTIDX_create_autocmd__buffer
                            != 0 as ::core::ffi::c_ulonglong;
                    buf = if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__buf
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        (*opts).buf
                    } else {
                        (*opts).buffer
                    };
                    if !(!((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong)
                        || !((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_ulonglong))
                    {
                        api_err_conflict(
                            err,
                            b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    } else if !(!((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong)
                        || !has_buf)
                    {
                        api_err_conflict(
                            err,
                            b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                            b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    } else {
                        patterns = get_patterns_from_pattern_or_buf(
                            (*opts).pattern,
                            has_buf,
                            buf,
                            b"*\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            arena,
                            err,
                        );
                        if (*err).type_0 as ::core::ffi::c_int
                            == kErrorTypeNone as ::core::ffi::c_int
                        {
                            if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                                & (1 as ::core::ffi::c_ulonglong)
                                    << KEYSET_OPTIDX_create_autocmd__desc
                                != 0 as ::core::ffi::c_ulonglong
                            {
                                desc = (*opts).desc.data;
                            }
                            if !(event_array.size > 0 as size_t) {
                                api_err_required(
                                    err,
                                    b"event\0".as_ptr() as *const ::core::ffi::c_char,
                                );
                            } else {
                                let c2rust_fresh18 = next_autocmd_id.get();
                                next_autocmd_id.set(next_autocmd_id.get() + 1);
                                autocmd_id = c2rust_fresh18;
                                let mut event_str_index: size_t = 0 as size_t;
                                loop {
                                    if event_str_index >= event_array.size {
                                        break '_cleanup;
                                    }
                                    let mut event_str: Object =
                                        *event_array.items.offset(event_str_index as isize);
                                    let mut event_nr: event_T =
                                        event_name2nr_str(event_str.data.string);
                                    if !((event_nr as ::core::ffi::c_uint)
                                        < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                                    {
                                        api_err_invalid(
                                            err,
                                            b"event\0".as_ptr() as *const ::core::ffi::c_char,
                                            event_str.data.string.data,
                                            0 as int64_t,
                                            true,
                                        );
                                        break '_cleanup;
                                    } else {
                                        let mut retval: ::core::ffi::c_int = 0;
                                        let mut pat_index: size_t = 0 as size_t;
                                        while pat_index < patterns.size {
                                            let mut pat: Object =
                                                *patterns.items.offset(pat_index as isize);
                                            let save_current_sctx: sctx_T =
                                                api_set_sctx(channel_id);
                                            retval = autocmd_register(
                                                autocmd_id,
                                                event_nr,
                                                pat.data.string.data,
                                                pat.data.string.size as ::core::ffi::c_int,
                                                au_group,
                                                (*opts).once as bool,
                                                (*opts).nested as bool,
                                                desc,
                                                handler_cmd,
                                                &raw mut handler_fn,
                                            );
                                            current_sctx.set(save_current_sctx);
                                            if retval == 0 as ::core::ffi::c_int {
                                                api_set_error(
                                                    err,
                                                    kErrorTypeException,
                                                    b"Failed to set autocmd\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                );
                                                break '_cleanup;
                                            } else {
                                                pat_index = pat_index.wrapping_add(1);
                                            }
                                        }
                                        event_str_index = event_str_index.wrapping_add(1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !handler_cmd.is_null() {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut handler_cmd as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    } else {
        callback_free(&raw mut handler_fn);
    }
    return autocmd_id as Integer;
}
pub unsafe extern "C" fn nvim_del_autocmd(mut id: Integer, mut err: *mut Error) {
    if !(id > 0 as Integer) {
        api_err_invalid(
            err,
            b"autocmd id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            id as int64_t,
            false_0 != 0,
        );
        return;
    }
    if !autocmd_delete_id(id as int64_t) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to delete autocmd\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
pub unsafe extern "C" fn nvim_clear_autocmds(
    mut opts: *mut KeyDict_clear_autocmds,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut event_array: Array = unpack_string_or_array(
        (*opts).event,
        b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        false_0 != 0,
        arena,
        err,
    );
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    let mut has_buf: bool = (*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_clear_autocmds__buf
        != 0 as ::core::ffi::c_ulonglong
        || (*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_clear_autocmds__buffer
            != 0 as ::core::ffi::c_ulonglong;
    let mut buf: ::core::ffi::c_int = if (*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_clear_autocmds__buf
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).buf as ::core::ffi::c_int
    } else {
        (*opts).buffer as ::core::ffi::c_int
    };
    if !(!((*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_ulonglong)
        || !((*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong))
    {
        api_err_conflict(
            err,
            b"buf\0".as_ptr() as *const ::core::ffi::c_char,
            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if !(!((*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_ulonglong)
        || !has_buf)
    {
        api_err_conflict(
            err,
            b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
            b"buf\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut au_group: ::core::ffi::c_int = get_augroup_from_object((*opts).group, err);
    if au_group == AUGROUP_ERROR as ::core::ffi::c_int {
        return;
    }
    let mut patterns: Array = get_patterns_from_pattern_or_buf(
        (*opts).pattern,
        has_buf,
        buf as Buffer,
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        arena,
        err,
    );
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    if event_array.size == 0 as size_t {
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            let mut pat_object_index: size_t = 0 as size_t;
            while pat_object_index < patterns.size {
                let mut pat_object: Object = *patterns.items.offset(pat_object_index as isize);
                let mut pat: *mut ::core::ffi::c_char = pat_object.data.string.data;
                if !clear_autocmd(event, pat, au_group, err) {
                    return;
                }
                pat_object_index = pat_object_index.wrapping_add(1);
            }
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    } else {
        let mut event_str_index: size_t = 0 as size_t;
        while event_str_index < event_array.size {
            let mut event_str: Object = *event_array.items.offset(event_str_index as isize);
            let mut event_nr: event_T = event_name2nr_str(event_str.data.string);
            if !((event_nr as ::core::ffi::c_uint)
                < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                api_err_invalid(
                    err,
                    b"event\0".as_ptr() as *const ::core::ffi::c_char,
                    event_str.data.string.data,
                    0 as int64_t,
                    true,
                );
                return;
            }
            let mut pat_object_index_0: size_t = 0 as size_t;
            while pat_object_index_0 < patterns.size {
                let mut pat_object_0: Object = *patterns.items.offset(pat_object_index_0 as isize);
                let mut pat_0: *mut ::core::ffi::c_char = pat_object_0.data.string.data;
                if !clear_autocmd(event_nr, pat_0, au_group, err) {
                    return;
                }
                pat_object_index_0 = pat_object_index_0.wrapping_add(1);
            }
            event_str_index = event_str_index.wrapping_add(1);
        }
    };
}
pub unsafe extern "C" fn nvim_create_augroup(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut opts: *mut KeyDict_create_augroup,
    mut err: *mut Error,
) -> Integer {
    let mut augroup_name_0: *mut ::core::ffi::c_char = name.data;
    let mut clear_autocmds: bool = if (*opts).is_set__create_augroup_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_augroup__clear
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).clear as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut augroup: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
    augroup = augroup_add(augroup_name_0);
    if augroup == AUGROUP_ERROR as ::core::ffi::c_int {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to set augroup\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return -1 as Integer;
    }
    if clear_autocmds {
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            aucmd_del_for_event_and_group(event, augroup);
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    }
    current_sctx.set(save_current_sctx);
    return augroup as Integer;
}
pub unsafe extern "C" fn nvim_del_augroup_by_id(mut id: Integer, mut err: *mut Error) {
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
    let mut name: *mut ::core::ffi::c_char = if id == 0 as Integer {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        augroup_name(id as ::core::ffi::c_int)
    };
    augroup_del(name, false);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_del_augroup_by_name(mut name: String_0, mut err: *mut Error) {
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
    augroup_del(name.data, false);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_exec_autocmds(
    mut event: Object,
    mut opts: *mut KeyDict_exec_autocmds,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut au_group: ::core::ffi::c_int = AUGROUP_ALL as ::core::ffi::c_int;
    let mut modeline: bool = true_0 != 0;
    let mut b: *mut buf_T = curbuf.get();
    let mut data: *mut Object = ::core::ptr::null_mut::<Object>();
    let mut event_array: Array = unpack_string_or_array(
        event,
        b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        true_0 != 0,
        arena,
        err,
    );
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match (*opts).group.type_0 as ::core::ffi::c_uint {
        0 => {}
        4 => {
            au_group = augroup_find((*opts).group.data.string.data);
            if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                api_err_invalid(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    (*opts).group.data.string.data,
                    0 as int64_t,
                    true_0 != 0,
                );
                return;
            }
        }
        2 => {
            au_group = (*opts).group.data.integer as ::core::ffi::c_int;
            name = if au_group == 0 as ::core::ffi::c_int {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                augroup_name(au_group)
            };
            if !augroup_exists(name) {
                api_err_invalid(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    au_group as int64_t,
                    false_0 != 0,
                );
                return;
            }
        }
        _ => {
            if true {
                api_err_exp(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    b"String or Integer\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename((*opts).group.type_0),
                );
                return;
            }
        }
    }
    let mut has_buf: bool = (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__buf
        != 0 as ::core::ffi::c_ulonglong
        || (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__buffer
            != 0 as ::core::ffi::c_ulonglong;
    let mut buf: Buffer = if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__buf
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).buf
    } else {
        (*opts).buffer
    };
    if !(!((*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_ulonglong)
        || !((*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong))
    {
        api_err_conflict(
            err,
            b"buf\0".as_ptr() as *const ::core::ffi::c_char,
            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if has_buf {
        if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong
        {
            api_err_conflict(
                err,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        b = find_buffer_by_handle(buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
    }
    let mut patterns: Array = get_patterns_from_pattern_or_buf(
        (*opts).pattern,
        has_buf,
        buf,
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        arena,
        err,
    );
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__data
        != 0 as ::core::ffi::c_ulonglong
    {
        data = &raw mut (*opts).data;
    }
    modeline = if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__modeline
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).modeline as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut did_aucmd: bool = false_0 != 0;
    let mut event_str_index: size_t = 0 as size_t;
    while event_str_index < event_array.size {
        let mut event_str: Object = *event_array.items.offset(event_str_index as isize);
        let mut event_nr: event_T = event_name2nr_str(event_str.data.string);
        if !((event_nr as ::core::ffi::c_uint)
            < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            api_err_invalid(
                err,
                b"event\0".as_ptr() as *const ::core::ffi::c_char,
                event_str.data.string.data,
                0 as int64_t,
                true,
            );
            return;
        }
        let mut pat_index: size_t = 0 as size_t;
        while pat_index < patterns.size {
            let mut pat: Object = *patterns.items.offset(pat_index as isize);
            let mut fname: *mut ::core::ffi::c_char = if !has_buf {
                pat.data.string.data
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            };
            did_aucmd = did_aucmd as ::core::ffi::c_int
                | apply_autocmds_group(
                    event_nr,
                    fname,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    true,
                    au_group,
                    b,
                    ::core::ptr::null_mut::<exarg_T>(),
                    data,
                ) as ::core::ffi::c_int
                != 0;
            pat_index = pat_index.wrapping_add(1);
        }
        event_str_index = event_str_index.wrapping_add(1);
    }
    if did_aucmd as ::core::ffi::c_int != 0 && modeline as ::core::ffi::c_int != 0 {
        do_modelines(0 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn unpack_string_or_array(
    mut v: Object,
    mut k: *mut ::core::ffi::c_char,
    mut required: bool,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    if v.type_0 as ::core::ffi::c_uint
        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut arr: Array = arena_array(arena, 1 as size_t);
        let c2rust_fresh23 = arr.size;
        arr.size = arr.size.wrapping_add(1);
        *arr.items.offset(c2rust_fresh23 as isize) = v;
        return arr;
    } else if v.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !check_string_array(v.data.array, k, true_0 != 0, err) {
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        return v.data.array;
    } else if !(!required
        && v.type_0 as ::core::ffi::c_uint
            == kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        api_err_exp(
            err,
            k,
            b"Array or String\0".as_ptr() as *const ::core::ffi::c_char,
            api_typename(v.type_0),
        );
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
    return Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
}
unsafe extern "C" fn get_augroup_from_object(
    mut group: Object,
    mut err: *mut Error,
) -> ::core::ffi::c_int {
    let mut au_group: ::core::ffi::c_int = AUGROUP_ERROR as ::core::ffi::c_int;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match group.type_0 as ::core::ffi::c_uint {
        0 => return AUGROUP_DEFAULT as ::core::ffi::c_int,
        4 => {
            au_group = augroup_find(group.data.string.data);
            if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                api_err_invalid(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    group.data.string.data,
                    0 as int64_t,
                    true_0 != 0,
                );
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
            return au_group;
        }
        2 => {
            au_group = group.data.integer as ::core::ffi::c_int;
            name = if au_group == 0 as ::core::ffi::c_int {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                augroup_name(au_group)
            };
            if !augroup_exists(name) {
                api_err_invalid(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    au_group as int64_t,
                    false_0 != 0,
                );
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
            return au_group;
        }
        _ => {
            if true {
                api_err_exp(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    b"String or Integer\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(group.type_0),
                );
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
        }
    }
    panic!("Reached end of non-void function without returning");
}
unsafe extern "C" fn get_patterns_from_pattern_or_buf(
    mut pattern: Object,
    mut has_buf: bool,
    mut buf: Buffer,
    mut fallback: *mut ::core::ffi::c_char,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut patterns: ArrayBuilder = ArrayBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
        init_array: [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 16],
    };
    patterns.capacity = ::core::mem::size_of::<[Object; 16]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 16]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    patterns.size = 0 as size_t;
    patterns.items = &raw mut patterns.init_array as *mut Object;
    if pattern.type_0 as ::core::ffi::c_uint
        != kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if pattern.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut pat: *const ::core::ffi::c_char = pattern.data.string.data;
            let mut patlen: size_t = aucmd_span_pattern(pat, &raw mut pat);
            while patlen != 0 {
                if patterns.size == patterns.capacity {
                    patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[Object; 16]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        patterns.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[Object; 16]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    patterns.items = (if patterns.capacity
                        == ::core::mem::size_of::<[Object; 16]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if patterns.items == &raw mut patterns.init_array as *mut Object {
                            patterns.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut patterns.init_array as *mut Object
                                    as *mut ::core::ffi::c_void,
                                patterns.items as *mut ::core::ffi::c_void,
                                patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    } else {
                        if patterns.items == &raw mut patterns.init_array as *mut Object {
                            memcpy(
                                xmalloc(
                                    patterns
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                ),
                                patterns.items as *const ::core::ffi::c_void,
                                patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        } else {
                            xrealloc(
                                patterns.items as *mut ::core::ffi::c_void,
                                patterns
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    }) as *mut Object;
                } else {
                };
                let c2rust_fresh19 = patterns.size;
                patterns.size = patterns.size.wrapping_add(1);
                *patterns.items.offset(c2rust_fresh19 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: arena_string(
                            arena,
                            String_0 {
                                data: pat as *mut ::core::ffi::c_char,
                                size: patlen,
                            },
                        ),
                    },
                };
                patlen = aucmd_span_pattern(pat.offset(patlen as isize), &raw mut pat);
            }
        } else if pattern.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if !check_string_array(
                pattern.data.array,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                true_0 != 0,
                err,
            ) {
                return Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
            }
            let mut array: Array = pattern.data.array;
            let mut entry_index: size_t = 0 as size_t;
            while entry_index < array.size {
                let mut entry: Object = *array.items.offset(entry_index as isize);
                let mut pat_0: *const ::core::ffi::c_char = entry.data.string.data;
                let mut patlen_0: size_t = aucmd_span_pattern(pat_0, &raw mut pat_0);
                while patlen_0 != 0 {
                    if patterns.size == patterns.capacity {
                        patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            patterns.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        patterns.items = (if patterns.capacity
                            == ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if patterns.items == &raw mut patterns.init_array as *mut Object {
                                patterns.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut patterns.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    patterns.items as *mut ::core::ffi::c_void,
                                    patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if patterns.items == &raw mut patterns.init_array as *mut Object {
                                memcpy(
                                    xmalloc(
                                        patterns
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    patterns.items as *const ::core::ffi::c_void,
                                    patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    patterns.items as *mut ::core::ffi::c_void,
                                    patterns
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh20 = patterns.size;
                    patterns.size = patterns.size.wrapping_add(1);
                    *patterns.items.offset(c2rust_fresh20 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: arena_string(
                                arena,
                                String_0 {
                                    data: pat_0 as *mut ::core::ffi::c_char,
                                    size: patlen_0,
                                },
                            ),
                        },
                    };
                    patlen_0 = aucmd_span_pattern(pat_0.offset(patlen_0 as isize), &raw mut pat_0);
                }
                entry_index = entry_index.wrapping_add(1);
            }
        } else if true {
            api_err_exp(
                err,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                b"String or Table\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(pattern.type_0),
            );
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
    } else if has_buf {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        if patterns.size == patterns.capacity {
            patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                patterns.capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            patterns.items = (if patterns.capacity
                == ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if patterns.items == &raw mut patterns.init_array as *mut Object {
                    patterns.items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut patterns.init_array as *mut Object as *mut ::core::ffi::c_void,
                        patterns.items as *mut ::core::ffi::c_void,
                        patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            } else {
                if patterns.items == &raw mut patterns.init_array as *mut Object {
                    memcpy(
                        xmalloc(
                            patterns
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<Object>()),
                        ),
                        patterns.items as *const ::core::ffi::c_void,
                        patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                } else {
                    xrealloc(
                        patterns.items as *mut ::core::ffi::c_void,
                        patterns
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            }) as *mut Object;
        } else {
        };
        let c2rust_fresh21 = patterns.size;
        patterns.size = patterns.size.wrapping_add(1);
        *patterns.items.offset(c2rust_fresh21 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: arena_printf(
                    arena,
                    b"<buffer=%d>\0".as_ptr() as *const ::core::ffi::c_char,
                    (*b).handle,
                ),
            },
        };
    }
    if patterns.size == 0 as size_t && !fallback.is_null() {
        if patterns.size == patterns.capacity {
            patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                patterns.capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            patterns.items = (if patterns.capacity
                == ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if patterns.items == &raw mut patterns.init_array as *mut Object {
                    patterns.items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut patterns.init_array as *mut Object as *mut ::core::ffi::c_void,
                        patterns.items as *mut ::core::ffi::c_void,
                        patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            } else {
                if patterns.items == &raw mut patterns.init_array as *mut Object {
                    memcpy(
                        xmalloc(
                            patterns
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<Object>()),
                        ),
                        patterns.items as *const ::core::ffi::c_void,
                        patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                } else {
                    xrealloc(
                        patterns.items as *mut ::core::ffi::c_void,
                        patterns
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            }) as *mut Object;
        } else {
        };
        let c2rust_fresh22 = patterns.size;
        patterns.size = patterns.size.wrapping_add(1);
        *patterns.items.offset(c2rust_fresh22 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(fallback),
            },
        };
    }
    return arena_take_arraybuilder(arena, &raw mut patterns);
}
unsafe extern "C" fn clear_autocmd(
    mut event: event_T,
    mut pat: *mut ::core::ffi::c_char,
    mut au_group: ::core::ffi::c_int,
    mut err: *mut Error,
) -> bool {
    if do_autocmd_event(
        event,
        pat,
        false_0 != 0,
        false_0,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
        au_group,
    ) == FAIL
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to clear autocmd\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    return true_0 != 0;
}
