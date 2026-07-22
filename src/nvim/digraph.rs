use crate::src::nvim::charset::{char2cells, getdigits_int, skiptowhite, skipwhite};
use crate::src::nvim::drawscreen::status_redraw_curbuf;
use crate::src::nvim::eval::typval::{
    tv_check_for_opt_bool_arg, tv_get_bool, tv_get_string_buf_chk, tv_get_string_chk,
    tv_list_alloc, tv_list_alloc_ret, tv_list_append_list, tv_list_append_string,
};
use crate::src::nvim::eval_1::eval_to_string;
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, getline_equal};
use crate::src::nvim::ex_getln::putcmdline;
use crate::src::nvim::getchar::plain_vgetc;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    allow_keys, cmdline_star, curbuf, curwin, e_number_exp, emsg_skip, got_int, msg_col,
    no_mapping, p_cpo, p_dg, p_enc, Columns,
};
use crate::src::nvim::mapping::do_map;
use crate::src::nvim::mbyte::{mb_cptr2char_adv, utf_char2bytes, utf_iscomposing_first};
use crate::src::nvim::memory::{xfree, xmalloc, xmemdupz, xstrdup};
use crate::src::nvim::message::{
    emsg, msg_advance, msg_ext_set_kind, msg_outtrans, msg_putchar, semsg,
};
use crate::src::nvim::normal::add_to_showcmd;
use crate::src::nvim::os::input::fast_breakcheck;
use crate::src::nvim::os::libc::{__assert_fail, gettext, strlen};
use crate::src::nvim::runtime::{getsourceline, source_runtime};
use crate::src::nvim::strings::vim_snprintf;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, ApiDispatchWrapper, Arena, Array, BoolVarValue, Boolean,
    BufUpdateCallbacks, CMD_index, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5,
    ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, EvalFuncData,
    ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridView, Integer, Intersection,
    KeyValuePair, LineGetter, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    MsgpackRpcRequestHandler, Object, ObjectType, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0, Terminal, Timestamp, VarLockStatus,
    VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit,
    WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T,
    chunksize_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_14,
    dict_T, dictvar_S, disptick_T, eslist_T, eslist_elem, exarg, exarg_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S, partial_T, pos_T,
    pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, ssize_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, taggy_T, terminal,
    time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
extern "C" {
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_append_via_ptr(gap: *mut garray_T, item_size: size_t) -> *mut ::core::ffi::c_void;
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct digr_T {
    pub char1: uint8_t,
    pub char2: uint8_t,
    pub result: result_T,
}
pub type result_T = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dg_header_entry {
    pub dg_start: ::core::ffi::c_int,
    pub dg_header: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kmap_T {
    pub from: *mut ::core::ffi::c_char,
    pub to: *mut ::core::ffi::c_char,
}
pub const MODE_LANGMAP: C2Rust_Unnamed_15 = 32;
pub const MAPTYPE_UNMAP: C2Rust_Unnamed_16 = 1;
pub const MAPTYPE_MAP: C2Rust_Unnamed_16 = 0;
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
pub const MODE_INSERT: C2Rust_Unnamed_15 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_15 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_15 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_15 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const MAPTYPE_UNMAP_LHS: C2Rust_Unnamed_16 = 3;
pub const MAPTYPE_NOREMAP: C2Rust_Unnamed_16 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const Ctrl_H: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static e_digraph_must_be_just_two_characters_str: GlobalCell<[::core::ffi::c_char; 47]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
            *b"E1214: Digraph must be just two characters: %s\0",
        )
    });
static e_digraph_argument_must_be_one_character_str: GlobalCell<[::core::ffi::c_char; 41]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
            *b"E1215: Digraph must be one character: %s\0",
        )
    });
