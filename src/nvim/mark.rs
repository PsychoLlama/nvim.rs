use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::autocmd::{aucmd_defer, has_event};
use crate::src::nvim::buffer::{
    bt_prompt, buflist_findnr, buflist_getfile, buflist_new, buflist_nr2name,
};
use crate::src::nvim::charset::{ptr2cells, skipwhite, vim_isprintc};
use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::diff::diff_mark_adjust;
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::typval::{
    tv_dict_add_list, tv_dict_add_str, tv_dict_alloc, tv_list_alloc, tv_list_append_dict,
    tv_list_append_number,
};
use crate::src::nvim::extmark::extmark_adjust;
use crate::src::nvim::fold::{foldMarkAdjust, hasFolding};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    cmdmod, curbuf, curtab, curwin, e_argreq, e_invarg, e_invarg2, e_markinval, e_marknotset,
    e_umark, first_tabpage, firstwin, global_busy, got_int, jop_flags, listcmd_busy, namedfm,
    saved_cursor, Columns, IObuff, NameBuff,
};
use crate::src::nvim::mbyte::{utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::{ml_get, ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{xfree, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    emsg, message_filtered, msg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts,
    msg_puts_title, semsg,
};
use crate::src::nvim::os::env::expand_env;
use crate::src::nvim::os::fs::os_dirname;
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{__assert_fail, gettext, memmove, snprintf};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::path::{path_fnamecmp, path_shorten_fname, vim_ispathsep_nocolon};
use crate::src::nvim::plines::linetabsize_eol;
use crate::src::nvim::r#move::set_topline;
use crate::src::nvim::strings::{vim_strchr, xstrnsave};
use crate::src::nvim::tag::tagstack_clear_entry;
use crate::src::nvim::textobject::{findpar, findsent};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, Array, BoolVarValue, Boolean, BufUpdateCallbacks, CMD_index,
    Callback, CallbackType, Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, Dict, Direction, ExtmarkMove, ExtmarkOp,
    ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative,
    GridView, Integer, Intersection, KeyValuePair, LineGetter, ListLenSpecials, LuaRef, MTKey,
    MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkAdjustMode, MarkGet, MarkMove, MarkMoveRes, MarkTree, MotionType,
    Object, ObjectType, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0, Terminal, Timestamp, UndoObjectType,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, __time_t, alist_T, auto_event, bcount_t, bhdr_T, blob_T, blobvar_S,
    blocknr_T, buf_T, bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, cmdmod_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_17, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    eslist_T, eslist_elem, event_T, exarg, exarg_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, getf_values,
    handle_T, hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed_13, oparg_T,
    partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T,
    queue, reg_extmatch_T, regmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T,
    synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union,
    u_entry, u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_6, varnumber_T, virt_line, visualinfo_T, win_T,
    window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn qf_mark_adjust(
        buf: *mut buf_T,
        wp: *mut win_T,
        line1: linenr_T,
        line2: linenr_T,
        amount: linenr_T,
        amount_after: linenr_T,
    ) -> bool;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_16 = 76;
pub const HLF_PRE: C2Rust_Unnamed_16 = 75;
pub const HLF_OK: C2Rust_Unnamed_16 = 74;
pub const HLF_SO: C2Rust_Unnamed_16 = 73;
pub const HLF_SE: C2Rust_Unnamed_16 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_16 = 71;
pub const HLF_TS: C2Rust_Unnamed_16 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_16 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_16 = 68;
pub const HLF_CU: C2Rust_Unnamed_16 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_16 = 66;
pub const HLF_WBR: C2Rust_Unnamed_16 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_16 = 64;
pub const HLF_MSG: C2Rust_Unnamed_16 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_16 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_16 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_16 = 60;
pub const HLF_0: C2Rust_Unnamed_16 = 59;
pub const HLF_QFL: C2Rust_Unnamed_16 = 58;
pub const HLF_MC: C2Rust_Unnamed_16 = 57;
pub const HLF_CUL: C2Rust_Unnamed_16 = 56;
pub const HLF_CUC: C2Rust_Unnamed_16 = 55;
pub const HLF_TPF: C2Rust_Unnamed_16 = 54;
pub const HLF_TPS: C2Rust_Unnamed_16 = 53;
pub const HLF_TP: C2Rust_Unnamed_16 = 52;
pub const HLF_PBR: C2Rust_Unnamed_16 = 51;
pub const HLF_PST: C2Rust_Unnamed_16 = 50;
pub const HLF_PSB: C2Rust_Unnamed_16 = 49;
pub const HLF_PSX: C2Rust_Unnamed_16 = 48;
pub const HLF_PNX: C2Rust_Unnamed_16 = 47;
pub const HLF_PSK: C2Rust_Unnamed_16 = 46;
pub const HLF_PNK: C2Rust_Unnamed_16 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_16 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_16 = 43;
pub const HLF_PSI: C2Rust_Unnamed_16 = 42;
pub const HLF_PNI: C2Rust_Unnamed_16 = 41;
pub const HLF_SPL: C2Rust_Unnamed_16 = 40;
pub const HLF_SPR: C2Rust_Unnamed_16 = 39;
pub const HLF_SPC: C2Rust_Unnamed_16 = 38;
pub const HLF_SPB: C2Rust_Unnamed_16 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_16 = 36;
pub const HLF_SC: C2Rust_Unnamed_16 = 35;
pub const HLF_TXA: C2Rust_Unnamed_16 = 34;
pub const HLF_TXD: C2Rust_Unnamed_16 = 33;
pub const HLF_DED: C2Rust_Unnamed_16 = 32;
pub const HLF_CHD: C2Rust_Unnamed_16 = 31;
pub const HLF_ADD: C2Rust_Unnamed_16 = 30;
pub const HLF_FC: C2Rust_Unnamed_16 = 29;
pub const HLF_FL: C2Rust_Unnamed_16 = 28;
pub const HLF_WM: C2Rust_Unnamed_16 = 27;
pub const HLF_W: C2Rust_Unnamed_16 = 26;
pub const HLF_VNC: C2Rust_Unnamed_16 = 25;
pub const HLF_V: C2Rust_Unnamed_16 = 24;
pub const HLF_T: C2Rust_Unnamed_16 = 23;
pub const HLF_VSP: C2Rust_Unnamed_16 = 22;
pub const HLF_C: C2Rust_Unnamed_16 = 21;
pub const HLF_SNC: C2Rust_Unnamed_16 = 20;
pub const HLF_S: C2Rust_Unnamed_16 = 19;
pub const HLF_R: C2Rust_Unnamed_16 = 18;
pub const HLF_CLF: C2Rust_Unnamed_16 = 17;
pub const HLF_CLS: C2Rust_Unnamed_16 = 16;
pub const HLF_CLN: C2Rust_Unnamed_16 = 15;
pub const HLF_LNB: C2Rust_Unnamed_16 = 14;
pub const HLF_LNA: C2Rust_Unnamed_16 = 13;
pub const HLF_N: C2Rust_Unnamed_16 = 12;
pub const HLF_CM: C2Rust_Unnamed_16 = 11;
pub const HLF_M: C2Rust_Unnamed_16 = 10;
pub const HLF_LC: C2Rust_Unnamed_16 = 9;
pub const HLF_L: C2Rust_Unnamed_16 = 8;
pub const HLF_I: C2Rust_Unnamed_16 = 7;
pub const HLF_E: C2Rust_Unnamed_16 = 6;
pub const HLF_D: C2Rust_Unnamed_16 = 5;
pub const HLF_AT: C2Rust_Unnamed_16 = 4;
pub const HLF_TERM: C2Rust_Unnamed_16 = 3;
pub const HLF_EOB: C2Rust_Unnamed_16 = 2;
pub const HLF_8: C2Rust_Unnamed_16 = 1;
pub const HLF_NONE: C2Rust_Unnamed_16 = 0;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const kMarkChangedView: MarkMoveRes = 64;
pub const kMarkChangedCursor: MarkMoveRes = 32;
pub const kMarkChangedLine: MarkMoveRes = 16;
pub const kMarkChangedCol: MarkMoveRes = 8;
pub const kMarkSwitchedBuf: MarkMoveRes = 4;
pub const kMarkMoveFailed: MarkMoveRes = 2;
pub const kMarkMoveSuccess: MarkMoveRes = 1;
pub const kMarkJumpList: MarkMove = 16;
pub const kMarkSetView: MarkMove = 8;
pub const KMarkNoContext: MarkMove = 4;
pub const kMarkContext: MarkMove = 2;
pub const kMarkBeginLine: MarkMove = 1;
pub const kMarkAllNoResolve: MarkGet = 2;
pub const kMarkAll: MarkGet = 1;
pub const kMarkBufLocal: MarkGet = 0;
pub const kMarkAdjustTerm: MarkAdjustMode = 2;
pub const kMarkAdjustApi: MarkAdjustMode = 1;
pub const kMarkAdjustNormal: MarkAdjustMode = 0;
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
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_18 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_18 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_18 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_18 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_18 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_18 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_18 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_18 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_18 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_18 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_18 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_18 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_18 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_18 = 1;
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
pub type C2Rust_Unnamed_19 = ::core::ffi::c_int;
pub const AUGROUP_DELETED: C2Rust_Unnamed_19 = -4;
pub const AUGROUP_ALL: C2Rust_Unnamed_19 = -3;
pub const AUGROUP_ERROR: C2Rust_Unnamed_19 = -2;
pub const AUGROUP_DEFAULT: C2Rust_Unnamed_19 = -1;
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const kOptJopFlagClean: C2Rust_Unnamed_20 = 4;
pub const kOptJopFlagView: C2Rust_Unnamed_20 = 2;
pub const kOptJopFlagStack: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_21 = 4;
pub const BL_SOL: C2Rust_Unnamed_21 = 2;
pub const BL_WHITE: C2Rust_Unnamed_21 = 1;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Dict = Dict {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<KeyValuePair>(),
};
pub const BUF_HAS_QF_ENTRY: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const BUF_HAS_LL_ENTRY: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EXTRA_MARKS: ::core::ffi::c_int =
    '9' as ::core::ffi::c_int - '0' as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const NMARKS: ::core::ffi::c_int =
    'z' as ::core::ffi::c_int - 'a' as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const NGLOBALMARKS: ::core::ffi::c_int = NMARKS + EXTRA_MARKS;
pub const NMARK_LOCAL_MAX: ::core::ffi::c_int = 126 as ::core::ffi::c_int;
pub const JUMPLISTSIZE: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
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
pub const ARRAY_DICT_INIT: Dict = KV_INITIAL_VALUE;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub unsafe extern "C" fn setmark(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut view: fmarkv_T = mark_view_make(curwin.get(), (*curwin.get()).w_cursor);
    return setmark_pos(
        c,
        &raw mut (*curwin.get()).w_cursor,
        (*curbuf.get()).handle as ::core::ffi::c_int,
        &raw mut view,
    );
}
pub unsafe extern "C" fn free_fmark(mut fm: fmark_T) {
    xfree(fm.additional_data as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn free_xfmark(mut fm: xfmark_T) {
    xfree(fm.fname as *mut ::core::ffi::c_void);
    free_fmark(fm.fmark);
}
pub unsafe extern "C" fn clear_fmark(fm: *mut fmark_T, timestamp: Timestamp) {
    free_fmark(*fm);
    *fm = fmark_T {
        mark: pos_T {
            lnum: 0 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        },
        fnum: 0 as ::core::ffi::c_int,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    (*fm).timestamp = timestamp;
}
unsafe extern "C" fn do_markset_autocmd(
    mut c: ::core::ffi::c_char,
    mut pos: *mut pos_T,
    mut buf: *mut buf_T,
) {
    if !has_event(EVENT_MARKSET) {
        return;
    }
    let mut data: Dict = ARRAY_DICT_INIT;
    let mut data__items: [KeyValuePair; 3] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_13 { boolean: false },
        },
    }; 3];
    data.capacity = 3 as size_t;
    data.items = &raw mut data__items as *mut KeyValuePair;
    let mut mark_str: [::core::ffi::c_char; 2] = [c, '\0' as ::core::ffi::c_char];
    let c2rust_fresh0 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh0 as isize) = key_value_pair {
        key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_13 {
                string: String_0 {
                    data: &raw mut mark_str as *mut ::core::ffi::c_char,
                    size: 1 as size_t,
                },
            },
        },
    };
    let c2rust_fresh1 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh1 as isize) = key_value_pair {
        key: cstr_as_string(b"line\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_13 {
                integer: (*pos).lnum as Integer,
            },
        },
    };
    let c2rust_fresh2 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh2 as isize) = key_value_pair {
        key: cstr_as_string(b"col\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed_13 {
                integer: (*pos).col as Integer,
            },
        },
    };
    let mut c2rust_lvalue: Object = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed_13 { dict: data },
    };
    aucmd_defer(
        EVENT_MARKSET,
        &raw mut mark_str as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        AUGROUP_ALL as ::core::ffi::c_int,
        buf,
        ::core::ptr::null_mut::<exarg_T>(),
        &raw mut c2rust_lvalue,
    );
}
pub unsafe extern "C" fn setmark_pos(
    mut c: ::core::ffi::c_int,
    mut pos: *mut pos_T,
    mut fnum: ::core::ffi::c_int,
    mut view_pt: *mut fmarkv_T,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut view: fmarkv_T = if !view_pt.is_null() {
        *view_pt
    } else {
        fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        }
    };
    if c < 0 as ::core::ffi::c_int {
        return FAIL;
    }
    if c == '\'' as ::core::ffi::c_int || c == '`' as ::core::ffi::c_int {
        if pos == &raw mut (*curwin.get()).w_cursor {
            setpcmark();
            (*curwin.get()).w_prev_pcmark = (*curwin.get()).w_pcmark;
        } else {
            (*curwin.get()).w_pcmark = *pos;
        }
        return OK;
    }
    let mut buf: *mut buf_T = buflist_findnr(fnum);
    if buf.is_null() {
        return FAIL;
    }
    if c == '"' as ::core::ffi::c_int {
        let fmarkp___: *mut fmark_T = &raw mut (*buf).b_last_cursor;
        free_fmark(*fmarkp___);
        let fmarkp__: *mut fmark_T = fmarkp___;
        (*fmarkp__).mark = *pos;
        (*fmarkp__).fnum = (*buf).handle as ::core::ffi::c_int;
        (*fmarkp__).timestamp = os_time();
        (*fmarkp__).view = view;
        (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        do_markset_autocmd(c as ::core::ffi::c_char, pos, buf);
        return OK;
    }
    if c == '[' as ::core::ffi::c_int {
        (*buf).b_op_start = *pos;
        do_markset_autocmd(c as ::core::ffi::c_char, pos, buf);
        return OK;
    }
    if c == ']' as ::core::ffi::c_int {
        (*buf).b_op_end = *pos;
        do_markset_autocmd(c as ::core::ffi::c_char, pos, buf);
        return OK;
    }
    if c == '<' as ::core::ffi::c_int || c == '>' as ::core::ffi::c_int {
        if c == '<' as ::core::ffi::c_int {
            (*buf).b_visual.vi_start = *pos;
        } else {
            (*buf).b_visual.vi_end = *pos;
        }
        if (*buf).b_visual.vi_mode == NUL {
            (*buf).b_visual.vi_mode = 'v' as ::core::ffi::c_int;
        }
        do_markset_autocmd(c as ::core::ffi::c_char, pos, buf);
        return OK;
    }
    if c == ':' as ::core::ffi::c_int && bt_prompt(buf) as ::core::ffi::c_int != 0 {
        let fmarkp____0: *mut fmark_T = &raw mut (*buf).b_prompt_start;
        free_fmark(*fmarkp____0);
        let fmarkp___0: *mut fmark_T = fmarkp____0;
        (*fmarkp___0).mark = *pos;
        (*fmarkp___0).fnum = (*buf).handle as ::core::ffi::c_int;
        (*fmarkp___0).timestamp = os_time();
        (*fmarkp___0).view = view;
        (*fmarkp___0).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        return OK;
    }
    if c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        i = c - 'a' as ::core::ffi::c_int;
        let fmarkp____1: *mut fmark_T =
            (&raw mut (*buf).b_namedm as *mut fmark_T).offset(i as isize);
        free_fmark(*fmarkp____1);
        let fmarkp___1: *mut fmark_T = fmarkp____1;
        (*fmarkp___1).mark = *pos;
        (*fmarkp___1).fnum = fnum;
        (*fmarkp___1).timestamp = os_time();
        (*fmarkp___1).view = view;
        (*fmarkp___1).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        do_markset_autocmd(c as ::core::ffi::c_char, pos, buf);
        return OK;
    }
    if c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || ascii_isdigit(c) as ::core::ffi::c_int != 0
    {
        if ascii_isdigit(c) {
            i = c - '0' as ::core::ffi::c_int + NMARKS;
        } else {
            i = c - 'A' as ::core::ffi::c_int;
        }
        let xfmarkp__: *mut xfmark_T = (namedfm.ptr() as *mut xfmark_T).offset(i as isize);
        free_xfmark(*xfmarkp__);
        (*xfmarkp__).fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let fmarkp___2: *mut fmark_T = &raw mut (*xfmarkp__).fmark;
        (*fmarkp___2).mark = *pos;
        (*fmarkp___2).fnum = fnum;
        (*fmarkp___2).timestamp = os_time();
        (*fmarkp___2).view = view;
        (*fmarkp___2).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        do_markset_autocmd(c as ::core::ffi::c_char, pos, buf);
        return OK;
    }
    return FAIL;
}
pub unsafe extern "C" fn mark_jumplist_forget_file(
    mut wp: *mut win_T,
    mut fnum: ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int = (*wp).w_jumplistlen - 1 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        if (*wp).w_jumplist[i as usize].fmark.fnum == fnum {
            free_xfmark((*wp).w_jumplist[i as usize]);
            if (*wp).w_jumplistidx > i {
                (*wp).w_jumplistidx -= 1;
            }
            (*wp).w_jumplistlen -= 1;
            memmove(
                (&raw mut (*wp).w_jumplist as *mut xfmark_T).offset(i as isize)
                    as *mut ::core::ffi::c_void,
                (&raw mut (*wp).w_jumplist as *mut xfmark_T)
                    .offset((i + 1 as ::core::ffi::c_int) as isize)
                    as *const ::core::ffi::c_void,
                (((*wp).w_jumplistlen - i) as size_t)
                    .wrapping_mul(::core::mem::size_of::<xfmark_T>()),
            );
        }
        i -= 1;
    }
}
pub unsafe extern "C" fn mark_forget_file(mut wp: *mut win_T, mut fnum: ::core::ffi::c_int) {
    mark_jumplist_forget_file(wp, fnum);
    let mut i: ::core::ffi::c_int = (*wp).w_tagstacklen - 1 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        if (*wp).w_tagstack[i as usize].fmark.fnum == fnum {
            tagstack_clear_entry((&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize));
            if (*wp).w_tagstackidx > i {
                (*wp).w_tagstackidx -= 1;
            }
            (*wp).w_tagstacklen -= 1;
            memmove(
                (&raw mut (*wp).w_tagstack as *mut taggy_T).offset(i as isize)
                    as *mut ::core::ffi::c_void,
                (&raw mut (*wp).w_tagstack as *mut taggy_T)
                    .offset((i + 1 as ::core::ffi::c_int) as isize)
                    as *const ::core::ffi::c_void,
                (((*wp).w_tagstacklen - i) as size_t)
                    .wrapping_mul(::core::mem::size_of::<taggy_T>()),
            );
        }
        i -= 1;
    }
}
pub unsafe extern "C" fn setpcmark() {
    let mut fm: *mut xfmark_T = ::core::ptr::null_mut::<xfmark_T>();
    if global_busy.get() != 0
        || listcmd_busy.get() as ::core::ffi::c_int != 0
        || (*cmdmod.ptr()).cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int != 0
    {
        return;
    }
    (*curwin.get()).w_prev_pcmark = (*curwin.get()).w_pcmark;
    (*curwin.get()).w_pcmark = (*curwin.get()).w_cursor;
    if (*curwin.get()).w_pcmark.lnum == 0 as linenr_T {
        (*curwin.get()).w_pcmark.lnum = 1 as ::core::ffi::c_int as linenr_T;
    }
    if jop_flags.get() & kOptJopFlagStack as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if (*curwin.get()).w_jumplistidx < (*curwin.get()).w_jumplistlen - 1 as ::core::ffi::c_int {
            (*curwin.get()).w_jumplistlen = (*curwin.get()).w_jumplistidx + 1 as ::core::ffi::c_int;
        }
    }
    (*curwin.get()).w_jumplistlen += 1;
    if (*curwin.get()).w_jumplistlen > JUMPLISTSIZE {
        (*curwin.get()).w_jumplistlen = JUMPLISTSIZE;
        free_xfmark((*curwin.get()).w_jumplist[0 as ::core::ffi::c_int as usize]);
        memmove(
            (&raw mut (*curwin.get()).w_jumplist as *mut xfmark_T)
                .offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            (&raw mut (*curwin.get()).w_jumplist as *mut xfmark_T)
                .offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            ((JUMPLISTSIZE - 1 as ::core::ffi::c_int) as size_t)
                .wrapping_mul(::core::mem::size_of::<xfmark_T>()),
        );
    }
    (*curwin.get()).w_jumplistidx = (*curwin.get()).w_jumplistlen;
    fm = (&raw mut (*curwin.get()).w_jumplist as *mut xfmark_T)
        .offset(((*curwin.get()).w_jumplistlen - 1 as ::core::ffi::c_int) as isize);
    let mut view: fmarkv_T = mark_view_make(curwin.get(), (*curwin.get()).w_pcmark);
    let xfmarkp__: *mut xfmark_T = fm;
    (*xfmarkp__).fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let fmarkp__: *mut fmark_T = &raw mut (*xfmarkp__).fmark;
    (*fmarkp__).mark = (*curwin.get()).w_pcmark;
    (*fmarkp__).fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
    (*fmarkp__).timestamp = os_time();
    (*fmarkp__).view = view;
    (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
}
pub unsafe extern "C" fn checkpcmark() {
    if (*curwin.get()).w_prev_pcmark.lnum != 0 as linenr_T
        && (equalpos((*curwin.get()).w_pcmark, (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
            || (*curwin.get()).w_pcmark.lnum == 0 as linenr_T)
    {
        (*curwin.get()).w_pcmark = (*curwin.get()).w_prev_pcmark;
    }
    (*curwin.get()).w_prev_pcmark.lnum = 0 as ::core::ffi::c_int as linenr_T;
}
pub unsafe extern "C" fn get_jumplist(
    mut win: *mut win_T,
    mut count: ::core::ffi::c_int,
) -> *mut fmark_T {
    let mut jmp: *mut xfmark_T = ::core::ptr::null_mut::<xfmark_T>();
    cleanup_jumplist(win, true_0 != 0);
    if (*win).w_jumplistlen == 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<fmark_T>();
    }
    loop {
        if (*win).w_jumplistidx + count < 0 as ::core::ffi::c_int
            || (*win).w_jumplistidx + count >= (*win).w_jumplistlen
        {
            return ::core::ptr::null_mut::<fmark_T>();
        }
        if (*win).w_jumplistidx == (*win).w_jumplistlen {
            setpcmark();
            (*win).w_jumplistidx -= 1;
            if (*win).w_jumplistidx + count < 0 as ::core::ffi::c_int {
                return ::core::ptr::null_mut::<fmark_T>();
            }
        }
        (*win).w_jumplistidx += count;
        jmp = (&raw mut (*win).w_jumplist as *mut xfmark_T).offset((*win).w_jumplistidx as isize);
        if (*jmp).fmark.fnum == 0 as ::core::ffi::c_int {
            fname2fnum(jmp);
        }
        if (*jmp).fmark.fnum == (*curbuf.get()).handle {
            break;
        }
        if !buflist_findnr((*jmp).fmark.fnum).is_null() {
            break;
        }
        count += if count < 0 as ::core::ffi::c_int {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
    return &raw mut (*jmp).fmark;
}
pub unsafe extern "C" fn get_changelist(
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut count: ::core::ffi::c_int,
) -> *mut fmark_T {
    let mut n: ::core::ffi::c_int = 0;
    let mut fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    if (*buf).b_changelistlen == 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<fmark_T>();
    }
    n = (*win).w_changelistidx;
    if n + count < 0 as ::core::ffi::c_int {
        if n == 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<fmark_T>();
        }
        n = 0 as ::core::ffi::c_int;
    } else if n + count >= (*buf).b_changelistlen {
        if n == (*buf).b_changelistlen - 1 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<fmark_T>();
        }
        n = (*buf).b_changelistlen - 1 as ::core::ffi::c_int;
    } else {
        n += count;
    }
    (*win).w_changelistidx = n;
    fm = (&raw mut (*buf).b_changelist as *mut fmark_T).offset(n as isize);
    (*fm).fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
    return (&raw mut (*buf).b_changelist as *mut fmark_T).offset(n as isize);
}
#[no_mangle]
pub unsafe extern "C" fn mark_get(
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut fmp: *mut fmark_T,
    mut flag: MarkGet,
    mut name: ::core::ffi::c_int,
) -> *mut fmark_T {
    let mut fm: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    if name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || ascii_isdigit(name) as ::core::ffi::c_int != 0
    {
        let mut xfm: *mut xfmark_T = mark_get_global(
            flag as ::core::ffi::c_uint
                != kMarkAllNoResolve as ::core::ffi::c_int as ::core::ffi::c_uint,
            name,
        );
        fm = &raw mut (*xfm).fmark;
        if flag as ::core::ffi::c_uint == kMarkBufLocal as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*xfm).fmark.fnum != (*buf).handle
        {
            return pos_to_mark(
                buf,
                ::core::ptr::null_mut::<fmark_T>(),
                pos_T {
                    lnum: 0 as linenr_T,
                    col: 0,
                    coladd: 0,
                },
            );
        }
    } else if name > 0 as ::core::ffi::c_int && name < NMARK_LOCAL_MAX {
        fm = mark_get_local(buf, win, name);
    }
    if !fmp.is_null() && !fm.is_null() {
        *fmp = *fm;
        return fmp;
    }
    return fm;
}
pub unsafe extern "C" fn mark_get_global(
    mut resolve: bool,
    mut name: ::core::ffi::c_int,
) -> *mut xfmark_T {
    let mut mark: *mut xfmark_T = ::core::ptr::null_mut::<xfmark_T>();
    if ascii_isdigit(name) {
        name = name - '0' as ::core::ffi::c_int + NMARKS;
    } else if name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        name -= 'A' as ::core::ffi::c_int;
    } else {
        '_c2rust_label: {
            __assert_fail(
                b"false\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                453 as ::core::ffi::c_uint,
                b"xfmark_T *mark_get_global(_Bool, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        };
    }
    mark = (namedfm.ptr() as *mut xfmark_T).offset(name as isize);
    if resolve as ::core::ffi::c_int != 0 && (*mark).fmark.fnum == 0 as ::core::ffi::c_int {
        fname2fnum(mark);
    }
    return mark;
}
pub unsafe extern "C" fn mark_get_local(
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut name: ::core::ffi::c_int,
) -> *mut fmark_T {
    let mut mark: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    if name as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
        && name as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        mark = (&raw mut (*buf).b_namedm as *mut fmark_T)
            .offset((name - 'a' as ::core::ffi::c_int) as isize);
    } else if name == '[' as ::core::ffi::c_int {
        mark = pos_to_mark(buf, ::core::ptr::null_mut::<fmark_T>(), (*buf).b_op_start);
    } else if name == ']' as ::core::ffi::c_int {
        mark = pos_to_mark(buf, ::core::ptr::null_mut::<fmark_T>(), (*buf).b_op_end);
    } else if name == '<' as ::core::ffi::c_int || name == '>' as ::core::ffi::c_int {
        mark = mark_get_visual(buf, name);
    } else if name == '\'' as ::core::ffi::c_int || name == '`' as ::core::ffi::c_int {
        mark = pos_to_mark(
            curbuf.get(),
            ::core::ptr::null_mut::<fmark_T>(),
            (*win).w_pcmark,
        );
    } else if name == '"' as ::core::ffi::c_int {
        mark = &raw mut (*buf).b_last_cursor;
    } else if name == '^' as ::core::ffi::c_int {
        mark = &raw mut (*buf).b_last_insert;
    } else if name == '.' as ::core::ffi::c_int {
        mark = &raw mut (*buf).b_last_change;
    } else if name == ':' as ::core::ffi::c_int && bt_prompt(buf) as ::core::ffi::c_int != 0 {
        mark = &raw mut (*buf).b_prompt_start;
    } else {
        mark = mark_get_motion(buf, win, name);
    }
    if !mark.is_null() {
        (*mark).fnum = (*buf).handle as ::core::ffi::c_int;
    }
    return mark;
}
pub unsafe extern "C" fn mark_get_motion(
    mut buf: *mut buf_T,
    mut win: *mut win_T,
    mut name: ::core::ffi::c_int,
) -> *mut fmark_T {
    let mut mark: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    let pos: pos_T = (*curwin.get()).w_cursor;
    let slcb: bool = listcmd_busy.get();
    listcmd_busy.set(true_0 != 0);
    if name == '{' as ::core::ffi::c_int || name == '}' as ::core::ffi::c_int {
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
        if findpar(
            &raw mut oa.inclusive,
            if name == '}' as ::core::ffi::c_int {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            },
            1 as ::core::ffi::c_int,
            NUL,
            false_0 != 0,
        ) {
            mark = pos_to_mark(buf, ::core::ptr::null_mut::<fmark_T>(), (*win).w_cursor);
        }
    } else if name == '(' as ::core::ffi::c_int || name == ')' as ::core::ffi::c_int {
        if findsent(
            (if name == ')' as ::core::ffi::c_int {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            }) as Direction,
            1 as ::core::ffi::c_int,
        ) != 0
        {
            mark = pos_to_mark(buf, ::core::ptr::null_mut::<fmark_T>(), (*win).w_cursor);
        }
    }
    (*curwin.get()).w_cursor = pos;
    listcmd_busy.set(slcb);
    return mark;
}
pub unsafe extern "C" fn mark_get_visual(
    mut buf: *mut buf_T,
    mut name: ::core::ffi::c_int,
) -> *mut fmark_T {
    let mut mark: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    if name == '<' as ::core::ffi::c_int || name == '>' as ::core::ffi::c_int {
        let mut startp: pos_T = (*buf).b_visual.vi_start;
        let mut endp: pos_T = (*buf).b_visual.vi_end;
        if ((name == '<' as ::core::ffi::c_int) as ::core::ffi::c_int
            == lt(startp, endp) as ::core::ffi::c_int
            || endp.lnum == 0 as linenr_T)
            && startp.lnum != 0 as linenr_T
        {
            mark = pos_to_mark(buf, ::core::ptr::null_mut::<fmark_T>(), startp);
        } else {
            mark = pos_to_mark(buf, ::core::ptr::null_mut::<fmark_T>(), endp);
        }
        if (*buf).b_visual.vi_mode == 'V' as ::core::ffi::c_int {
            if name == '<' as ::core::ffi::c_int {
                (*mark).mark.col = 0 as ::core::ffi::c_int as colnr_T;
            } else {
                (*mark).mark.col = MAXCOL as ::core::ffi::c_int as colnr_T;
            }
            (*mark).mark.coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
    }
    return mark;
}
pub unsafe extern "C" fn pos_to_mark(
    mut buf: *mut buf_T,
    mut fmp: *mut fmark_T,
    mut pos: pos_T,
) -> *mut fmark_T {
    static fms: GlobalCell<fmark_T> = GlobalCell::new(fmark_T {
        mark: pos_T {
            lnum: 0 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        },
        fnum: 0 as ::core::ffi::c_int,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    });
    let mut fm: *mut fmark_T = if fmp.is_null() { fms.ptr() } else { fmp };
    (*fm).fnum = (*buf).handle as ::core::ffi::c_int;
    (*fm).mark = pos;
    return fm;
}
unsafe extern "C" fn switch_to_mark_buf(
    mut fm: *mut fmark_T,
    mut pcmark_on_switch: bool,
) -> MarkMoveRes {
    if (*fm).fnum != (*curbuf.get()).handle {
        let mut getfile_flag: ::core::ffi::c_int = if pcmark_on_switch as ::core::ffi::c_int != 0 {
            GETF_SETMARK as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        let mut res: bool =
            buflist_getfile((*fm).fnum, (*fm).mark.lnum, getfile_flag, false_0) == OK;
        return (if res as ::core::ffi::c_int == true_0 {
            kMarkSwitchedBuf as ::core::ffi::c_int
        } else {
            kMarkMoveFailed as ::core::ffi::c_int
        }) as MarkMoveRes;
    }
    return 0 as MarkMoveRes;
}
pub unsafe extern "C" fn mark_move_to(mut fm: *mut fmark_T, mut flags: MarkMove) -> MarkMoveRes {
    let mut prev_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    static fm_copy: GlobalCell<fmark_T> = GlobalCell::new(fmark_T {
        mark: pos_T {
            lnum: 0 as linenr_T,
            col: 0 as colnr_T,
            coladd: 0 as colnr_T,
        },
        fnum: 0 as ::core::ffi::c_int,
        timestamp: 0 as Timestamp,
        view: fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    });
    let mut res: MarkMoveRes = kMarkMoveSuccess;
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    '_end: {
        if !mark_check(fm, &raw mut errormsg) {
            if !errormsg.is_null() {
                emsg(errormsg);
            }
            res = kMarkMoveFailed;
        } else {
            if (*fm).fnum != (*curbuf.get()).handle {
                fm_copy.set(*fm);
                fm = fm_copy.ptr();
                res = (res as ::core::ffi::c_uint
                    | switch_to_mark_buf(
                        fm,
                        flags as ::core::ffi::c_uint
                            & kMarkJumpList as ::core::ffi::c_int as ::core::ffi::c_uint
                            == 0,
                    ) as ::core::ffi::c_uint) as MarkMoveRes;
                if res as ::core::ffi::c_uint
                    & kMarkMoveFailed as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                {
                    break '_end;
                } else if !mark_check_line_bounds(curbuf.get(), fm, &raw mut errormsg) {
                    if !errormsg.is_null() {
                        emsg(errormsg);
                    }
                    res = (res as ::core::ffi::c_uint
                        | kMarkMoveFailed as ::core::ffi::c_int as ::core::ffi::c_uint)
                        as MarkMoveRes;
                    break '_end;
                }
            } else if flags as ::core::ffi::c_uint
                & kMarkContext as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
            {
                setpcmark();
            }
            prev_pos = (*curwin.get()).w_cursor;
            pos = (*fm).mark;
            (*curwin.get()).w_cursor = (*fm).mark;
            if flags as ::core::ffi::c_uint
                & kMarkBeginLine as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
            {
                beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
            }
            res = (if prev_pos.lnum != pos.lnum {
                res as ::core::ffi::c_uint
                    | kMarkChangedLine as ::core::ffi::c_int as ::core::ffi::c_uint
                    | kMarkChangedCursor as ::core::ffi::c_int as ::core::ffi::c_uint
            } else {
                res as ::core::ffi::c_uint
            }) as MarkMoveRes;
            res = (if prev_pos.col != pos.col {
                res as ::core::ffi::c_uint
                    | kMarkChangedCol as ::core::ffi::c_int as ::core::ffi::c_uint
                    | kMarkChangedCursor as ::core::ffi::c_int as ::core::ffi::c_uint
            } else {
                res as ::core::ffi::c_uint
            }) as MarkMoveRes;
            if flags as ::core::ffi::c_uint
                & kMarkSetView as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
            {
                mark_view_restore(fm);
            }
            if res as ::core::ffi::c_uint
                & kMarkSwitchedBuf as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
                || res as ::core::ffi::c_uint
                    & kMarkChangedCursor as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
            {
                check_cursor(curwin.get());
            }
        }
    }
    return res;
}
pub unsafe extern "C" fn mark_view_restore(mut fm: *mut fmark_T) {
    if !fm.is_null() && (*fm).view.topline_offset >= 0 as linenr_T {
        let mut topline: linenr_T = (*fm).mark.lnum - (*fm).view.topline_offset;
        if topline >= 1 as linenr_T {
            set_topline(curwin.get(), topline);
            (*curwin.get()).w_skipcol = (if (*fm).view.skipcol > 0 as ::core::ffi::c_int
                && !hasFolding(
                    curwin.get(),
                    topline,
                    ::core::ptr::null_mut::<linenr_T>(),
                    ::core::ptr::null_mut::<linenr_T>(),
                )
                && (*fm).view.skipcol < linetabsize_eol(curwin.get(), topline)
            {
                (*fm).view.skipcol as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
        }
    }
}
pub unsafe extern "C" fn mark_view_make(mut wp: *const win_T, mut pos: pos_T) -> fmarkv_T {
    return fmarkv_T {
        topline_offset: pos.lnum - (*wp).w_topline,
        skipcol: (*wp).w_skipcol,
    };
}
pub unsafe extern "C" fn getnextmark(
    mut startpos: *mut pos_T,
    mut dir: ::core::ffi::c_int,
    mut begin_line: ::core::ffi::c_int,
) -> *mut fmark_T {
    let mut result: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    let mut pos: pos_T = *startpos;
    if dir == BACKWARD as ::core::ffi::c_int && begin_line != 0 {
        pos.col = 0 as ::core::ffi::c_int as colnr_T;
    } else if dir == FORWARD as ::core::ffi::c_int && begin_line != 0 {
        pos.col = MAXCOL as ::core::ffi::c_int as colnr_T;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NMARKS {
        if (*curbuf.get()).b_namedm[i as usize].mark.lnum > 0 as linenr_T {
            if dir == FORWARD as ::core::ffi::c_int {
                if (result.is_null()
                    || lt((*curbuf.get()).b_namedm[i as usize].mark, (*result).mark)
                        as ::core::ffi::c_int
                        != 0)
                    && lt(pos, (*curbuf.get()).b_namedm[i as usize].mark) as ::core::ffi::c_int != 0
                {
                    result = (&raw mut (*curbuf.get()).b_namedm as *mut fmark_T).offset(i as isize);
                }
            } else if (result.is_null()
                || lt((*result).mark, (*curbuf.get()).b_namedm[i as usize].mark)
                    as ::core::ffi::c_int
                    != 0)
                && lt((*curbuf.get()).b_namedm[i as usize].mark, pos) as ::core::ffi::c_int != 0
            {
                result = (&raw mut (*curbuf.get()).b_namedm as *mut fmark_T).offset(i as isize);
            }
        }
        i += 1;
    }
    return result;
}
unsafe extern "C" fn fname2fnum(mut fm: *mut xfmark_T) {
    if (*fm).fname.is_null() {
        return;
    }
    if *(*fm).fname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '~' as ::core::ffi::c_int
        && vim_ispathsep_nocolon(
            *(*fm).fname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        ) as ::core::ffi::c_int
            != 0
    {
        let mut len: size_t = expand_env(
            b"~/\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            MAXPATHL,
        );
        xstrlcpy(
            (NameBuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
            (*fm).fname.offset(2 as ::core::ffi::c_int as isize),
            (MAXPATHL as size_t).wrapping_sub(len),
        );
    } else {
        xstrlcpy(
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            (*fm).fname,
            MAXPATHL as size_t,
        );
    }
    os_dirname(IObuff.ptr() as *mut ::core::ffi::c_char, IOSIZE as size_t);
    let mut p: *mut ::core::ffi::c_char = path_shorten_fname(
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        IObuff.ptr() as *mut ::core::ffi::c_char,
    );
    buflist_new(
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        p,
        1 as linenr_T,
        0 as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn fmarks_check_names(mut buf: *mut buf_T) {
    let mut name: *mut ::core::ffi::c_char = (*buf).b_ffname;
    if (*buf).b_ffname.is_null() {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NGLOBALMARKS {
        fmarks_check_one(
            (namedfm.ptr() as *mut xfmark_T).offset(i as isize),
            name,
            buf,
        );
        i += 1;
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*wp).w_jumplistlen {
            fmarks_check_one(
                (&raw mut (*wp).w_jumplist as *mut xfmark_T).offset(i_0 as isize),
                name,
                buf,
            );
            i_0 += 1;
        }
        wp = (*wp).w_next;
    }
}
unsafe extern "C" fn fmarks_check_one(
    mut fm: *mut xfmark_T,
    mut name: *mut ::core::ffi::c_char,
    mut buf: *mut buf_T,
) {
    if (*fm).fmark.fnum == 0 as ::core::ffi::c_int
        && !(*fm).fname.is_null()
        && path_fnamecmp(name, (*fm).fname) == 0 as ::core::ffi::c_int
    {
        (*fm).fmark.fnum = (*buf).handle as ::core::ffi::c_int;
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*fm).fname as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
}
pub unsafe extern "C" fn mark_check(
    mut fm: *mut fmark_T,
    mut errormsg: *mut *const ::core::ffi::c_char,
) -> bool {
    if fm.is_null() {
        *errormsg = gettext(&raw const e_umark as *const ::core::ffi::c_char);
        return false_0 != 0;
    } else if (*fm).mark.lnum <= 0 as linenr_T {
        if (*fm).mark.lnum == 0 as linenr_T {
            *errormsg = gettext(&raw const e_marknotset as *const ::core::ffi::c_char);
        }
        return false_0 != 0;
    }
    if (*fm).fnum == (*curbuf.get()).handle && !mark_check_line_bounds(curbuf.get(), fm, errormsg) {
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn mark_check_line_bounds(
    mut buf: *mut buf_T,
    mut fm: *mut fmark_T,
    mut errormsg: *mut *const ::core::ffi::c_char,
) -> bool {
    if !buf.is_null() && (*fm).mark.lnum > (*buf).b_ml.ml_line_count {
        *errormsg = gettext(&raw const e_markinval as *const ::core::ffi::c_char);
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn clrallmarks(buf: *mut buf_T, timestamp: Timestamp) {
    let mut i: size_t = 0 as size_t;
    while i < NMARKS as size_t {
        clear_fmark(
            (&raw mut (*buf).b_namedm as *mut fmark_T).offset(i as isize),
            timestamp,
        );
        i = i.wrapping_add(1);
    }
    clear_fmark(&raw mut (*buf).b_last_cursor, timestamp);
    (*buf).b_last_cursor.mark.lnum = 1 as ::core::ffi::c_int as linenr_T;
    clear_fmark(&raw mut (*buf).b_last_insert, timestamp);
    clear_fmark(&raw mut (*buf).b_last_change, timestamp);
    (*buf).b_op_start.lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*buf).b_op_end.lnum = 0 as ::core::ffi::c_int as linenr_T;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*buf).b_changelistlen {
        clear_fmark(
            (&raw mut (*buf).b_changelist as *mut fmark_T).offset(i_0 as isize),
            timestamp,
        );
        i_0 += 1;
    }
    (*buf).b_changelistlen = 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn fm_getname(
    mut fmark: *mut fmark_T,
    mut lead_len: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if (*fmark).fnum == (*curbuf.get()).handle {
        return mark_line(&raw mut (*fmark).mark, lead_len);
    }
    return buflist_nr2name((*fmark).fnum, false_0, true_0);
}
unsafe extern "C" fn mark_line(
    mut mp: *mut pos_T,
    mut lead_len: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*mp).lnum == 0 as linenr_T || (*mp).lnum > (*curbuf.get()).b_ml.ml_line_count {
        return xstrdup(b"-invalid-\0".as_ptr() as *const ::core::ffi::c_char);
    }
    '_c2rust_label: {
        if Columns.get() >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"Columns >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                896 as ::core::ffi::c_uint,
                b"char *mark_line(pos_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut s: *mut ::core::ffi::c_char = xstrnsave(
        skipwhite(ml_get((*mp).lnum)),
        (Columns.get() as size_t).wrapping_mul(5 as size_t),
    );
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    p = s;
    while *p as ::core::ffi::c_int != NUL {
        len += ptr2cells(p);
        if len >= Columns.get() - lead_len {
            break;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    *p = NUL as ::core::ffi::c_char;
    return s;
}
pub unsafe extern "C" fn ex_marks(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut posp: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    if !arg.is_null() && *arg as ::core::ffi::c_int == NUL {
        arg = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    show_one_mark(
        '\'' as ::core::ffi::c_int,
        arg,
        &raw mut (*curwin.get()).w_pcmark,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NMARKS {
        show_one_mark(
            i + 'a' as ::core::ffi::c_int,
            arg,
            &raw mut (*(&raw mut (*curbuf.get()).b_namedm as *mut fmark_T).offset(i as isize)).mark,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0,
        );
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < NGLOBALMARKS {
        if (*namedfm.ptr())[i_0 as usize].fmark.fnum != 0 as ::core::ffi::c_int {
            name = fm_getname(
                &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(i_0 as isize)).fmark,
                15 as ::core::ffi::c_int,
            );
        } else {
            name = (*namedfm.ptr())[i_0 as usize].fname;
        }
        if !name.is_null() {
            show_one_mark(
                if i_0 >= NMARKS {
                    i_0 - NMARKS + '0' as ::core::ffi::c_int
                } else {
                    i_0 + 'A' as ::core::ffi::c_int
                },
                arg,
                &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(i_0 as isize))
                    .fmark
                    .mark,
                name,
                ((*namedfm.ptr())[i_0 as usize].fmark.fnum == (*curbuf.get()).handle)
                    as ::core::ffi::c_int,
            );
            if (*namedfm.ptr())[i_0 as usize].fmark.fnum != 0 as ::core::ffi::c_int {
                xfree(name as *mut ::core::ffi::c_void);
            }
        }
        i_0 += 1;
    }
    show_one_mark(
        '"' as ::core::ffi::c_int,
        arg,
        &raw mut (*curbuf.get()).b_last_cursor.mark,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0,
    );
    show_one_mark(
        '[' as ::core::ffi::c_int,
        arg,
        &raw mut (*curbuf.get()).b_op_start,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0,
    );
    show_one_mark(
        ']' as ::core::ffi::c_int,
        arg,
        &raw mut (*curbuf.get()).b_op_end,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0,
    );
    show_one_mark(
        '^' as ::core::ffi::c_int,
        arg,
        &raw mut (*curbuf.get()).b_last_insert.mark,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0,
    );
    show_one_mark(
        '.' as ::core::ffi::c_int,
        arg,
        &raw mut (*curbuf.get()).b_last_change.mark,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0,
    );
    if bt_prompt(curbuf.get()) {
        show_one_mark(
            ':' as ::core::ffi::c_int,
            arg,
            &raw mut (*curbuf.get()).b_prompt_start.mark,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0,
        );
    }
    let mut startp: *mut pos_T = &raw mut (*curbuf.get()).b_visual.vi_start;
    let mut endp: *mut pos_T = &raw mut (*curbuf.get()).b_visual.vi_end;
    if (lt(*startp, *endp) as ::core::ffi::c_int != 0 || (*endp).lnum == 0 as linenr_T)
        && (*startp).lnum != 0 as linenr_T
    {
        posp = startp;
    } else {
        posp = endp;
    }
    show_one_mark(
        '<' as ::core::ffi::c_int,
        arg,
        posp,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0,
    );
    show_one_mark(
        '>' as ::core::ffi::c_int,
        arg,
        if posp == startp { endp } else { startp },
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0,
    );
    show_one_mark(
        -1 as ::core::ffi::c_int,
        arg,
        ::core::ptr::null_mut::<pos_T>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0,
    );
}
unsafe extern "C" fn show_one_mark(
    mut c: ::core::ffi::c_int,
    mut arg: *mut ::core::ffi::c_char,
    mut p: *mut pos_T,
    mut name_arg: *mut ::core::ffi::c_char,
    mut current: ::core::ffi::c_int,
) {
    static did_title: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut mustfree: bool = false_0 != 0;
    let mut name: *mut ::core::ffi::c_char = name_arg;
    if c == -1 as ::core::ffi::c_int {
        if did_title.get() {
            did_title.set(false_0 != 0);
        } else if arg.is_null() {
            msg(
                gettext(b"No marks set\0".as_ptr() as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
        } else {
            semsg(
                gettext(b"E283: No marks matching \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                arg,
            );
        }
    } else if !got_int.get()
        && (arg.is_null() || !vim_strchr(arg, c).is_null())
        && (*p).lnum != 0 as linenr_T
    {
        if name.is_null() && current != 0 {
            name = mark_line(p, 15 as ::core::ffi::c_int);
            mustfree = true_0 != 0;
        }
        if !message_filtered(name) {
            if !did_title.get() {
                msg_puts_title(gettext(
                    b"\nmark line  col file/text\0".as_ptr() as *const ::core::ffi::c_char
                ));
                did_title.set(true_0 != 0);
            }
            msg_putchar('\n' as ::core::ffi::c_int);
            if !got_int.get() {
                snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b" %c %6d %4d \0".as_ptr() as *const ::core::ffi::c_char,
                    c,
                    (*p).lnum,
                    (*p).col,
                );
                msg_outtrans(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                if !name.is_null() {
                    msg_outtrans(
                        name,
                        if current != 0 {
                            HLF_D as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        },
                        false_0 != 0,
                    );
                }
            }
        }
        if mustfree {
            xfree(name as *mut ::core::ffi::c_void);
        }
    }
}
pub unsafe extern "C" fn ex_delmarks(mut eap: *mut exarg_T) {
    let mut from: ::core::ffi::c_int = 0;
    let mut to: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut pos: pos_T = pos_T {
        lnum: 0 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    };
    if *(*eap).arg as ::core::ffi::c_int == NUL && (*eap).forceit != 0 {
        let mut i: size_t = 0 as size_t;
        while i < NMARKS as size_t {
            if (*curbuf.get()).b_namedm[i as usize].mark.lnum != 0 as linenr_T {
                do_markset_autocmd(
                    i.wrapping_add('a' as size_t) as ::core::ffi::c_char,
                    &raw mut pos,
                    curbuf.get(),
                );
            }
            i = i.wrapping_add(1);
        }
        if (*curbuf.get()).b_last_cursor.mark.lnum != 0 as linenr_T {
            do_markset_autocmd('"' as ::core::ffi::c_char, &raw mut pos, curbuf.get());
        }
        if (*curbuf.get()).b_last_insert.mark.lnum != 0 as linenr_T {
            do_markset_autocmd('^' as ::core::ffi::c_char, &raw mut pos, curbuf.get());
        }
        if (*curbuf.get()).b_last_change.mark.lnum != 0 as linenr_T {
            do_markset_autocmd('.' as ::core::ffi::c_char, &raw mut pos, curbuf.get());
        }
        if (*curbuf.get()).b_op_start.lnum != 0 as linenr_T {
            do_markset_autocmd('[' as ::core::ffi::c_char, &raw mut pos, curbuf.get());
        }
        if (*curbuf.get()).b_op_end.lnum != 0 as linenr_T {
            do_markset_autocmd(']' as ::core::ffi::c_char, &raw mut pos, curbuf.get());
        }
        clrallmarks(curbuf.get(), os_time());
    } else if (*eap).forceit != 0 {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
    } else if *(*eap).arg as ::core::ffi::c_int == NUL {
        emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
    } else {
        let timestamp: Timestamp = os_time();
        let mut p: *mut ::core::ffi::c_char = (*eap).arg;
        while *p as ::core::ffi::c_int != NUL {
            let mut lower: bool = *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint;
            let mut digit: bool = ascii_isdigit(*p as ::core::ffi::c_int);
            if lower as ::core::ffi::c_int != 0
                || digit as ::core::ffi::c_int != 0
                || *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            {
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '-' as ::core::ffi::c_int
                {
                    from = *p as uint8_t as ::core::ffi::c_int;
                    to = *p.offset(2 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int;
                    if (if lower as ::core::ffi::c_int != 0 {
                        (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'a' as ::core::ffi::c_uint
                            && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'z' as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                    } else {
                        if digit as ::core::ffi::c_int != 0 {
                            ascii_isdigit(
                                *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                        } else {
                            (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                >= 'A' as ::core::ffi::c_uint
                                && *p.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_uint
                                    <= 'Z' as ::core::ffi::c_uint)
                                as ::core::ffi::c_int
                        }
                    }) == 0
                        || to < from
                    {
                        semsg(
                            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                            p,
                        );
                        return;
                    }
                    p = p.offset(2 as ::core::ffi::c_int as isize);
                } else {
                    to = *p as uint8_t as ::core::ffi::c_int;
                    from = to;
                }
                let mut i_0: ::core::ffi::c_int = from;
                while i_0 <= to {
                    if lower {
                        if (*curbuf.get()).b_namedm[(i_0 - 'a' as ::core::ffi::c_int) as usize]
                            .mark
                            .lnum
                            != 0 as linenr_T
                        {
                            do_markset_autocmd(
                                i_0 as ::core::ffi::c_char,
                                &raw mut pos,
                                curbuf.get(),
                            );
                        }
                        (*curbuf.get()).b_namedm[(i_0 - 'a' as ::core::ffi::c_int) as usize]
                            .mark
                            .lnum = 0 as ::core::ffi::c_int as linenr_T;
                        (*curbuf.get()).b_namedm[(i_0 - 'a' as ::core::ffi::c_int) as usize]
                            .timestamp = timestamp;
                    } else {
                        if digit {
                            n = i_0 - '0' as ::core::ffi::c_int + NMARKS;
                        } else {
                            n = i_0 - 'A' as ::core::ffi::c_int;
                        }
                        if (*namedfm.ptr())[n as usize].fmark.mark.lnum != 0 as linenr_T {
                            let mut buf: *mut buf_T =
                                buflist_findnr((*namedfm.ptr())[n as usize].fmark.fnum);
                            if buf.is_null() {
                                buf = curbuf.get();
                            }
                            do_markset_autocmd(i_0 as ::core::ffi::c_char, &raw mut pos, buf);
                        }
                        (*namedfm.ptr())[n as usize].fmark.mark.lnum =
                            0 as ::core::ffi::c_int as linenr_T;
                        (*namedfm.ptr())[n as usize].fmark.fnum = 0 as ::core::ffi::c_int;
                        (*namedfm.ptr())[n as usize].fmark.timestamp = timestamp;
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(n as isize)).fname
                                as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL;
                        let _ = *ptr_;
                    }
                    i_0 += 1;
                }
            } else {
                match *p as ::core::ffi::c_int {
                    34 => {
                        if (*curbuf.get()).b_last_cursor.mark.lnum != 0 as linenr_T {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        clear_fmark(&raw mut (*curbuf.get()).b_last_cursor, timestamp);
                    }
                    94 => {
                        if (*curbuf.get()).b_last_insert.mark.lnum != 0 as linenr_T {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        clear_fmark(&raw mut (*curbuf.get()).b_last_insert, timestamp);
                    }
                    46 => {
                        if (*curbuf.get()).b_last_change.mark.lnum != 0 as linenr_T {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        clear_fmark(&raw mut (*curbuf.get()).b_last_change, timestamp);
                    }
                    91 => {
                        if (*curbuf.get()).b_op_start.lnum != 0 as linenr_T {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        (*curbuf.get()).b_op_start.lnum = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    93 => {
                        if (*curbuf.get()).b_op_end.lnum != 0 as linenr_T {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        (*curbuf.get()).b_op_end.lnum = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    60 => {
                        if (*curbuf.get()).b_visual.vi_start.lnum != 0 as linenr_T {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        (*curbuf.get()).b_visual.vi_start.lnum =
                            0 as ::core::ffi::c_int as linenr_T;
                    }
                    62 => {
                        if (*curbuf.get()).b_visual.vi_end.lnum != 0 as linenr_T {
                            do_markset_autocmd(*p, &raw mut pos, curbuf.get());
                        }
                        (*curbuf.get()).b_visual.vi_end.lnum = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    58 | 32 => {}
                    _ => {
                        semsg(
                            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                            p,
                        );
                        return;
                    }
                }
            }
            p = p.offset(1);
        }
    };
}
pub unsafe extern "C" fn ex_jumps(mut _eap: *mut exarg_T) {
    cleanup_jumplist(curwin.get(), true_0 != 0);
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    msg_puts_title(gettext(
        b"\n jump line  col file/text\0".as_ptr() as *const ::core::ffi::c_char
    ));
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*curwin.get()).w_jumplistlen && !got_int.get() {
        if (*curwin.get()).w_jumplist[i as usize].fmark.mark.lnum != 0 as linenr_T {
            let mut name: *mut ::core::ffi::c_char = fm_getname(
                &raw mut (*(&raw mut (*curwin.get()).w_jumplist as *mut xfmark_T)
                    .offset(i as isize))
                .fmark,
                16 as ::core::ffi::c_int,
            );
            if name.is_null() && i == (*curwin.get()).w_jumplistidx {
                name = xstrdup(b"-invalid-\0".as_ptr() as *const ::core::ffi::c_char);
            }
            if name.is_null() || message_filtered(name) as ::core::ffi::c_int != 0 {
                xfree(name as *mut ::core::ffi::c_void);
            } else {
                msg_putchar('\n' as ::core::ffi::c_int);
                if got_int.get() {
                    xfree(name as *mut ::core::ffi::c_void);
                    break;
                } else {
                    snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        b"%c %2d %5d %4d \0".as_ptr() as *const ::core::ffi::c_char,
                        if i == (*curwin.get()).w_jumplistidx {
                            '>' as ::core::ffi::c_int
                        } else {
                            ' ' as ::core::ffi::c_int
                        },
                        if i > (*curwin.get()).w_jumplistidx {
                            i - (*curwin.get()).w_jumplistidx
                        } else {
                            (*curwin.get()).w_jumplistidx - i
                        },
                        (*curwin.get()).w_jumplist[i as usize].fmark.mark.lnum,
                        (*curwin.get()).w_jumplist[i as usize].fmark.mark.col,
                    );
                    msg_outtrans(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                    msg_outtrans(
                        name,
                        if (*curwin.get()).w_jumplist[i as usize].fmark.fnum
                            == (*curbuf.get()).handle
                        {
                            HLF_D as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        },
                        false_0 != 0,
                    );
                    xfree(name as *mut ::core::ffi::c_void);
                    os_breakcheck();
                }
            }
        }
        i += 1;
    }
    if (*curwin.get()).w_jumplistidx == (*curwin.get()).w_jumplistlen {
        msg_puts(b"\n>\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn ex_clearjumps(mut _eap: *mut exarg_T) {
    free_jumplist(curwin.get());
    (*curwin.get()).w_jumplistlen = 0 as ::core::ffi::c_int;
    (*curwin.get()).w_jumplistidx = 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ex_changes(mut _eap: *mut exarg_T) {
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    msg_puts_title(gettext(
        b"\nchange line  col text\0".as_ptr() as *const ::core::ffi::c_char
    ));
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*curbuf.get()).b_changelistlen && !got_int.get() {
        if (*curbuf.get()).b_changelist[i as usize].mark.lnum != 0 as linenr_T {
            msg_putchar('\n' as ::core::ffi::c_int);
            if got_int.get() {
                break;
            }
            snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%c %3d %5d %4d \0".as_ptr() as *const ::core::ffi::c_char,
                if i == (*curwin.get()).w_changelistidx {
                    '>' as ::core::ffi::c_int
                } else {
                    ' ' as ::core::ffi::c_int
                },
                if i > (*curwin.get()).w_changelistidx {
                    i - (*curwin.get()).w_changelistidx
                } else {
                    (*curwin.get()).w_changelistidx - i
                },
                (*curbuf.get()).b_changelist[i as usize].mark.lnum,
                (*curbuf.get()).b_changelist[i as usize].mark.col,
            );
            msg_outtrans(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
            let mut name: *mut ::core::ffi::c_char = mark_line(
                &raw mut (*(&raw mut (*curbuf.get()).b_changelist as *mut fmark_T)
                    .offset(i as isize))
                .mark,
                17 as ::core::ffi::c_int,
            );
            msg_outtrans(name, HLF_D as ::core::ffi::c_int, false_0 != 0);
            xfree(name as *mut ::core::ffi::c_void);
            os_breakcheck();
        }
        i += 1;
    }
    if (*curwin.get()).w_changelistidx == (*curbuf.get()).b_changelistlen {
        msg_puts(b"\n>\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn mark_adjust(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
    mut op: ExtmarkOp,
) {
    mark_adjust_buf(
        curbuf.get(),
        line1,
        line2,
        amount,
        amount_after,
        true_0 != 0,
        kMarkAdjustNormal,
        op,
    );
}
pub unsafe extern "C" fn mark_adjust_nofold(
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
    mut op: ExtmarkOp,
) {
    mark_adjust_buf(
        curbuf.get(),
        line1,
        line2,
        amount,
        amount_after,
        false_0 != 0,
        kMarkAdjustNormal,
        op,
    );
}
#[no_mangle]
pub unsafe extern "C" fn mark_adjust_buf(
    mut buf: *mut buf_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
    mut adjust_folds: bool,
    mut mode: MarkAdjustMode,
    mut op: ExtmarkOp,
) {
    let mut fnum: ::core::ffi::c_int = (*buf).handle as ::core::ffi::c_int;
    let mut lp: *mut linenr_T = ::core::ptr::null_mut::<linenr_T>();
    static initpos: GlobalCell<pos_T> = GlobalCell::new(pos_T {
        lnum: 1 as linenr_T,
        col: 0 as colnr_T,
        coladd: 0 as colnr_T,
    });
    if line2 < line1 && amount_after == 0 as linenr_T {
        return;
    }
    let mut by_api: bool =
        mode as ::core::ffi::c_uint == kMarkAdjustApi as ::core::ffi::c_int as ::core::ffi::c_uint;
    let mut by_term: bool =
        mode as ::core::ffi::c_uint == kMarkAdjustTerm as ::core::ffi::c_int as ::core::ffi::c_uint;
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < NMARKS {
            lp = &raw mut (*(&raw mut (*buf).b_namedm as *mut fmark_T).offset(i as isize))
                .mark
                .lnum;
            if *lp >= line1 && *lp <= line2 {
                if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                    *lp = 0 as ::core::ffi::c_int as linenr_T;
                } else {
                    *lp += amount;
                }
            } else if amount_after != 0 && *lp > line2 {
                *lp += amount_after;
            }
            if (*namedfm.ptr())[i as usize].fmark.fnum == fnum {
                lp = &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(i as isize))
                    .fmark
                    .mark
                    .lnum;
                if *lp >= line1 && *lp <= line2 {
                    if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                        *lp = line1;
                    } else {
                        *lp += amount;
                    }
                } else if amount_after != 0 && *lp > line2 {
                    *lp += amount_after;
                }
            }
            i += 1;
        }
        let mut i_0: ::core::ffi::c_int = NMARKS;
        while i_0 < NGLOBALMARKS {
            if (*namedfm.ptr())[i_0 as usize].fmark.fnum == fnum {
                lp = &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(i_0 as isize))
                    .fmark
                    .mark
                    .lnum;
                if *lp >= line1 && *lp <= line2 {
                    if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                        *lp = line1;
                    } else {
                        *lp += amount;
                    }
                } else if amount_after != 0 && *lp > line2 {
                    *lp += amount_after;
                }
            }
            i_0 += 1;
        }
        lp = &raw mut (*buf).b_last_insert.mark.lnum;
        if *lp >= line1 && *lp <= line2 {
            if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                *lp = 0 as ::core::ffi::c_int as linenr_T;
            } else {
                *lp += amount;
            }
        } else if amount_after != 0 && *lp > line2 {
            *lp += amount_after;
        }
        lp = &raw mut (*buf).b_last_change.mark.lnum;
        if *lp >= line1 && *lp <= line2 {
            if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                *lp = 0 as ::core::ffi::c_int as linenr_T;
            } else {
                *lp += amount;
            }
        } else if amount_after != 0 && *lp > line2 {
            *lp += amount_after;
        }
        if !equalpos((*buf).b_last_cursor.mark, initpos.get())
            && (!by_term || (*buf).b_last_cursor.mark.lnum < (*buf).b_ml.ml_line_count)
        {
            lp = &raw mut (*buf).b_last_cursor.mark.lnum;
            if *lp >= line1 && *lp <= line2 {
                if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                    *lp = 0 as ::core::ffi::c_int as linenr_T;
                } else {
                    *lp += amount;
                }
            } else if amount_after != 0 && *lp > line2 {
                *lp += amount_after;
            }
        }
        if bt_prompt(buf) {
            lp = &raw mut (*buf).b_prompt_start.mark.lnum;
            if *lp >= line1 && *lp <= line2 {
                if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                    *lp = line1;
                } else {
                    *lp += amount;
                }
            } else if amount_after != 0 && *lp > line2 {
                *lp += amount_after;
            }
        }
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < (*buf).b_changelistlen {
            lp = &raw mut (*(&raw mut (*buf).b_changelist as *mut fmark_T).offset(i_1 as isize))
                .mark
                .lnum;
            if *lp >= line1 && *lp <= line2 {
                if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                    *lp = line1;
                } else {
                    *lp += amount;
                }
            } else if amount_after != 0 && *lp > line2 {
                *lp += amount_after;
            }
            i_1 += 1;
        }
        lp = &raw mut (*buf).b_visual.vi_start.lnum;
        if *lp >= line1 && *lp <= line2 {
            if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                *lp = line1;
            } else {
                *lp += amount;
            }
        } else if amount_after != 0 && *lp > line2 {
            *lp += amount_after;
        }
        lp = &raw mut (*buf).b_visual.vi_end.lnum;
        if *lp >= line1 && *lp <= line2 {
            if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                *lp = line1;
            } else {
                *lp += amount;
            }
        } else if amount_after != 0 && *lp > line2 {
            *lp += amount_after;
        }
        if !qf_mark_adjust(
            buf,
            ::core::ptr::null_mut::<win_T>(),
            line1,
            line2,
            amount,
            amount_after,
        ) {
            (*buf).b_has_qf_entry &= !BUF_HAS_QF_ENTRY;
        }
        let mut found_one: bool = false_0 != 0;
        let mut tab: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tab.is_null() {
            let mut win: *mut win_T = if tab == curtab.get() {
                firstwin.get()
            } else {
                (*tab).tp_firstwin
            };
            while !win.is_null() {
                found_one = found_one as ::core::ffi::c_int
                    | qf_mark_adjust(buf, win, line1, line2, amount, amount_after)
                        as ::core::ffi::c_int
                    != 0;
                win = (*win).w_next;
            }
            tab = (*tab).tp_next as *mut tabpage_T;
        }
        if !found_one {
            (*buf).b_has_qf_entry &= !BUF_HAS_LL_ENTRY;
        }
    }
    if op as ::core::ffi::c_uint != kExtmarkNOOP as ::core::ffi::c_int as ::core::ffi::c_uint {
        extmark_adjust(buf, line1, line2, amount, amount_after, op);
    }
    if (*curwin.get()).w_buffer == buf {
        lp = &raw mut (*curwin.get()).w_pcmark.lnum;
        if *lp >= line1 && *lp <= line2 {
            if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                *lp = 0 as ::core::ffi::c_int as linenr_T;
            } else {
                *lp += amount;
            }
        } else if amount_after != 0 && *lp > line2 {
            *lp += amount_after;
        }
        lp = &raw mut (*curwin.get()).w_prev_pcmark.lnum;
        if *lp >= line1 && *lp <= line2 {
            if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                *lp = 0 as ::core::ffi::c_int as linenr_T;
            } else {
                *lp += amount;
            }
        } else if amount_after != 0 && *lp > line2 {
            *lp += amount_after;
        }
        if (*saved_cursor.ptr()).lnum != 0 as linenr_T {
            lp = &raw mut (*saved_cursor.ptr()).lnum;
            if *lp >= line1 && *lp <= line2 {
                if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                    *lp = line1;
                } else {
                    *lp += amount;
                }
            } else if amount_after != 0 && *lp > line2 {
                *lp += amount_after;
            }
        }
    }
    let mut tab_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tab_0.is_null() {
        let mut win_0: *mut win_T = if tab_0 == curtab.get() {
            firstwin.get()
        } else {
            (*tab_0).tp_firstwin
        };
        while !win_0.is_null() {
            if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_2 < (*win_0).w_jumplistlen {
                    if (*win_0).w_jumplist[i_2 as usize].fmark.fnum == fnum {
                        lp = &raw mut (*(&raw mut (*win_0).w_jumplist as *mut xfmark_T)
                            .offset(i_2 as isize))
                        .fmark
                        .mark
                        .lnum;
                        if *lp >= line1 && *lp <= line2 {
                            if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                                *lp = line1;
                            } else {
                                *lp += amount;
                            }
                        } else if amount_after != 0 && *lp > line2 {
                            *lp += amount_after;
                        }
                    }
                    i_2 += 1;
                }
            }
            if (*win_0).w_buffer == buf {
                if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
                {
                    let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i_3 < (*win_0).w_tagstacklen {
                        if (*win_0).w_tagstack[i_3 as usize].fmark.fnum == fnum {
                            lp = &raw mut (*(&raw mut (*win_0).w_tagstack as *mut taggy_T)
                                .offset(i_3 as isize))
                            .fmark
                            .mark
                            .lnum;
                            if *lp >= line1 && *lp <= line2 {
                                if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                                    *lp = line1;
                                } else {
                                    *lp += amount;
                                }
                            } else if amount_after != 0 && *lp > line2 {
                                *lp += amount_after;
                            }
                        }
                        i_3 += 1;
                    }
                }
                if (*win_0).w_old_cursor_lnum != 0 as linenr_T {
                    lp = &raw mut (*win_0).w_old_cursor_lnum;
                    if *lp >= line1 && *lp <= line2 {
                        if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                            *lp = line1;
                        } else {
                            *lp += amount;
                        }
                    } else if amount_after != 0 && *lp > line2 {
                        *lp += amount_after;
                    }
                    lp = &raw mut (*win_0).w_old_visual_lnum;
                    if *lp >= line1 && *lp <= line2 {
                        if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                            *lp = line1;
                        } else {
                            *lp += amount;
                        }
                    } else if amount_after != 0 && *lp > line2 {
                        *lp += amount_after;
                    }
                }
                if by_api as ::core::ffi::c_int != 0
                    || (if by_term as ::core::ffi::c_int != 0 {
                        ((*win_0).w_cursor.lnum < (*buf).b_ml.ml_line_count) as ::core::ffi::c_int
                    } else {
                        (win_0 != curwin.get()) as ::core::ffi::c_int
                    }) != 0
                {
                    if (*win_0).w_topline >= line1 && (*win_0).w_topline <= line2 {
                        if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                            if !(by_api as ::core::ffi::c_int != 0
                                && amount_after > line1 - line2 - 1 as linenr_T)
                            {
                                (*win_0).w_topline = if line1 - 1 as linenr_T > 1 as linenr_T {
                                    line1 - 1 as linenr_T
                                } else {
                                    1 as linenr_T
                                };
                            }
                        } else if (*win_0).w_topline > line1 {
                            (*win_0).w_topline += amount;
                        }
                        (*win_0).w_topfill = 0 as ::core::ffi::c_int;
                    } else if amount_after != 0
                        && (*win_0).w_topline
                            > line2
                                + (if by_api as ::core::ffi::c_int != 0 && line2 < line1 {
                                    1 as linenr_T
                                } else {
                                    0 as linenr_T
                                })
                    {
                        (*win_0).w_topline += amount_after;
                        (*win_0).w_topfill = 0 as ::core::ffi::c_int;
                    }
                }
                if !by_api
                    && (if by_term as ::core::ffi::c_int != 0 {
                        ((*win_0).w_cursor.lnum < (*buf).b_ml.ml_line_count) as ::core::ffi::c_int
                    } else {
                        (win_0 != curwin.get()) as ::core::ffi::c_int
                    }) != 0
                {
                    let mut posp: *mut pos_T = &raw mut (*win_0).w_cursor;
                    if (*posp).lnum >= line1 && (*posp).lnum <= line2 {
                        if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                            (*posp).lnum = if line1 - 1 as linenr_T > 1 as linenr_T {
                                line1 - 1 as linenr_T
                            } else {
                                1 as linenr_T
                            };
                            (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
                        } else {
                            (*posp).lnum += amount;
                        }
                    } else if amount_after != 0 && (*posp).lnum > line2 {
                        (*posp).lnum += amount_after;
                    }
                }
                if adjust_folds {
                    foldMarkAdjust(win_0, line1, line2, amount, amount_after);
                }
            }
            win_0 = (*win_0).w_next;
        }
        tab_0 = (*tab_0).tp_next as *mut tabpage_T;
    }
    diff_mark_adjust(buf, line1, line2, amount, amount_after);
    let mut i_4: size_t = 0 as size_t;
    while i_4 < (*buf).b_wininfo.size {
        let mut wip: *mut WinInfo = *(*buf).b_wininfo.items.offset(i_4 as isize);
        if !by_term || (*wip).wi_mark.mark.lnum < (*buf).b_ml.ml_line_count {
            let mut posp_0: *mut pos_T = &raw mut (*wip).wi_mark.mark;
            if (*posp_0).lnum >= line1 && (*posp_0).lnum <= line2 {
                if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                    (*posp_0).lnum = if line1 - 1 as linenr_T > 1 as linenr_T {
                        line1 - 1 as linenr_T
                    } else {
                        1 as linenr_T
                    };
                    (*posp_0).col = 0 as ::core::ffi::c_int as colnr_T;
                } else {
                    (*posp_0).lnum += amount;
                }
            } else if amount_after != 0 && (*posp_0).lnum > line2 {
                (*posp_0).lnum += amount_after;
            }
        }
        i_4 = i_4.wrapping_add(1);
    }
}
pub unsafe extern "C" fn mark_col_adjust(
    mut lnum: linenr_T,
    mut mincol: colnr_T,
    mut lnum_amount: linenr_T,
    mut col_amount: colnr_T,
    mut spaces_removed: ::core::ffi::c_int,
) {
    let mut fnum: ::core::ffi::c_int = (*curbuf.get()).handle as ::core::ffi::c_int;
    let mut posp: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    if col_amount == 0 as ::core::ffi::c_int && lnum_amount == 0 as linenr_T
        || (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0
    {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NMARKS {
        posp =
            &raw mut (*(&raw mut (*curbuf.get()).b_namedm as *mut fmark_T).offset(i as isize)).mark;
        if (*posp).lnum == lnum && (*posp).col >= mincol {
            (*posp).lnum += lnum_amount;
            '_c2rust_label: {
                if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                    && col_amount <= 2147483647 as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1499 as ::core::ffi::c_uint,
                        b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
                (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
            } else if (*posp).col < spaces_removed {
                (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
            } else {
                (*posp).col += col_amount;
            }
        }
        if (*namedfm.ptr())[i as usize].fmark.fnum == fnum {
            posp = &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(i as isize))
                .fmark
                .mark;
            if (*posp).lnum == lnum && (*posp).col >= mincol {
                (*posp).lnum += lnum_amount;
                '_c2rust_label_0: {
                    if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                        && col_amount <= 2147483647 as ::core::ffi::c_int
                    {
                    } else {
                        __assert_fail(
                            b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            1501 as ::core::ffi::c_uint,
                            b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
                    (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
                } else if (*posp).col < spaces_removed {
                    (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
                } else {
                    (*posp).col += col_amount;
                }
            }
        }
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = NMARKS;
    while i_0 < NGLOBALMARKS {
        if (*namedfm.ptr())[i_0 as usize].fmark.fnum == fnum {
            posp = &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(i_0 as isize))
                .fmark
                .mark;
            if (*posp).lnum == lnum && (*posp).col >= mincol {
                (*posp).lnum += lnum_amount;
                '_c2rust_label_1: {
                    if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                        && col_amount <= 2147483647 as ::core::ffi::c_int
                    {
                    } else {
                        __assert_fail(
                            b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            1506 as ::core::ffi::c_uint,
                            b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
                    (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
                } else if (*posp).col < spaces_removed {
                    (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
                } else {
                    (*posp).col += col_amount;
                }
            }
        }
        i_0 += 1;
    }
    posp = &raw mut (*curbuf.get()).b_last_insert.mark;
    if (*posp).lnum == lnum && (*posp).col >= mincol {
        (*posp).lnum += lnum_amount;
        '_c2rust_label_2: {
            if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                && col_amount <= 2147483647 as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1511 as ::core::ffi::c_uint,
                    b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
            (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
        } else if (*posp).col < spaces_removed {
            (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
        } else {
            (*posp).col += col_amount;
        }
    }
    posp = &raw mut (*curbuf.get()).b_last_change.mark;
    if (*posp).lnum == lnum && (*posp).col >= mincol {
        (*posp).lnum += lnum_amount;
        '_c2rust_label_3: {
            if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                && col_amount <= 2147483647 as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1514 as ::core::ffi::c_uint,
                    b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
            (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
        } else if (*posp).col < spaces_removed {
            (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
        } else {
            (*posp).col += col_amount;
        }
    }
    if bt_prompt(curbuf.get()) {
        posp = &raw mut (*curbuf.get()).b_prompt_start.mark;
        if (*posp).lnum == lnum && (*posp).col >= mincol {
            (*posp).lnum += lnum_amount;
            '_c2rust_label_4: {
                if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                    && col_amount <= 2147483647 as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1517 as ::core::ffi::c_uint,
                        b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
                (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
            } else if (*posp).col < spaces_removed {
                (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
            } else {
                (*posp).col += col_amount;
            }
        }
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < (*curbuf.get()).b_changelistlen {
        posp = &raw mut (*(&raw mut (*curbuf.get()).b_changelist as *mut fmark_T)
            .offset(i_1 as isize))
        .mark;
        if (*posp).lnum == lnum && (*posp).col >= mincol {
            (*posp).lnum += lnum_amount;
            '_c2rust_label_5: {
                if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                    && col_amount <= 2147483647 as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1522 as ::core::ffi::c_uint,
                        b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
                (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
            } else if (*posp).col < spaces_removed {
                (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
            } else {
                (*posp).col += col_amount;
            }
        }
        i_1 += 1;
    }
    posp = &raw mut (*curbuf.get()).b_visual.vi_start;
    if (*posp).lnum == lnum && (*posp).col >= mincol {
        (*posp).lnum += lnum_amount;
        '_c2rust_label_6: {
            if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                && col_amount <= 2147483647 as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1526 as ::core::ffi::c_uint,
                    b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
            (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
        } else if (*posp).col < spaces_removed {
            (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
        } else {
            (*posp).col += col_amount;
        }
    }
    posp = &raw mut (*curbuf.get()).b_visual.vi_end;
    if (*posp).lnum == lnum && (*posp).col >= mincol {
        (*posp).lnum += lnum_amount;
        '_c2rust_label_7: {
            if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                && col_amount <= 2147483647 as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1527 as ::core::ffi::c_uint,
                    b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
            (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
        } else if (*posp).col < spaces_removed {
            (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
        } else {
            (*posp).col += col_amount;
        }
    }
    posp = &raw mut (*curwin.get()).w_pcmark;
    if (*posp).lnum == lnum && (*posp).col >= mincol {
        (*posp).lnum += lnum_amount;
        '_c2rust_label_8: {
            if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                && col_amount <= 2147483647 as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1530 as ::core::ffi::c_uint,
                    b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
            (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
        } else if (*posp).col < spaces_removed {
            (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
        } else {
            (*posp).col += col_amount;
        }
    }
    posp = &raw mut (*curwin.get()).w_prev_pcmark;
    if (*posp).lnum == lnum && (*posp).col >= mincol {
        (*posp).lnum += lnum_amount;
        '_c2rust_label_9: {
            if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                && col_amount <= 2147483647 as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1533 as ::core::ffi::c_uint,
                    b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
            (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
        } else if (*posp).col < spaces_removed {
            (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
        } else {
            (*posp).col += col_amount;
        }
    }
    posp = saved_cursor.ptr();
    if (*posp).lnum == lnum && (*posp).col >= mincol {
        (*posp).lnum += lnum_amount;
        '_c2rust_label_10: {
            if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                && col_amount <= 2147483647 as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1536 as ::core::ffi::c_uint,
                    b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
            (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
        } else if (*posp).col < spaces_removed {
            (*posp).col = (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
        } else {
            (*posp).col += col_amount;
        }
    }
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_2 < (*win).w_jumplistlen {
            if (*win).w_jumplist[i_2 as usize].fmark.fnum == fnum {
                posp = &raw mut (*(&raw mut (*win).w_jumplist as *mut xfmark_T)
                    .offset(i_2 as isize))
                .fmark
                .mark;
                if (*posp).lnum == lnum && (*posp).col >= mincol {
                    (*posp).lnum += lnum_amount;
                    '_c2rust_label_11: {
                        if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                            && col_amount <= 2147483647 as ::core::ffi::c_int
                        {
                        } else {
                            __assert_fail(
                                b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                1543 as ::core::ffi::c_uint,
                                b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
                        (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
                    } else if (*posp).col < spaces_removed {
                        (*posp).col =
                            (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
                    } else {
                        (*posp).col += col_amount;
                    }
                }
            }
            i_2 += 1;
        }
        if (*win).w_buffer == curbuf.get() {
            let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_3 < (*win).w_tagstacklen {
                if (*win).w_tagstack[i_3 as usize].fmark.fnum == fnum {
                    posp = &raw mut (*(&raw mut (*win).w_tagstack as *mut taggy_T)
                        .offset(i_3 as isize))
                    .fmark
                    .mark;
                    if (*posp).lnum == lnum && (*posp).col >= mincol {
                        (*posp).lnum += lnum_amount;
                        '_c2rust_label_12: {
                            if col_amount
                                > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                && col_amount <= 2147483647 as ::core::ffi::c_int
                            {
                            } else {
                                __assert_fail(
                                    b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/mark.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    1551 as ::core::ffi::c_uint,
                                    b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
                            (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
                        } else if (*posp).col < spaces_removed {
                            (*posp).col =
                                (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
                        } else {
                            (*posp).col += col_amount;
                        }
                    }
                }
                i_3 += 1;
            }
            if win != curwin.get() {
                posp = &raw mut (*win).w_cursor;
                if (*posp).lnum == lnum && (*posp).col >= mincol {
                    (*posp).lnum += lnum_amount;
                    '_c2rust_label_13: {
                        if col_amount > -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                            && col_amount <= 2147483647 as ::core::ffi::c_int
                        {
                        } else {
                            __assert_fail(
                                b"col_amount > INT_MIN && col_amount <= INT_MAX\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/mark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                1557 as ::core::ffi::c_uint,
                                b"void mark_col_adjust(linenr_T, colnr_T, linenr_T, colnr_T, int)\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if col_amount < 0 as ::core::ffi::c_int && (*posp).col <= -col_amount {
                        (*posp).col = 0 as ::core::ffi::c_int as colnr_T;
                    } else if (*posp).col < spaces_removed {
                        (*posp).col =
                            (col_amount as ::core::ffi::c_int + spaces_removed) as colnr_T;
                    } else {
                        (*posp).col += col_amount;
                    }
                }
            }
        }
        win = (*win).w_next;
    }
}
pub unsafe extern "C" fn cleanup_jumplist(mut wp: *mut win_T, mut loadfiles: bool) {
    let mut i: ::core::ffi::c_int = 0;
    if loadfiles {
        i = 0 as ::core::ffi::c_int;
        while i < (*wp).w_jumplistlen {
            if (*wp).w_jumplist[i as usize].fmark.fnum == 0 as ::core::ffi::c_int
                && (*wp).w_jumplist[i as usize].fmark.mark.lnum != 0 as linenr_T
            {
                fname2fnum((&raw mut (*wp).w_jumplist as *mut xfmark_T).offset(i as isize));
            }
            i += 1;
        }
    }
    let mut to: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut from: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while from < (*wp).w_jumplistlen {
        if (*wp).w_jumplistidx == from {
            (*wp).w_jumplistidx = to;
        }
        i = from + 1 as ::core::ffi::c_int;
        while i < (*wp).w_jumplistlen {
            if (*wp).w_jumplist[i as usize].fmark.fnum == (*wp).w_jumplist[from as usize].fmark.fnum
                && (*wp).w_jumplist[from as usize].fmark.fnum != 0 as ::core::ffi::c_int
                && (*wp).w_jumplist[i as usize].fmark.mark.lnum
                    == (*wp).w_jumplist[from as usize].fmark.mark.lnum
            {
                break;
            }
            i += 1;
        }
        let mut mustfree: bool = false;
        if i >= (*wp).w_jumplistlen {
            mustfree = false_0 != 0;
        } else if i > from + 1 as ::core::ffi::c_int {
            mustfree = jop_flags.get()
                & kOptJopFlagStack as ::core::ffi::c_int as ::core::ffi::c_uint
                == 0;
        } else {
            mustfree = true_0 != 0;
        }
        if mustfree {
            xfree((*wp).w_jumplist[from as usize].fname as *mut ::core::ffi::c_void);
        } else {
            if to != from {
                (*wp).w_jumplist[to as usize] = (*wp).w_jumplist[from as usize];
            }
            to += 1;
        }
        from += 1;
    }
    if (*wp).w_jumplistidx == (*wp).w_jumplistlen {
        (*wp).w_jumplistidx = to;
    }
    (*wp).w_jumplistlen = to;
    if loadfiles as ::core::ffi::c_int != 0
        && (*wp).w_jumplistlen != 0
        && (*wp).w_jumplistidx == (*wp).w_jumplistlen
    {
        let mut fm_last: *const xfmark_T = (&raw mut (*wp).w_jumplist as *mut xfmark_T)
            .offset(((*wp).w_jumplistlen - 1 as ::core::ffi::c_int) as isize);
        if (*fm_last).fmark.fnum == (*curbuf.get()).handle
            && (*fm_last).fmark.mark.lnum == (*wp).w_cursor.lnum
        {
            xfree((*fm_last).fname as *mut ::core::ffi::c_void);
            (*wp).w_jumplistlen -= 1;
            (*wp).w_jumplistidx -= 1;
        }
    }
}
pub unsafe extern "C" fn copy_jumplist(mut from: *mut win_T, mut to: *mut win_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*from).w_jumplistlen {
        (*to).w_jumplist[i as usize] = (*from).w_jumplist[i as usize];
        if !(*from).w_jumplist[i as usize].fname.is_null() {
            (*to).w_jumplist[i as usize].fname = xstrdup((*from).w_jumplist[i as usize].fname);
        }
        i += 1;
    }
    (*to).w_jumplistlen = (*from).w_jumplistlen;
    (*to).w_jumplistidx = (*from).w_jumplistidx;
}
pub unsafe extern "C" fn mark_jumplist_iter(
    iter: *const ::core::ffi::c_void,
    win: *const win_T,
    fm: *mut xfmark_T,
) -> *const ::core::ffi::c_void {
    if iter.is_null() && (*win).w_jumplistlen == 0 as ::core::ffi::c_int {
        *fm = xfmark_T {
            fmark: fmark_T {
                mark: pos_T {
                    lnum: 0 as linenr_T,
                    col: 0 as colnr_T,
                    coladd: 0 as colnr_T,
                },
                fnum: 0 as ::core::ffi::c_int,
                timestamp: 0 as Timestamp,
                view: fmarkv_T {
                    topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
                    skipcol: 0 as colnr_T,
                },
                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
            },
            fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    let iter_mark: *const xfmark_T = if iter.is_null() {
        (&raw const (*win).w_jumplist as *const xfmark_T).offset(0 as ::core::ffi::c_int as isize)
    } else {
        iter as *const xfmark_T
    };
    *fm = *iter_mark;
    if iter_mark
        == (&raw const (*win).w_jumplist as *const xfmark_T)
            .offset(((*win).w_jumplistlen - 1 as ::core::ffi::c_int) as isize)
    {
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    return iter_mark.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void;
}
pub unsafe extern "C" fn mark_global_iter(
    iter: *const ::core::ffi::c_void,
    name: *mut ::core::ffi::c_char,
    fm: *mut xfmark_T,
) -> *const ::core::ffi::c_void {
    *name = NUL as ::core::ffi::c_char;
    let mut iter_mark: *const xfmark_T = if iter.is_null() {
        (namedfm.ptr() as *mut xfmark_T).offset(0 as ::core::ffi::c_int as isize) as *const xfmark_T
    } else {
        iter as *const xfmark_T
    };
    while (iter_mark
        .offset_from((namedfm.ptr() as *mut xfmark_T).offset(0 as ::core::ffi::c_int as isize))
        as size_t)
        < ::core::mem::size_of::<[xfmark_T; 36]>()
            .wrapping_div(::core::mem::size_of::<xfmark_T>())
            .wrapping_div(
                (::core::mem::size_of::<[xfmark_T; 36]>()
                    .wrapping_rem(::core::mem::size_of::<xfmark_T>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        && (*iter_mark).fmark.mark.lnum == 0
    {
        iter_mark = iter_mark.offset(1);
    }
    if iter_mark
        .offset_from((namedfm.ptr() as *mut xfmark_T).offset(0 as ::core::ffi::c_int as isize))
        as size_t
        == ::core::mem::size_of::<[xfmark_T; 36]>()
            .wrapping_div(::core::mem::size_of::<xfmark_T>())
            .wrapping_div(
                (::core::mem::size_of::<[xfmark_T; 36]>()
                    .wrapping_rem(::core::mem::size_of::<xfmark_T>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        || (*iter_mark).fmark.mark.lnum == 0
    {
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    let mut iter_off: size_t = iter_mark
        .offset_from((namedfm.ptr() as *mut xfmark_T).offset(0 as ::core::ffi::c_int as isize))
        as size_t;
    *name = (if iter_off < NMARKS as size_t {
        'A' as ::core::ffi::c_int + iter_off as ::core::ffi::c_char as ::core::ffi::c_int
    } else {
        '0' as ::core::ffi::c_int
            + iter_off.wrapping_sub(NMARKS as size_t) as ::core::ffi::c_char as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
    *fm = *iter_mark;
    loop {
        iter_mark = iter_mark.offset(1);
        if (iter_mark
            .offset_from((namedfm.ptr() as *mut xfmark_T).offset(0 as ::core::ffi::c_int as isize))
            as size_t)
            >= ::core::mem::size_of::<[xfmark_T; 36]>()
                .wrapping_div(::core::mem::size_of::<xfmark_T>())
                .wrapping_div(
                    (::core::mem::size_of::<[xfmark_T; 36]>()
                        .wrapping_rem(::core::mem::size_of::<xfmark_T>())
                        == 0) as ::core::ffi::c_int as usize,
                )
        {
            break;
        }
        if (*iter_mark).fmark.mark.lnum != 0 {
            return iter_mark as *const ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null::<::core::ffi::c_void>();
}
#[inline]
unsafe extern "C" fn next_buffer_mark(
    buf: *const buf_T,
    mark_name: *mut ::core::ffi::c_char,
) -> *const fmark_T {
    match *mark_name as ::core::ffi::c_int {
        NUL => {
            *mark_name = '"' as ::core::ffi::c_char;
            return &raw const (*buf).b_last_cursor;
        }
        34 => {
            *mark_name = '^' as ::core::ffi::c_char;
            return &raw const (*buf).b_last_insert;
        }
        94 => {
            *mark_name = '.' as ::core::ffi::c_char;
            return &raw const (*buf).b_last_change;
        }
        46 => {
            *mark_name = 'a' as ::core::ffi::c_char;
            return (&raw const (*buf).b_namedm as *const fmark_T)
                .offset(0 as ::core::ffi::c_int as isize);
        }
        122 => return ::core::ptr::null::<fmark_T>(),
        _ => {
            *mark_name += 1;
            return (&raw const (*buf).b_namedm as *const fmark_T)
                .offset((*mark_name as ::core::ffi::c_int - 'a' as ::core::ffi::c_int) as isize);
        }
    };
}
pub unsafe extern "C" fn mark_buffer_iter(
    iter: *const ::core::ffi::c_void,
    buf: *const buf_T,
    name: *mut ::core::ffi::c_char,
    fm: *mut fmark_T,
) -> *const ::core::ffi::c_void {
    *name = NUL as ::core::ffi::c_char;
    let mut mark_name: ::core::ffi::c_char = (if iter.is_null() {
        NUL as isize
    } else if iter == &raw const (*buf).b_last_cursor as *const ::core::ffi::c_void {
        '"' as isize
    } else if iter == &raw const (*buf).b_last_insert as *const ::core::ffi::c_void {
        '^' as isize
    } else if iter == &raw const (*buf).b_last_change as *const ::core::ffi::c_void {
        '.' as isize
    } else {
        (iter as *const fmark_T)
            .offset('a' as ::core::ffi::c_int as isize)
            .offset_from(
                (&raw const (*buf).b_namedm as *const fmark_T)
                    .offset(0 as ::core::ffi::c_int as isize),
            )
    }) as ::core::ffi::c_char;
    let mut iter_mark: *const fmark_T = next_buffer_mark(buf, &raw mut mark_name);
    while !iter_mark.is_null() && (*iter_mark).mark.lnum == 0 as linenr_T {
        iter_mark = next_buffer_mark(buf, &raw mut mark_name);
    }
    if iter_mark.is_null() {
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    let mut iter_off: size_t = iter_mark.offset_from(
        (&raw const (*buf).b_namedm as *const fmark_T).offset(0 as ::core::ffi::c_int as isize),
    ) as size_t;
    if mark_name != 0 {
        *name = mark_name;
    } else {
        *name = ('a' as ::core::ffi::c_int + iter_off as ::core::ffi::c_char as ::core::ffi::c_int)
            as ::core::ffi::c_char;
    }
    *fm = *iter_mark;
    return iter_mark as *const ::core::ffi::c_void;
}
pub unsafe extern "C" fn mark_set_global(
    name: ::core::ffi::c_char,
    fm: xfmark_T,
    update: bool,
) -> bool {
    let idx: ::core::ffi::c_int = mark_global_index(name);
    if idx == -1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let fm_tgt: *mut xfmark_T = (namedfm.ptr() as *mut xfmark_T).offset(idx as isize);
    if update as ::core::ffi::c_int != 0 && fm.fmark.timestamp <= (*fm_tgt).fmark.timestamp {
        return false_0 != 0;
    }
    if (*fm_tgt).fmark.mark.lnum != 0 as linenr_T {
        free_xfmark(*fm_tgt);
    }
    *fm_tgt = fm;
    return true_0 != 0;
}
pub unsafe extern "C" fn mark_set_local(
    name: ::core::ffi::c_char,
    buf: *mut buf_T,
    fm: fmark_T,
    update: bool,
) -> bool {
    let mut fm_tgt: *mut fmark_T = ::core::ptr::null_mut::<fmark_T>();
    if name as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
        && name as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        fm_tgt = (&raw mut (*buf).b_namedm as *mut fmark_T)
            .offset((name as ::core::ffi::c_int - 'a' as ::core::ffi::c_int) as isize);
    } else if name as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
        fm_tgt = &raw mut (*buf).b_last_cursor;
    } else if name as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
        fm_tgt = &raw mut (*buf).b_last_insert;
    } else if name as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
        fm_tgt = &raw mut (*buf).b_prompt_start;
    } else if name as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
        fm_tgt = &raw mut (*buf).b_last_change;
    } else {
        return false_0 != 0;
    }
    if update as ::core::ffi::c_int != 0 && fm.timestamp <= (*fm_tgt).timestamp {
        return false_0 != 0;
    }
    if (*fm_tgt).mark.lnum != 0 as linenr_T {
        free_fmark(*fm_tgt);
    }
    *fm_tgt = fm;
    return true_0 != 0;
}
pub unsafe extern "C" fn free_jumplist(mut wp: *mut win_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*wp).w_jumplistlen {
        free_xfmark((*wp).w_jumplist[i as usize]);
        i += 1;
    }
    (*wp).w_jumplistlen = 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn set_last_cursor(mut win: *mut win_T) {
    if !(*win).w_buffer.is_null() {
        let fmarkp___: *mut fmark_T = &raw mut (*(*win).w_buffer).b_last_cursor;
        free_fmark(*fmarkp___);
        let fmarkp__: *mut fmark_T = fmarkp___;
        (*fmarkp__).mark = (*win).w_cursor;
        (*fmarkp__).fnum = 0 as ::core::ffi::c_int;
        (*fmarkp__).timestamp = os_time();
        (*fmarkp__).view = fmarkv_T {
            topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
            skipcol: 0 as colnr_T,
        };
        (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    }
}
pub unsafe extern "C" fn mark_mb_adjustpos(mut buf: *mut buf_T, mut lp: *mut pos_T) {
    if (*lp).col > 0 as ::core::ffi::c_int || (*lp).coladd > 1 as ::core::ffi::c_int {
        let p: *const ::core::ffi::c_char = ml_get_buf(buf, (*lp).lnum);
        if *p as ::core::ffi::c_int == NUL || ml_get_buf_len(buf, (*lp).lnum) < (*lp).col {
            (*lp).col = 0 as ::core::ffi::c_int as colnr_T;
        } else {
            (*lp).col -= utf_head_off(p, p.offset((*lp).col as isize));
        }
        if (*lp).coladd == 1 as ::core::ffi::c_int
            && *p.offset((*lp).col as isize) as ::core::ffi::c_int != TAB
            && vim_isprintc(utf_ptr2char(p.offset((*lp).col as isize))) as ::core::ffi::c_int != 0
            && ptr2cells(p.offset((*lp).col as isize)) > 1 as ::core::ffi::c_int
        {
            (*lp).coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
    }
}
unsafe extern "C" fn add_mark(
    mut l: *mut list_T,
    mut mname: *const ::core::ffi::c_char,
    mut pos: *const pos_T,
    mut bufnr: ::core::ffi::c_int,
    mut fname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*pos).lnum <= 0 as linenr_T {
        return OK;
    }
    let mut d: *mut dict_T = tv_dict_alloc();
    tv_list_append_dict(l, d);
    let mut lpos: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    tv_list_append_number(lpos, bufnr as varnumber_T);
    tv_list_append_number(lpos, (*pos).lnum as varnumber_T);
    tv_list_append_number(
        lpos,
        (if (*pos).col < MAXCOL as ::core::ffi::c_int {
            (*pos).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        } else {
            MAXCOL as ::core::ffi::c_int
        }) as varnumber_T,
    );
    tv_list_append_number(lpos, (*pos).coladd as varnumber_T);
    if tv_dict_add_str(
        d,
        b"mark\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        mname,
    ) == FAIL
        || tv_dict_add_list(
            d,
            b"pos\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            lpos,
        ) == FAIL
        || !fname.is_null()
            && tv_dict_add_str(
                d,
                b"file\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                fname,
            ) == FAIL
    {
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn get_buf_local_marks(mut buf: *const buf_T, mut l: *mut list_T) {
    let mut mname: [::core::ffi::c_char; 3] =
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"' \0");
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NMARKS {
        mname[1 as ::core::ffi::c_int as usize] =
            ('a' as ::core::ffi::c_int + i) as ::core::ffi::c_char;
        add_mark(
            l,
            &raw mut mname as *mut ::core::ffi::c_char,
            &raw const (*(&raw const (*buf).b_namedm as *const fmark_T).offset(i as isize)).mark,
            (*buf).handle as ::core::ffi::c_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        i += 1;
    }
    add_mark(
        l,
        b"''\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut (*curwin.get()).w_pcmark,
        (*curbuf.get()).handle as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    add_mark(
        l,
        b"'\"\0".as_ptr() as *const ::core::ffi::c_char,
        &raw const (*buf).b_last_cursor.mark,
        (*buf).handle as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    add_mark(
        l,
        b"'[\0".as_ptr() as *const ::core::ffi::c_char,
        &raw const (*buf).b_op_start,
        (*buf).handle as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    add_mark(
        l,
        b"']\0".as_ptr() as *const ::core::ffi::c_char,
        &raw const (*buf).b_op_end,
        (*buf).handle as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    add_mark(
        l,
        b"'^\0".as_ptr() as *const ::core::ffi::c_char,
        &raw const (*buf).b_last_insert.mark,
        (*buf).handle as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    add_mark(
        l,
        b"'.\0".as_ptr() as *const ::core::ffi::c_char,
        &raw const (*buf).b_last_change.mark,
        (*buf).handle as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    add_mark(
        l,
        b"'<\0".as_ptr() as *const ::core::ffi::c_char,
        &raw const (*buf).b_visual.vi_start,
        (*buf).handle as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    add_mark(
        l,
        b"'>\0".as_ptr() as *const ::core::ffi::c_char,
        &raw const (*buf).b_visual.vi_end,
        (*buf).handle as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
}
pub unsafe extern "C" fn get_raw_global_mark(mut name: ::core::ffi::c_char) -> xfmark_T {
    return (*namedfm.ptr())[mark_global_index(name) as usize];
}
pub unsafe extern "C" fn get_global_marks(mut l: *mut list_T) {
    let mut mname: [::core::ffi::c_char; 3] =
        ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"' \0");
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NMARKS + EXTRA_MARKS {
        if (*namedfm.ptr())[i as usize].fmark.fnum != 0 as ::core::ffi::c_int {
            name = buflist_nr2name((*namedfm.ptr())[i as usize].fmark.fnum, true_0, true_0);
        } else {
            name = (*namedfm.ptr())[i as usize].fname;
        }
        if !name.is_null() {
            mname[1 as ::core::ffi::c_int as usize] = (if i >= NMARKS {
                (i - NMARKS + '0' as ::core::ffi::c_int) as ::core::ffi::c_char
                    as ::core::ffi::c_int
            } else {
                (i + 'A' as ::core::ffi::c_int) as ::core::ffi::c_char as ::core::ffi::c_int
            }) as ::core::ffi::c_char;
            add_mark(
                l,
                &raw mut mname as *mut ::core::ffi::c_char,
                &raw mut (*(namedfm.ptr() as *mut xfmark_T).offset(i as isize))
                    .fmark
                    .mark,
                (*namedfm.ptr())[i as usize].fmark.fnum,
                name,
            );
            if (*namedfm.ptr())[i as usize].fmark.fnum != 0 as ::core::ffi::c_int {
                xfree(name as *mut ::core::ffi::c_void);
            }
        }
        i += 1;
    }
}
#[inline]
unsafe extern "C" fn mark_global_index(name: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return if name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        name as ::core::ffi::c_int - 'A' as ::core::ffi::c_int
    } else if ascii_isdigit(name as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
        NMARKS + (name as ::core::ffi::c_int - '0' as ::core::ffi::c_int)
    } else {
        -1 as ::core::ffi::c_int
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
