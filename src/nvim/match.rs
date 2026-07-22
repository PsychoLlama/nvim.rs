use crate::src::nvim::charset::{skiptowhite, skipwhite};
use crate::src::nvim::drawscreen::{redraw_later, redraw_win_range_later};
use crate::src::nvim::eval::funcs::get_optional_window;
use crate::src::nvim::eval::typval::{
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find,
    tv_dict_get_number, tv_dict_get_string, tv_dict_get_string_buf, tv_get_number,
    tv_get_number_chk, tv_get_string, tv_get_string_buf_chk, tv_list_alloc, tv_list_alloc_ret,
    tv_list_append_dict, tv_list_append_number, tv_list_append_string, tv_list_append_tv,
    tv_list_idx_of_item, tv_list_unref,
};
use crate::src::nvim::eval::window::find_win_by_nr_or_id;
use crate::src::nvim::ex_docmd::{ends_excmd, ex_errmsg, find_nextcmd, set_no_hlsearch};
use crate::src::nvim::fold::hasFolding;

use crate::src::nvim::highlight_group::{syn_check_group, syn_id2attr, syn_id2name, syn_name2id};
use crate::src::nvim::main::{
    called_emsg, curwin, e_dictreq, e_invalwindow, e_invarg2, e_invcmd, e_listarg, e_listreq,
    e_trailing_arg, got_int, hl_attr_active, ns_hl_fast, p_cpo, p_rdt, search_first_line,
    search_hl_has_cursor_lnum, search_last_line,
};
use crate::src::nvim::mbyte::{utf_char2bytes, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memline::ml_get_buf;
use crate::src::nvim::memory::{xcalloc, xfree, xmemdupz, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::libc::{__assert_fail, gettext, snprintf, strlen, strncasecmp};
use crate::src::nvim::profile::{profile_passed_limit, profile_setlimit};
use crate::src::nvim::regexp::skip_regexp;
use crate::src::nvim::strings::vim_strchr;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CMD_index, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, EvalFuncData, ExtmarkUndoObject, FileID, FloatAnchor,
    FloatRelative, GridView, Intersection, LineGetter, ListLenSpecials, LuaRef, MTKey, MTNode,
    MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MsgpackRpcRequestHandler, OptInt, ScopeDictDictItem, ScopeType,
    ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal, Timestamp, VarLockStatus, VarType,
    VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle,
    Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T,
    chunksize_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_15,
    dict_T, dictitem_T, dictvar_S, disptick_T, eslist_T, eslist_elem, exarg, exarg_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, ssize_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T,
    synblock_T, synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, xfmark_T, NS, QUEUE,
};
extern "C" {
    fn re_multiline(prog: *const regprog_T) -> ::core::ffi::c_int;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_multi(
        rmp: *mut regmmatch_T,
        win: *mut win_T,
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        tm: *mut proftime_T,
        timed_out: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed = 2147483647;
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
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_13 = 76;
pub const HLF_PRE: C2Rust_Unnamed_13 = 75;
pub const HLF_OK: C2Rust_Unnamed_13 = 74;
pub const HLF_SO: C2Rust_Unnamed_13 = 73;
pub const HLF_SE: C2Rust_Unnamed_13 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_13 = 71;
pub const HLF_TS: C2Rust_Unnamed_13 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_13 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_13 = 68;
pub const HLF_CU: C2Rust_Unnamed_13 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_13 = 66;
pub const HLF_WBR: C2Rust_Unnamed_13 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_13 = 64;
pub const HLF_MSG: C2Rust_Unnamed_13 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_13 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_13 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_13 = 60;
pub const HLF_0: C2Rust_Unnamed_13 = 59;
pub const HLF_QFL: C2Rust_Unnamed_13 = 58;
pub const HLF_MC: C2Rust_Unnamed_13 = 57;
pub const HLF_CUL: C2Rust_Unnamed_13 = 56;
pub const HLF_CUC: C2Rust_Unnamed_13 = 55;
pub const HLF_TPF: C2Rust_Unnamed_13 = 54;
pub const HLF_TPS: C2Rust_Unnamed_13 = 53;
pub const HLF_TP: C2Rust_Unnamed_13 = 52;
pub const HLF_PBR: C2Rust_Unnamed_13 = 51;
pub const HLF_PST: C2Rust_Unnamed_13 = 50;
pub const HLF_PSB: C2Rust_Unnamed_13 = 49;
pub const HLF_PSX: C2Rust_Unnamed_13 = 48;
pub const HLF_PNX: C2Rust_Unnamed_13 = 47;
pub const HLF_PSK: C2Rust_Unnamed_13 = 46;
pub const HLF_PNK: C2Rust_Unnamed_13 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_13 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_13 = 43;
pub const HLF_PSI: C2Rust_Unnamed_13 = 42;
pub const HLF_PNI: C2Rust_Unnamed_13 = 41;
pub const HLF_SPL: C2Rust_Unnamed_13 = 40;
pub const HLF_SPR: C2Rust_Unnamed_13 = 39;
pub const HLF_SPC: C2Rust_Unnamed_13 = 38;
pub const HLF_SPB: C2Rust_Unnamed_13 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_13 = 36;
pub const HLF_SC: C2Rust_Unnamed_13 = 35;
pub const HLF_TXA: C2Rust_Unnamed_13 = 34;
pub const HLF_TXD: C2Rust_Unnamed_13 = 33;
pub const HLF_DED: C2Rust_Unnamed_13 = 32;
pub const HLF_CHD: C2Rust_Unnamed_13 = 31;
pub const HLF_ADD: C2Rust_Unnamed_13 = 30;
pub const HLF_FC: C2Rust_Unnamed_13 = 29;
pub const HLF_FL: C2Rust_Unnamed_13 = 28;
pub const HLF_WM: C2Rust_Unnamed_13 = 27;
pub const HLF_W: C2Rust_Unnamed_13 = 26;
pub const HLF_VNC: C2Rust_Unnamed_13 = 25;
pub const HLF_V: C2Rust_Unnamed_13 = 24;
pub const HLF_T: C2Rust_Unnamed_13 = 23;
pub const HLF_VSP: C2Rust_Unnamed_13 = 22;
pub const HLF_C: C2Rust_Unnamed_13 = 21;
pub const HLF_SNC: C2Rust_Unnamed_13 = 20;
pub const HLF_S: C2Rust_Unnamed_13 = 19;
pub const HLF_R: C2Rust_Unnamed_13 = 18;
pub const HLF_CLF: C2Rust_Unnamed_13 = 17;
pub const HLF_CLS: C2Rust_Unnamed_13 = 16;
pub const HLF_CLN: C2Rust_Unnamed_13 = 15;
pub const HLF_LNB: C2Rust_Unnamed_13 = 14;
pub const HLF_LNA: C2Rust_Unnamed_13 = 13;
pub const HLF_N: C2Rust_Unnamed_13 = 12;
pub const HLF_CM: C2Rust_Unnamed_13 = 11;
pub const HLF_M: C2Rust_Unnamed_13 = 10;
pub const HLF_LC: C2Rust_Unnamed_13 = 9;
pub const HLF_L: C2Rust_Unnamed_13 = 8;
pub const HLF_I: C2Rust_Unnamed_13 = 7;
pub const HLF_E: C2Rust_Unnamed_13 = 6;
pub const HLF_D: C2Rust_Unnamed_13 = 5;
pub const HLF_AT: C2Rust_Unnamed_13 = 4;
pub const HLF_TERM: C2Rust_Unnamed_13 = 3;
pub const HLF_EOB: C2Rust_Unnamed_13 = 2;
pub const HLF_8: C2Rust_Unnamed_13 = 1;
pub const HLF_NONE: C2Rust_Unnamed_13 = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_14 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_14 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_14 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_14 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_14 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_14 = 20;
pub const UPD_VALID: C2Rust_Unnamed_14 = 10;
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
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 56] = unsafe {
    ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
        *b"void f_getmatches(typval_T *, typval_T *, EvalFuncData)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_SEARCH: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
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
pub const SEARCH_HL_PRIORITY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn match_add(
    mut wp: *mut win_T,
    grp: *const ::core::ffi::c_char,
    pat: *const ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
    mut id: ::core::ffi::c_int,
    mut pos_list: *mut list_T,
    conceal_char: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut cur_0: *mut matchitem_T = ::core::ptr::null_mut::<matchitem_T>();
    let mut prev: *mut matchitem_T = ::core::ptr::null_mut::<matchitem_T>();
    let mut hlg_id: ::core::ffi::c_int = 0;
    let mut regprog: *mut regprog_T = ::core::ptr::null_mut::<regprog_T>();
    let mut rtype: ::core::ffi::c_int = UPD_SOME_VALID as ::core::ffi::c_int;
    if *grp as ::core::ffi::c_int == NUL || !pat.is_null() && *pat as ::core::ffi::c_int == NUL {
        return -1 as ::core::ffi::c_int;
    }
    if id < -1 as ::core::ffi::c_int || id == 0 as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E799: Invalid ID: %ld (must be greater than or equal to 1)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            id as int64_t,
        );
        return -1 as ::core::ffi::c_int;
    }
    if id == -1 as ::core::ffi::c_int {
        let c2rust_fresh0 = (*wp).w_next_match_id;
        (*wp).w_next_match_id = (*wp).w_next_match_id + 1;
        id = c2rust_fresh0;
    } else {
        let mut cur: *mut matchitem_T = (*wp).w_match_head;
        while !cur.is_null() {
            if (*cur).mit_id == id {
                semsg(
                    gettext(b"E801: ID already taken: %ld\0".as_ptr() as *const ::core::ffi::c_char),
                    id as int64_t,
                );
                return -1 as ::core::ffi::c_int;
            }
            cur = (*cur).mit_next;
        }
        if (*wp).w_next_match_id < id + 100 as ::core::ffi::c_int {
            (*wp).w_next_match_id = id + 100 as ::core::ffi::c_int;
        }
    }
    hlg_id = syn_check_group(grp, strlen(grp));
    if hlg_id == 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if !pat.is_null() && {
        regprog = vim_regcomp(pat, RE_MAGIC);
        regprog.is_null()
    } {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            pat,
        );
        return -1 as ::core::ffi::c_int;
    }
    let mut m: *mut matchitem_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<matchitem_T>()) as *mut matchitem_T;
    if tv_list_len(pos_list) > 0 as ::core::ffi::c_int {
        (*m).mit_pos_array = xcalloc(
            tv_list_len(pos_list) as size_t,
            ::core::mem::size_of::<llpos_T>(),
        ) as *mut llpos_T;
        (*m).mit_pos_count = tv_list_len(pos_list);
    }
    (*m).mit_id = id;
    (*m).mit_priority = prio;
    (*m).mit_pattern = if pat.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup(pat)
    };
    (*m).mit_hlg_id = hlg_id;
    (*m).mit_match.regprog = regprog;
    (*m).mit_match.rmm_ic = false_0;
    (*m).mit_match.rmm_maxcol = 0 as ::core::ffi::c_int as colnr_T;
    (*m).mit_conceal_char = 0 as ::core::ffi::c_int;
    if !conceal_char.is_null() {
        (*m).mit_conceal_char = utf_ptr2char(conceal_char);
    }
    if !pos_list.is_null() {
        let mut toplnum: linenr_T = 0 as linenr_T;
        let mut botlnum: linenr_T = 0 as linenr_T;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let l_: *mut list_T = pos_list;
        's_369: {
            if !l_.is_null() {
                let mut li: *mut listitem_T = (*l_).lv_first;
                '_fail: loop {
                    if li.is_null() {
                        break 's_369;
                    }
                    let mut lnum: linenr_T = 0 as linenr_T;
                    let mut col: colnr_T = 0 as colnr_T;
                    let mut len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    let mut error: bool = false;
                    's_183: {
                        if (*li).li_tv.v_type as ::core::ffi::c_uint
                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            let subl: *const list_T = (*li).li_tv.vval.v_list;
                            let mut subli: *const listitem_T = tv_list_first(subl);
                            if subli.is_null() {
                                semsg(
                                    gettext(b"E5030: Empty list at position %d\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    tv_list_idx_of_item(pos_list, li),
                                );
                                break '_fail;
                            } else {
                                lnum = tv_get_number_chk(&raw const (*subli).li_tv, &raw mut error)
                                    as linenr_T;
                                if error {
                                    break '_fail;
                                }
                                if lnum <= 0 as linenr_T {
                                    break 's_183;
                                } else {
                                    (*(*m).mit_pos_array.offset(i as isize)).lnum = lnum;
                                    subli = (*subli).li_next;
                                    if !subli.is_null() {
                                        col = tv_get_number_chk(
                                            &raw const (*subli).li_tv,
                                            &raw mut error,
                                        ) as colnr_T;
                                        if error {
                                            break '_fail;
                                        }
                                        if col < 0 as ::core::ffi::c_int {
                                            break 's_183;
                                        } else {
                                            subli = (*subli).li_next;
                                            if !subli.is_null() {
                                                len = tv_get_number_chk(
                                                    &raw const (*subli).li_tv,
                                                    &raw mut error,
                                                )
                                                    as colnr_T
                                                    as ::core::ffi::c_int;
                                                if len < 0 as ::core::ffi::c_int {
                                                    break 's_183;
                                                } else if error {
                                                    break '_fail;
                                                }
                                            }
                                        }
                                    }
                                    (*(*m).mit_pos_array.offset(i as isize)).col = col;
                                    (*(*m).mit_pos_array.offset(i as isize)).len = len;
                                }
                            }
                        } else if (*li).li_tv.v_type as ::core::ffi::c_uint
                            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if (*li).li_tv.vval.v_number <= 0 as varnumber_T {
                                break 's_183;
                            } else {
                                (*(*m).mit_pos_array.offset(i as isize)).lnum =
                                    (*li).li_tv.vval.v_number as linenr_T;
                                (*(*m).mit_pos_array.offset(i as isize)).col =
                                    0 as ::core::ffi::c_int as colnr_T;
                                (*(*m).mit_pos_array.offset(i as isize)).len =
                                    0 as ::core::ffi::c_int;
                            }
                        } else {
                            semsg(
                                gettext(
                                    b"E5031: List or number required at position %d\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ),
                                tv_list_idx_of_item(pos_list, li),
                            );
                            break '_fail;
                        }
                        if toplnum == 0 as linenr_T || lnum < toplnum {
                            toplnum = lnum;
                        }
                        if botlnum == 0 as linenr_T || lnum >= botlnum {
                            botlnum = lnum + 1 as linenr_T;
                        }
                        i += 1;
                    }
                    li = (*li).li_next;
                }
                vim_regfree(regprog);
                xfree((*m).mit_pattern as *mut ::core::ffi::c_void);
                xfree((*m).mit_pos_array as *mut ::core::ffi::c_void);
                xfree(m as *mut ::core::ffi::c_void);
                return -1 as ::core::ffi::c_int;
            }
        }
        if toplnum != 0 as linenr_T {
            redraw_win_range_later(wp, toplnum, botlnum);
            (*m).mit_toplnum = toplnum;
            (*m).mit_botlnum = botlnum;
            rtype = UPD_VALID as ::core::ffi::c_int;
        }
    }
    cur_0 = (*wp).w_match_head;
    prev = cur_0;
    while !cur_0.is_null() && prio >= (*cur_0).mit_priority {
        prev = cur_0;
        cur_0 = (*cur_0).mit_next;
    }
    if cur_0 == prev {
        (*wp).w_match_head = m;
    } else {
        (*prev).mit_next = m;
    }
    (*m).mit_next = cur_0;
    redraw_later(wp, rtype);
    return id;
}
unsafe extern "C" fn match_delete(
    mut wp: *mut win_T,
    mut id: ::core::ffi::c_int,
    mut perr: bool,
) -> ::core::ffi::c_int {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut prev: *mut matchitem_T = cur;
    let mut rtype: ::core::ffi::c_int = UPD_SOME_VALID as ::core::ffi::c_int;
    if id < 1 as ::core::ffi::c_int {
        if perr {
            semsg(
                gettext(
                    b"E802: Invalid ID: %ld (must be greater than or equal to 1)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                id as int64_t,
            );
        }
        return -1 as ::core::ffi::c_int;
    }
    while !cur.is_null() && (*cur).mit_id != id {
        prev = cur;
        cur = (*cur).mit_next;
    }
    if cur.is_null() {
        if perr {
            semsg(
                gettext(b"E803: ID not found: %ld\0".as_ptr() as *const ::core::ffi::c_char),
                id as int64_t,
            );
        }
        return -1 as ::core::ffi::c_int;
    }
    if cur == prev {
        (*wp).w_match_head = (*cur).mit_next;
    } else {
        (*prev).mit_next = (*cur).mit_next;
    }
    vim_regfree((*cur).mit_match.regprog);
    xfree((*cur).mit_pattern as *mut ::core::ffi::c_void);
    if (*cur).mit_toplnum != 0 as linenr_T {
        redraw_win_range_later(wp, (*cur).mit_toplnum, (*cur).mit_botlnum);
        rtype = UPD_VALID as ::core::ffi::c_int;
    }
    xfree((*cur).mit_pos_array as *mut ::core::ffi::c_void);
    xfree(cur as *mut ::core::ffi::c_void);
    redraw_later(wp, rtype);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn clear_matches(mut wp: *mut win_T) {
    while !(*wp).w_match_head.is_null() {
        let mut m: *mut matchitem_T = (*(*wp).w_match_head).mit_next;
        vim_regfree((*(*wp).w_match_head).mit_match.regprog);
        xfree((*(*wp).w_match_head).mit_pattern as *mut ::core::ffi::c_void);
        xfree((*(*wp).w_match_head).mit_pos_array as *mut ::core::ffi::c_void);
        xfree((*wp).w_match_head as *mut ::core::ffi::c_void);
        (*wp).w_match_head = m;
    }
    redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
}
unsafe extern "C" fn get_match(mut wp: *mut win_T, mut id: ::core::ffi::c_int) -> *mut matchitem_T {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    while !cur.is_null() && (*cur).mit_id != id {
        cur = (*cur).mit_next;
    }
    return cur;
}
#[no_mangle]
pub unsafe extern "C" fn init_search_hl(mut wp: *mut win_T, mut search_hl: *mut match_T) {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    while !cur.is_null() {
        (*cur).mit_hl.rm = (*cur).mit_match;
        if (*cur).mit_hlg_id == 0 as ::core::ffi::c_int {
            (*cur).mit_hl.attr = 0 as ::core::ffi::c_int;
        } else {
            (*cur).mit_hl.attr = syn_id2attr((*cur).mit_hlg_id);
        }
        (*cur).mit_hl.buf = (*wp).w_buffer;
        (*cur).mit_hl.lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*cur).mit_hl.first_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*cur).mit_hl.tm = profile_setlimit(p_rdt.get() as int64_t);
        cur = (*cur).mit_next;
    }
    (*search_hl).buf = (*wp).w_buffer;
    (*search_hl).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*search_hl).first_lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*search_hl).attr = win_hl_attr(wp, HLF_L as ::core::ffi::c_int);
}
unsafe extern "C" fn next_search_hl_pos(
    mut shl: *mut match_T,
    mut lnum: linenr_T,
    mut match_0: *mut matchitem_T,
    mut mincol: colnr_T,
) -> ::core::ffi::c_int {
    let mut found: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
    let mut i: ::core::ffi::c_int = (*match_0).mit_pos_cur;
    while i < (*match_0).mit_pos_count {
        let mut pos: *mut llpos_T = (*match_0).mit_pos_array.offset(i as isize);
        if (*pos).lnum == 0 as linenr_T {
            break;
        }
        if !((*pos).len == 0 as ::core::ffi::c_int && (*pos).col < mincol) {
            if (*pos).lnum == lnum {
                if found >= 0 as ::core::ffi::c_int {
                    if (*pos).col < (*(*match_0).mit_pos_array.offset(found as isize)).col {
                        let mut tmp: llpos_T = *pos;
                        *pos = *(*match_0).mit_pos_array.offset(found as isize);
                        *(*match_0).mit_pos_array.offset(found as isize) = tmp;
                    }
                } else {
                    found = i;
                }
            }
        }
        i += 1;
    }
    (*match_0).mit_pos_cur = 0 as ::core::ffi::c_int;
    if found >= 0 as ::core::ffi::c_int {
        let mut start: colnr_T =
            if (*(*match_0).mit_pos_array.offset(found as isize)).col == 0 as ::core::ffi::c_int {
                0 as colnr_T
            } else {
                (*(*match_0).mit_pos_array.offset(found as isize)).col - 1 as colnr_T
            };
        let mut end: colnr_T =
            if (*(*match_0).mit_pos_array.offset(found as isize)).col == 0 as ::core::ffi::c_int {
                MAXCOL as ::core::ffi::c_int
            } else {
                start + (*(*match_0).mit_pos_array.offset(found as isize)).len as colnr_T
            };
        (*shl).lnum = lnum;
        (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum =
            0 as ::core::ffi::c_int as linenr_T;
        (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col = start;
        (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum =
            0 as ::core::ffi::c_int as linenr_T;
        (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col = end;
        (*shl).is_addpos = true_0 != 0;
        (*shl).has_cursor = false_0 != 0;
        (*match_0).mit_pos_cur = found + 1 as ::core::ffi::c_int;
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn next_search_hl(
    mut win: *mut win_T,
    mut search_hl: *mut match_T,
    mut shl: *mut match_T,
    mut lnum: linenr_T,
    mut mincol: colnr_T,
    mut cur: *mut matchitem_T,
) {
    let mut matchcol: colnr_T = 0;
    let mut nmatched: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    if (lnum < search_first_line.get() || lnum > search_last_line.get()) && cur.is_null() {
        (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
        return;
    }
    if (*shl).lnum != 0 as linenr_T {
        let mut l: linenr_T = (*shl).lnum + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
            - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
        if lnum > l {
            (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
        } else if lnum < l || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col > mincol {
            return;
        }
    }
    loop {
        if profile_passed_limit((*shl).tm) {
            (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
            break;
        } else {
            if (*shl).lnum == 0 as linenr_T {
                matchcol = 0 as ::core::ffi::c_int as colnr_T;
            } else if vim_strchr(p_cpo.get(), CPO_SEARCH).is_null()
                || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum == 0 as linenr_T
                    && (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col
                        <= (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col
            {
                matchcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
                let mut ml: *mut ::core::ffi::c_char =
                    ml_get_buf((*shl).buf, lnum).offset(matchcol as isize);
                if *ml as ::core::ffi::c_int == NUL {
                    matchcol += 1;
                    (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                    break;
                } else {
                    matchcol += utfc_ptr2len(ml);
                }
            } else {
                matchcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
            }
            (*shl).lnum = lnum;
            if !(*shl).rm.regprog.is_null() {
                let mut regprog_is_copy: bool = shl != search_hl
                    && !cur.is_null()
                    && shl == &raw mut (*cur).mit_hl
                    && (*cur).mit_match.regprog == (*cur).mit_hl.rm.regprog;
                let mut timed_out: ::core::ffi::c_int = false_0;
                nmatched = vim_regexec_multi(
                    &raw mut (*shl).rm,
                    win,
                    (*shl).buf,
                    lnum,
                    matchcol,
                    &raw mut (*shl).tm,
                    &raw mut timed_out,
                );
                if regprog_is_copy {
                    (*cur).mit_match.regprog = (*cur).mit_hl.rm.regprog;
                }
                if called_emsg.get() > called_emsg_before
                    || got_int.get() as ::core::ffi::c_int != 0
                    || timed_out != 0
                {
                    if shl == search_hl {
                        vim_regfree((*shl).rm.regprog);
                        set_no_hlsearch(true_0 != 0);
                    }
                    (*shl).rm.regprog = ::core::ptr::null_mut::<regprog_T>();
                    (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                    got_int.set(false_0 != 0);
                    break;
                }
            } else if !cur.is_null() {
                nmatched = next_search_hl_pos(shl, lnum, cur, matchcol);
            }
            if nmatched == 0 as ::core::ffi::c_int {
                (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                break;
            } else {
                if !((*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum > 0 as linenr_T
                    || (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col >= mincol
                    || nmatched > 1 as ::core::ffi::c_int
                    || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col > mincol)
                {
                    continue;
                }
                (*shl).lnum += (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
                break;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn prepare_search_hl(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut lnum: linenr_T,
) {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    while !cur.is_null() || shl_flag as ::core::ffi::c_int == false_0 {
        if shl_flag as ::core::ffi::c_int == false_0 {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if !(*shl).rm.regprog.is_null()
            && (*shl).lnum == 0 as linenr_T
            && re_multiline((*shl).rm.regprog) != 0
        {
            if (*shl).first_lnum == 0 as linenr_T {
                (*shl).first_lnum = lnum;
                while (*shl).first_lnum > (*wp).w_topline {
                    if hasFolding(
                        wp,
                        (*shl).first_lnum - 1 as linenr_T,
                        ::core::ptr::null_mut::<linenr_T>(),
                        ::core::ptr::null_mut::<linenr_T>(),
                    ) {
                        break;
                    }
                    (*shl).first_lnum -= 1;
                }
            }
            if !cur.is_null() {
                (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
            }
            let mut pos_inprogress: bool = true_0 != 0;
            let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while (*shl).first_lnum < lnum
                && (!(*shl).rm.regprog.is_null()
                    || !cur.is_null() && pos_inprogress as ::core::ffi::c_int != 0)
            {
                next_search_hl(
                    wp,
                    search_hl,
                    shl,
                    (*shl).first_lnum,
                    n,
                    if shl == search_hl {
                        ::core::ptr::null_mut::<matchitem_T>()
                    } else {
                        cur
                    },
                );
                pos_inprogress = !(cur.is_null() || (*cur).mit_pos_cur == 0 as ::core::ffi::c_int);
                if (*shl).lnum != 0 as linenr_T {
                    (*shl).first_lnum = (*shl).lnum
                        + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
                        - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
                    n = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col
                        as ::core::ffi::c_int;
                } else {
                    (*shl).first_lnum += 1;
                    n = 0 as ::core::ffi::c_int;
                }
            }
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
}
unsafe extern "C" fn check_cur_search_hl(mut wp: *mut win_T, mut shl: *mut match_T) {
    let mut linecount: linenr_T = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
        - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
    if (*wp).w_cursor.lnum >= (*shl).lnum
        && (*wp).w_cursor.lnum <= (*shl).lnum + linecount
        && ((*wp).w_cursor.lnum > (*shl).lnum
            || (*wp).w_cursor.col >= (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col)
        && ((*wp).w_cursor.lnum < (*shl).lnum + linecount
            || (*wp).w_cursor.col < (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col)
    {
        (*shl).has_cursor = true_0 != 0;
    } else {
        (*shl).has_cursor = false_0 != 0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn prepare_search_hl_line(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut mincol: colnr_T,
    mut line: *mut *mut ::core::ffi::c_char,
    mut search_hl: *mut match_T,
    mut search_attr: *mut ::core::ffi::c_int,
    mut search_attr_from_match: *mut bool,
) -> bool {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    let mut area_highlighting: bool = false_0 != 0;
    while !cur.is_null() || !shl_flag {
        if !shl_flag {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        (*shl).startcol = MAXCOL as ::core::ffi::c_int as colnr_T;
        (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
        (*shl).attr_cur = 0 as ::core::ffi::c_int;
        (*shl).is_addpos = false_0 != 0;
        (*shl).has_cursor = false_0 != 0;
        if !cur.is_null() {
            (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
        }
        next_search_hl(
            wp,
            search_hl,
            shl,
            lnum,
            mincol,
            if shl == search_hl {
                ::core::ptr::null_mut::<matchitem_T>()
            } else {
                cur
            },
        );
        *line = ml_get_buf((*wp).w_buffer, lnum);
        if (*shl).lnum != 0 as linenr_T && (*shl).lnum <= lnum {
            if (*shl).lnum == lnum {
                (*shl).startcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
            } else {
                (*shl).startcol = 0 as ::core::ffi::c_int as colnr_T;
            }
            if lnum
                == (*shl).lnum + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
                    - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum
            {
                (*shl).endcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
            } else {
                (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
            }
            if shl == search_hl {
                check_cur_search_hl(wp, shl);
            }
            if (*shl).startcol == (*shl).endcol {
                if *(*line).offset((*shl).endcol as isize) as ::core::ffi::c_int != NUL {
                    (*shl).endcol += utfc_ptr2len((*line).offset((*shl).endcol as isize));
                } else {
                    (*shl).endcol += 1;
                }
            }
            if (*shl).startcol < mincol {
                (*shl).attr_cur = (*shl).attr;
                *search_attr = (*shl).attr;
                *search_attr_from_match = shl != search_hl;
            }
            area_highlighting = true_0 != 0;
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
    return area_highlighting;
}
#[no_mangle]
pub unsafe extern "C" fn update_search_hl(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut line: *mut *mut ::core::ffi::c_char,
    mut search_hl: *mut match_T,
    mut has_match_conc: *mut ::core::ffi::c_int,
    mut match_conc: *mut ::core::ffi::c_int,
    mut lcs_eol_todo: bool,
    mut on_last_col: *mut bool,
    mut search_attr_from_match: *mut bool,
) -> ::core::ffi::c_int {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    let mut search_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !cur.is_null() || !shl_flag {
        if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if !cur.is_null() {
            (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
        }
        let mut pos_inprogress: bool = true_0 != 0;
        while !(*shl).rm.regprog.is_null()
            || !cur.is_null() && pos_inprogress as ::core::ffi::c_int != 0
        {
            if (*shl).startcol != MAXCOL as ::core::ffi::c_int
                && col >= (*shl).startcol
                && col < (*shl).endcol
            {
                let mut next_col: ::core::ffi::c_int =
                    col as ::core::ffi::c_int + utfc_ptr2len((*line).offset(col as isize));
                if (*shl).endcol < next_col {
                    (*shl).endcol = next_col as colnr_T;
                }
                if shl == search_hl && (*shl).has_cursor as ::core::ffi::c_int != 0 {
                    (*shl).attr_cur = win_hl_attr(wp, HLF_LC as ::core::ffi::c_int);
                    if (*shl).attr_cur != (*shl).attr {
                        search_hl_has_cursor_lnum.set(lnum);
                    }
                } else {
                    (*shl).attr_cur = (*shl).attr;
                }
                if !cur.is_null()
                    && shl != search_hl
                    && syn_name2id(b"Conceal\0".as_ptr() as *const ::core::ffi::c_char)
                        == (*cur).mit_hlg_id
                {
                    *has_match_conc = if col == (*shl).startcol {
                        2 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    };
                    *match_conc = (*cur).mit_conceal_char;
                } else {
                    *has_match_conc = 0 as ::core::ffi::c_int;
                }
                break;
            } else {
                if col != (*shl).endcol {
                    break;
                }
                (*shl).attr_cur = 0 as ::core::ffi::c_int;
                next_search_hl(
                    wp,
                    search_hl,
                    shl,
                    lnum,
                    col,
                    if shl == search_hl {
                        ::core::ptr::null_mut::<matchitem_T>()
                    } else {
                        cur
                    },
                );
                pos_inprogress = !(cur.is_null() || (*cur).mit_pos_cur == 0 as ::core::ffi::c_int);
                *line = ml_get_buf((*wp).w_buffer, lnum);
                if (*shl).lnum != lnum {
                    break;
                }
                (*shl).startcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
                if (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum == 0 as linenr_T {
                    (*shl).endcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
                } else {
                    (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
                }
                if shl == search_hl {
                    check_cur_search_hl(wp, shl);
                }
                if (*shl).startcol == (*shl).endcol {
                    let mut p: *mut ::core::ffi::c_char = (*line).offset((*shl).endcol as isize);
                    if *p as ::core::ffi::c_int == NUL {
                        (*shl).endcol += 1;
                    } else {
                        (*shl).endcol += utfc_ptr2len(p);
                    }
                }
            }
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
    *search_attr_from_match = false_0 != 0;
    search_attr = (*search_hl).attr_cur;
    cur = (*wp).w_match_head;
    shl_flag = false_0 != 0;
    while !cur.is_null() || !shl_flag {
        if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if (*shl).attr_cur != 0 as ::core::ffi::c_int {
            search_attr = (*shl).attr_cur;
            *on_last_col = col as ::core::ffi::c_int + 1 as ::core::ffi::c_int >= (*shl).endcol;
            *search_attr_from_match = shl != search_hl;
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
    if *(*line).offset(col as isize) as ::core::ffi::c_int == NUL
        && ((*wp).w_onebuf_opt.wo_list != 0 && !lcs_eol_todo)
    {
        search_attr = 0 as ::core::ffi::c_int;
    }
    return search_attr;
}
#[no_mangle]
pub unsafe extern "C" fn get_prevcol_hl_flag(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut curcol: colnr_T,
) -> bool {
    let mut prevcol: colnr_T = curcol;
    if (if (*wp).w_onebuf_opt.wo_wrap != 0 {
        (*wp).w_skipcol
    } else {
        (*wp).w_leftcol
    }) > prevcol
    {
        prevcol += 1;
    }
    if !(*search_hl).is_addpos
        && (prevcol == (*search_hl).startcol
            || prevcol > (*search_hl).startcol
                && (*search_hl).endcol == MAXCOL as ::core::ffi::c_int)
    {
        return true_0 != 0;
    }
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    while !cur.is_null() {
        if !(*cur).mit_hl.is_addpos
            && (prevcol == (*cur).mit_hl.startcol
                || prevcol > (*cur).mit_hl.startcol
                    && (*cur).mit_hl.endcol == MAXCOL as ::core::ffi::c_int)
        {
            return true_0 != 0;
        }
        cur = (*cur).mit_next;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn get_search_match_hl(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut col: colnr_T,
    mut char_attr: *mut ::core::ffi::c_int,
) {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    while !cur.is_null() || !shl_flag {
        if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if col as ::core::ffi::c_int - 1 as ::core::ffi::c_int == (*shl).startcol
            && (shl == search_hl || !(*shl).is_addpos)
        {
            *char_attr = (*shl).attr;
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
}
unsafe extern "C" fn matchadd_dict_arg(
    mut tv: *mut typval_T,
    mut conceal_char: *mut *const ::core::ffi::c_char,
    mut win: *mut *mut win_T,
) -> ::core::ffi::c_int {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    if (*tv).v_type as ::core::ffi::c_uint != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
        return FAIL;
    }
    di = tv_dict_find(
        (*tv).vval.v_dict,
        b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        *conceal_char = tv_get_string(&raw mut (*di).di_tv);
    }
    di = tv_dict_find(
        (*tv).vval.v_dict,
        b"window\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if di.is_null() {
        return OK;
    }
    *win = find_win_by_nr_or_id(&raw mut (*di).di_tv);
    if (*win).is_null() {
        emsg(gettext(
            &raw const e_invalwindow as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn f_clearmatches(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut win: *mut win_T = get_optional_window(argvars, 0 as ::core::ffi::c_int);
    if !win.is_null() {
        clear_matches(win);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_getmatches(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut win: *mut win_T = get_optional_window(argvars, 0 as ::core::ffi::c_int);
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if win.is_null() {
        return;
    }
    let mut cur: *mut matchitem_T = (*win).w_match_head;
    while !cur.is_null() {
        let mut dict: *mut dict_T = tv_dict_alloc();
        if (*cur).mit_match.regprog.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*cur).mit_pos_count {
                let mut llpos: *mut llpos_T = ::core::ptr::null_mut::<llpos_T>();
                let mut buf: [::core::ffi::c_char; 30] = [0; 30];
                llpos = (*cur).mit_pos_array.offset(i as isize);
                if (*llpos).lnum == 0 as linenr_T {
                    break;
                }
                let l: *mut list_T = tv_list_alloc(
                    (1 as ::core::ffi::c_int
                        + (if (*llpos).col > 0 as ::core::ffi::c_int {
                            2 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        })) as ptrdiff_t,
                );
                tv_list_append_number(l, (*llpos).lnum as varnumber_T);
                if (*llpos).col > 0 as ::core::ffi::c_int {
                    tv_list_append_number(l, (*llpos).col as varnumber_T);
                    tv_list_append_number(l, (*llpos).len as varnumber_T);
                }
                let mut len: ::core::ffi::c_int = snprintf(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                    b"pos%d\0".as_ptr() as *const ::core::ffi::c_char,
                    i + 1 as ::core::ffi::c_int,
                );
                '_c2rust_label: {
                    if (len as size_t) < ::core::mem::size_of::<[::core::ffi::c_char; 30]>() {
                    } else {
                        __assert_fail(
                            b"(size_t)len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/match.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            898 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                tv_dict_add_list(
                    dict,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    len as size_t,
                    l,
                );
                i += 1;
            }
        } else {
            tv_dict_add_str(
                dict,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                (*cur).mit_pattern,
            );
        }
        tv_dict_add_str(
            dict,
            b"group\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            syn_id2name((*cur).mit_hlg_id),
        );
        tv_dict_add_nr(
            dict,
            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            (*cur).mit_priority as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            (*cur).mit_id as varnumber_T,
        );
        if (*cur).mit_conceal_char != 0 {
            let mut buf_0: [::core::ffi::c_char; 7] = [0; 7];
            buf_0[utf_char2bytes(
                (*cur).mit_conceal_char,
                &raw mut buf_0 as *mut ::core::ffi::c_char,
            ) as usize] = NUL as ::core::ffi::c_char;
            tv_dict_add_str(
                dict,
                b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                &raw mut buf_0 as *mut ::core::ffi::c_char,
            );
        }
        tv_list_append_dict((*rettv).vval.v_list, dict);
        cur = (*cur).mit_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_setmatches(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut s: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut win: *mut win_T = get_optional_window(argvars, 1 as ::core::ffi::c_int);
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    }
    if win.is_null() {
        return;
    }
    let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let mut li_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                || {
                    d = (*li).li_tv.vval.v_dict;
                    d.is_null()
                }
            {
                semsg(
                    gettext(
                        b"E474: List item %d is either not a dictionary or an empty one\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    li_idx,
                );
                return;
            }
            if !(!tv_dict_find(
                d,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            )
            .is_null()
                && (!tv_dict_find(
                    d,
                    b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                )
                .is_null()
                    || !tv_dict_find(
                        d,
                        b"pos1\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                            as ptrdiff_t,
                    )
                    .is_null())
                && !tv_dict_find(
                    d,
                    b"priority\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                )
                .is_null()
                && !tv_dict_find(
                    d,
                    b"id\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                )
                .is_null())
            {
                semsg(
                    gettext(
                        b"E474: List item %d is missing one of the required keys\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    li_idx,
                );
                return;
            }
            li_idx += 1;
            li = (*li).li_next;
        }
    }
    clear_matches(win);
    let mut match_add_failed: bool = false_0 != 0;
    let l__0: *const list_T = l;
    if !l__0.is_null() {
        let mut li_0: *const listitem_T = (*l__0).lv_first;
        while !li_0.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            d = (*li_0).li_tv.vval.v_dict;
            let di: *mut dictitem_T = tv_dict_find(
                d,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if di.is_null() {
                if s.is_null() {
                    s = tv_list_alloc(9 as ptrdiff_t);
                }
                i = 1 as ::core::ffi::c_int;
                while i < 9 as ::core::ffi::c_int {
                    let mut buf: [::core::ffi::c_char; 30] = [0; 30];
                    snprintf(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                        b"pos%d\0".as_ptr() as *const ::core::ffi::c_char,
                        i,
                    );
                    let pos_di: *mut dictitem_T =
                        tv_dict_find(d, &raw mut buf as *mut ::core::ffi::c_char, -1 as ptrdiff_t);
                    if pos_di.is_null() {
                        break;
                    }
                    if (*pos_di).di_tv.v_type as ::core::ffi::c_uint
                        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        return;
                    }
                    tv_list_append_tv(s, &raw mut (*pos_di).di_tv);
                    tv_list_ref(s);
                    i += 1;
                }
            }
            let mut group_buf: [::core::ffi::c_char; 65] = [0; 65];
            let group: *const ::core::ffi::c_char = tv_dict_get_string_buf(
                d,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut group_buf as *mut ::core::ffi::c_char,
            );
            let priority: ::core::ffi::c_int =
                tv_dict_get_number(d, b"priority\0".as_ptr() as *const ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            let id: ::core::ffi::c_int =
                tv_dict_get_number(d, b"id\0".as_ptr() as *const ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            let conceal_di: *mut dictitem_T = tv_dict_find(
                d,
                b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            let conceal: *const ::core::ffi::c_char = if !conceal_di.is_null() {
                tv_get_string(&raw mut (*conceal_di).di_tv)
            } else {
                ::core::ptr::null::<::core::ffi::c_char>()
            };
            if i == 0 as ::core::ffi::c_int {
                if match_add(
                    win,
                    group,
                    tv_dict_get_string(
                        d,
                        b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                        false,
                    ),
                    priority,
                    id,
                    ::core::ptr::null_mut::<list_T>(),
                    conceal,
                ) != id
                {
                    match_add_failed = true;
                }
            } else {
                if match_add(
                    win,
                    group,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    priority,
                    id,
                    s,
                    conceal,
                ) != id
                {
                    match_add_failed = true;
                }
                tv_list_unref(s);
                s = ::core::ptr::null_mut::<list_T>();
            }
            li_0 = (*li_0).li_next;
        }
    }
    if !match_add_failed {
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_matchadd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut grpbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let grp: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut grpbuf as *mut ::core::ffi::c_char,
    );
    let pat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut patbuf as *mut ::core::ffi::c_char,
    );
    let mut prio: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    let mut conceal_char: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut win: *mut win_T = curwin.get();
    (*rettv).vval.v_number = -1 as varnumber_T;
    if grp.is_null() || pat.is_null() {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        prio = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            id = tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && matchadd_dict_arg(
                    argvars.offset(4 as ::core::ffi::c_int as isize),
                    &raw mut conceal_char,
                    &raw mut win,
                ) == FAIL
            {
                return;
            }
        }
    }
    if error {
        return;
    }
    if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E798: ID is reserved for \":match\": %d\0".as_ptr() as *const ::core::ffi::c_char
            ),
            id,
        );
        return;
    }
    (*rettv).vval.v_number = match_add(
        win,
        grp,
        pat,
        prio,
        id,
        ::core::ptr::null_mut::<list_T>(),
        conceal_char,
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_matchaddpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let group: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    if group.is_null() {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            b"matchaddpos()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    l = (*argvars.offset(1 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if tv_list_len(l) == 0 as ::core::ffi::c_int {
        return;
    }
    let mut error: bool = false_0 != 0;
    let mut prio: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut conceal_char: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut win: *mut win_T = curwin.get();
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        prio = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            id = tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && matchadd_dict_arg(
                    argvars.offset(4 as ::core::ffi::c_int as isize),
                    &raw mut conceal_char,
                    &raw mut win,
                ) == FAIL
            {
                return;
            }
        }
    }
    if error as ::core::ffi::c_int == true_0 {
        return;
    }
    if id == 1 as ::core::ffi::c_int || id == 2 as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E798: ID is reserved for \"match\": %d\0".as_ptr() as *const ::core::ffi::c_char
            ),
            id,
        );
        return;
    }
    (*rettv).vval.v_number = match_add(
        win,
        group,
        ::core::ptr::null::<::core::ffi::c_char>(),
        prio,
        id,
        l,
        conceal_char,
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_matcharg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    tv_list_alloc_ret(
        rettv,
        (if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
            2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as ptrdiff_t,
    );
    if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
        let m: *mut matchitem_T = get_match(curwin.get(), id);
        if !m.is_null() {
            tv_list_append_string(
                (*rettv).vval.v_list,
                syn_id2name((*m).mit_hlg_id),
                -1 as ssize_t,
            );
            tv_list_append_string((*rettv).vval.v_list, (*m).mit_pattern, -1 as ssize_t);
        } else {
            tv_list_append_string(
                (*rettv).vval.v_list,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ssize_t,
            );
            tv_list_append_string(
                (*rettv).vval.v_list,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ssize_t,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_matchdelete(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut win: *mut win_T = get_optional_window(argvars, 1 as ::core::ffi::c_int);
    if win.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = match_delete(
            win,
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
            true_0 != 0,
        ) as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_match(mut eap: *mut exarg_T) {
    let mut g: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = 0;
    if (*eap).line2 <= 3 as linenr_T {
        id = (*eap).line2 as ::core::ffi::c_int;
    } else {
        emsg(&raw const e_invcmd as *const ::core::ffi::c_char);
        return;
    }
    if (*eap).skip == 0 {
        match_delete(curwin.get(), id, false_0 != 0);
    }
    if ends_excmd(*(*eap).arg as ::core::ffi::c_int) != 0 {
        end = (*eap).arg;
    } else if strncasecmp(
        (*eap).arg,
        b"none\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        4 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && (ascii_iswhite(*(*eap).arg.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
            || ends_excmd(
                *(*eap).arg.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) != 0)
    {
        end = (*eap).arg.offset(4 as ::core::ffi::c_int as isize);
    } else {
        let mut p: *mut ::core::ffi::c_char = skiptowhite((*eap).arg);
        if (*eap).skip == 0 {
            g = xmemdupz(
                (*eap).arg as *const ::core::ffi::c_void,
                p.offset_from((*eap).arg) as size_t,
            ) as *mut ::core::ffi::c_char;
        }
        p = skipwhite(p);
        if *p as ::core::ffi::c_int == NUL {
            xfree(g as *mut ::core::ffi::c_void);
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
            return;
        }
        end = skip_regexp(
            p.offset(1 as ::core::ffi::c_int as isize),
            *p as ::core::ffi::c_int,
            true_0,
        );
        if (*eap).skip == 0 {
            if *end as ::core::ffi::c_int != NUL
                && ends_excmd(
                    *skipwhite(end.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                ) == 0
            {
                xfree(g as *mut ::core::ffi::c_void);
                (*eap).errmsg =
                    ex_errmsg(&raw const e_trailing_arg as *const ::core::ffi::c_char, end);
                return;
            }
            if *end as ::core::ffi::c_int != *p as ::core::ffi::c_int {
                xfree(g as *mut ::core::ffi::c_void);
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    p,
                );
                return;
            }
            let mut c: ::core::ffi::c_int = *end as uint8_t as ::core::ffi::c_int;
            *end = NUL as ::core::ffi::c_char;
            match_add(
                curwin.get(),
                g,
                p.offset(1 as ::core::ffi::c_int as isize),
                10 as ::core::ffi::c_int,
                id,
                ::core::ptr::null_mut::<list_T>(),
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            xfree(g as *mut ::core::ffi::c_void);
            *end = c as ::core::ffi::c_char;
        }
    }
    (*eap).nextcmd = find_nextcmd(end);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