static e_digraph_setlist_argument_must_be_list_of_lists_with_two_items: GlobalCell<
    [::core::ffi::c_char; 73],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 73], [::core::ffi::c_char; 73]>(
        *b"E1216: digraph_setlist() argument must be a list of lists with two items\0",
    )
});
static user_digraphs: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<digr_T>() as ::core::ffi::c_int,
    ga_growsize: 10 as ::core::ffi::c_int,
    ga_data: NULL,
});
static digraphdefault: GlobalCell<[digr_T; 1367]> = GlobalCell::new([
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0xa as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x1 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'X' as uint8_t,
        result: 0x2 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'X' as uint8_t,
        result: 0x3 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x4 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'Q' as uint8_t,
        result: 0x5 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: 'K' as uint8_t,
        result: 0x6 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x7 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x8 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x9 as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: 'F' as uint8_t,
        result: 0xa as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0xb as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: 'F' as uint8_t,
        result: 0xc as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0xd as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0xe as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0xf as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x10 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x11 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x12 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x13 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x14 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'K' as uint8_t,
        result: 0x15 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'Y' as uint8_t,
        result: 0x16 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'B' as uint8_t,
        result: 0x17 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: 'N' as uint8_t,
        result: 0x18 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x19 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'B' as uint8_t,
        result: 0x1a as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x1b as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x1c as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x1d as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x1e as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x1f as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'P' as uint8_t,
        result: 0x20 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'b' as uint8_t,
        result: 0x23 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0x24 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: 't' as uint8_t,
        result: 0x40 as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x5b as result_T,
    },
    digr_T {
        char1: '/' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x5c as result_T,
    },
    digr_T {
        char1: ')' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x5d as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x5e as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x60 as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x7b as result_T,
    },
    digr_T {
        char1: '!' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x7c as result_T,
    },
    digr_T {
        char1: '!' as uint8_t,
        char2: ')' as uint8_t,
        result: 0x7d as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x7e as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x7f as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0x80 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0x81 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x82 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x83 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'N' as uint8_t,
        result: 0x84 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x85 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0x86 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x87 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x88 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'J' as uint8_t,
        result: 0x89 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x8a as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'D' as uint8_t,
        result: 0x8b as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0x8c as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0x8d as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x8e as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x8f as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x90 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x91 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x92 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x93 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x94 as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'W' as uint8_t,
        result: 0x95 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'G' as uint8_t,
        result: 0x96 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'G' as uint8_t,
        result: 0x97 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x98 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x99 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x9a as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0x9b as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x9c as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x9d as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x9e as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x9f as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0xa0 as result_T,
    },
    digr_T {
        char1: '!' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0xa1 as result_T,
    },
    digr_T {
        char1: '~' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xa1 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: 't' as uint8_t,
        result: 0xa2 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '|' as uint8_t,
        result: 0xa2 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'd' as uint8_t,
        result: 0xa3 as result_T,
    },
    digr_T {
        char1: '$' as uint8_t,
        char2: '$' as uint8_t,
        result: 0xa3 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0xa4 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: 'x' as uint8_t,
        result: 0xa4 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0xa5 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '-' as uint8_t,
        result: 0xa5 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'B' as uint8_t,
        result: 0xa6 as result_T,
    },
    digr_T {
        char1: '|' as uint8_t,
        char2: '|' as uint8_t,
        result: 0xa6 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'E' as uint8_t,
        result: 0xa7 as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xa8 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0xa9 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0xa9 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0xaa as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '<' as uint8_t,
        result: 0xab as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0xac as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: ',' as uint8_t,
        result: 0xac as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: '-' as uint8_t,
        result: 0xad as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'g' as uint8_t,
        result: 0xae as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: 'm' as uint8_t,
        result: 0xaf as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: '=' as uint8_t,
        result: 0xaf as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'G' as uint8_t,
        result: 0xb0 as result_T,
    },
    digr_T {
        char1: '~' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0xb0 as result_T,
    },
    digr_T {
        char1: '+' as uint8_t,
        char2: '-' as uint8_t,
        result: 0xb1 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0xb2 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: '2' as uint8_t,
        result: 0xb2 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0xb3 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '3' as uint8_t,
        result: 0xb3 as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xb4 as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'y' as uint8_t,
        result: 0xb5 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0xb6 as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: 'p' as uint8_t,
        result: 0xb6 as result_T,
    },
    digr_T {
        char1: '.' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0xb7 as result_T,
    },
    digr_T {
        char1: '~' as uint8_t,
        char2: '.' as uint8_t,
        result: 0xb7 as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: ',' as uint8_t,
        result: 0xb8 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0xb9 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '1' as uint8_t,
        result: 0xb9 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0xba as result_T,
    },
    digr_T {
        char1: '>' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xbb as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '4' as uint8_t,
        result: 0xbc as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '2' as uint8_t,
        result: 0xbd as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '4' as uint8_t,
        result: 0xbe as result_T,
    },
    digr_T {
        char1: '?' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0xbf as result_T,
    },
    digr_T {
        char1: '~' as uint8_t,
        char2: '?' as uint8_t,
        result: 0xbf as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xc0 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '`' as uint8_t,
        result: 0xc0 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xc1 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xc2 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '^' as uint8_t,
        result: 0xc2 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '?' as uint8_t,
        result: 0xc3 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '~' as uint8_t,
        result: 0xc3 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xc4 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '"' as uint8_t,
        result: 0xc4 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0xc5 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '@' as uint8_t,
        result: 0xc5 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: 'E' as uint8_t,
        result: 0xc6 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: ',' as uint8_t,
        result: 0xc7 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xc8 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '`' as uint8_t,
        result: 0xc8 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xc9 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xca as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '^' as uint8_t,
        result: 0xca as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xcb as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '"' as uint8_t,
        result: 0xcb as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xcc as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '`' as uint8_t,
        result: 0xcc as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xcd as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xce as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '^' as uint8_t,
        result: 0xce as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xcf as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '"' as uint8_t,
        result: 0xcf as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '-' as uint8_t,
        result: 0xd0 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '?' as uint8_t,
        result: 0xd1 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '~' as uint8_t,
        result: 0xd1 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xd2 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '`' as uint8_t,
        result: 0xd2 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xd3 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xd4 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '^' as uint8_t,
        result: 0xd4 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '?' as uint8_t,
        result: 0xd5 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '~' as uint8_t,
        result: 0xd5 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xd6 as result_T,
    },
    digr_T {
        char1: '*' as uint8_t,
        char2: 'X' as uint8_t,
        result: 0xd7 as result_T,
    },
    digr_T {
        char1: '/' as uint8_t,
        char2: '\\' as uint8_t,
        result: 0xd7 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '/' as uint8_t,
        result: 0xd8 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xd9 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '`' as uint8_t,
        result: 0xd9 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xda as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xdb as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '^' as uint8_t,
        result: 0xdb as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xdc as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xdd as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0xde as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'p' as uint8_t,
        result: 0xde as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 's' as uint8_t,
        result: 0xdf as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xe0 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '`' as uint8_t,
        result: 0xe0 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xe1 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xe2 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '^' as uint8_t,
        result: 0xe2 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '?' as uint8_t,
        result: 0xe3 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '~' as uint8_t,
        result: 0xe3 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xe4 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '"' as uint8_t,
        result: 0xe4 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0xe5 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '@' as uint8_t,
        result: 0xe5 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0xe6 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: ',' as uint8_t,
        result: 0xe7 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xe8 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '`' as uint8_t,
        result: 0xe8 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xe9 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xea as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '^' as uint8_t,
        result: 0xea as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xeb as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '"' as uint8_t,
        result: 0xeb as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xec as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '`' as uint8_t,
        result: 0xec as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xed as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xee as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '^' as uint8_t,
        result: 0xee as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xef as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: '-' as uint8_t,
        result: 0xf0 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '?' as uint8_t,
        result: 0xf1 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '~' as uint8_t,
        result: 0xf1 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xf2 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '`' as uint8_t,
        result: 0xf2 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xf3 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xf4 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '^' as uint8_t,
        result: 0xf4 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '?' as uint8_t,
        result: 0xf5 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '~' as uint8_t,
        result: 0xf5 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xf6 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xf7 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '/' as uint8_t,
        result: 0xf8 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '!' as uint8_t,
        result: 0xf9 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '`' as uint8_t,
        result: 0xf9 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xfa as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '>' as uint8_t,
        result: 0xfb as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '^' as uint8_t,
        result: 0xfb as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xfc as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0xfd as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0xfe as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: ':' as uint8_t,
        result: 0xff as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '"' as uint8_t,
        result: 0xff as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x100 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x101 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x102 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x103 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x104 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x105 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x106 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x107 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x108 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x109 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x10a as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x10b as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x10c as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x10d as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x10e as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x10f as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x110 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x111 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x112 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x113 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x114 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x115 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x116 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x117 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x118 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x119 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x11a as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x11b as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x11c as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x11d as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x11e as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x11f as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x120 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x121 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x122 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x123 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x124 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x125 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x126 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x127 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x128 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x129 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x12a as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x12b as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x12c as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x12d as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x12e as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x12f as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x130 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x131 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'J' as uint8_t,
        result: 0x132 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x133 as result_T,
    },
    digr_T {
        char1: 'J' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x134 as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x135 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x136 as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x137 as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: 'k' as uint8_t,
        result: 0x138 as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x139 as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x13a as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x13b as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x13c as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x13d as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x13e as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x13f as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x140 as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x141 as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x142 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x143 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x144 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x145 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x146 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x147 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x148 as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: 'n' as uint8_t,
        result: 0x149 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'G' as uint8_t,
        result: 0x14a as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: 'g' as uint8_t,
        result: 0x14b as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x14c as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x14d as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x14e as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x14f as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x150 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x151 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: 'E' as uint8_t,
        result: 0x152 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x153 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x154 as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x155 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x156 as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x157 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x158 as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x159 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x15a as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x15b as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x15c as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x15d as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x15e as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x15f as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x160 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x161 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x162 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x163 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x164 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x165 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x166 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x167 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x168 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x169 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x16a as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x16b as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x16c as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x16d as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '0' as uint8_t,
        result: 0x16e as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '0' as uint8_t,
        result: 0x16f as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x170 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x171 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x172 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x173 as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x174 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x175 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x176 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x177 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x178 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x179 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x17a as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x17b as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x17c as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x17d as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x17e as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '9' as uint8_t,
        result: 0x1a0 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '9' as uint8_t,
        result: 0x1a1 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0x1a2 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x1a3 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x1a6 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '9' as uint8_t,
        result: 0x1af as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '9' as uint8_t,
        result: 0x1b0 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x1b5 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x1b6 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'D' as uint8_t,
        result: 0x1b7 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1cd as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1ce as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1cf as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1d0 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1d1 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1d2 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1d3 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1d4 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x1de as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x1df as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '7' as uint8_t,
        result: 0x1e0 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '7' as uint8_t,
        result: 0x1e1 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x1e2 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x1e3 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x1e4 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x1e5 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1e6 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1e7 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1e8 as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1e9 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x1ea as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x1eb as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x1ec as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x1ed as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'Z' as uint8_t,
        result: 0x1ee as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: 'z' as uint8_t,
        result: 0x1ef as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x1f0 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1f4 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1f5 as result_T,
    },
    digr_T {
        char1: ';' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2bf as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x2c7 as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x2d8 as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x2d9 as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '0' as uint8_t,
        result: 0x2da as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x2db as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x2dd as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x386 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x388 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x389 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x38a as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x38c as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x38e as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x38f as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x390 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x391 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x392 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x393 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x394 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x395 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x396 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x397 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x398 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x399 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x39a as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x39b as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x39c as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x39d as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x39e as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x39f as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3a0 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3a1 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3a3 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3a4 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3a5 as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3a6 as result_T,
    },
    digr_T {
        char1: 'X' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3a7 as result_T,
    },
    digr_T {
        char1: 'Q' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3a8 as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3a9 as result_T,
    },
    digr_T {
        char1: 'J' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3aa as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3ab as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x3ac as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x3ad as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x3ae as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x3af as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x3b0 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3b1 as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3b2 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3b3 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3b4 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3b5 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3b6 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3b7 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3b8 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3b9 as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3ba as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3bb as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3bc as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3bd as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3be as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3bf as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3c0 as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3c1 as result_T,
    },
    digr_T {
        char1: '*' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x3c2 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3c3 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3c4 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3c5 as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3c6 as result_T,
    },
    digr_T {
        char1: 'x' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3c7 as result_T,
    },
    digr_T {
        char1: 'q' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3c8 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3c9 as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3ca as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x3cb as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x3cc as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x3cd as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x3ce as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: 'G' as uint8_t,
        result: 0x3d8 as result_T,
    },
    digr_T {
        char1: ',' as uint8_t,
        char2: 'G' as uint8_t,
        result: 0x3d9 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x3da as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x3db as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x3dc as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x3dd as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x3de as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x3df as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x3e0 as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x3e1 as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x3f4 as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x3f5 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0x401 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x402 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x403 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'E' as uint8_t,
        result: 0x404 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x405 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0x406 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0x407 as result_T,
    },
    digr_T {
        char1: 'J' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x408 as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: 'J' as uint8_t,
        result: 0x409 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'J' as uint8_t,
        result: 0x40a as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x40b as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: 'J' as uint8_t,
        result: 0x40c as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x40e as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'Z' as uint8_t,
        result: 0x40f as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x410 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x411 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x412 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x413 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x414 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x415 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x416 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x417 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x418 as result_T,
    },
    digr_T {
        char1: 'J' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x419 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x41a as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x41b as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x41c as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x41d as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x41e as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x41f as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x420 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x421 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x422 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x423 as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x424 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x425 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x426 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x427 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x428 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x429 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x42a as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x42b as result_T,
    },
    digr_T {
        char1: '%' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x42c as result_T,
    },
    digr_T {
        char1: 'J' as uint8_t,
        char2: 'E' as uint8_t,
        result: 0x42d as result_T,
    },
    digr_T {
        char1: 'J' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0x42e as result_T,
    },
    digr_T {
        char1: 'J' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0x42f as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x430 as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x431 as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x432 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x433 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x434 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x435 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x436 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x437 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x438 as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x439 as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x43a as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x43b as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x43c as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x43d as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x43e as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x43f as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x440 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x441 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x442 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x443 as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x444 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x445 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x446 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x447 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x448 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x449 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x44a as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x44b as result_T,
    },
    digr_T {
        char1: '%' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x44c as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x44d as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x44e as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x44f as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x451 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x452 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x453 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x454 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x455 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x456 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x457 as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x458 as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x459 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x45a as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x45b as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x45c as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x45e as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'z' as uint8_t,
        result: 0x45f as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x462 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x463 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x46a as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x46b as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x472 as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x473 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x474 as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x475 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x480 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x481 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x490 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x491 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5d0 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5d1 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5d2 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5d3 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5d4 as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5d5 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5d6 as result_T,
    },
    digr_T {
        char1: 'X' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5d7 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x5d8 as result_T,
    },
    digr_T {
        char1: 'J' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5d9 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x5da as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5db as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5dc as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x5dd as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5de as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x5df as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5e0 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5e1 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5e2 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: '%' as uint8_t,
        result: 0x5e3 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5e4 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x5e5 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: 'J' as uint8_t,
        result: 0x5e6 as result_T,
    },
    digr_T {
        char1: 'Q' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5e7 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5e8 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x5e9 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x5ea as result_T,
    },
    digr_T {
        char1: ',' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x60c as result_T,
    },
    digr_T {
        char1: ';' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x61b as result_T,
    },
    digr_T {
        char1: '?' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x61f as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x621 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x622 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x623 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x624 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x625 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x626 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x627 as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x628 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 'm' as uint8_t,
        result: 0x629 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x62a as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 'k' as uint8_t,
        result: 0x62b as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x62c as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: 'k' as uint8_t,
        result: 0x62d as result_T,
    },
    digr_T {
        char1: 'x' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x62e as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x62f as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'k' as uint8_t,
        result: 0x630 as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x631 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x632 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x633 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 'n' as uint8_t,
        result: 0x634 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x635 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'd' as uint8_t,
        result: 0x636 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x637 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x638 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x639 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x63a as result_T,
    },
    digr_T {
        char1: '+' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x640 as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x641 as result_T,
    },
    digr_T {
        char1: 'q' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x642 as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x643 as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x644 as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x645 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x646 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x647 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x648 as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x649 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x64a as result_T,
    },
    digr_T {
        char1: ':' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x64b as result_T,
    },
    digr_T {
        char1: '"' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x64c as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x64d as result_T,
    },
    digr_T {
        char1: '/' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x64e as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x64f as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x650 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x651 as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x652 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x670 as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x67e as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x6a4 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: 'f' as uint8_t,
        result: 0x6af as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x6f0 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x6f1 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x6f2 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x6f3 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x6f4 as result_T,
    },
    digr_T {
        char1: '5' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x6f5 as result_T,
    },
    digr_T {
        char1: '6' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x6f6 as result_T,
    },
    digr_T {
        char1: '7' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x6f7 as result_T,
    },
    digr_T {
        char1: '8' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x6f8 as result_T,
    },
    digr_T {
        char1: '9' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x6f9 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e02 as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e03 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e06 as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e07 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e0a as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e0b as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e0e as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e0f as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x1e10 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x1e11 as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e1e as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e1f as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x1e20 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x1e21 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e22 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e23 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x1e26 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x1e27 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x1e28 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x1e29 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1e30 as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1e31 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e34 as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e35 as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e3a as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e3b as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1e3e as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1e3f as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e40 as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e41 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e44 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e45 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e48 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e49 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1e54 as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1e55 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e56 as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e57 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e58 as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e59 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e5e as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e5f as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e60 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e61 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e6a as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e6b as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e6e as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e6f as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x1e7c as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x1e7d as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x1e80 as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: '`' as uint8_t,
        result: 0x1e80 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x1e81 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: '`' as uint8_t,
        result: 0x1e81 as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1e82 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1e83 as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x1e84 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x1e85 as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e86 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e87 as result_T,
    },
    digr_T {
        char1: 'X' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e8a as result_T,
    },
    digr_T {
        char1: 'x' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e8b as result_T,
    },
    digr_T {
        char1: 'X' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x1e8c as result_T,
    },
    digr_T {
        char1: 'x' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x1e8d as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e8e as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x1e8f as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x1e90 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x1e91 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e94 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e95 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x1e96 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x1e97 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: '0' as uint8_t,
        result: 0x1e98 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '0' as uint8_t,
        result: 0x1e99 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ea2 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ea3 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1eba as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ebb as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x1ebc as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x1ebd as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ec8 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ec9 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ece as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ecf as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ee6 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ee7 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x1ef2 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '`' as uint8_t,
        result: 0x1ef2 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x1ef3 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '`' as uint8_t,
        result: 0x1ef3 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ef6 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x1ef7 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x1ef8 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x1ef9 as result_T,
    },
    digr_T {
        char1: ';' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1f00 as result_T,
    },
    digr_T {
        char1: ',' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x1f01 as result_T,
    },
    digr_T {
        char1: ';' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x1f02 as result_T,
    },
    digr_T {
        char1: ',' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x1f03 as result_T,
    },
    digr_T {
        char1: '?' as uint8_t,
        char2: ';' as uint8_t,
        result: 0x1f04 as result_T,
    },
    digr_T {
        char1: '?' as uint8_t,
        char2: ',' as uint8_t,
        result: 0x1f05 as result_T,
    },
    digr_T {
        char1: '!' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x1f06 as result_T,
    },
    digr_T {
        char1: '?' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x1f07 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'N' as uint8_t,
        result: 0x2002 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x2003 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x2004 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x2005 as result_T,
    },
    digr_T {
        char1: '6' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x2006 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x2009 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x200a as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x2010 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 'N' as uint8_t,
        result: 0x2013 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x2014 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x2015 as result_T,
    },
    digr_T {
        char1: '!' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x2016 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x2017 as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x2018 as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '9' as uint8_t,
        result: 0x2019 as result_T,
    },
    digr_T {
        char1: '.' as uint8_t,
        char2: '9' as uint8_t,
        result: 0x201a as result_T,
    },
    digr_T {
        char1: '9' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x201b as result_T,
    },
    digr_T {
        char1: '"' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x201c as result_T,
    },
    digr_T {
        char1: '"' as uint8_t,
        char2: '9' as uint8_t,
        result: 0x201d as result_T,
    },
    digr_T {
        char1: ':' as uint8_t,
        char2: '9' as uint8_t,
        result: 0x201e as result_T,
    },
    digr_T {
        char1: '9' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x201f as result_T,
    },
    digr_T {
        char1: '/' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x2020 as result_T,
    },
    digr_T {
        char1: '/' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x2021 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x2022 as result_T,
    },
    digr_T {
        char1: '.' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x2025 as result_T,
    },
    digr_T {
        char1: ',' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x2026 as result_T,
    },
    digr_T {
        char1: '%' as uint8_t,
        char2: '0' as uint8_t,
        result: 0x2030 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x2032 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x2033 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x2034 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x2057 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x2035 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x2036 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x2037 as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x2038 as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x2039 as result_T,
    },
    digr_T {
        char1: '>' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x203a as result_T,
    },
    digr_T {
        char1: ':' as uint8_t,
        char2: 'X' as uint8_t,
        result: 0x203b as result_T,
    },
    digr_T {
        char1: '\'' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x203e as result_T,
    },
    digr_T {
        char1: '/' as uint8_t,
        char2: 'f' as uint8_t,
        result: 0x2044 as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2070 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2074 as result_T,
    },
    digr_T {
        char1: '5' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2075 as result_T,
    },
    digr_T {
        char1: '6' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2076 as result_T,
    },
    digr_T {
        char1: '7' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2077 as result_T,
    },
    digr_T {
        char1: '8' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2078 as result_T,
    },
    digr_T {
        char1: '9' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2079 as result_T,
    },
    digr_T {
        char1: '+' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x207a as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x207b as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x207c as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x207d as result_T,
    },
    digr_T {
        char1: ')' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x207e as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x207f as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2080 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2081 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2082 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2083 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2084 as result_T,
    },
    digr_T {
        char1: '5' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2085 as result_T,
    },
    digr_T {
        char1: '6' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2086 as result_T,
    },
    digr_T {
        char1: '7' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2087 as result_T,
    },
    digr_T {
        char1: '8' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2088 as result_T,
    },
    digr_T {
        char1: '9' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2089 as result_T,
    },
    digr_T {
        char1: '+' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x208a as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x208b as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x208c as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x208d as result_T,
    },
    digr_T {
        char1: ')' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x208e as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x20a4 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 't' as uint8_t,
        result: 0x20a7 as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x20a9 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x20ac as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x20ac as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x20bd as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: 'P' as uint8_t,
        result: 0x20bd as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x2103 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x2105 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: 'F' as uint8_t,
        result: 0x2109 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '0' as uint8_t,
        result: 0x2116 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0x2117 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'x' as uint8_t,
        result: 0x211e as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x2120 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x2122 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: 'm' as uint8_t,
        result: 0x2126 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0x212b as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x2153 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x2154 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x2155 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x2156 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x2157 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x2158 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x2159 as result_T,
    },
    digr_T {
        char1: '5' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x215a as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '8' as uint8_t,
        result: 0x215b as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '8' as uint8_t,
        result: 0x215c as result_T,
    },
    digr_T {
        char1: '5' as uint8_t,
        char2: '8' as uint8_t,
        result: 0x215d as result_T,
    },
    digr_T {
        char1: '7' as uint8_t,
        char2: '8' as uint8_t,
        result: 0x215e as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2160 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2161 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2162 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2163 as result_T,
    },
    digr_T {
        char1: '5' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2164 as result_T,
    },
    digr_T {
        char1: '6' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2165 as result_T,
    },
    digr_T {
        char1: '7' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2166 as result_T,
    },
    digr_T {
        char1: '8' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2167 as result_T,
    },
    digr_T {
        char1: '9' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2168 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2169 as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x216a as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x216b as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2170 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2171 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2172 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2173 as result_T,
    },
    digr_T {
        char1: '5' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2174 as result_T,
    },
    digr_T {
        char1: '6' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2175 as result_T,
    },
    digr_T {
        char1: '7' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2176 as result_T,
    },
    digr_T {
        char1: '8' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2177 as result_T,
    },
    digr_T {
        char1: '9' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2178 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2179 as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x217a as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x217b as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x2190 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x2191 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x2192 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 'v' as uint8_t,
        result: 0x2193 as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x2194 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: 'D' as uint8_t,
        result: 0x2195 as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x21d0 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x21d2 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x21d4 as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0x2200 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'P' as uint8_t,
        result: 0x2202 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'E' as uint8_t,
        result: 0x2203 as result_T,
    },
    digr_T {
        char1: '/' as uint8_t,
        char2: '0' as uint8_t,
        result: 0x2205 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'E' as uint8_t,
        result: 0x2206 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'B' as uint8_t,
        result: 0x2207 as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x2208 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: ')' as uint8_t,
        result: 0x220b as result_T,
    },
    digr_T {
        char1: '*' as uint8_t,
        char2: 'P' as uint8_t,
        result: 0x220f as result_T,
    },
    digr_T {
        char1: '+' as uint8_t,
        char2: 'Z' as uint8_t,
        result: 0x2211 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x2212 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x2213 as result_T,
    },
    digr_T {
        char1: '*' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x2217 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: 'b' as uint8_t,
        result: 0x2218 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'b' as uint8_t,
        result: 0x2219 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x221a as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: '(' as uint8_t,
        result: 0x221d as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: '0' as uint8_t,
        result: 0x221e as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x221f as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 'V' as uint8_t,
        result: 0x2220 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'P' as uint8_t,
        result: 0x2225 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: 'N' as uint8_t,
        result: 0x2227 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2228 as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0x2229 as result_T,
    },
    digr_T {
        char1: ')' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0x222a as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'n' as uint8_t,
        result: 0x222b as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0x222c as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x222e as result_T,
    },
    digr_T {
        char1: '.' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x2234 as result_T,
    },
    digr_T {
        char1: ':' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x2235 as result_T,
    },
    digr_T {
        char1: ':' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2236 as result_T,
    },
    digr_T {
        char1: ':' as uint8_t,
        char2: ':' as uint8_t,
        result: 0x2237 as result_T,
    },
    digr_T {
        char1: '?' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x223c as result_T,
    },
    digr_T {
        char1: 'C' as uint8_t,
        char2: 'G' as uint8_t,
        result: 0x223e as result_T,
    },
    digr_T {
        char1: '?' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x2243 as result_T,
    },
    digr_T {
        char1: '?' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x2245 as result_T,
    },
    digr_T {
        char1: '?' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x2248 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x224c as result_T,
    },
    digr_T {
        char1: '.' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x2250 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0x2253 as result_T,
    },
    digr_T {
        char1: '!' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x2260 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x2261 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x2264 as result_T,
    },
    digr_T {
        char1: '>' as uint8_t,
        char2: '=' as uint8_t,
        result: 0x2265 as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '*' as uint8_t,
        result: 0x226a as result_T,
    },
    digr_T {
        char1: '*' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x226b as result_T,
    },
    digr_T {
        char1: '!' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x226e as result_T,
    },
    digr_T {
        char1: '!' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x226f as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x2282 as result_T,
    },
    digr_T {
        char1: ')' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x2283 as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x2286 as result_T,
    },
    digr_T {
        char1: ')' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x2287 as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x2299 as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x229a as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x22a5 as result_T,
    },
    digr_T {
        char1: '.' as uint8_t,
        char2: 'P' as uint8_t,
        result: 0x22c5 as result_T,
    },
    digr_T {
        char1: ':' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x22ee as result_T,
    },
    digr_T {
        char1: '.' as uint8_t,
        char2: '3' as uint8_t,
        result: 0x22ef as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x2302 as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '7' as uint8_t,
        result: 0x2308 as result_T,
    },
    digr_T {
        char1: '>' as uint8_t,
        char2: '7' as uint8_t,
        result: 0x2309 as result_T,
    },
    digr_T {
        char1: '7' as uint8_t,
        char2: '<' as uint8_t,
        result: 0x230a as result_T,
    },
    digr_T {
        char1: '7' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x230b as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0x2310 as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0x2312 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2315 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x2320 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'l' as uint8_t,
        result: 0x2321 as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '[' as uint8_t,
        result: 0x27e8 as result_T,
    },
    digr_T {
        char1: ']' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x27e9 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 's' as uint8_t,
        result: 0x2423 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x2440 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x2441 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x2442 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x2443 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x2446 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x2447 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x2448 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: 'j' as uint8_t,
        result: 0x2449 as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x2488 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x2489 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x248a as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x248b as result_T,
    },
    digr_T {
        char1: '5' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x248c as result_T,
    },
    digr_T {
        char1: '6' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x248d as result_T,
    },
    digr_T {
        char1: '7' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x248e as result_T,
    },
    digr_T {
        char1: '8' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x248f as result_T,
    },
    digr_T {
        char1: '9' as uint8_t,
        char2: '.' as uint8_t,
        result: 0x2490 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x2500 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x2501 as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: 'v' as uint8_t,
        result: 0x2502 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'V' as uint8_t,
        result: 0x2503 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x2504 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x2505 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x2506 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x2507 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: '-' as uint8_t,
        result: 0x2508 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x2509 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: '!' as uint8_t,
        result: 0x250a as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x250b as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x250c as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x250d as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x250e as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x250f as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'l' as uint8_t,
        result: 0x2510 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x2511 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'l' as uint8_t,
        result: 0x2512 as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: 'D' as uint8_t,
        result: 0x2513 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2514 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2515 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2516 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2517 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: 'l' as uint8_t,
        result: 0x2518 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x2519 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: 'l' as uint8_t,
        result: 0x251a as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x251b as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x251c as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x251d as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x2520 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x2523 as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: 'l' as uint8_t,
        result: 0x2524 as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x2525 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'l' as uint8_t,
        result: 0x2528 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x252b as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x252c as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x252f as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x2530 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x2533 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x2534 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x2537 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x2538 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x253b as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x253c as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x253f as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x2542 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x254b as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: 'D' as uint8_t,
        result: 0x2571 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'D' as uint8_t,
        result: 0x2572 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'B' as uint8_t,
        result: 0x2580 as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: 'B' as uint8_t,
        result: 0x2584 as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: 'B' as uint8_t,
        result: 0x2588 as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: 'B' as uint8_t,
        result: 0x258c as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'B' as uint8_t,
        result: 0x2590 as result_T,
    },
    digr_T {
        char1: '.' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2591 as result_T,
    },
    digr_T {
        char1: ':' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2592 as result_T,
    },
    digr_T {
        char1: '?' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2593 as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x25a0 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x25a1 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0x25a2 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x25a3 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'F' as uint8_t,
        result: 0x25a4 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'Y' as uint8_t,
        result: 0x25a5 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x25a6 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'Z' as uint8_t,
        result: 0x25a7 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'K' as uint8_t,
        result: 0x25a8 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'X' as uint8_t,
        result: 0x25a9 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 'B' as uint8_t,
        result: 0x25aa as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x25ac as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x25ad as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x25b2 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x25b3 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x25b6 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x25b7 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 't' as uint8_t,
        result: 0x25bc as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x25bd as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x25c0 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'l' as uint8_t,
        result: 0x25c1 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'b' as uint8_t,
        result: 0x25c6 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'w' as uint8_t,
        result: 0x25c7 as result_T,
    },
    digr_T {
        char1: 'L' as uint8_t,
        char2: 'Z' as uint8_t,
        result: 0x25ca as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: 'm' as uint8_t,
        result: 0x25cb as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x25ce as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: 'M' as uint8_t,
        result: 0x25cf as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: 'L' as uint8_t,
        result: 0x25d0 as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: 'R' as uint8_t,
        result: 0x25d1 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'n' as uint8_t,
        result: 0x25d8 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x25d9 as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: 'd' as uint8_t,
        result: 0x25e2 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'd' as uint8_t,
        result: 0x25e3 as result_T,
    },
    digr_T {
        char1: '*' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x2605 as result_T,
    },
    digr_T {
        char1: '*' as uint8_t,
        char2: '1' as uint8_t,
        result: 0x2606 as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x261c as result_T,
    },
    digr_T {
        char1: '>' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x261e as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x263a as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0x263b as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0x263c as result_T,
    },
    digr_T {
        char1: 'F' as uint8_t,
        char2: 'm' as uint8_t,
        result: 0x2640 as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'l' as uint8_t,
        result: 0x2642 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x2660 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: 'H' as uint8_t,
        result: 0x2661 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: 'D' as uint8_t,
        result: 0x2662 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: 'C' as uint8_t,
        result: 0x2663 as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'd' as uint8_t,
        result: 0x2669 as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: '8' as uint8_t,
        result: 0x266a as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: '2' as uint8_t,
        result: 0x266b as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'b' as uint8_t,
        result: 0x266d as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'x' as uint8_t,
        result: 0x266e as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'X' as uint8_t,
        result: 0x266f as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: 'K' as uint8_t,
        result: 0x2713 as result_T,
    },
    digr_T {
        char1: 'X' as uint8_t,
        char2: 'X' as uint8_t,
        result: 0x2717 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: 'X' as uint8_t,
        result: 0x2720 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: 'S' as uint8_t,
        result: 0x3000 as result_T,
    },
    digr_T {
        char1: ',' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x3001 as result_T,
    },
    digr_T {
        char1: '.' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x3002 as result_T,
    },
    digr_T {
        char1: '+' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x3003 as result_T,
    },
    digr_T {
        char1: '+' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x3004 as result_T,
    },
    digr_T {
        char1: '*' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x3005 as result_T,
    },
    digr_T {
        char1: ';' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x3006 as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x3007 as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '/' as uint8_t,
        result: 0x3008 as result_T,
    },
    digr_T {
        char1: '/' as uint8_t,
        char2: '>' as uint8_t,
        result: 0x3009 as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x300a as result_T,
    },
    digr_T {
        char1: '>' as uint8_t,
        char2: '+' as uint8_t,
        result: 0x300b as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x300c as result_T,
    },
    digr_T {
        char1: '>' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x300d as result_T,
    },
    digr_T {
        char1: '<' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x300e as result_T,
    },
    digr_T {
        char1: '>' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x300f as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x3010 as result_T,
    },
    digr_T {
        char1: ')' as uint8_t,
        char2: '"' as uint8_t,
        result: 0x3011 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: 'T' as uint8_t,
        result: 0x3012 as result_T,
    },
    digr_T {
        char1: '=' as uint8_t,
        char2: '_' as uint8_t,
        result: 0x3013 as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x3014 as result_T,
    },
    digr_T {
        char1: ')' as uint8_t,
        char2: '\'' as uint8_t,
        result: 0x3015 as result_T,
    },
    digr_T {
        char1: '(' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0x3016 as result_T,
    },
    digr_T {
        char1: ')' as uint8_t,
        char2: 'I' as uint8_t,
        result: 0x3017 as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: '?' as uint8_t,
        result: 0x301c as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x3041 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x3042 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x3043 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x3044 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x3045 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x3046 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x3047 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x3048 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x3049 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x304a as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x304b as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x304c as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x304d as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x304e as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x304f as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3050 as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x3051 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x3052 as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x3053 as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x3054 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x3055 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x3056 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x3057 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x3058 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3059 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x305a as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x305b as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x305c as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x305d as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x305e as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x305f as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x3060 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x3061 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x3062 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0x3063 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3064 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3065 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x3066 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x3067 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x3068 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x3069 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x306a as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x306b as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x306c as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x306d as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x306e as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x306f as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x3070 as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x3071 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x3072 as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x3073 as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x3074 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3075 as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3076 as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3077 as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x3078 as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x3079 as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x307a as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x307b as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x307c as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x307d as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x307e as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x307f as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3080 as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x3081 as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x3082 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0x3083 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x3084 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0x3085 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3086 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0x3087 as result_T,
    },
    digr_T {
        char1: 'y' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x3088 as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x3089 as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x308a as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x308b as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x308c as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x308d as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0x308e as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x308f as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x3090 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x3091 as result_T,
    },
    digr_T {
        char1: 'w' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x3092 as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x3093 as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3094 as result_T,
    },
    digr_T {
        char1: '"' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x309b as result_T,
    },
    digr_T {
        char1: '0' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x309c as result_T,
    },
    digr_T {
        char1: '*' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x309d as result_T,
    },
    digr_T {
        char1: '+' as uint8_t,
        char2: '5' as uint8_t,
        result: 0x309e as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30a1 as result_T,
    },
    digr_T {
        char1: 'A' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30a2 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30a3 as result_T,
    },
    digr_T {
        char1: 'I' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30a4 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30a5 as result_T,
    },
    digr_T {
        char1: 'U' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30a6 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30a7 as result_T,
    },
    digr_T {
        char1: 'E' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30a8 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30a9 as result_T,
    },
    digr_T {
        char1: 'O' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30aa as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30ab as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30ac as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30ad as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30ae as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30af as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30b0 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30b1 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30b2 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30b3 as result_T,
    },
    digr_T {
        char1: 'G' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30b4 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30b5 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30b6 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30b7 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30b8 as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30b9 as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30ba as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30bb as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30bc as result_T,
    },
    digr_T {
        char1: 'S' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30bd as result_T,
    },
    digr_T {
        char1: 'Z' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30be as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30bf as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30c0 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30c1 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30c2 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0x30c3 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30c4 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30c5 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30c6 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30c7 as result_T,
    },
    digr_T {
        char1: 'T' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30c8 as result_T,
    },
    digr_T {
        char1: 'D' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30c9 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30ca as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30cb as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30cc as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30cd as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30ce as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30cf as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30d0 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30d1 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30d2 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30d3 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30d4 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30d5 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30d6 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30d7 as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30d8 as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30d9 as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30da as result_T,
    },
    digr_T {
        char1: 'H' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30db as result_T,
    },
    digr_T {
        char1: 'B' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30dc as result_T,
    },
    digr_T {
        char1: 'P' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30dd as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30de as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30df as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30e0 as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30e1 as result_T,
    },
    digr_T {
        char1: 'M' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30e2 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0x30e3 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30e4 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: 'U' as uint8_t,
        result: 0x30e5 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30e6 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: 'O' as uint8_t,
        result: 0x30e7 as result_T,
    },
    digr_T {
        char1: 'Y' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30e8 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30e9 as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30ea as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30eb as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30ec as result_T,
    },
    digr_T {
        char1: 'R' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30ed as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0x30ee as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30ef as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30f0 as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30f1 as result_T,
    },
    digr_T {
        char1: 'W' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30f2 as result_T,
    },
    digr_T {
        char1: 'N' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30f3 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x30f4 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: 'A' as uint8_t,
        result: 0x30f5 as result_T,
    },
    digr_T {
        char1: 'K' as uint8_t,
        char2: 'E' as uint8_t,
        result: 0x30f6 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'a' as uint8_t,
        result: 0x30f7 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x30f8 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'e' as uint8_t,
        result: 0x30f9 as result_T,
    },
    digr_T {
        char1: 'V' as uint8_t,
        char2: 'o' as uint8_t,
        result: 0x30fa as result_T,
    },
    digr_T {
        char1: '.' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30fb as result_T,
    },
    digr_T {
        char1: '-' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30fc as result_T,
    },
    digr_T {
        char1: '*' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30fd as result_T,
    },
    digr_T {
        char1: '+' as uint8_t,
        char2: '6' as uint8_t,
        result: 0x30fe as result_T,
    },
    digr_T {
        char1: 'b' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3105 as result_T,
    },
    digr_T {
        char1: 'p' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3106 as result_T,
    },
    digr_T {
        char1: 'm' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3107 as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3108 as result_T,
    },
    digr_T {
        char1: 'd' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3109 as result_T,
    },
    digr_T {
        char1: 't' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x310a as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x310b as result_T,
    },
    digr_T {
        char1: 'l' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x310c as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x310d as result_T,
    },
    digr_T {
        char1: 'k' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x310e as result_T,
    },
    digr_T {
        char1: 'h' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x310f as result_T,
    },
    digr_T {
        char1: 'j' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3110 as result_T,
    },
    digr_T {
        char1: 'q' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3111 as result_T,
    },
    digr_T {
        char1: 'x' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3112 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x3113 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x3114 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 'h' as uint8_t,
        result: 0x3115 as result_T,
    },
    digr_T {
        char1: 'r' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3116 as result_T,
    },
    digr_T {
        char1: 'z' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3117 as result_T,
    },
    digr_T {
        char1: 'c' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3118 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3119 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x311a as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x311b as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x311c as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x311e as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0x311f as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3120 as result_T,
    },
    digr_T {
        char1: 'o' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3121 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'n' as uint8_t,
        result: 0x3122 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: 'n' as uint8_t,
        result: 0x3123 as result_T,
    },
    digr_T {
        char1: 'a' as uint8_t,
        char2: 'N' as uint8_t,
        result: 0x3124 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: 'N' as uint8_t,
        result: 0x3125 as result_T,
    },
    digr_T {
        char1: 'e' as uint8_t,
        char2: 'r' as uint8_t,
        result: 0x3126 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3127 as result_T,
    },
    digr_T {
        char1: 'u' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x3128 as result_T,
    },
    digr_T {
        char1: 'i' as uint8_t,
        char2: 'u' as uint8_t,
        result: 0x3129 as result_T,
    },
    digr_T {
        char1: 'v' as uint8_t,
        char2: '4' as uint8_t,
        result: 0x312a as result_T,
    },
    digr_T {
        char1: 'n' as uint8_t,
        char2: 'G' as uint8_t,
        result: 0x312b as result_T,
    },
    digr_T {
        char1: 'g' as uint8_t,
        char2: 'n' as uint8_t,
        result: 0x312c as result_T,
    },
    digr_T {
        char1: '1' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x3220 as result_T,
    },
    digr_T {
        char1: '2' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x3221 as result_T,
    },
    digr_T {
        char1: '3' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x3222 as result_T,
    },
    digr_T {
        char1: '4' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x3223 as result_T,
    },
    digr_T {
        char1: '5' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x3224 as result_T,
    },
    digr_T {
        char1: '6' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x3225 as result_T,
    },
    digr_T {
        char1: '7' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x3226 as result_T,
    },
    digr_T {
        char1: '8' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x3227 as result_T,
    },
    digr_T {
        char1: '9' as uint8_t,
        char2: 'c' as uint8_t,
        result: 0x3228 as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: 'f' as uint8_t,
        result: 0xfb00 as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: 'i' as uint8_t,
        result: 0xfb01 as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: 'l' as uint8_t,
        result: 0xfb02 as result_T,
    },
    digr_T {
        char1: 'f' as uint8_t,
        char2: 't' as uint8_t,
        result: 0xfb05 as result_T,
    },
    digr_T {
        char1: 's' as uint8_t,
        char2: 't' as uint8_t,
        result: 0xfb06 as result_T,
    },
    digr_T {
        char1: NUL as uint8_t,
        char2: NUL as uint8_t,
        result: NUL,
    },
]);
pub const DG_START_LATIN: ::core::ffi::c_int = 0xa1 as ::core::ffi::c_int;
pub const DG_START_GREEK: ::core::ffi::c_int = 0x386 as ::core::ffi::c_int;
pub const DG_START_CYRILLIC: ::core::ffi::c_int = 0x401 as ::core::ffi::c_int;
pub const DG_START_HEBREW: ::core::ffi::c_int = 0x5d0 as ::core::ffi::c_int;
pub const DG_START_ARABIC: ::core::ffi::c_int = 0x60c as ::core::ffi::c_int;
pub const DG_START_LATIN_EXTENDED: ::core::ffi::c_int = 0x1e02 as ::core::ffi::c_int;
pub const DG_START_GREEK_EXTENDED: ::core::ffi::c_int = 0x1f00 as ::core::ffi::c_int;
pub const DG_START_PUNCTUATION: ::core::ffi::c_int = 0x2002 as ::core::ffi::c_int;
pub const DG_START_SUB_SUPER: ::core::ffi::c_int = 0x2070 as ::core::ffi::c_int;
pub const DG_START_CURRENCY: ::core::ffi::c_int = 0x20a4 as ::core::ffi::c_int;
pub const DG_START_OTHER1: ::core::ffi::c_int = 0x2103 as ::core::ffi::c_int;
pub const DG_START_ROMAN: ::core::ffi::c_int = 0x2160 as ::core::ffi::c_int;
pub const DG_START_ARROWS: ::core::ffi::c_int = 0x2190 as ::core::ffi::c_int;
pub const DG_START_MATH: ::core::ffi::c_int = 0x2200 as ::core::ffi::c_int;
pub const DG_START_TECHNICAL: ::core::ffi::c_int = 0x2302 as ::core::ffi::c_int;
pub const DG_START_OTHER2: ::core::ffi::c_int = 0x2423 as ::core::ffi::c_int;
pub const DG_START_DRAWING: ::core::ffi::c_int = 0x2500 as ::core::ffi::c_int;
pub const DG_START_BLOCK: ::core::ffi::c_int = 0x2580 as ::core::ffi::c_int;
pub const DG_START_SHAPES: ::core::ffi::c_int = 0x25a0 as ::core::ffi::c_int;
pub const DG_START_SYMBOLS: ::core::ffi::c_int = 0x2605 as ::core::ffi::c_int;
pub const DG_START_DINGBATS: ::core::ffi::c_int = 0x2713 as ::core::ffi::c_int;
pub const DG_START_CJK_SYMBOLS: ::core::ffi::c_int = 0x3000 as ::core::ffi::c_int;
pub const DG_START_HIRAGANA: ::core::ffi::c_int = 0x3041 as ::core::ffi::c_int;
pub const DG_START_KATAKANA: ::core::ffi::c_int = 0x30a1 as ::core::ffi::c_int;
pub const DG_START_BOPOMOFO: ::core::ffi::c_int = 0x3105 as ::core::ffi::c_int;
pub const DG_START_OTHER3: ::core::ffi::c_int = 0x3220 as ::core::ffi::c_int;
pub unsafe extern "C" fn do_digraph(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    static backspaced: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    static lastchar: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    if c == -1 as ::core::ffi::c_int {
        backspaced.set(-1 as ::core::ffi::c_int);
    } else if p_dg.get() != 0 {
        if backspaced.get() >= 0 as ::core::ffi::c_int {
            c = digraph_get(backspaced.get(), c, false_0 != 0);
        }
        backspaced.set(-1 as ::core::ffi::c_int);
        if (c == K_BS || c == Ctrl_H) && lastchar.get() >= 0 as ::core::ffi::c_int {
            backspaced.set(lastchar.get());
        }
    }
    lastchar.set(c);
    return c;
}
pub unsafe extern "C" fn get_digraph_for_char(
    mut val_arg: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let val: ::core::ffi::c_int = val_arg;
    let mut dp: *const digr_T = ::core::ptr::null::<digr_T>();
    static r: GlobalCell<[::core::ffi::c_char; 3]> = GlobalCell::new([0; 3]);
    let mut use_defaults: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while use_defaults <= 1 as ::core::ffi::c_int {
        if use_defaults == 0 as ::core::ffi::c_int {
            dp = (*user_digraphs.ptr()).ga_data as *const digr_T;
        } else {
            dp = digraphdefault.ptr() as *mut digr_T;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while if use_defaults != 0 {
            ((*dp).char1 as ::core::ffi::c_int != NUL) as ::core::ffi::c_int
        } else {
            (i < (*user_digraphs.ptr()).ga_len) as ::core::ffi::c_int
        } != 0
        {
            if (*dp).result == val {
                (*r.ptr())[0 as ::core::ffi::c_int as usize] = (*dp).char1 as ::core::ffi::c_char;
                (*r.ptr())[1 as ::core::ffi::c_int as usize] = (*dp).char2 as ::core::ffi::c_char;
                (*r.ptr())[2 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                return r.ptr() as *mut ::core::ffi::c_char;
            }
            dp = dp.offset(1);
            i += 1;
        }
        use_defaults += 1;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn get_digraph(mut cmdline: bool) -> ::core::ffi::c_int {
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    let mut c: ::core::ffi::c_int = plain_vgetc();
    (*no_mapping.ptr()) -= 1;
    (*allow_keys.ptr()) -= 1;
    if c == ESC {
        return NUL;
    }
    if c < 0 as ::core::ffi::c_int {
        return c;
    }
    if cmdline {
        if char2cells(c) == 1 as ::core::ffi::c_int
            && c < 128 as ::core::ffi::c_int
            && cmdline_star.get() == 0 as ::core::ffi::c_int
        {
            putcmdline(c as ::core::ffi::c_char, true_0 != 0);
        }
    } else {
        add_to_showcmd(c);
    }
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    let mut cc: ::core::ffi::c_int = plain_vgetc();
    (*no_mapping.ptr()) -= 1;
    (*allow_keys.ptr()) -= 1;
    if cc != ESC {
        return digraph_get(c, cc, true_0 != 0);
    }
    return NUL;
}
unsafe extern "C" fn getexactdigraph(
    mut char1: ::core::ffi::c_int,
    mut char2: ::core::ffi::c_int,
    mut meta_char: bool,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if char1 < 0 as ::core::ffi::c_int || char2 < 0 as ::core::ffi::c_int {
        return char2;
    }
    let mut dp: *const digr_T = (*user_digraphs.ptr()).ga_data as *const digr_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*user_digraphs.ptr()).ga_len {
        if (*dp).char1 as ::core::ffi::c_int == char1 && (*dp).char2 as ::core::ffi::c_int == char2
        {
            retval = (*dp).result as ::core::ffi::c_int;
            break;
        } else {
            dp = dp.offset(1);
            i += 1;
        }
    }
    if retval == 0 as ::core::ffi::c_int {
        dp = digraphdefault.ptr() as *mut digr_T;
        while (*dp).char1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if (*dp).char1 as ::core::ffi::c_int == char1
                && (*dp).char2 as ::core::ffi::c_int == char2
            {
                retval = (*dp).result as ::core::ffi::c_int;
                break;
            } else {
                dp = dp.offset(1);
            }
        }
    }
    if retval == 0 as ::core::ffi::c_int {
        if char1 == ' ' as ::core::ffi::c_int && meta_char as ::core::ffi::c_int != 0 {
            return char2 | 0x80 as ::core::ffi::c_int;
        }
        return char2;
    }
    return retval;
}
pub unsafe extern "C" fn digraph_get(
    mut char1: ::core::ffi::c_int,
    mut char2: ::core::ffi::c_int,
    mut meta_char: bool,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0;
    retval = getexactdigraph(char1, char2, meta_char);
    if retval == char2 && char1 != char2 && {
        retval = getexactdigraph(char2, char1, meta_char);
        retval == char1
    } {
        return char2;
    }
    return retval;
}
unsafe extern "C" fn registerdigraph(
    mut char1: ::core::ffi::c_int,
    mut char2: ::core::ffi::c_int,
    mut n: ::core::ffi::c_int,
) {
    let mut dp: *mut digr_T = (*user_digraphs.ptr()).ga_data as *mut digr_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*user_digraphs.ptr()).ga_len {
        if (*dp).char1 as ::core::ffi::c_int == char1 && (*dp).char2 as ::core::ffi::c_int == char2
        {
            (*dp).result = n as result_T;
            return;
        }
        dp = dp.offset(1);
        i += 1;
    }
    dp = ga_append_via_ptr(user_digraphs.ptr(), ::core::mem::size_of::<digr_T>()) as *mut digr_T;
    (*dp).char1 = char1 as uint8_t;
    (*dp).char2 = char2 as uint8_t;
    (*dp).result = n as result_T;
}
pub unsafe extern "C" fn check_digraph_chars_valid(
    mut char1: ::core::ffi::c_int,
    mut char2: ::core::ffi::c_int,
) -> bool {
    if char2 == 0 as ::core::ffi::c_int {
        let mut msg: [::core::ffi::c_char; 7] = [0; 7];
        msg[utf_char2bytes(char1, &raw mut msg as *mut ::core::ffi::c_char) as usize] =
            NUL as ::core::ffi::c_char;
        semsg(
            gettext(
                (e_digraph_must_be_just_two_characters_str.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            &raw mut msg as *mut ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    if char1 == ESC || char2 == ESC {
        emsg(gettext(
            b"E104: Escape not allowed in digraph\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn putdigraph(mut str: *mut ::core::ffi::c_char) {
    while *str as ::core::ffi::c_int != NUL {
        str = skipwhite(str);
        if *str as ::core::ffi::c_int == NUL {
            return;
        }
        let c2rust_fresh0 = str;
        str = str.offset(1);
        let mut char1: uint8_t = *c2rust_fresh0 as uint8_t;
        let c2rust_fresh1 = str;
        str = str.offset(1);
        let mut char2: uint8_t = *c2rust_fresh1 as uint8_t;
        if !check_digraph_chars_valid(char1 as ::core::ffi::c_int, char2 as ::core::ffi::c_int) {
            return;
        }
        str = skipwhite(str);
        if !ascii_isdigit(*str as ::core::ffi::c_int) {
            emsg(gettext(
                &raw const e_number_exp as *const ::core::ffi::c_char,
            ));
            return;
        }
        let mut n: ::core::ffi::c_int =
            getdigits_int(&raw mut str, true_0 != 0, 0 as ::core::ffi::c_int);
        registerdigraph(char1 as ::core::ffi::c_int, char2 as ::core::ffi::c_int, n);
    }
}
unsafe extern "C" fn digraph_header(mut msg: *const ::core::ffi::c_char) {
    if msg_col.get() > 0 as ::core::ffi::c_int {
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    msg_outtrans(msg, HLF_CM as ::core::ffi::c_int, false_0 != 0);
    msg_putchar('\n' as ::core::ffi::c_int);
}
pub unsafe extern "C" fn listdigraphs(mut use_headers: bool) {
    let mut previous: result_T = 0 as result_T;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    msg_putchar('\n' as ::core::ffi::c_int);
    let mut dp: *const digr_T = digraphdefault.ptr() as *mut digr_T;
    while (*dp).char1 as ::core::ffi::c_int != NUL && !got_int.get() {
        let mut tmp: digr_T = digr_T {
            char1: 0,
            char2: 0,
            result: 0,
        };
        tmp.char1 = (*dp).char1;
        tmp.char2 = (*dp).char2;
        tmp.result = getexactdigraph(
            tmp.char1 as ::core::ffi::c_int,
            tmp.char2 as ::core::ffi::c_int,
            false_0 != 0,
        ) as result_T;
        if tmp.result != 0 as ::core::ffi::c_int && tmp.result != tmp.char2 as ::core::ffi::c_int {
            printdigraph(
                &raw mut tmp,
                if use_headers as ::core::ffi::c_int != 0 {
                    &raw mut previous
                } else {
                    ::core::ptr::null_mut::<result_T>()
                },
            );
        }
        dp = dp.offset(1);
        fast_breakcheck();
    }
    dp = (*user_digraphs.ptr()).ga_data as *const digr_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*user_digraphs.ptr()).ga_len && !got_int.get() {
        if previous >= 0 as ::core::ffi::c_int && use_headers as ::core::ffi::c_int != 0 {
            digraph_header(gettext(b"Custom\0".as_ptr() as *const ::core::ffi::c_char));
        }
        previous = -1 as ::core::ffi::c_int as result_T;
        printdigraph(dp, ::core::ptr::null_mut::<result_T>());
        fast_breakcheck();
        dp = dp.offset(1);
        i += 1;
    }
}
unsafe extern "C" fn digraph_getlist_appendpair(mut dp: *const digr_T, mut l: *mut list_T) {
    let mut l2: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
    tv_list_append_list(l, l2);
    let mut buf: [::core::ffi::c_char; 30] = [0; 30];
    buf[0 as ::core::ffi::c_int as usize] = (*dp).char1 as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = (*dp).char2 as ::core::ffi::c_char;
    buf[2 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    tv_list_append_string(l2, &raw mut buf as *mut ::core::ffi::c_char, -1 as ssize_t);
    let mut p: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
    p = p.offset(utf_char2bytes((*dp).result as ::core::ffi::c_int, p) as isize);
    *p = NUL as ::core::ffi::c_char;
    tv_list_append_string(l2, &raw mut buf as *mut ::core::ffi::c_char, -1 as ssize_t);
}
pub unsafe extern "C" fn digraph_getlist_common(mut list_all: bool, mut rettv: *mut typval_T) {
    tv_list_alloc_ret(
        rettv,
        (::core::mem::size_of::<[digr_T; 1367]>() as ::core::ffi::c_int
            + (*user_digraphs.ptr()).ga_len) as ptrdiff_t,
    );
    let mut dp: *const digr_T = ::core::ptr::null::<digr_T>();
    if list_all {
        dp = digraphdefault.ptr() as *mut digr_T;
        while (*dp).char1 as ::core::ffi::c_int != NUL && !got_int.get() {
            let mut tmp: digr_T = digr_T {
                char1: 0,
                char2: 0,
                result: 0,
            };
            tmp.char1 = (*dp).char1;
            tmp.char2 = (*dp).char2;
            tmp.result = getexactdigraph(
                tmp.char1 as ::core::ffi::c_int,
                tmp.char2 as ::core::ffi::c_int,
                false_0 != 0,
            ) as result_T;
            if tmp.result != 0 as ::core::ffi::c_int
                && tmp.result != tmp.char2 as ::core::ffi::c_int
            {
                digraph_getlist_appendpair(&raw mut tmp, (*rettv).vval.v_list);
            }
            dp = dp.offset(1);
        }
    }
    dp = (*user_digraphs.ptr()).ga_data as *const digr_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*user_digraphs.ptr()).ga_len && !got_int.get() {
        digraph_getlist_appendpair(dp, (*rettv).vval.v_list);
        dp = dp.offset(1);
        i += 1;
    }
}
static header_table: GlobalCell<[dg_header_entry; 27]> = GlobalCell::new([
    dg_header_entry {
        dg_start: DG_START_LATIN,
        dg_header: b"Latin supplement\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_GREEK,
        dg_header: b"Greek and Coptic\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_CYRILLIC,
        dg_header: b"Cyrillic\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_HEBREW,
        dg_header: b"Hebrew\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_ARABIC,
        dg_header: b"Arabic\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_LATIN_EXTENDED,
        dg_header: b"Latin extended\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_GREEK_EXTENDED,
        dg_header: b"Greek extended\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_PUNCTUATION,
        dg_header: b"Punctuation\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_SUB_SUPER,
        dg_header: b"Super- and subscripts\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_CURRENCY,
        dg_header: b"Currency\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_OTHER1,
        dg_header: b"Other\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_ROMAN,
        dg_header: b"Roman numbers\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_ARROWS,
        dg_header: b"Arrows\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_MATH,
        dg_header: b"Mathematical operators\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_TECHNICAL,
        dg_header: b"Technical\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_OTHER2,
        dg_header: b"Other\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_DRAWING,
        dg_header: b"Box drawing\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_BLOCK,
        dg_header: b"Block elements\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_SHAPES,
        dg_header: b"Geometric shapes\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_SYMBOLS,
        dg_header: b"Symbols\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_DINGBATS,
        dg_header: b"Dingbats\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_CJK_SYMBOLS,
        dg_header: b"CJK symbols and punctuation\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_HIRAGANA,
        dg_header: b"Hiragana\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_KATAKANA,
        dg_header: b"Katakana\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_BOPOMOFO,
        dg_header: b"Bopomofo\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: DG_START_OTHER3,
        dg_header: b"Other\0".as_ptr() as *const ::core::ffi::c_char,
    },
    dg_header_entry {
        dg_start: 0xfffffff as ::core::ffi::c_int,
        dg_header: ::core::ptr::null::<::core::ffi::c_char>(),
    },
]);
unsafe extern "C" fn printdigraph(mut dp: *const digr_T, mut previous: *mut result_T) {
    let mut buf: [::core::ffi::c_char; 30] = [0; 30];
    let mut list_width: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
    if (*dp).result == 0 as ::core::ffi::c_int {
        return;
    }
    if !previous.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !(*header_table.ptr())[i as usize].dg_header.is_null() {
            if *previous < (*header_table.ptr())[i as usize].dg_start
                && (*dp).result >= (*header_table.ptr())[i as usize].dg_start
                && (*dp).result
                    < (*header_table.ptr())[(i + 1 as ::core::ffi::c_int) as usize].dg_start
            {
                digraph_header(gettext((*header_table.ptr())[i as usize].dg_header));
                break;
            } else {
                i += 1;
            }
        }
        *previous = (*dp).result;
    }
    if msg_col.get() > Columns.get() - list_width {
        msg_putchar('\n' as ::core::ffi::c_int);
    }
    if msg_col.get() % list_width != 0 as ::core::ffi::c_int {
        msg_advance((msg_col.get() / list_width + 1 as ::core::ffi::c_int) * list_width);
    }
    let mut p: *mut ::core::ffi::c_char =
        (&raw mut buf as *mut ::core::ffi::c_char).offset(0 as ::core::ffi::c_int as isize);
    let c2rust_fresh2 = p;
    p = p.offset(1);
    *c2rust_fresh2 = (*dp).char1 as ::core::ffi::c_char;
    let c2rust_fresh3 = p;
    p = p.offset(1);
    *c2rust_fresh3 = (*dp).char2 as ::core::ffi::c_char;
    let c2rust_fresh4 = p;
    p = p.offset(1);
    *c2rust_fresh4 = ' ' as ::core::ffi::c_char;
    *p = NUL as ::core::ffi::c_char;
    msg_outtrans(
        &raw mut buf as *mut ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        false_0 != 0,
    );
    p = &raw mut buf as *mut ::core::ffi::c_char;
    if utf_iscomposing_first((*dp).result as ::core::ffi::c_int) {
        let c2rust_fresh5 = p;
        p = p.offset(1);
        *c2rust_fresh5 = ' ' as ::core::ffi::c_char;
    }
    p = p.offset(utf_char2bytes((*dp).result as ::core::ffi::c_int, p) as isize);
    *p = NUL as ::core::ffi::c_char;
    msg_outtrans(
        &raw mut buf as *mut ::core::ffi::c_char,
        HLF_8 as ::core::ffi::c_int,
        false_0 != 0,
    );
    p = &raw mut buf as *mut ::core::ffi::c_char;
    if char2cells((*dp).result as ::core::ffi::c_int) == 1 as ::core::ffi::c_int {
        let c2rust_fresh6 = p;
        p = p.offset(1);
        *c2rust_fresh6 = ' ' as ::core::ffi::c_char;
    }
    '_c2rust_label: {
        if p >= &raw mut buf as *mut ::core::ffi::c_char {
        } else {
            __assert_fail(
                b"p >= buf\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/digraph.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1877 as ::core::ffi::c_uint,
                b"void printdigraph(const digr_T *, result_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    vim_snprintf(
        p,
        ::core::mem::size_of::<[::core::ffi::c_char; 30]>()
            .wrapping_sub(p.offset_from(&raw mut buf as *mut ::core::ffi::c_char) as size_t),
        b" %3d\0".as_ptr() as *const ::core::ffi::c_char,
        (*dp).result,
    );
    msg_outtrans(
        &raw mut buf as *mut ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        false_0 != 0,
    );
}
unsafe extern "C" fn get_digraph_chars(
    mut arg: *const typval_T,
    mut char1: *mut ::core::ffi::c_int,
    mut char2: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut buf_chars: [::core::ffi::c_char; 65] = [0; 65];
    let mut chars: *const ::core::ffi::c_char =
        tv_get_string_buf_chk(arg, &raw mut buf_chars as *mut ::core::ffi::c_char);
    let mut p: *const ::core::ffi::c_char = chars;
    if !p.is_null() {
        if *p as ::core::ffi::c_int != NUL {
            *char1 = mb_cptr2char_adv(&raw mut p);
            if *p as ::core::ffi::c_int != NUL {
                *char2 = mb_cptr2char_adv(&raw mut p);
                if *p as ::core::ffi::c_int == NUL {
                    if check_digraph_chars_valid(*char1, *char2) {
                        return OK;
                    }
                    return FAIL;
                }
            }
        }
    }
    semsg(
        gettext(
            (e_digraph_must_be_just_two_characters_str.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ),
        chars,
    );
    return FAIL;
}
unsafe extern "C" fn digraph_set_common(
    mut argchars: *const typval_T,
    mut argdigraph: *const typval_T,
) -> bool {
    let mut char1: ::core::ffi::c_int = 0;
    let mut char2: ::core::ffi::c_int = 0;
    if get_digraph_chars(argchars, &raw mut char1, &raw mut char2) == FAIL {
        return false_0 != 0;
    }
    let mut buf_digraph: [::core::ffi::c_char; 65] = [0; 65];
    let mut digraph: *const ::core::ffi::c_char =
        tv_get_string_buf_chk(argdigraph, &raw mut buf_digraph as *mut ::core::ffi::c_char);
    if digraph.is_null() {
        return false_0 != 0;
    }
    let mut p: *const ::core::ffi::c_char = digraph;
    let mut n: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut p);
    if *p as ::core::ffi::c_int != NUL {
        semsg(
            gettext(
                (e_digraph_argument_must_be_one_character_str.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            digraph,
        );
        return false_0 != 0;
    }
    registerdigraph(char1, char2, n);
    return true_0 != 0;
}
pub unsafe extern "C" fn f_digraph_get(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut digraphs: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if digraphs.is_null() {
        return;
    }
    if strlen(digraphs) != 2 as size_t {
        semsg(
            gettext(
                (e_digraph_must_be_just_two_characters_str.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            digraphs,
        );
        return;
    }
    let mut code: ::core::ffi::c_int = digraph_get(
        *digraphs.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        *digraphs.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        false_0 != 0,
    );
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    buf[utf_char2bytes(code, &raw mut buf as *mut ::core::ffi::c_char) as usize] =
        NUL as ::core::ffi::c_char;
    (*rettv).vval.v_string = xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
}
pub unsafe extern "C" fn f_digraph_getlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_opt_bool_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut flag_list_all: bool = false;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flag_list_all = false_0 != 0;
    } else {
        let mut flag: varnumber_T = tv_get_bool(argvars.offset(0 as ::core::ffi::c_int as isize));
        flag_list_all = flag != 0 as varnumber_T;
    }
    digraph_getlist_common(flag_list_all, rettv);
}
pub unsafe extern "C" fn f_digraph_set(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_BOOL;
    (*rettv).vval.v_bool = kBoolVarFalse;
    if !digraph_set_common(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        argvars.offset(1 as ::core::ffi::c_int as isize),
    ) {
        return;
    }
    (*rettv).vval.v_bool = kBoolVarTrue;
}
pub unsafe extern "C" fn f_digraph_setlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_BOOL;
    (*rettv).vval.v_bool = kBoolVarFalse;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(
            (e_digraph_setlist_argument_must_be_list_of_lists_with_two_items.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut pl: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if pl.is_null() {
        (*rettv).vval.v_bool = kBoolVarTrue;
        return;
    }
    let l_: *const list_T = pl;
    if !l_.is_null() {
        let mut pli: *const listitem_T = (*l_).lv_first;
        while !pli.is_null() {
            if (*pli).li_tv.v_type as ::core::ffi::c_uint
                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(
                    (e_digraph_setlist_argument_must_be_list_of_lists_with_two_items.ptr()
                        as *const _) as *const ::core::ffi::c_char,
                ));
                return;
            }
            let mut l: *mut list_T = (*pli).li_tv.vval.v_list;
            if l.is_null() || tv_list_len(l) != 2 as ::core::ffi::c_int {
                emsg(gettext(
                    (e_digraph_setlist_argument_must_be_list_of_lists_with_two_items.ptr()
                        as *const _) as *const ::core::ffi::c_char,
                ));
                return;
            }
            if !digraph_set_common(
                &raw mut (*(tv_list_first
                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                    l
                ))
                .li_tv,
                &raw mut (*(*(tv_list_first
                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                    l
                ))
                .li_next)
                    .li_tv,
            ) {
                return;
            }
            pli = (*pli).li_next;
        }
    }
    (*rettv).vval.v_bool = kBoolVarTrue;
}
pub unsafe extern "C" fn keymap_init() -> *mut ::core::ffi::c_char {
    (*curbuf.get()).b_kmap_state =
        ((*curbuf.get()).b_kmap_state as ::core::ffi::c_int & !KEYMAP_INIT) as int16_t;
    if *(*curbuf.get()).b_p_keymap as ::core::ffi::c_int == NUL {
        keymap_unload();
        do_cmdline_cmd(b"unlet! b:keymap_name\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        let mut buflen: size_t = strlen((*curbuf.get()).b_p_keymap)
            .wrapping_add(strlen(p_enc.get()))
            .wrapping_add(14 as size_t);
        let mut buf: *mut ::core::ffi::c_char = xmalloc(buflen) as *mut ::core::ffi::c_char;
        vim_snprintf(
            buf,
            buflen,
            b"keymap/%s_%s.vim\0".as_ptr() as *const ::core::ffi::c_char,
            (*curbuf.get()).b_p_keymap,
            p_enc.get(),
        );
        if source_runtime(buf, 0 as ::core::ffi::c_int) == FAIL {
            vim_snprintf(
                buf,
                buflen,
                b"keymap/%s.vim\0".as_ptr() as *const ::core::ffi::c_char,
                (*curbuf.get()).b_p_keymap,
            );
            if source_runtime(buf, 0 as ::core::ffi::c_int) == FAIL {
                xfree(buf as *mut ::core::ffi::c_void);
                return b"E544: Keymap file not found\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
        }
        xfree(buf as *mut ::core::ffi::c_void);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn ex_loadkeymap(mut eap: *mut exarg_T) {
    let mut buf: [::core::ffi::c_char; 211] = [0; 211];
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    if !getline_equal(
        (*eap).ea_getline,
        (*eap).cookie,
        Some(
            getsourceline
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                    ::core::ffi::c_int,
                    bool,
                ) -> *mut ::core::ffi::c_char,
        ),
    ) {
        emsg(gettext(
            b"E105: Using :loadkeymap not in a sourced file\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    keymap_unload();
    (*curbuf.get()).b_kmap_state = 0 as int16_t;
    ga_init(
        &raw mut (*curbuf.get()).b_kmap_ga,
        ::core::mem::size_of::<kmap_T>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    p_cpo.set(b"C\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
    loop {
        let mut line: *mut ::core::ffi::c_char =
            (*eap).ea_getline.expect("non-null function pointer")(
                0 as ::core::ffi::c_int,
                (*eap).cookie,
                0 as ::core::ffi::c_int,
                true_0 != 0,
            );
        if line.is_null() {
            break;
        }
        let mut p: *mut ::core::ffi::c_char = skipwhite(line);
        if *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL
        {
            let mut kp: *mut kmap_T = ga_append_via_ptr(
                &raw mut (*curbuf.get()).b_kmap_ga,
                ::core::mem::size_of::<kmap_T>(),
            ) as *mut kmap_T;
            let mut s: *mut ::core::ffi::c_char = skiptowhite(p);
            (*kp).from = xmemdupz(p as *const ::core::ffi::c_void, s.offset_from(p) as size_t)
                as *mut ::core::ffi::c_char;
            p = skipwhite(s);
            s = skiptowhite(p);
            (*kp).to = xmemdupz(p as *const ::core::ffi::c_void, s.offset_from(p) as size_t)
                as *mut ::core::ffi::c_char;
            if strlen((*kp).from).wrapping_add(strlen((*kp).to)) >= KMAP_LLEN as size_t
                || *(*kp).from as ::core::ffi::c_int == NUL
                || *(*kp).to as ::core::ffi::c_int == NUL
            {
                if *(*kp).to as ::core::ffi::c_int == NUL {
                    emsg(gettext(
                        b"E791: Empty keymap entry\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                xfree((*kp).from as *mut ::core::ffi::c_void);
                xfree((*kp).to as *mut ::core::ffi::c_void);
                (*curbuf.get()).b_kmap_ga.ga_len -= 1;
            }
        }
        xfree(line as *mut ::core::ffi::c_void);
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*curbuf.get()).b_kmap_ga.ga_len {
        vim_snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 211]>(),
            b"<buffer> %s %s\0".as_ptr() as *const ::core::ffi::c_char,
            (*((*curbuf.get()).b_kmap_ga.ga_data as *mut kmap_T).offset(i as isize)).from,
            (*((*curbuf.get()).b_kmap_ga.ga_data as *mut kmap_T).offset(i as isize)).to,
        );
        do_map(
            MAPTYPE_MAP as ::core::ffi::c_int,
            &raw mut buf as *mut ::core::ffi::c_char,
            MODE_LANGMAP as ::core::ffi::c_int,
            false_0 != 0,
        );
        i += 1;
    }
    p_cpo.set(save_cpo);
    (*curbuf.get()).b_kmap_state =
        ((*curbuf.get()).b_kmap_state as ::core::ffi::c_int | KEYMAP_LOADED) as int16_t;
    status_redraw_curbuf();
}
pub const KMAP_LLEN: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
pub unsafe extern "C" fn keymap_ga_clear(mut kmap_ga: *mut garray_T) {
    let mut kp: *mut kmap_T = (*kmap_ga).ga_data as *mut kmap_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*kmap_ga).ga_len {
        xfree((*kp.offset(i as isize)).from as *mut ::core::ffi::c_void);
        xfree((*kp.offset(i as isize)).to as *mut ::core::ffi::c_void);
        i += 1;
    }
}
unsafe extern "C" fn keymap_unload() {
    let mut buf: [::core::ffi::c_char; 30] = [0; 30];
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    if (*curbuf.get()).b_kmap_state as ::core::ffi::c_int & KEYMAP_LOADED == 0 {
        return;
    }
    p_cpo.set(b"C\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
    let mut kp: *mut kmap_T = (*curbuf.get()).b_kmap_ga.ga_data as *mut kmap_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*curbuf.get()).b_kmap_ga.ga_len {
        vim_snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
            b"<buffer> %s\0".as_ptr() as *const ::core::ffi::c_char,
            (*kp.offset(i as isize)).from,
        );
        do_map(
            MAPTYPE_UNMAP as ::core::ffi::c_int,
            &raw mut buf as *mut ::core::ffi::c_char,
            MODE_LANGMAP as ::core::ffi::c_int,
            false_0 != 0,
        );
        i += 1;
    }
    keymap_ga_clear(&raw mut (*curbuf.get()).b_kmap_ga);
    p_cpo.set(save_cpo);
    ga_clear(&raw mut (*curbuf.get()).b_kmap_ga);
    (*curbuf.get()).b_kmap_state =
        ((*curbuf.get()).b_kmap_state as ::core::ffi::c_int & !KEYMAP_LOADED) as int16_t;
    status_redraw_curbuf();
}
pub unsafe extern "C" fn get_keymap_str(
    mut wp: *mut win_T,
    mut fmt: *mut ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*(*wp).w_buffer).b_p_iminsert != B_IMODE_LMAP as OptInt {
        return 0 as ::core::ffi::c_int;
    }
    let mut old_curbuf: *mut buf_T = curbuf.get();
    let mut old_curwin: *mut win_T = curwin.get();
    let mut to_evaluate: [::core::ffi::c_char; 14] =
        ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"b:keymap_name\0");
    curbuf.set((*wp).w_buffer);
    curwin.set(wp);
    (*emsg_skip.ptr()) += 1;
    p = eval_to_string(
        &raw mut to_evaluate as *mut ::core::ffi::c_char,
        false_0 != 0,
        false_0 != 0,
    );
    let mut s: *mut ::core::ffi::c_char = p;
    (*emsg_skip.ptr()) -= 1;
    curbuf.set(old_curbuf);
    curwin.set(old_curwin);
    if p.is_null() || *p as ::core::ffi::c_int == NUL {
        if (*(*wp).w_buffer).b_kmap_state as ::core::ffi::c_int & KEYMAP_LOADED != 0 {
            p = (*(*wp).w_buffer).b_p_keymap;
        } else {
            p = b"lang\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
    }
    let mut plen: ::core::ffi::c_int = vim_snprintf(buf, len as size_t, fmt, p);
    xfree(s as *mut ::core::ffi::c_void);
    if plen < 0 as ::core::ffi::c_int || plen > len - 1 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        plen = 0 as ::core::ffi::c_int;
    }
    return plen;
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
pub const B_IMODE_LMAP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYMAP_INIT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYMAP_LOADED: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const K_BS: ::core::ffi::c_int =
    -('k' as ::core::ffi::c_int + (('b' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
