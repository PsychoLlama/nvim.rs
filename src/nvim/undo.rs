use crate::src::nvim::autocmd::{block_autocmds, unblock_autocmds};
use crate::src::nvim::buffer::{bt_dontwrite, bt_prompt, buf_is_empty};
use crate::src::nvim::buffer_updates::{buf_updates_changedtick, buf_updates_unload};
use crate::src::nvim::change::{
    change_warning, changed, changed_bytes, changed_lines, file_ff_differs, unchanged,
};
use crate::src::nvim::cursor::{
    check_cursor, check_cursor_col, check_cursor_lnum, check_pos, coladvance, getviscol,
};
use crate::src::nvim::drawscreen::{redrawWinline, redraw_later};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::funcs::get_buf_arg;
use crate::src::nvim::eval::typval::{
    tv_dict_add_list, tv_dict_add_nr, tv_dict_alloc, tv_dict_alloc_ret, tv_get_string,
    tv_list_alloc, tv_list_append_dict,
};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_docmd::expr_map_locked;
use crate::src::nvim::ex_getln::{text_locked, text_locked_msg};
use crate::src::nvim::extmark::{extmark_apply_undo, extmark_splice_cols};
use crate::src::nvim::fileio::{get2c, get4c, get8ctime, read_eintr};
use crate::src::nvim::fold::foldOpenCursor;
use crate::src::nvim::garray::{ga_clear_strings, ga_grow, ga_init};
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    curbuf, curtab, curwin, e_modifiable, e_sandbox, e_textlock, fdo_flags, firstbuf, firstwin,
    global_busy, got_int, no_u_sync, p_cpo, p_fs, p_udir, p_ul, p_verbose, sandbox, textlock,
    IObuff, KeyTyped, VIsual, VIsual_active,
};
use crate::src::nvim::mark::{free_fmark, mark_adjust, setpcmark};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memline::{
    ml_append_flags, ml_delete, ml_get, ml_get_buf, ml_replace, resolve_symlink,
};
use crate::src::nvim::memory::{
    time_to_bytes, xcalloc, xfree, xmalloc, xmallocz, xrealloc, xstrdup, xstrlcat,
};
use crate::src::nvim::message::{
    emsg, give_warning, iemsg, internal_error, messaging, msg, msg_end, msg_ext_set_kind,
    msg_putchar, msg_puts, msg_puts_hl, msg_start, semsg, smsg, smsg_keep, verb_msg, verbose_enter,
    verbose_leave,
};
use crate::src::nvim::option::copy_option_part;
use crate::src::nvim::os::fs::{
    os_fchown, os_fileinfo, os_fopen, os_free_acl, os_fsync, os_get_acl, os_getperm, os_isdir,
    os_mkdir_recurse, os_open, os_path_exists, os_remove, os_set_acl, os_setperm,
};
use crate::src::nvim::os::input::fast_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, abort, close, fclose, fdopen, fflush, fread, fwrite, getc, gettext, getuid,
    memcmp, memmove, memset, ngettext, strcmp, strftime, strlen, time,
};
use crate::src::nvim::os::time::{os_localtime_r, os_time};
use crate::src::nvim::path::{concat_fnames, path_tail, vim_ispathsep, FullName_save};
use crate::src::nvim::sha256::{Sha256, SHA256_SUM_SIZE};
use crate::src::nvim::spell::spell_check_window;
use crate::src::nvim::state::virtual_active;
use crate::src::nvim::strings::{sort_strings, vim_snprintf, vim_snprintf_add, vim_strchr};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CMD_index, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, EvalFuncData, ExtmarkMove, ExtmarkOp, ExtmarkSavePos,
    ExtmarkSplice, ExtmarkUndoObject, FileID, FileInfo, FloatAnchor, FloatRelative, GridView,
    Intersection, LineGetter, ListLenSpecials, LuaRef, MTKey, MTNode, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    MsgpackRpcRequestHandler, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal, Timestamp, UndoObjectType,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, __gid_t,
    __off64_t, __off_t, __time_t, __uid_t, alist_T, bcount_t, bhdr_T, blob_T, blobvar_S, blocknr_T,
    buf_T, bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_17, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    eslist_T, eslist_elem, exarg, exarg_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_11,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, gid_t, handle_T,
    hash_T, hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T,
    list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, ssize_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T,
    synblock_T, synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, tm, typval_T,
    typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uid_t, uint16_t, uint32_t, uint64_t, uint8_t, uintmax_t, undo_object,
    undo_object_data as C2Rust_Unnamed_6, uv_gid_t, uv_stat_t, uv_timespec_t, uv_uid_t,
    varnumber_T, vim_acl_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T,
    xfmark_T, FILE, QUEUE, _IO_FILE,
};
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
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_14 = 76;
pub const HLF_PRE: C2Rust_Unnamed_14 = 75;
pub const HLF_OK: C2Rust_Unnamed_14 = 74;
pub const HLF_SO: C2Rust_Unnamed_14 = 73;
pub const HLF_SE: C2Rust_Unnamed_14 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_14 = 71;
pub const HLF_TS: C2Rust_Unnamed_14 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_14 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_14 = 68;
pub const HLF_CU: C2Rust_Unnamed_14 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_14 = 66;
pub const HLF_WBR: C2Rust_Unnamed_14 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_14 = 64;
pub const HLF_MSG: C2Rust_Unnamed_14 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_14 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_14 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_14 = 60;
pub const HLF_0: C2Rust_Unnamed_14 = 59;
pub const HLF_QFL: C2Rust_Unnamed_14 = 58;
pub const HLF_MC: C2Rust_Unnamed_14 = 57;
pub const HLF_CUL: C2Rust_Unnamed_14 = 56;
pub const HLF_CUC: C2Rust_Unnamed_14 = 55;
pub const HLF_TPF: C2Rust_Unnamed_14 = 54;
pub const HLF_TPS: C2Rust_Unnamed_14 = 53;
pub const HLF_TP: C2Rust_Unnamed_14 = 52;
pub const HLF_PBR: C2Rust_Unnamed_14 = 51;
pub const HLF_PST: C2Rust_Unnamed_14 = 50;
pub const HLF_PSB: C2Rust_Unnamed_14 = 49;
pub const HLF_PSX: C2Rust_Unnamed_14 = 48;
pub const HLF_PNX: C2Rust_Unnamed_14 = 47;
pub const HLF_PSK: C2Rust_Unnamed_14 = 46;
pub const HLF_PNK: C2Rust_Unnamed_14 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_14 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_14 = 43;
pub const HLF_PSI: C2Rust_Unnamed_14 = 42;
pub const HLF_PNI: C2Rust_Unnamed_14 = 41;
pub const HLF_SPL: C2Rust_Unnamed_14 = 40;
pub const HLF_SPR: C2Rust_Unnamed_14 = 39;
pub const HLF_SPC: C2Rust_Unnamed_14 = 38;
pub const HLF_SPB: C2Rust_Unnamed_14 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_14 = 36;
pub const HLF_SC: C2Rust_Unnamed_14 = 35;
pub const HLF_TXA: C2Rust_Unnamed_14 = 34;
pub const HLF_TXD: C2Rust_Unnamed_14 = 33;
pub const HLF_DED: C2Rust_Unnamed_14 = 32;
pub const HLF_CHD: C2Rust_Unnamed_14 = 31;
pub const HLF_ADD: C2Rust_Unnamed_14 = 30;
pub const HLF_FC: C2Rust_Unnamed_14 = 29;
pub const HLF_FL: C2Rust_Unnamed_14 = 28;
pub const HLF_WM: C2Rust_Unnamed_14 = 27;
pub const HLF_W: C2Rust_Unnamed_14 = 26;
pub const HLF_VNC: C2Rust_Unnamed_14 = 25;
pub const HLF_V: C2Rust_Unnamed_14 = 24;
pub const HLF_T: C2Rust_Unnamed_14 = 23;
pub const HLF_VSP: C2Rust_Unnamed_14 = 22;
pub const HLF_C: C2Rust_Unnamed_14 = 21;
pub const HLF_SNC: C2Rust_Unnamed_14 = 20;
pub const HLF_S: C2Rust_Unnamed_14 = 19;
pub const HLF_R: C2Rust_Unnamed_14 = 18;
pub const HLF_CLF: C2Rust_Unnamed_14 = 17;
pub const HLF_CLS: C2Rust_Unnamed_14 = 16;
pub const HLF_CLN: C2Rust_Unnamed_14 = 15;
pub const HLF_LNB: C2Rust_Unnamed_14 = 14;
pub const HLF_LNA: C2Rust_Unnamed_14 = 13;
pub const HLF_N: C2Rust_Unnamed_14 = 12;
pub const HLF_CM: C2Rust_Unnamed_14 = 11;
pub const HLF_M: C2Rust_Unnamed_14 = 10;
pub const HLF_LC: C2Rust_Unnamed_14 = 9;
pub const HLF_L: C2Rust_Unnamed_14 = 8;
pub const HLF_I: C2Rust_Unnamed_14 = 7;
pub const HLF_E: C2Rust_Unnamed_14 = 6;
pub const HLF_D: C2Rust_Unnamed_14 = 5;
pub const HLF_AT: C2Rust_Unnamed_14 = 4;
pub const HLF_TERM: C2Rust_Unnamed_14 = 3;
pub const HLF_EOB: C2Rust_Unnamed_14 = 2;
pub const HLF_8: C2Rust_Unnamed_14 = 1;
pub const HLF_NONE: C2Rust_Unnamed_14 = 0;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const UNDO_HASH_SIZE: C2Rust_Unnamed_15 = 32;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const UH_RELOAD: C2Rust_Unnamed_16 = 4;
pub const UH_EMPTYBUF: C2Rust_Unnamed_16 = 2;
pub const UH_CHANGED: C2Rust_Unnamed_16 = 1;
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
pub const UPD_CLEAR: C2Rust_Unnamed_18 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_18 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_18 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_18 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_18 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_18 = 20;
pub const UPD_VALID: C2Rust_Unnamed_18 = 10;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_19 = 4;
pub const BL_SOL: C2Rust_Unnamed_19 = 2;
pub const BL_WHITE: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const kOptFdoFlagJump: C2Rust_Unnamed_20 = 1024;
pub const kOptFdoFlagUndo: C2Rust_Unnamed_20 = 512;
pub const kOptFdoFlagInsert: C2Rust_Unnamed_20 = 256;
pub const kOptFdoFlagTag: C2Rust_Unnamed_20 = 128;
pub const kOptFdoFlagSearch: C2Rust_Unnamed_20 = 64;
pub const kOptFdoFlagQuickfix: C2Rust_Unnamed_20 = 32;
pub const kOptFdoFlagPercent: C2Rust_Unnamed_20 = 16;
pub const kOptFdoFlagMark: C2Rust_Unnamed_20 = 8;
pub const kOptFdoFlagHor: C2Rust_Unnamed_20 = 4;
pub const kOptFdoFlagBlock: C2Rust_Unnamed_20 = 2;
pub const kOptFdoFlagAll: C2Rust_Unnamed_20 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufinfo_T {
    pub bi_buf: *mut buf_T,
    pub bi_fp: *mut FILE,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_EXCL: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const __O_NOFOLLOW: ::core::ffi::c_int = 0o400000 as ::core::ffi::c_int;
pub const O_NOFOLLOW: ::core::ffi::c_int = __O_NOFOLLOW;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const NMARKS: ::core::ffi::c_int =
    'z' as ::core::ffi::c_int - 'a' as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn clearpos(mut a: *mut pos_T) {
    (*a).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*a).col = 0 as ::core::ffi::c_int as colnr_T;
    (*a).coladd = 0 as ::core::ffi::c_int as colnr_T;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const CPO_UNDO: ::core::ffi::c_int = 'u' as ::core::ffi::c_int;
pub const NO_LOCAL_UNDOLEVEL: ::core::ffi::c_int = -123456 as ::core::ffi::c_int;
static e_undo_list_corrupt: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E439: Undo list corrupt\0")
});
static e_undo_line_missing: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E440: Undo line missing\0")
});
static e_write_error_in_undo_file_str: GlobalCell<[::core::ffi::c_char; 35]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
            *b"E829: Write error in undo file: %s\0",
        )
    });
static u_newcount: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static u_oldcount: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static undo_undoes: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static lastmark: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub unsafe extern "C" fn u_save_cursor() -> ::core::ffi::c_int {
    let mut cur: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut top: linenr_T = if cur > 0 as linenr_T {
        cur - 1 as linenr_T
    } else {
        0 as linenr_T
    };
    let mut bot: linenr_T = cur + 1 as linenr_T;
    return u_save(top, bot);
}
pub unsafe extern "C" fn u_save(mut top: linenr_T, mut bot: linenr_T) -> ::core::ffi::c_int {
    return u_save_buf(curbuf.get(), top, bot);
}
pub unsafe extern "C" fn u_save_buf(
    mut buf: *mut buf_T,
    mut top: linenr_T,
    mut bot: linenr_T,
) -> ::core::ffi::c_int {
    if top >= bot || bot > (*buf).b_ml.ml_line_count + 1 as linenr_T {
        return FAIL;
    }
    if top + 2 as linenr_T == bot {
        u_saveline(buf, top + 1 as linenr_T);
    }
    return u_savecommon(buf, top, bot, 0 as linenr_T, false_0 != 0);
}
pub unsafe extern "C" fn u_savesub(mut lnum: linenr_T) -> ::core::ffi::c_int {
    return u_savecommon(
        curbuf.get(),
        lnum - 1 as linenr_T,
        lnum + 1 as linenr_T,
        lnum + 1 as linenr_T,
        false_0 != 0,
    );
}
pub unsafe extern "C" fn u_inssub(mut lnum: linenr_T) -> ::core::ffi::c_int {
    return u_savecommon(
        curbuf.get(),
        lnum - 1 as linenr_T,
        lnum,
        lnum + 1 as linenr_T,
        false_0 != 0,
    );
}
pub unsafe extern "C" fn u_savedel(mut lnum: linenr_T, mut nlines: linenr_T) -> ::core::ffi::c_int {
    return u_savecommon(
        curbuf.get(),
        lnum - 1 as linenr_T,
        lnum + nlines,
        if nlines == (*curbuf.get()).b_ml.ml_line_count {
            2 as linenr_T
        } else {
            lnum
        },
        false_0 != 0,
    );
}
pub unsafe extern "C" fn undo_allowed(mut buf: *mut buf_T) -> bool {
    if (*buf).b_p_ma == 0 {
        emsg(gettext(
            &raw const e_modifiable as *const ::core::ffi::c_char,
        ));
        return false_0 != 0;
    }
    if sandbox.get() != 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_sandbox as *const ::core::ffi::c_char));
        return false_0 != 0;
    }
    if textlock.get() != 0 as ::core::ffi::c_int || expr_map_locked() as ::core::ffi::c_int != 0 {
        emsg(gettext(&raw const e_textlock as *const ::core::ffi::c_char));
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn get_undolevel(mut buf: *mut buf_T) -> OptInt {
    if (*buf).b_p_ul == NO_LOCAL_UNDOLEVEL as OptInt {
        return p_ul.get();
    }
    return (*buf).b_p_ul;
}
#[inline]
unsafe extern "C" fn zero_fmark_additional_data(mut fmarks: *mut fmark_T) {
    let mut i: size_t = 0 as size_t;
    while i < NMARKS as size_t {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*fmarks.offset(i as isize)).additional_data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn u_savecommon(
    mut buf: *mut buf_T,
    mut top: linenr_T,
    mut bot: linenr_T,
    mut newbot: linenr_T,
    mut reload: bool,
) -> ::core::ffi::c_int {
    if !reload {
        if !undo_allowed(buf) {
            return FAIL;
        }
        if buf == curbuf.get() {
            change_warning(buf, 0 as ::core::ffi::c_int);
        }
        if bot > (*buf).b_ml.ml_line_count + 1 as linenr_T {
            emsg(gettext(
                b"E881: Line count changed unexpectedly\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return FAIL;
        }
    }
    let mut uep: *mut u_entry_T = ::core::ptr::null_mut::<u_entry_T>();
    let mut prev_uep: *mut u_entry_T = ::core::ptr::null_mut::<u_entry_T>();
    let mut size: linenr_T = bot - top - 1 as linenr_T;
    if (*buf).b_u_synced {
        (*buf).b_new_change = true_0 != 0;
        let mut uhp: *mut u_header_T = ::core::ptr::null_mut::<u_header_T>();
        if get_undolevel(buf) >= 0 as OptInt {
            uhp = xmalloc(::core::mem::size_of::<u_header_T>()) as *mut u_header_T;
            (*uhp).uh_extmark.capacity = 0 as size_t;
            (*uhp).uh_extmark.size = (*uhp).uh_extmark.capacity;
            (*uhp).uh_extmark.items = ::core::ptr::null_mut::<ExtmarkUndoObject>();
        } else {
            uhp = ::core::ptr::null_mut::<u_header_T>();
        }
        let mut old_curhead: *mut u_header_T = (*buf).b_u_curhead;
        if !old_curhead.is_null() {
            (*buf).b_u_newhead = (*old_curhead).uh_next.ptr;
            (*buf).b_u_curhead = ::core::ptr::null_mut::<u_header_T>();
        }
        while (*buf).b_u_numhead as OptInt > get_undolevel(buf) && !(*buf).b_u_oldhead.is_null() {
            let mut uhfree: *mut u_header_T = (*buf).b_u_oldhead;
            if uhfree == old_curhead {
                u_freebranch(buf, uhfree, &raw mut old_curhead);
            } else if (*uhfree).uh_alt_next.ptr.is_null() {
                u_freeheader(buf, uhfree, &raw mut old_curhead);
            } else {
                while !(*uhfree).uh_alt_next.ptr.is_null() {
                    uhfree = (*uhfree).uh_alt_next.ptr;
                }
                u_freebranch(buf, uhfree, &raw mut old_curhead);
            }
        }
        if uhp.is_null() {
            if !old_curhead.is_null() {
                u_freebranch(buf, old_curhead, ::core::ptr::null_mut::<*mut u_header_T>());
            }
            (*buf).b_u_synced = false_0 != 0;
            return OK;
        }
        (*uhp).uh_prev.ptr = ::core::ptr::null_mut::<u_header_T>();
        (*uhp).uh_next.ptr = (*buf).b_u_newhead;
        (*uhp).uh_alt_next.ptr = old_curhead;
        if !old_curhead.is_null() {
            (*uhp).uh_alt_prev.ptr = (*old_curhead).uh_alt_prev.ptr;
            if !(*uhp).uh_alt_prev.ptr.is_null() {
                (*(*uhp).uh_alt_prev.ptr).uh_alt_next.ptr = uhp;
            }
            (*old_curhead).uh_alt_prev.ptr = uhp;
            if (*buf).b_u_oldhead == old_curhead {
                (*buf).b_u_oldhead = uhp;
            }
        } else {
            (*uhp).uh_alt_prev.ptr = ::core::ptr::null_mut::<u_header_T>();
        }
        if !(*buf).b_u_newhead.is_null() {
            (*(*buf).b_u_newhead).uh_prev.ptr = uhp;
        }
        (*buf).b_u_seq_last += 1;
        (*uhp).uh_seq = (*buf).b_u_seq_last;
        (*buf).b_u_seq_cur = (*uhp).uh_seq;
        (*uhp).uh_time = time(::core::ptr::null_mut::<time_t>());
        (*uhp).uh_save_nr = 0 as ::core::ffi::c_int;
        (*buf).b_u_time_cur = (*uhp).uh_time + 1 as time_t;
        (*uhp).uh_walk = 0 as ::core::ffi::c_int;
        (*uhp).uh_entry = ::core::ptr::null_mut::<u_entry_T>();
        (*uhp).uh_getbot_entry = ::core::ptr::null_mut::<u_entry_T>();
        (*uhp).uh_cursor = (*curwin.get()).w_cursor;
        if virtual_active(curwin.get()) as ::core::ffi::c_int != 0
            && (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
        {
            (*uhp).uh_cursor_vcol = getviscol() as colnr_T;
        } else {
            (*uhp).uh_cursor_vcol = -1 as ::core::ffi::c_int as colnr_T;
        }
        (*uhp).uh_flags = (if (*buf).b_changed != 0 {
            UH_CHANGED as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) + (if (*buf).b_ml.ml_flags & ML_EMPTY != 0 {
            UH_EMPTYBUF as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
        zero_fmark_additional_data(&raw mut (*buf).b_namedm as *mut fmark_T);
        memmove(
            &raw mut (*uhp).uh_namedm as *mut fmark_T as *mut ::core::ffi::c_void,
            &raw mut (*buf).b_namedm as *mut fmark_T as *const ::core::ffi::c_void,
            ::core::mem::size_of::<fmark_T>().wrapping_mul(NMARKS as size_t),
        );
        (*uhp).uh_visual = (*buf).b_visual;
        (*buf).b_u_newhead = uhp;
        if (*buf).b_u_oldhead.is_null() {
            (*buf).b_u_oldhead = uhp;
        }
        (*buf).b_u_numhead += 1;
    } else {
        if get_undolevel(buf) < 0 as OptInt {
            return OK;
        }
        if size == 1 as linenr_T {
            uep = u_get_headentry(buf);
            prev_uep = ::core::ptr::null_mut::<u_entry_T>();
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 10 as ::core::ffi::c_int {
                if uep.is_null() {
                    break;
                }
                if (if (*(*buf).b_u_newhead).uh_getbot_entry != uep {
                    ((*uep).ue_top + (*uep).ue_size + 1 as linenr_T
                        != (if (*uep).ue_bot == 0 as linenr_T {
                            (*buf).b_ml.ml_line_count + 1 as linenr_T
                        } else {
                            (*uep).ue_bot
                        })) as ::core::ffi::c_int
                } else {
                    ((*uep).ue_lcount != (*buf).b_ml.ml_line_count) as ::core::ffi::c_int
                }) != 0
                    || (*uep).ue_size > 1 as linenr_T
                        && top >= (*uep).ue_top
                        && top + 2 as linenr_T <= (*uep).ue_top + (*uep).ue_size + 1 as linenr_T
                {
                    break;
                }
                if (*uep).ue_size == 1 as linenr_T && (*uep).ue_top == top {
                    if i > 0 as ::core::ffi::c_int {
                        u_getbot(buf);
                        (*buf).b_u_synced = false_0 != 0;
                        (*prev_uep).ue_next = (*uep).ue_next;
                        (*uep).ue_next = (*(*buf).b_u_newhead).uh_entry;
                        (*(*buf).b_u_newhead).uh_entry = uep;
                    }
                    if newbot != 0 as linenr_T {
                        (*uep).ue_bot = newbot;
                    } else if bot > (*buf).b_ml.ml_line_count {
                        (*uep).ue_bot = 0 as ::core::ffi::c_int as linenr_T;
                    } else {
                        (*uep).ue_lcount = (*buf).b_ml.ml_line_count;
                        (*(*buf).b_u_newhead).uh_getbot_entry = uep;
                    }
                    return OK;
                }
                prev_uep = uep;
                uep = (*uep).ue_next;
                i += 1;
            }
        }
        u_getbot(buf);
    }
    uep = xmalloc(::core::mem::size_of::<u_entry_T>()) as *mut u_entry_T;
    memset(
        uep as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<u_entry_T>(),
    );
    (*uep).ue_size = size;
    (*uep).ue_top = top;
    if newbot != 0 as linenr_T {
        (*uep).ue_bot = newbot;
    } else if bot > (*buf).b_ml.ml_line_count {
        (*uep).ue_bot = 0 as ::core::ffi::c_int as linenr_T;
    } else {
        (*uep).ue_lcount = (*buf).b_ml.ml_line_count;
        (*(*buf).b_u_newhead).uh_getbot_entry = uep;
    }
    if size > 0 as linenr_T {
        (*uep).ue_array = xmalloc(
            ::core::mem::size_of::<*mut ::core::ffi::c_char>().wrapping_mul(size as size_t),
        ) as *mut *mut ::core::ffi::c_char;
        let mut lnum: linenr_T = 0;
        let mut i_0: ::core::ffi::c_int = 0;
        i_0 = 0 as ::core::ffi::c_int;
        lnum = top + 1 as linenr_T;
        while (i_0 as linenr_T) < size {
            fast_breakcheck();
            if got_int.get() {
                u_freeentry(uep, i_0);
                return FAIL;
            }
            let c2rust_fresh0 = lnum;
            lnum = lnum + 1;
            *(*uep).ue_array.offset(i_0 as isize) = u_save_line_buf(buf, c2rust_fresh0);
            i_0 += 1;
        }
    } else {
        (*uep).ue_array = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
    (*uep).ue_next = (*(*buf).b_u_newhead).uh_entry;
    (*(*buf).b_u_newhead).uh_entry = uep;
    if reload {
        (*(*buf).b_u_newhead).uh_flags |= UH_RELOAD as ::core::ffi::c_int;
    }
    (*buf).b_u_synced = false_0 != 0;
    undo_undoes.set(false_0 != 0);
    return OK;
}
pub const UF_START_MAGIC: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"Vim\x9FUnDo\xE5\0") };
pub const UF_START_MAGIC_LEN: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const UF_HEADER_MAGIC: ::core::ffi::c_int = 0x5fd0 as ::core::ffi::c_int;
pub const UF_HEADER_END_MAGIC: ::core::ffi::c_int = 0xe7aa as ::core::ffi::c_int;
pub const UF_ENTRY_MAGIC: ::core::ffi::c_int = 0xf518 as ::core::ffi::c_int;
pub const UF_ENTRY_END_MAGIC: ::core::ffi::c_int = 0x3581 as ::core::ffi::c_int;
pub const UF_VERSION: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const UF_LAST_SAVE_NR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const UHP_SAVE_NR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static e_not_open: GlobalCell<[::core::ffi::c_char; 44]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
        *b"E828: Cannot open undo file for writing: %s\0",
    )
});
#[no_mangle]
pub unsafe extern "C" fn u_compute_hash(mut buf: *mut buf_T, mut hash: *mut uint8_t) {
    let mut ctx = Sha256::new();
    let mut lnum: linenr_T = 1 as linenr_T;
    while lnum <= (*buf).b_ml.ml_line_count {
        let mut p: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum);
        // Include the terminating NUL as a line separator.
        ctx.update(::core::slice::from_raw_parts(p as *const u8, strlen(p) + 1));
        lnum += 1;
    }
    ::core::slice::from_raw_parts_mut(hash, SHA256_SUM_SIZE).copy_from_slice(&ctx.finish());
}
#[no_mangle]
pub unsafe extern "C" fn u_get_undo_file_name(
    buf_ffname: *const ::core::ffi::c_char,
    reading: bool,
) -> *mut ::core::ffi::c_char {
    let mut ffname: *const ::core::ffi::c_char = buf_ffname;
    if ffname.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut fname_buf: [::core::ffi::c_char; 4096] = [0; 4096];
    if resolve_symlink(ffname, &raw mut fname_buf as *mut ::core::ffi::c_char) == OK {
        ffname = &raw mut fname_buf as *mut ::core::ffi::c_char;
    }
    let mut dir_name: [::core::ffi::c_char; 4097] = [0; 4097];
    let mut munged_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut undo_file_name: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dirp: *mut ::core::ffi::c_char = p_udir.get();
    while *dirp as ::core::ffi::c_int != NUL {
        let mut dir_len: size_t = copy_option_part(
            &raw mut dirp,
            &raw mut dir_name as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if dir_len == 1 as size_t
            && dir_name[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
        {
            let ffname_len: size_t = strlen(ffname);
            undo_file_name =
                xmalloc(ffname_len.wrapping_add(6 as size_t)) as *mut ::core::ffi::c_char;
            memmove(
                undo_file_name as *mut ::core::ffi::c_void,
                ffname as *const ::core::ffi::c_void,
                ffname_len.wrapping_add(1 as size_t),
            );
            let tail: *mut ::core::ffi::c_char = path_tail(undo_file_name);
            let tail_len: size_t = strlen(tail);
            memmove(
                tail.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                tail as *const ::core::ffi::c_void,
                tail_len.wrapping_add(1 as size_t),
            );
            *tail = '.' as ::core::ffi::c_char;
            memmove(
                tail.offset(tail_len as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                b".un~\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>(),
            );
        } else {
            dir_name[dir_len as usize] = NUL as ::core::ffi::c_char;
            let mut p: *mut ::core::ffi::c_char = (&raw mut dir_name as *mut ::core::ffi::c_char)
                .offset(dir_len.wrapping_sub(1 as size_t) as isize);
            while dir_len > 1 as size_t
                && vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            {
                let c2rust_fresh1 = p;
                p = p.offset(-1);
                *c2rust_fresh1 = NUL as ::core::ffi::c_char;
            }
            let mut has_directory: bool = os_isdir(&raw mut dir_name as *mut ::core::ffi::c_char);
            if !has_directory && *dirp as ::core::ffi::c_int == NUL && !reading {
                let mut ret: ::core::ffi::c_int = 0;
                let mut failed_dir: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                ret = os_mkdir_recurse(
                    &raw mut dir_name as *mut ::core::ffi::c_char,
                    0o755 as int32_t,
                    &raw mut failed_dir,
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                );
                if ret != 0 as ::core::ffi::c_int {
                    semsg(
                        gettext(
                            b"E5003: Unable to create directory \"%s\" for undo file: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        failed_dir,
                        uv_strerror(ret),
                    );
                    xfree(failed_dir as *mut ::core::ffi::c_void);
                } else {
                    has_directory = true_0 != 0;
                }
            }
            if has_directory {
                if munged_name.is_null() {
                    munged_name = xstrdup(ffname);
                    let mut c: *mut ::core::ffi::c_char = munged_name;
                    while *c as ::core::ffi::c_int != NUL {
                        if vim_ispathsep(*c as ::core::ffi::c_int) {
                            *c = '%' as ::core::ffi::c_char;
                        }
                        c = c.offset(utfc_ptr2len(c) as isize);
                    }
                }
                undo_file_name = concat_fnames(
                    &raw mut dir_name as *mut ::core::ffi::c_char,
                    munged_name,
                    true_0 != 0,
                );
            }
        }
        if !undo_file_name.is_null()
            && (!reading || os_path_exists(undo_file_name) as ::core::ffi::c_int != 0)
        {
            break;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut undo_file_name as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
    xfree(munged_name as *mut ::core::ffi::c_void);
    return undo_file_name;
}
unsafe extern "C" fn corruption_error(
    mesg: *const ::core::ffi::c_char,
    file_name: *const ::core::ffi::c_char,
) {
    semsg(
        gettext(b"E825: Corrupted undo file (%s): %s\0".as_ptr() as *const ::core::ffi::c_char),
        mesg,
        file_name,
    );
}
unsafe extern "C" fn u_free_uhp(mut uhp: *mut u_header_T) {
    let mut uep: *mut u_entry_T = (*uhp).uh_entry;
    while !uep.is_null() {
        let mut nuep: *mut u_entry_T = (*uep).ue_next;
        u_freeentry(uep, (*uep).ue_size as ::core::ffi::c_int);
        uep = nuep;
    }
    xfree(uhp as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn serialize_header(mut bi: *mut bufinfo_T, mut hash: *mut uint8_t) -> bool {
    let mut buf: *mut buf_T = (*bi).bi_buf;
    let mut fp: *mut FILE = (*bi).bi_fp;
    if fwrite(
        UF_START_MAGIC.as_ptr() as *const ::core::ffi::c_void,
        UF_START_MAGIC_LEN as size_t,
        1 as size_t,
        fp,
    ) != 1 as ::core::ffi::c_ulong
    {
        return false_0 != 0;
    }
    undo_write_bytes(bi, UF_VERSION as uintmax_t, 2 as size_t);
    if !undo_write(bi, hash, UNDO_HASH_SIZE as ::core::ffi::c_int as size_t) {
        return false_0 != 0;
    }
    undo_write_bytes(bi, (*buf).b_ml.ml_line_count as uintmax_t, 4 as size_t);
    let mut len: size_t = if !(*buf).b_u_line_ptr.is_null() {
        strlen((*buf).b_u_line_ptr)
    } else {
        0 as size_t
    };
    undo_write_bytes(bi, len as uintmax_t, 4 as size_t);
    if len > 0 as size_t && !undo_write(bi, (*buf).b_u_line_ptr as *mut uint8_t, len) {
        return false_0 != 0;
    }
    undo_write_bytes(bi, (*buf).b_u_line_lnum as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, (*buf).b_u_line_colnr as uintmax_t, 4 as size_t);
    put_header_ptr(bi, (*buf).b_u_oldhead);
    put_header_ptr(bi, (*buf).b_u_newhead);
    put_header_ptr(bi, (*buf).b_u_curhead);
    undo_write_bytes(bi, (*buf).b_u_numhead as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, (*buf).b_u_seq_last as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, (*buf).b_u_seq_cur as uintmax_t, 4 as size_t);
    let mut time_buf: [uint8_t; 8] = [0; 8];
    time_to_bytes((*buf).b_u_time_cur, &raw mut time_buf as *mut uint8_t);
    undo_write(
        bi,
        &raw mut time_buf as *mut uint8_t,
        ::core::mem::size_of::<[uint8_t; 8]>(),
    );
    undo_write_bytes(bi, 4 as uintmax_t, 1 as size_t);
    undo_write_bytes(bi, UF_LAST_SAVE_NR as uintmax_t, 1 as size_t);
    undo_write_bytes(bi, (*buf).b_u_save_nr_last as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, 0 as uintmax_t, 1 as size_t);
    return true_0 != 0;
}
unsafe extern "C" fn serialize_uhp(mut bi: *mut bufinfo_T, mut uhp: *mut u_header_T) -> bool {
    if !undo_write_bytes(bi, UF_HEADER_MAGIC as uintmax_t, 2 as size_t) {
        return false_0 != 0;
    }
    put_header_ptr(bi, (*uhp).uh_next.ptr);
    put_header_ptr(bi, (*uhp).uh_prev.ptr);
    put_header_ptr(bi, (*uhp).uh_alt_next.ptr);
    put_header_ptr(bi, (*uhp).uh_alt_prev.ptr);
    undo_write_bytes(bi, (*uhp).uh_seq as uintmax_t, 4 as size_t);
    serialize_pos(bi, (*uhp).uh_cursor);
    undo_write_bytes(bi, (*uhp).uh_cursor_vcol as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, (*uhp).uh_flags as uintmax_t, 2 as size_t);
    let mut i: size_t = 0 as size_t;
    while i < NMARKS as size_t {
        serialize_pos(bi, (*uhp).uh_namedm[i as usize].mark);
        i = i.wrapping_add(1);
    }
    serialize_visualinfo(bi, &raw mut (*uhp).uh_visual);
    let mut time_buf: [uint8_t; 8] = [0; 8];
    time_to_bytes((*uhp).uh_time, &raw mut time_buf as *mut uint8_t);
    undo_write(
        bi,
        &raw mut time_buf as *mut uint8_t,
        ::core::mem::size_of::<[uint8_t; 8]>(),
    );
    undo_write_bytes(bi, 4 as uintmax_t, 1 as size_t);
    undo_write_bytes(bi, UHP_SAVE_NR as uintmax_t, 1 as size_t);
    undo_write_bytes(bi, (*uhp).uh_save_nr as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, 0 as uintmax_t, 1 as size_t);
    let mut uep: *mut u_entry_T = (*uhp).uh_entry;
    while !uep.is_null() {
        undo_write_bytes(bi, UF_ENTRY_MAGIC as uintmax_t, 2 as size_t);
        if !serialize_uep(bi, uep) {
            return false_0 != 0;
        }
        uep = (*uep).ue_next;
    }
    undo_write_bytes(bi, UF_ENTRY_END_MAGIC as uintmax_t, 2 as size_t);
    let mut i_0: size_t = 0 as size_t;
    while i_0 < (*uhp).uh_extmark.size {
        if !serialize_extmark(bi, *(*uhp).uh_extmark.items.offset(i_0 as isize)) {
            return false_0 != 0;
        }
        i_0 = i_0.wrapping_add(1);
    }
    undo_write_bytes(bi, UF_ENTRY_END_MAGIC as uintmax_t, 2 as size_t);
    return true_0 != 0;
}
unsafe extern "C" fn unserialize_uhp(
    mut bi: *mut bufinfo_T,
    mut file_name: *const ::core::ffi::c_char,
) -> *mut u_header_T {
    let mut uhp: *mut u_header_T = xmalloc(::core::mem::size_of::<u_header_T>()) as *mut u_header_T;
    memset(
        uhp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<u_header_T>(),
    );
    (*uhp).uh_next.seq = undo_read_4c(bi);
    (*uhp).uh_prev.seq = undo_read_4c(bi);
    (*uhp).uh_alt_next.seq = undo_read_4c(bi);
    (*uhp).uh_alt_prev.seq = undo_read_4c(bi);
    (*uhp).uh_seq = undo_read_4c(bi);
    if (*uhp).uh_seq <= 0 as ::core::ffi::c_int {
        corruption_error(
            b"uh_seq\0".as_ptr() as *const ::core::ffi::c_char,
            file_name,
        );
        xfree(uhp as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<u_header_T>();
    }
    unserialize_pos(bi, &raw mut (*uhp).uh_cursor);
    (*uhp).uh_cursor_vcol = undo_read_4c(bi) as colnr_T;
    (*uhp).uh_flags = undo_read_2c(bi);
    let cur_timestamp: Timestamp = os_time();
    let mut i: size_t = 0 as size_t;
    while i < NMARKS as size_t {
        unserialize_pos(
            bi,
            &raw mut (*(&raw mut (*uhp).uh_namedm as *mut fmark_T).offset(i as isize)).mark,
        );
        (*uhp).uh_namedm[i as usize].timestamp = cur_timestamp;
        (*uhp).uh_namedm[i as usize].fnum = 0 as ::core::ffi::c_int;
        i = i.wrapping_add(1);
    }
    unserialize_visualinfo(bi, &raw mut (*uhp).uh_visual);
    (*uhp).uh_time = undo_read_time(bi);
    loop {
        let mut len: ::core::ffi::c_int = undo_read_byte(bi);
        if len == EOF {
            corruption_error(
                b"truncated\0".as_ptr() as *const ::core::ffi::c_char,
                file_name,
            );
            u_free_uhp(uhp);
            return ::core::ptr::null_mut::<u_header_T>();
        }
        if len == 0 as ::core::ffi::c_int {
            break;
        }
        let mut what: ::core::ffi::c_int = undo_read_byte(bi);
        match what {
            UHP_SAVE_NR => {
                (*uhp).uh_save_nr = undo_read_4c(bi);
            }
            _ => loop {
                len -= 1;
                if len < 0 as ::core::ffi::c_int {
                    break;
                }
                undo_read_byte(bi);
            },
        }
    }
    let mut last_uep: *mut u_entry_T = ::core::ptr::null_mut::<u_entry_T>();
    let mut c: ::core::ffi::c_int = 0;
    loop {
        c = undo_read_2c(bi);
        if c != UF_ENTRY_MAGIC {
            break;
        }
        let mut error: bool = false_0 != 0;
        let mut uep: *mut u_entry_T = unserialize_uep(bi, &raw mut error, file_name);
        if last_uep.is_null() {
            (*uhp).uh_entry = uep;
        } else {
            (*last_uep).ue_next = uep;
        }
        last_uep = uep;
        if uep.is_null() || error as ::core::ffi::c_int != 0 {
            u_free_uhp(uhp);
            return ::core::ptr::null_mut::<u_header_T>();
        }
    }
    if c != UF_ENTRY_END_MAGIC {
        corruption_error(
            b"entry end\0".as_ptr() as *const ::core::ffi::c_char,
            file_name,
        );
        u_free_uhp(uhp);
        return ::core::ptr::null_mut::<u_header_T>();
    }
    (*uhp).uh_extmark.capacity = 0 as size_t;
    (*uhp).uh_extmark.size = (*uhp).uh_extmark.capacity;
    (*uhp).uh_extmark.items = ::core::ptr::null_mut::<ExtmarkUndoObject>();
    loop {
        c = undo_read_2c(bi);
        if c != UF_ENTRY_MAGIC {
            break;
        }
        let mut error_0: bool = false_0 != 0;
        let mut extup: *mut ExtmarkUndoObject =
            unserialize_extmark(bi, &raw mut error_0, file_name);
        if error_0 {
            xfree((*uhp).uh_extmark.items as *mut ::core::ffi::c_void);
            (*uhp).uh_extmark.capacity = 0 as size_t;
            (*uhp).uh_extmark.size = (*uhp).uh_extmark.capacity;
            (*uhp).uh_extmark.items = ::core::ptr::null_mut::<ExtmarkUndoObject>();
            xfree(extup as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<u_header_T>();
        }
        if (*uhp).uh_extmark.size == (*uhp).uh_extmark.capacity {
            (*uhp).uh_extmark.capacity = if (*uhp).uh_extmark.capacity != 0 {
                (*uhp).uh_extmark.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*uhp).uh_extmark.items = xrealloc(
                (*uhp).uh_extmark.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<ExtmarkUndoObject>()
                    .wrapping_mul((*uhp).uh_extmark.capacity),
            ) as *mut ExtmarkUndoObject;
        } else {
        };
        let c2rust_fresh3 = (*uhp).uh_extmark.size;
        (*uhp).uh_extmark.size = (*uhp).uh_extmark.size.wrapping_add(1);
        *(*uhp).uh_extmark.items.offset(c2rust_fresh3 as isize) = *extup;
        xfree(extup as *mut ::core::ffi::c_void);
    }
    if c != UF_ENTRY_END_MAGIC {
        corruption_error(
            b"entry end\0".as_ptr() as *const ::core::ffi::c_char,
            file_name,
        );
        u_free_uhp(uhp);
        return ::core::ptr::null_mut::<u_header_T>();
    }
    return uhp;
}
unsafe extern "C" fn serialize_extmark(
    mut bi: *mut bufinfo_T,
    mut extup: ExtmarkUndoObject,
) -> bool {
    if extup.type_0 as ::core::ffi::c_uint
        == kExtmarkSplice as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        undo_write_bytes(bi, UF_ENTRY_MAGIC as uintmax_t, 2 as size_t);
        undo_write_bytes(bi, extup.type_0 as uintmax_t, 4 as size_t);
        if !undo_write(
            bi,
            &raw mut extup.data.splice as *mut uint8_t,
            ::core::mem::size_of::<ExtmarkSplice>(),
        ) {
            return false_0 != 0;
        }
    } else if extup.type_0 as ::core::ffi::c_uint
        == kExtmarkMove as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        undo_write_bytes(bi, UF_ENTRY_MAGIC as uintmax_t, 2 as size_t);
        undo_write_bytes(bi, extup.type_0 as uintmax_t, 4 as size_t);
        if !undo_write(
            bi,
            &raw mut extup.data.move_0 as *mut uint8_t,
            ::core::mem::size_of::<ExtmarkMove>(),
        ) {
            return false_0 != 0;
        }
    }
    return true_0 != 0;
}
unsafe extern "C" fn unserialize_extmark(
    mut bi: *mut bufinfo_T,
    mut error: *mut bool,
    mut _filename: *const ::core::ffi::c_char,
) -> *mut ExtmarkUndoObject {
    let mut buf: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut extup: *mut ExtmarkUndoObject =
        xmalloc(::core::mem::size_of::<ExtmarkUndoObject>()) as *mut ExtmarkUndoObject;
    let mut type_0: UndoObjectType = undo_read_4c(bi) as UndoObjectType;
    (*extup).type_0 = type_0;
    '_error: {
        if type_0 as ::core::ffi::c_uint
            == kExtmarkSplice as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut n_elems: size_t = ::core::mem::size_of::<ExtmarkSplice>()
                .wrapping_div(::core::mem::size_of::<uint8_t>());
            buf = xcalloc(n_elems, ::core::mem::size_of::<uint8_t>()) as *mut uint8_t;
            if !undo_read(bi, buf, n_elems) {
                break '_error;
            } else {
                (*extup).data.splice = *(buf as *mut ExtmarkSplice);
            }
        } else if type_0 as ::core::ffi::c_uint
            == kExtmarkMove as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut n_elems_0: size_t = ::core::mem::size_of::<ExtmarkMove>()
                .wrapping_div(::core::mem::size_of::<uint8_t>());
            buf = xcalloc(n_elems_0, ::core::mem::size_of::<uint8_t>()) as *mut uint8_t;
            if !undo_read(bi, buf, n_elems_0) {
                break '_error;
            } else {
                (*extup).data.move_0 = *(buf as *mut ExtmarkMove);
            }
        } else {
            break '_error;
        }
        xfree(buf as *mut ::core::ffi::c_void);
        return extup;
    }
    xfree(extup as *mut ::core::ffi::c_void);
    if !buf.is_null() {
        xfree(buf as *mut ::core::ffi::c_void);
    }
    *error = true_0 != 0;
    return ::core::ptr::null_mut::<ExtmarkUndoObject>();
}
unsafe extern "C" fn serialize_uep(mut bi: *mut bufinfo_T, mut uep: *mut u_entry_T) -> bool {
    undo_write_bytes(bi, (*uep).ue_top as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, (*uep).ue_bot as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, (*uep).ue_lcount as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, (*uep).ue_size as uintmax_t, 4 as size_t);
    let mut i: size_t = 0 as size_t;
    while i < (*uep).ue_size as size_t {
        let mut len: size_t = strlen(*(*uep).ue_array.offset(i as isize));
        if !undo_write_bytes(bi, len as uintmax_t, 4 as size_t) {
            return false_0 != 0;
        }
        if len > 0 as size_t
            && !undo_write(bi, *(*uep).ue_array.offset(i as isize) as *mut uint8_t, len)
        {
            return false_0 != 0;
        }
        i = i.wrapping_add(1);
    }
    return true_0 != 0;
}
unsafe extern "C" fn unserialize_uep(
    mut bi: *mut bufinfo_T,
    mut error: *mut bool,
    mut file_name: *const ::core::ffi::c_char,
) -> *mut u_entry_T {
    let mut uep: *mut u_entry_T = xmalloc(::core::mem::size_of::<u_entry_T>()) as *mut u_entry_T;
    memset(
        uep as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<u_entry_T>(),
    );
    (*uep).ue_top = undo_read_4c(bi) as linenr_T;
    (*uep).ue_bot = undo_read_4c(bi) as linenr_T;
    (*uep).ue_lcount = undo_read_4c(bi) as linenr_T;
    (*uep).ue_size = undo_read_4c(bi) as linenr_T;
    let mut array: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    if (*uep).ue_size > 0 as linenr_T {
        if ((*uep).ue_size as size_t)
            < (SIZE_MAX as usize).wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
        {
            array = xmalloc(
                ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                    .wrapping_mul((*uep).ue_size as size_t),
            ) as *mut *mut ::core::ffi::c_char;
            memset(
                array as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                    .wrapping_mul((*uep).ue_size as size_t),
            );
        }
    }
    (*uep).ue_array = array;
    let mut i: size_t = 0 as size_t;
    while i < (*uep).ue_size as size_t {
        let mut line_len: ::core::ffi::c_int = undo_read_4c(bi);
        let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if line_len >= 0 as ::core::ffi::c_int {
            line = undo_read_string(bi, line_len as size_t);
        } else {
            line = ::core::ptr::null_mut::<::core::ffi::c_char>();
            corruption_error(
                b"line length\0".as_ptr() as *const ::core::ffi::c_char,
                file_name,
            );
        }
        if line.is_null() {
            *error = true_0 != 0;
            return uep;
        }
        *array.offset(i as isize) = line;
        i = i.wrapping_add(1);
    }
    return uep;
}
unsafe extern "C" fn serialize_pos(mut bi: *mut bufinfo_T, mut pos: pos_T) {
    undo_write_bytes(bi, pos.lnum as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, pos.col as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, pos.coladd as uintmax_t, 4 as size_t);
}
unsafe extern "C" fn unserialize_pos(mut bi: *mut bufinfo_T, mut pos: *mut pos_T) {
    (*pos).lnum = undo_read_4c(bi) as linenr_T;
    (*pos).lnum = if (*pos).lnum > 0 as linenr_T {
        (*pos).lnum
    } else {
        0 as linenr_T
    };
    (*pos).col = undo_read_4c(bi) as colnr_T;
    (*pos).col = (if (*pos).col > 0 as ::core::ffi::c_int {
        (*pos).col as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as colnr_T;
    (*pos).coladd = undo_read_4c(bi) as colnr_T;
    (*pos).coladd = (if (*pos).coladd > 0 as ::core::ffi::c_int {
        (*pos).coladd as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as colnr_T;
}
unsafe extern "C" fn serialize_visualinfo(mut bi: *mut bufinfo_T, mut info: *mut visualinfo_T) {
    serialize_pos(bi, (*info).vi_start);
    serialize_pos(bi, (*info).vi_end);
    undo_write_bytes(bi, (*info).vi_mode as uintmax_t, 4 as size_t);
    undo_write_bytes(bi, (*info).vi_curswant as uintmax_t, 4 as size_t);
}
unsafe extern "C" fn unserialize_visualinfo(mut bi: *mut bufinfo_T, mut info: *mut visualinfo_T) {
    unserialize_pos(bi, &raw mut (*info).vi_start);
    unserialize_pos(bi, &raw mut (*info).vi_end);
    (*info).vi_mode = undo_read_4c(bi);
    (*info).vi_curswant = undo_read_4c(bi) as colnr_T;
}
#[no_mangle]
pub unsafe extern "C" fn u_write_undo(
    name: *const ::core::ffi::c_char,
    forceit: bool,
    buf: *mut buf_T,
    hash: *mut uint8_t,
) {
    let mut mark: ::core::ffi::c_int = 0;
    let mut uhp: *mut u_header_T = ::core::ptr::null_mut::<u_header_T>();
    let mut fd_0: ::core::ffi::c_int = 0;
    let mut file_info_old: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    let mut file_info_new: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    let mut bi: bufinfo_T = bufinfo_T {
        bi_buf: ::core::ptr::null_mut::<buf_T>(),
        bi_fp: ::core::ptr::null_mut::<FILE>(),
    };
    let mut file_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fp: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut write_ok: bool = false_0 != 0;
    if name.is_null() {
        file_name = u_get_undo_file_name((*buf).b_ffname, false_0 != 0);
        if file_name.is_null() {
            if p_verbose.get() > 0 as OptInt {
                verbose_enter();
                smsg(
                    0 as ::core::ffi::c_int,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(
                        b"Cannot write undo file in any directory in 'undodir'\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                );
                verbose_leave();
            }
            return;
        }
    } else {
        file_name = name as *mut ::core::ffi::c_char;
    }
    let mut perm: ::core::ffi::c_int = 0o600 as ::core::ffi::c_int;
    if !(*buf).b_ffname.is_null() {
        perm = os_getperm((*buf).b_ffname) as ::core::ffi::c_int;
        if perm < 0 as ::core::ffi::c_int {
            perm = 0o600 as ::core::ffi::c_int;
        }
    }
    perm = perm & 0o666 as ::core::ffi::c_int;
    '_theend: {
        if os_path_exists(file_name) {
            if name.is_null() || !forceit {
                let mut fd: ::core::ffi::c_int =
                    os_open(file_name, O_RDONLY, 0 as ::core::ffi::c_int);
                if fd < 0 as ::core::ffi::c_int {
                    if !name.is_null() || p_verbose.get() > 0 as OptInt {
                        if name.is_null() {
                            verbose_enter();
                        }
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(
                                b"Will not overwrite with undo file, cannot read: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ),
                            file_name,
                        );
                        if name.is_null() {
                            verbose_leave();
                        }
                    }
                    break '_theend;
                } else {
                    let mut mbuf: [::core::ffi::c_char; 9] = [0; 9];
                    let mut len: ssize_t = read_eintr(
                        fd,
                        &raw mut mbuf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        UF_START_MAGIC_LEN as size_t,
                    );
                    close(fd);
                    if len < UF_START_MAGIC_LEN as ssize_t
                        || memcmp(
                            &raw mut mbuf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                            UF_START_MAGIC.as_ptr() as *const ::core::ffi::c_void,
                            UF_START_MAGIC_LEN as size_t,
                        ) != 0 as ::core::ffi::c_int
                    {
                        if !name.is_null() || p_verbose.get() > 0 as OptInt {
                            if name.is_null() {
                                verbose_enter();
                            }
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(
                                    b"Will not overwrite, this is not an undo file: %s\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ),
                                file_name,
                            );
                            if name.is_null() {
                                verbose_leave();
                            }
                        }
                        break '_theend;
                    }
                }
            }
            os_remove(file_name);
        }
        if (*buf).b_u_numhead == 0 as ::core::ffi::c_int && (*buf).b_u_line_ptr.is_null() {
            if p_verbose.get() > 0 as OptInt {
                verb_msg(gettext(
                    b"Skipping undo file write, nothing to undo\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            }
        } else {
            fd_0 = os_open(file_name, O_CREAT | O_WRONLY | O_EXCL | O_NOFOLLOW, perm);
            if fd_0 < 0 as ::core::ffi::c_int {
                semsg(
                    gettext((e_not_open.ptr() as *const _) as *const ::core::ffi::c_char),
                    file_name,
                );
            } else {
                os_setperm(file_name, perm);
                if p_verbose.get() > 0 as OptInt {
                    verbose_enter();
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Writing undo file: %s\0".as_ptr() as *const ::core::ffi::c_char),
                        file_name,
                    );
                    verbose_leave();
                }
                file_info_old = FileInfo {
                    stat: uv_stat_t {
                        st_dev: 0,
                        st_mode: 0,
                        st_nlink: 0,
                        st_uid: 0,
                        st_gid: 0,
                        st_rdev: 0,
                        st_ino: 0,
                        st_size: 0,
                        st_blksize: 0,
                        st_blocks: 0,
                        st_flags: 0,
                        st_gen: 0,
                        st_atim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_mtim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_ctim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_birthtim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                    },
                };
                file_info_new = FileInfo {
                    stat: uv_stat_t {
                        st_dev: 0,
                        st_mode: 0,
                        st_nlink: 0,
                        st_uid: 0,
                        st_gid: 0,
                        st_rdev: 0,
                        st_ino: 0,
                        st_size: 0,
                        st_blksize: 0,
                        st_blocks: 0,
                        st_flags: 0,
                        st_gen: 0,
                        st_atim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_mtim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_ctim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_birthtim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                    },
                };
                if !(*buf).b_ffname.is_null()
                    && os_fileinfo((*buf).b_ffname, &raw mut file_info_old) as ::core::ffi::c_int
                        != 0
                    && os_fileinfo(file_name, &raw mut file_info_new) as ::core::ffi::c_int != 0
                    && file_info_old.stat.st_gid != file_info_new.stat.st_gid
                    && os_fchown(
                        fd_0,
                        -1 as ::core::ffi::c_int as uv_uid_t,
                        file_info_old.stat.st_gid as uv_gid_t,
                    ) != 0
                {
                    os_setperm(
                        file_name,
                        perm & 0o707 as ::core::ffi::c_int
                            | (perm & 0o7 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int,
                    );
                }
                fp = fdopen(fd_0, b"w\0".as_ptr() as *const ::core::ffi::c_char);
                if fp.is_null() {
                    semsg(
                        gettext((e_not_open.ptr() as *const _) as *const ::core::ffi::c_char),
                        file_name,
                    );
                    close(fd_0);
                    os_remove(file_name);
                } else {
                    u_sync(true_0 != 0);
                    bi = bufinfo_T {
                        bi_buf: buf,
                        bi_fp: fp,
                    };
                    '_write_error: {
                        if serialize_header(&raw mut bi, hash) {
                            (*lastmark.ptr()) += 1;
                            mark = lastmark.get();
                            uhp = (*buf).b_u_oldhead;
                            while !uhp.is_null() {
                                if (*uhp).uh_walk != mark {
                                    (*uhp).uh_walk = mark;
                                    if !serialize_uhp(&raw mut bi, uhp) {
                                        break '_write_error;
                                    }
                                }
                                if !(*uhp).uh_prev.ptr.is_null()
                                    && (*(*uhp).uh_prev.ptr).uh_walk != mark
                                {
                                    uhp = (*uhp).uh_prev.ptr;
                                } else if !(*uhp).uh_alt_next.ptr.is_null()
                                    && (*(*uhp).uh_alt_next.ptr).uh_walk != mark
                                {
                                    uhp = (*uhp).uh_alt_next.ptr;
                                } else if !(*uhp).uh_next.ptr.is_null()
                                    && (*uhp).uh_alt_prev.ptr.is_null()
                                    && (*(*uhp).uh_next.ptr).uh_walk != mark
                                {
                                    uhp = (*uhp).uh_next.ptr;
                                } else if !(*uhp).uh_alt_prev.ptr.is_null() {
                                    uhp = (*uhp).uh_alt_prev.ptr;
                                } else {
                                    uhp = (*uhp).uh_next.ptr;
                                }
                            }
                            if undo_write_bytes(
                                &raw mut bi,
                                UF_HEADER_END_MAGIC as uintmax_t,
                                2 as size_t,
                            ) {
                                write_ok = true_0 != 0;
                            }
                            if (if (*buf).b_p_fs >= 0 as ::core::ffi::c_int {
                                (*buf).b_p_fs
                            } else {
                                p_fs.get()
                            }) != 0
                                && fflush(fp) == 0 as ::core::ffi::c_int
                                && os_fsync(fd_0) != 0 as ::core::ffi::c_int
                            {
                                write_ok = false_0 != 0;
                            }
                        }
                    }
                    fclose(fp);
                    if !write_ok {
                        semsg(
                            gettext(
                                (e_write_error_in_undo_file_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            file_name,
                        );
                    }
                    if !(*buf).b_ffname.is_null() {
                        let mut acl: vim_acl_T = os_get_acl((*buf).b_ffname);
                        os_set_acl(file_name, acl);
                        os_free_acl(acl);
                    }
                }
            }
        }
    }
    if file_name != name as *mut ::core::ffi::c_char {
        xfree(file_name as *mut ::core::ffi::c_void);
    }
}
pub unsafe extern "C" fn u_read_undo(
    mut name: *mut ::core::ffi::c_char,
    mut hash: *const uint8_t,
    mut orig_name: *const ::core::ffi::c_char,
) {
    let mut bi: bufinfo_T = bufinfo_T {
        bi_buf: ::core::ptr::null_mut::<buf_T>(),
        bi_fp: ::core::ptr::null_mut::<FILE>(),
    };
    let mut magic_buf: [::core::ffi::c_char; 9] = [0; 9];
    let mut version: ::core::ffi::c_int = 0;
    let mut read_hash: [uint8_t; 32] = [0; 32];
    let mut line_count: linenr_T = 0;
    let mut str_len: ::core::ffi::c_int = 0;
    let mut line_lnum: linenr_T = 0;
    let mut line_colnr: colnr_T = 0;
    let mut old_header_seq: ::core::ffi::c_int = 0;
    let mut new_header_seq: ::core::ffi::c_int = 0;
    let mut cur_header_seq: ::core::ffi::c_int = 0;
    let mut num_head: ::core::ffi::c_int = 0;
    let mut seq_last: ::core::ffi::c_int = 0;
    let mut seq_cur: ::core::ffi::c_int = 0;
    let mut seq_time: time_t = 0;
    let mut last_save_nr: ::core::ffi::c_int = 0;
    let mut num_read_uhps: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut old_idx: int16_t = 0;
    let mut new_idx: int16_t = 0;
    let mut cur_idx: int16_t = 0;
    let mut uhp_table: *mut *mut u_header_T = ::core::ptr::null_mut::<*mut u_header_T>();
    let mut line_ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut file_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if name.is_null() {
        file_name = u_get_undo_file_name((*curbuf.get()).b_ffname, true_0 != 0);
        if file_name.is_null() {
            return;
        }
        let mut file_info_orig: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        let mut file_info_undo: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        if os_fileinfo(orig_name, &raw mut file_info_orig) as ::core::ffi::c_int != 0
            && os_fileinfo(file_name, &raw mut file_info_undo) as ::core::ffi::c_int != 0
            && file_info_orig.stat.st_uid != file_info_undo.stat.st_uid
            && file_info_undo.stat.st_uid != getuid() as uint64_t
        {
            if p_verbose.get() > 0 as OptInt {
                verbose_enter();
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Not reading undo file, owner differs: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    file_name,
                );
                verbose_leave();
            }
            return;
        }
    } else {
        file_name = name;
    }
    if p_verbose.get() > 0 as OptInt {
        verbose_enter();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Reading undo file: %s\0".as_ptr() as *const ::core::ffi::c_char),
            file_name,
        );
        verbose_leave();
    }
    let mut fp: *mut FILE = os_fopen(file_name, b"r\0".as_ptr() as *const ::core::ffi::c_char);
    '_theend: {
        '_error: {
            if fp.is_null() {
                if !name.is_null() || p_verbose.get() > 0 as OptInt {
                    semsg(
                        gettext(b"E822: Cannot open undo file for reading: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        file_name,
                    );
                }
            } else {
                bi = bufinfo_T {
                    bi_buf: curbuf.get(),
                    bi_fp: fp,
                };
                magic_buf = [0; 9];
                if fread(
                    &raw mut magic_buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    UF_START_MAGIC_LEN as size_t,
                    1 as size_t,
                    fp,
                ) != 1 as ::core::ffi::c_ulong
                    || memcmp(
                        &raw mut magic_buf as *mut ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        UF_START_MAGIC.as_ptr() as *const ::core::ffi::c_void,
                        UF_START_MAGIC_LEN as size_t,
                    ) != 0 as ::core::ffi::c_int
                {
                    semsg(
                        gettext(
                            b"E823: Not an undo file: %s\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                        file_name,
                    );
                } else {
                    version = get2c(fp);
                    if version != UF_VERSION {
                        semsg(
                            gettext(b"E824: Incompatible undo file: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            file_name,
                        );
                    } else {
                        read_hash = [0; 32];
                        if !undo_read(
                            &raw mut bi,
                            &raw mut read_hash as *mut uint8_t,
                            UNDO_HASH_SIZE as ::core::ffi::c_int as size_t,
                        ) {
                            corruption_error(
                                b"hash\0".as_ptr() as *const ::core::ffi::c_char,
                                file_name,
                            );
                        } else {
                            line_count = undo_read_4c(&raw mut bi) as linenr_T;
                            if memcmp(
                                hash as *const ::core::ffi::c_void,
                                &raw mut read_hash as *mut uint8_t as *const ::core::ffi::c_void,
                                UNDO_HASH_SIZE as ::core::ffi::c_int as size_t,
                            ) != 0 as ::core::ffi::c_int
                                || line_count != (*curbuf.get()).b_ml.ml_line_count
                            {
                                if p_verbose.get() > 0 as OptInt || !name.is_null() {
                                    if name.is_null() {
                                        verbose_enter();
                                    }
                                    give_warning(
                                        gettext(
                                            b"File contents changed, cannot use undo info\0"
                                                .as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ),
                                        true_0 != 0,
                                        true_0 != 0,
                                    );
                                    if name.is_null() {
                                        verbose_leave();
                                    }
                                }
                            } else {
                                str_len = undo_read_4c(&raw mut bi);
                                if str_len >= 0 as ::core::ffi::c_int {
                                    if str_len > 0 as ::core::ffi::c_int {
                                        line_ptr = undo_read_string(&raw mut bi, str_len as size_t);
                                    }
                                    line_lnum = undo_read_4c(&raw mut bi) as linenr_T;
                                    line_colnr = undo_read_4c(&raw mut bi);
                                    if line_lnum < 0 as linenr_T
                                        || line_colnr < 0 as ::core::ffi::c_int
                                    {
                                        corruption_error(
                                            b"line lnum/col\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                            file_name,
                                        );
                                    } else {
                                        old_header_seq = undo_read_4c(&raw mut bi);
                                        new_header_seq = undo_read_4c(&raw mut bi);
                                        cur_header_seq = undo_read_4c(&raw mut bi);
                                        num_head = undo_read_4c(&raw mut bi);
                                        seq_last = undo_read_4c(&raw mut bi);
                                        seq_cur = undo_read_4c(&raw mut bi);
                                        seq_time = undo_read_time(&raw mut bi);
                                        last_save_nr = 0 as ::core::ffi::c_int;
                                        loop {
                                            let mut len: ::core::ffi::c_int =
                                                undo_read_byte(&raw mut bi);
                                            if len == 0 as ::core::ffi::c_int || len == EOF {
                                                break;
                                            }
                                            let mut what: ::core::ffi::c_int =
                                                undo_read_byte(&raw mut bi);
                                            match what {
                                                UF_LAST_SAVE_NR => {
                                                    last_save_nr = undo_read_4c(&raw mut bi);
                                                }
                                                _ => loop {
                                                    len -= 1;
                                                    if len < 0 as ::core::ffi::c_int {
                                                        break;
                                                    }
                                                    undo_read_byte(&raw mut bi);
                                                },
                                            }
                                        }
                                        if num_head > 0 as ::core::ffi::c_int {
                                            if (num_head as size_t)
                                                < (SIZE_MAX as usize).wrapping_div(
                                                    ::core::mem::size_of::<*mut u_header_T>(),
                                                )
                                            {
                                                uhp_table =
                                                    xmalloc((num_head as size_t).wrapping_mul(
                                                        ::core::mem::size_of::<*mut u_header_T>(),
                                                    ))
                                                        as *mut *mut u_header_T;
                                            }
                                        }
                                        num_read_uhps = 0 as ::core::ffi::c_int;
                                        c = 0;
                                        loop {
                                            c = undo_read_2c(&raw mut bi);
                                            if c != UF_HEADER_MAGIC {
                                                break;
                                            }
                                            if num_read_uhps >= num_head {
                                                corruption_error(
                                                    b"num_head too small\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                    file_name,
                                                );
                                                break '_error;
                                            } else {
                                                let mut uhp: *mut u_header_T =
                                                    unserialize_uhp(&raw mut bi, file_name);
                                                if uhp.is_null() {
                                                    break '_error;
                                                }
                                                let c2rust_fresh2 = num_read_uhps;
                                                num_read_uhps = num_read_uhps + 1;
                                                let c2rust_lvalue_ptr = &raw mut *uhp_table
                                                    .offset(c2rust_fresh2 as isize);
                                                *c2rust_lvalue_ptr = uhp;
                                            }
                                        }
                                        if num_read_uhps != num_head {
                                            corruption_error(
                                                b"num_head\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                file_name,
                                            );
                                        } else if c != UF_HEADER_END_MAGIC {
                                            corruption_error(
                                                b"end marker\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                file_name,
                                            );
                                        } else {
                                            old_idx = -1 as int16_t;
                                            new_idx = -1 as int16_t;
                                            cur_idx = -1 as int16_t;
                                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                            while i < num_head {
                                                let mut uhp_0: *mut u_header_T =
                                                    *uhp_table.offset(i as isize);
                                                if !uhp_0.is_null() {
                                                    let mut j: ::core::ffi::c_int =
                                                        0 as ::core::ffi::c_int;
                                                    while j < num_head {
                                                        if !(*uhp_table.offset(j as isize))
                                                            .is_null()
                                                            && i != j
                                                            && (**uhp_table.offset(i as isize))
                                                                .uh_seq
                                                                == (**uhp_table.offset(j as isize))
                                                                    .uh_seq
                                                        {
                                                            corruption_error(
                                                                b"duplicate uh_seq\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                                file_name,
                                                            );
                                                            break '_error;
                                                        } else {
                                                            j += 1;
                                                        }
                                                    }
                                                    let seq: ::core::ffi::c_int =
                                                        (*uhp_0).uh_next.seq;
                                                    (*uhp_0).uh_next.ptr =
                                                        ::core::ptr::null_mut::<u_header_T>();
                                                    let mut j_0: ::core::ffi::c_int =
                                                        0 as ::core::ffi::c_int;
                                                    while j_0 < num_head {
                                                        if !(*uhp_table.offset(j_0 as isize))
                                                            .is_null()
                                                            && i != j_0
                                                            && (**uhp_table.offset(j_0 as isize))
                                                                .uh_seq
                                                                == seq
                                                        {
                                                            (*uhp_0).uh_next.ptr =
                                                                *uhp_table.offset(j_0 as isize);
                                                            break;
                                                        } else {
                                                            j_0 += 1;
                                                        }
                                                    }
                                                    let seq_0: ::core::ffi::c_int =
                                                        (*uhp_0).uh_prev.seq;
                                                    (*uhp_0).uh_prev.ptr =
                                                        ::core::ptr::null_mut::<u_header_T>();
                                                    let mut j_1: ::core::ffi::c_int =
                                                        0 as ::core::ffi::c_int;
                                                    while j_1 < num_head {
                                                        if !(*uhp_table.offset(j_1 as isize))
                                                            .is_null()
                                                            && i != j_1
                                                            && (**uhp_table.offset(j_1 as isize))
                                                                .uh_seq
                                                                == seq_0
                                                        {
                                                            (*uhp_0).uh_prev.ptr =
                                                                *uhp_table.offset(j_1 as isize);
                                                            break;
                                                        } else {
                                                            j_1 += 1;
                                                        }
                                                    }
                                                    let seq_1: ::core::ffi::c_int =
                                                        (*uhp_0).uh_alt_next.seq;
                                                    (*uhp_0).uh_alt_next.ptr =
                                                        ::core::ptr::null_mut::<u_header_T>();
                                                    let mut j_2: ::core::ffi::c_int =
                                                        0 as ::core::ffi::c_int;
                                                    while j_2 < num_head {
                                                        if !(*uhp_table.offset(j_2 as isize))
                                                            .is_null()
                                                            && i != j_2
                                                            && (**uhp_table.offset(j_2 as isize))
                                                                .uh_seq
                                                                == seq_1
                                                        {
                                                            (*uhp_0).uh_alt_next.ptr =
                                                                *uhp_table.offset(j_2 as isize);
                                                            break;
                                                        } else {
                                                            j_2 += 1;
                                                        }
                                                    }
                                                    let seq_2: ::core::ffi::c_int =
                                                        (*uhp_0).uh_alt_prev.seq;
                                                    (*uhp_0).uh_alt_prev.ptr =
                                                        ::core::ptr::null_mut::<u_header_T>();
                                                    let mut j_3: ::core::ffi::c_int =
                                                        0 as ::core::ffi::c_int;
                                                    while j_3 < num_head {
                                                        if !(*uhp_table.offset(j_3 as isize))
                                                            .is_null()
                                                            && i != j_3
                                                            && (**uhp_table.offset(j_3 as isize))
                                                                .uh_seq
                                                                == seq_2
                                                        {
                                                            (*uhp_0).uh_alt_prev.ptr =
                                                                *uhp_table.offset(j_3 as isize);
                                                            break;
                                                        } else {
                                                            j_3 += 1;
                                                        }
                                                    }
                                                    if old_header_seq > 0 as ::core::ffi::c_int
                                                        && (old_idx as ::core::ffi::c_int)
                                                            < 0 as ::core::ffi::c_int
                                                        && (*uhp_0).uh_seq == old_header_seq
                                                    {
                                                        '_c2rust_label: {
                                                            if i <= 32767 as ::core::ffi::c_int {
                                                            } else {
                                                                __assert_fail(
                                                                    b"i <= INT16_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                                                    b"src/nvim/undo.rs\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                    1613 as ::core::ffi::c_uint,
                                                                    b"void u_read_undo(char *, const uint8_t *, const char *)\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                );
                                                            }
                                                        };
                                                        old_idx = i as int16_t;
                                                    }
                                                    if new_header_seq > 0 as ::core::ffi::c_int
                                                        && (new_idx as ::core::ffi::c_int)
                                                            < 0 as ::core::ffi::c_int
                                                        && (*uhp_0).uh_seq == new_header_seq
                                                    {
                                                        '_c2rust_label_0: {
                                                            if i <= 32767 as ::core::ffi::c_int {
                                                            } else {
                                                                __assert_fail(
                                                                    b"i <= INT16_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                                                    b"src/nvim/undo.rs\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                    1618 as ::core::ffi::c_uint,
                                                                    b"void u_read_undo(char *, const uint8_t *, const char *)\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                );
                                                            }
                                                        };
                                                        new_idx = i as int16_t;
                                                    }
                                                    if cur_header_seq > 0 as ::core::ffi::c_int
                                                        && (cur_idx as ::core::ffi::c_int)
                                                            < 0 as ::core::ffi::c_int
                                                        && (*uhp_0).uh_seq == cur_header_seq
                                                    {
                                                        '_c2rust_label_1: {
                                                            if i <= 32767 as ::core::ffi::c_int {
                                                            } else {
                                                                __assert_fail(
                                                                    b"i <= INT16_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                                                    b"src/nvim/undo.rs\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                    1623 as ::core::ffi::c_uint,
                                                                    b"void u_read_undo(char *, const uint8_t *, const char *)\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                );
                                                            }
                                                        };
                                                        cur_idx = i as int16_t;
                                                    }
                                                }
                                                i += 1;
                                            }
                                            u_blockfree(curbuf.get());
                                            (*curbuf.get()).b_u_oldhead = if (old_idx
                                                as ::core::ffi::c_int)
                                                < 0 as ::core::ffi::c_int
                                            {
                                                ::core::ptr::null_mut::<u_header_T>()
                                            } else {
                                                *uhp_table.offset(old_idx as isize)
                                            };
                                            (*curbuf.get()).b_u_newhead = if (new_idx
                                                as ::core::ffi::c_int)
                                                < 0 as ::core::ffi::c_int
                                            {
                                                ::core::ptr::null_mut::<u_header_T>()
                                            } else {
                                                *uhp_table.offset(new_idx as isize)
                                            };
                                            (*curbuf.get()).b_u_curhead = if (cur_idx
                                                as ::core::ffi::c_int)
                                                < 0 as ::core::ffi::c_int
                                            {
                                                ::core::ptr::null_mut::<u_header_T>()
                                            } else {
                                                *uhp_table.offset(cur_idx as isize)
                                            };
                                            (*curbuf.get()).b_u_line_ptr = line_ptr;
                                            (*curbuf.get()).b_u_line_lnum = line_lnum;
                                            (*curbuf.get()).b_u_line_colnr = line_colnr;
                                            (*curbuf.get()).b_u_numhead = num_head;
                                            (*curbuf.get()).b_u_seq_last = seq_last;
                                            (*curbuf.get()).b_u_seq_cur = seq_cur;
                                            (*curbuf.get()).b_u_time_cur = seq_time;
                                            (*curbuf.get()).b_u_save_nr_last = last_save_nr;
                                            (*curbuf.get()).b_u_save_nr_cur = last_save_nr;
                                            (*curbuf.get()).b_u_synced = true_0 != 0;
                                            xfree(uhp_table as *mut ::core::ffi::c_void);
                                            if !name.is_null() {
                                                smsg(
                                                    0 as ::core::ffi::c_int,
                                                    gettext(
                                                        b"Finished reading undo file %s\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ),
                                                    file_name,
                                                );
                                            }
                                            break '_theend;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        xfree(line_ptr as *mut ::core::ffi::c_void);
        if !uhp_table.is_null() {
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < num_read_uhps {
                if !(*uhp_table.offset(i_0 as isize)).is_null() {
                    u_free_uhp(*uhp_table.offset(i_0 as isize));
                }
                i_0 += 1;
            }
            xfree(uhp_table as *mut ::core::ffi::c_void);
        }
    }
    if !fp.is_null() {
        fclose(fp);
    }
    if file_name != name {
        xfree(file_name as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn undo_write(
    mut bi: *mut bufinfo_T,
    mut ptr: *mut uint8_t,
    mut len: size_t,
) -> bool {
    return fwrite(
        ptr as *const ::core::ffi::c_void,
        len,
        1 as size_t,
        (*bi).bi_fp,
    ) == 1 as ::core::ffi::c_ulong;
}
unsafe extern "C" fn undo_write_bytes(
    mut bi: *mut bufinfo_T,
    mut nr: uintmax_t,
    mut len: size_t,
) -> bool {
    '_c2rust_label: {
        if len > 0 as size_t {
        } else {
            __assert_fail(
                b"len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/undo.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1707 as ::core::ffi::c_uint,
                b"_Bool undo_write_bytes(bufinfo_T *, uintmax_t, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut buf: [uint8_t; 8] = [0; 8];
    let mut i: size_t = len.wrapping_sub(1 as size_t);
    let mut bufi: size_t = 0 as size_t;
    while bufi < len {
        buf[bufi as usize] = (nr >> i.wrapping_mul(8 as size_t)) as uint8_t;
        i = i.wrapping_sub(1);
        bufi = bufi.wrapping_add(1);
    }
    return undo_write(bi, &raw mut buf as *mut uint8_t, len);
}
unsafe extern "C" fn put_header_ptr(mut bi: *mut bufinfo_T, mut uhp: *mut u_header_T) {
    '_c2rust_label: {
        if uhp.is_null() || (*uhp).uh_seq >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"uhp == NULL || uhp->uh_seq >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/undo.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1722 as ::core::ffi::c_uint,
                b"void put_header_ptr(bufinfo_T *, u_header_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    undo_write_bytes(
        bi,
        (if !uhp.is_null() {
            (*uhp).uh_seq
        } else {
            0 as ::core::ffi::c_int
        }) as uintmax_t,
        4 as size_t,
    );
}
unsafe extern "C" fn undo_read_4c(mut bi: *mut bufinfo_T) -> ::core::ffi::c_int {
    return get4c((*bi).bi_fp);
}
unsafe extern "C" fn undo_read_2c(mut bi: *mut bufinfo_T) -> ::core::ffi::c_int {
    return get2c((*bi).bi_fp);
}
unsafe extern "C" fn undo_read_byte(mut bi: *mut bufinfo_T) -> ::core::ffi::c_int {
    return getc((*bi).bi_fp);
}
unsafe extern "C" fn undo_read_time(mut bi: *mut bufinfo_T) -> time_t {
    return get8ctime((*bi).bi_fp);
}
unsafe extern "C" fn undo_read(
    mut bi: *mut bufinfo_T,
    mut buffer: *mut uint8_t,
    mut size: size_t,
) -> bool {
    let retval: bool = fread(
        buffer as *mut ::core::ffi::c_void,
        size,
        1 as size_t,
        (*bi).bi_fp,
    ) == 1 as ::core::ffi::c_ulong;
    if !retval {
        memset(
            buffer as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            size,
        );
    }
    return retval;
}
unsafe extern "C" fn undo_read_string(
    mut bi: *mut bufinfo_T,
    mut len: size_t,
) -> *mut ::core::ffi::c_char {
    let mut ptr: *mut ::core::ffi::c_char = xmallocz(len) as *mut ::core::ffi::c_char;
    if len > 0 as size_t && !undo_read(bi, ptr as *mut uint8_t, len) {
        xfree(ptr as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return ptr;
}
pub unsafe extern "C" fn u_undo(mut count: ::core::ffi::c_int) {
    if (*curbuf.get()).b_u_synced as ::core::ffi::c_int == false_0 {
        u_sync(true_0 != 0);
        count = 1 as ::core::ffi::c_int;
    }
    if vim_strchr(p_cpo.get(), CPO_UNDO).is_null() {
        undo_undoes.set(true_0 != 0);
    } else {
        undo_undoes.set(!undo_undoes.get());
    }
    u_doit(count, false_0 != 0, true_0 != 0);
}
pub unsafe extern "C" fn u_redo(mut count: ::core::ffi::c_int) {
    if vim_strchr(p_cpo.get(), CPO_UNDO).is_null() {
        undo_undoes.set(false_0 != 0);
    }
    u_doit(count, false_0 != 0, true_0 != 0);
}
pub unsafe extern "C" fn u_undo_and_forget(
    mut count: ::core::ffi::c_int,
    mut do_buf_event: bool,
) -> bool {
    if (*curbuf.get()).b_u_synced as ::core::ffi::c_int == false_0 {
        u_sync(true_0 != 0);
        count = 1 as ::core::ffi::c_int;
    }
    undo_undoes.set(true_0 != 0);
    u_doit(count, true_0 != 0, do_buf_event);
    if (*curbuf.get()).b_u_curhead.is_null() {
        return false_0 != 0;
    }
    let mut to_forget: *mut u_header_T = (*curbuf.get()).b_u_curhead;
    (*curbuf.get()).b_u_newhead = (*to_forget).uh_next.ptr;
    (*curbuf.get()).b_u_curhead = (*to_forget).uh_alt_next.ptr;
    if !(*curbuf.get()).b_u_curhead.is_null() {
        (*to_forget).uh_alt_next.ptr = ::core::ptr::null_mut::<u_header_T>();
        (*(*curbuf.get()).b_u_curhead).uh_alt_prev.ptr = (*to_forget).uh_alt_prev.ptr;
        (*curbuf.get()).b_u_seq_cur = if !(*(*curbuf.get()).b_u_curhead).uh_next.ptr.is_null() {
            (*(*(*curbuf.get()).b_u_curhead).uh_next.ptr).uh_seq
        } else {
            0 as ::core::ffi::c_int
        };
    } else if !(*curbuf.get()).b_u_newhead.is_null() {
        (*curbuf.get()).b_u_seq_cur = (*(*curbuf.get()).b_u_newhead).uh_seq;
    }
    if !(*to_forget).uh_alt_prev.ptr.is_null() {
        (*(*to_forget).uh_alt_prev.ptr).uh_alt_next.ptr = (*curbuf.get()).b_u_curhead;
    }
    if !(*curbuf.get()).b_u_newhead.is_null() {
        (*(*curbuf.get()).b_u_newhead).uh_prev.ptr = (*curbuf.get()).b_u_curhead;
    }
    if (*curbuf.get()).b_u_seq_last == (*to_forget).uh_seq {
        (*curbuf.get()).b_u_seq_last -= 1;
    }
    u_freebranch(
        curbuf.get(),
        to_forget,
        ::core::ptr::null_mut::<*mut u_header_T>(),
    );
    return true_0 != 0;
}
unsafe extern "C" fn u_doit(
    mut startcount: ::core::ffi::c_int,
    mut quiet: bool,
    mut do_buf_event: bool,
) {
    if !undo_allowed(curbuf.get()) {
        return;
    }
    u_newcount.set(0 as ::core::ffi::c_int);
    u_oldcount.set(0 as ::core::ffi::c_int);
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        u_oldcount.set(-1 as ::core::ffi::c_int);
    }
    msg_ext_set_kind(b"undo\0".as_ptr() as *const ::core::ffi::c_char);
    let mut count: ::core::ffi::c_int = startcount;
    loop {
        let c2rust_fresh4 = count;
        count = count - 1;
        if c2rust_fresh4 == 0 {
            break;
        }
        change_warning(curbuf.get(), 0 as ::core::ffi::c_int);
        if undo_undoes.get() {
            if (*curbuf.get()).b_u_curhead.is_null() {
                (*curbuf.get()).b_u_curhead = (*curbuf.get()).b_u_newhead;
            } else if get_undolevel(curbuf.get()) > 0 as OptInt {
                (*curbuf.get()).b_u_curhead = (*(*curbuf.get()).b_u_curhead).uh_next.ptr;
            }
            if (*curbuf.get()).b_u_numhead == 0 as ::core::ffi::c_int
                || (*curbuf.get()).b_u_curhead.is_null()
            {
                (*curbuf.get()).b_u_curhead = (*curbuf.get()).b_u_oldhead;
                beep_flush();
                if count == startcount - 1 as ::core::ffi::c_int {
                    msg(
                        gettext(
                            b"Already at oldest change\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                        0 as ::core::ffi::c_int,
                    );
                    return;
                }
                break;
            } else {
                u_undoredo(true_0 != 0, do_buf_event);
            }
        } else if (*curbuf.get()).b_u_curhead.is_null()
            || get_undolevel(curbuf.get()) <= 0 as OptInt
        {
            beep_flush();
            if count == startcount - 1 as ::core::ffi::c_int {
                msg(
                    gettext(b"Already at newest change\0".as_ptr() as *const ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                );
                return;
            }
            break;
        } else {
            u_undoredo(false_0 != 0, do_buf_event);
            if (*(*curbuf.get()).b_u_curhead).uh_prev.ptr.is_null() {
                (*curbuf.get()).b_u_newhead = (*curbuf.get()).b_u_curhead;
            }
            (*curbuf.get()).b_u_curhead = (*(*curbuf.get()).b_u_curhead).uh_prev.ptr;
        }
    }
    u_undo_end(undo_undoes.get(), false_0 != 0, quiet);
}
pub unsafe extern "C" fn undo_time(
    mut step: ::core::ffi::c_int,
    mut sec: bool,
    mut file: bool,
    mut absolute: bool,
) {
    if text_locked() {
        text_locked_msg();
        return;
    }
    if (*curbuf.get()).b_u_synced as ::core::ffi::c_int == false_0 {
        u_sync(true_0 != 0);
    }
    u_newcount.set(0 as ::core::ffi::c_int);
    u_oldcount.set(0 as ::core::ffi::c_int);
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        u_oldcount.set(-1 as ::core::ffi::c_int);
    }
    let mut target: ::core::ffi::c_int = 0;
    let mut closest: ::core::ffi::c_int = 0;
    let mut uhp: *mut u_header_T = ::core::ptr::null_mut::<u_header_T>();
    let mut dosec: bool = sec;
    let mut dofile: bool = file;
    let mut above: bool = false_0 != 0;
    let mut did_undo: bool = true_0 != 0;
    if absolute {
        target = step;
        closest = -1 as ::core::ffi::c_int;
    } else {
        if dosec {
            target = (*curbuf.get()).b_u_time_cur as ::core::ffi::c_int + step;
        } else if dofile {
            if step < 0 as ::core::ffi::c_int {
                uhp = (*curbuf.get()).b_u_curhead;
                if !uhp.is_null() {
                    uhp = (*uhp).uh_next.ptr;
                } else {
                    uhp = (*curbuf.get()).b_u_newhead;
                }
                if !uhp.is_null() && (*uhp).uh_save_nr != 0 as ::core::ffi::c_int {
                    target = (*curbuf.get()).b_u_save_nr_cur + step;
                } else {
                    target = (*curbuf.get()).b_u_save_nr_cur + step + 1 as ::core::ffi::c_int;
                }
                if target <= 0 as ::core::ffi::c_int {
                    dofile = false_0 != 0;
                }
            } else {
                target = (*curbuf.get()).b_u_save_nr_cur + step;
                if target > (*curbuf.get()).b_u_save_nr_last {
                    target = (*curbuf.get()).b_u_seq_last + 1 as ::core::ffi::c_int;
                    dofile = false_0 != 0;
                }
            }
        } else {
            target = (*curbuf.get()).b_u_seq_cur + step;
        }
        if step < 0 as ::core::ffi::c_int {
            target = if target > 0 as ::core::ffi::c_int {
                target
            } else {
                0 as ::core::ffi::c_int
            };
            closest = -1 as ::core::ffi::c_int;
        } else {
            if dosec {
                closest = os_time().wrapping_add(1 as Timestamp) as ::core::ffi::c_int;
            } else if dofile {
                closest = (*curbuf.get()).b_u_save_nr_last + 2 as ::core::ffi::c_int;
            } else {
                closest = (*curbuf.get()).b_u_seq_last + 2 as ::core::ffi::c_int;
            }
            if target >= closest {
                target = closest - 1 as ::core::ffi::c_int;
            }
        }
    }
    let mut closest_start: ::core::ffi::c_int = closest;
    let mut closest_seq: ::core::ffi::c_int = (*curbuf.get()).b_u_seq_cur;
    let mut mark: ::core::ffi::c_int = 0;
    let mut nomark: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if target == 0 as ::core::ffi::c_int {
        mark = lastmark.get();
    } else {
        let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while round <= 2 as ::core::ffi::c_int {
            (*lastmark.ptr()) += 1;
            mark = lastmark.get();
            (*lastmark.ptr()) += 1;
            nomark = lastmark.get();
            if (*curbuf.get()).b_u_curhead.is_null() {
                uhp = (*curbuf.get()).b_u_newhead;
            } else {
                uhp = (*curbuf.get()).b_u_curhead;
            }
            while !uhp.is_null() {
                (*uhp).uh_walk = mark;
                let mut val: ::core::ffi::c_int = if dosec as ::core::ffi::c_int != 0 {
                    (*uhp).uh_time as ::core::ffi::c_int
                } else if dofile as ::core::ffi::c_int != 0 {
                    (*uhp).uh_save_nr
                } else {
                    (*uhp).uh_seq
                };
                if round == 1 as ::core::ffi::c_int
                    && !(dofile as ::core::ffi::c_int != 0 && val == 0 as ::core::ffi::c_int)
                {
                    if (if step < 0 as ::core::ffi::c_int {
                        ((*uhp).uh_seq <= (*curbuf.get()).b_u_seq_cur) as ::core::ffi::c_int
                    } else {
                        ((*uhp).uh_seq > (*curbuf.get()).b_u_seq_cur) as ::core::ffi::c_int
                    }) != 0
                        && (if dosec as ::core::ffi::c_int != 0 && val == closest {
                            if step < 0 as ::core::ffi::c_int {
                                ((*uhp).uh_seq < closest_seq) as ::core::ffi::c_int
                            } else {
                                ((*uhp).uh_seq > closest_seq) as ::core::ffi::c_int
                            }
                        } else {
                            (closest == closest_start
                                || (if val > target {
                                    if closest > target {
                                        (val - target <= closest - target) as ::core::ffi::c_int
                                    } else {
                                        (val - target <= target - closest) as ::core::ffi::c_int
                                    }
                                } else {
                                    if closest > target {
                                        (target - val <= closest - target) as ::core::ffi::c_int
                                    } else {
                                        (target - val <= target - closest) as ::core::ffi::c_int
                                    }
                                }) != 0) as ::core::ffi::c_int
                        }) != 0
                    {
                        closest = val;
                        closest_seq = (*uhp).uh_seq;
                    }
                }
                if target == val && !dosec {
                    target = (*uhp).uh_seq;
                    break;
                } else if !(*uhp).uh_prev.ptr.is_null()
                    && (*(*uhp).uh_prev.ptr).uh_walk != nomark
                    && (*(*uhp).uh_prev.ptr).uh_walk != mark
                {
                    uhp = (*uhp).uh_prev.ptr;
                } else if !(*uhp).uh_alt_next.ptr.is_null()
                    && (*(*uhp).uh_alt_next.ptr).uh_walk != nomark
                    && (*(*uhp).uh_alt_next.ptr).uh_walk != mark
                {
                    uhp = (*uhp).uh_alt_next.ptr;
                } else if !(*uhp).uh_next.ptr.is_null()
                    && (*uhp).uh_alt_prev.ptr.is_null()
                    && (*(*uhp).uh_next.ptr).uh_walk != nomark
                    && (*(*uhp).uh_next.ptr).uh_walk != mark
                {
                    if uhp == (*curbuf.get()).b_u_curhead {
                        (*uhp).uh_walk = nomark;
                    }
                    uhp = (*uhp).uh_next.ptr;
                } else {
                    (*uhp).uh_walk = nomark;
                    if !(*uhp).uh_alt_prev.ptr.is_null() {
                        uhp = (*uhp).uh_alt_prev.ptr;
                    } else {
                        uhp = (*uhp).uh_next.ptr;
                    }
                }
            }
            if !uhp.is_null() {
                break;
            }
            if absolute {
                semsg(
                    gettext(
                        b"E830: Undo number %ld not found\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    step as int64_t,
                );
                return;
            }
            if closest == closest_start {
                if step < 0 as ::core::ffi::c_int {
                    msg(
                        gettext(
                            b"Already at oldest change\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                        0 as ::core::ffi::c_int,
                    );
                } else {
                    msg(
                        gettext(
                            b"Already at newest change\0".as_ptr() as *const ::core::ffi::c_char
                        ),
                        0 as ::core::ffi::c_int,
                    );
                }
                return;
            }
            target = closest_seq;
            dosec = false_0 != 0;
            dofile = false_0 != 0;
            if step < 0 as ::core::ffi::c_int {
                above = true_0 != 0;
            }
            round += 1;
        }
    }
    if !uhp.is_null() || target == 0 as ::core::ffi::c_int {
        while !got_int.get() {
            change_warning(curbuf.get(), 0 as ::core::ffi::c_int);
            uhp = (*curbuf.get()).b_u_curhead;
            if uhp.is_null() {
                uhp = (*curbuf.get()).b_u_newhead;
            } else {
                uhp = (*uhp).uh_next.ptr;
            }
            if uhp.is_null()
                || target > 0 as ::core::ffi::c_int && (*uhp).uh_walk != mark
                || (*uhp).uh_seq == target && !above
            {
                break;
            }
            (*curbuf.get()).b_u_curhead = uhp;
            u_undoredo(true_0 != 0, true_0 != 0);
            if target > 0 as ::core::ffi::c_int {
                (*uhp).uh_walk = nomark;
            }
        }
        if target > 0 as ::core::ffi::c_int {
            while !got_int.get() {
                change_warning(curbuf.get(), 0 as ::core::ffi::c_int);
                uhp = (*curbuf.get()).b_u_curhead;
                if uhp.is_null() {
                    break;
                }
                while !(*uhp).uh_alt_prev.ptr.is_null() && (*(*uhp).uh_alt_prev.ptr).uh_walk == mark
                {
                    uhp = (*uhp).uh_alt_prev.ptr;
                }
                let mut last: *mut u_header_T = uhp;
                while !(*last).uh_alt_next.ptr.is_null()
                    && (*(*last).uh_alt_next.ptr).uh_walk == mark
                {
                    last = (*last).uh_alt_next.ptr;
                }
                if last != uhp {
                    while !(*uhp).uh_alt_prev.ptr.is_null() {
                        uhp = (*uhp).uh_alt_prev.ptr;
                    }
                    if !(*last).uh_alt_next.ptr.is_null() {
                        (*(*last).uh_alt_next.ptr).uh_alt_prev.ptr = (*last).uh_alt_prev.ptr;
                    }
                    (*(*last).uh_alt_prev.ptr).uh_alt_next.ptr = (*last).uh_alt_next.ptr;
                    (*last).uh_alt_prev.ptr = ::core::ptr::null_mut::<u_header_T>();
                    (*last).uh_alt_next.ptr = uhp;
                    (*uhp).uh_alt_prev.ptr = last;
                    if (*curbuf.get()).b_u_oldhead == uhp {
                        (*curbuf.get()).b_u_oldhead = last;
                    }
                    uhp = last;
                    if !(*uhp).uh_next.ptr.is_null() {
                        (*(*uhp).uh_next.ptr).uh_prev.ptr = uhp;
                    }
                }
                (*curbuf.get()).b_u_curhead = uhp;
                if (*uhp).uh_walk != mark {
                    break;
                }
                if (*uhp).uh_seq == target && above as ::core::ffi::c_int != 0 {
                    (*curbuf.get()).b_u_seq_cur = target - 1 as ::core::ffi::c_int;
                    break;
                } else {
                    u_undoredo(false_0 != 0, true_0 != 0);
                    if (*uhp).uh_prev.ptr.is_null() {
                        (*curbuf.get()).b_u_newhead = uhp;
                    }
                    (*curbuf.get()).b_u_curhead = (*uhp).uh_prev.ptr;
                    did_undo = false_0 != 0;
                    if (*uhp).uh_seq == target {
                        break;
                    }
                    uhp = (*uhp).uh_prev.ptr;
                    if !(uhp.is_null() || (*uhp).uh_walk != mark) {
                        continue;
                    }
                    internal_error(b"undo_time()\0".as_ptr() as *const ::core::ffi::c_char);
                    break;
                }
            }
        }
    }
    u_undo_end(did_undo, absolute, false_0 != 0);
}
unsafe extern "C" fn u_undoredo(mut undo: bool, mut do_buf_event: bool) {
    let mut newarray: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut newlnum: linenr_T = MAXLNUM as ::core::ffi::c_int as linenr_T;
    let mut new_curpos: pos_T = (*curwin.get()).w_cursor;
    let mut nuep: *mut u_entry_T = ::core::ptr::null_mut::<u_entry_T>();
    let mut newlist: *mut u_entry_T = ::core::ptr::null_mut::<u_entry_T>();
    let mut namedm: [fmark_T; 26] = [fmark_T {
        mark: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        fnum: 0,
        timestamp: 0,
        view: fmarkv_T {
            topline_offset: 0,
            skipcol: 0,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    }; 26];
    let mut curhead: *mut u_header_T = (*curbuf.get()).b_u_curhead;
    block_autocmds();
    let mut old_flags: ::core::ffi::c_int = (*curhead).uh_flags;
    let mut new_flags: ::core::ffi::c_int = (if (*curbuf.get()).b_changed != 0 {
        UH_CHANGED as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) | (if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        UH_EMPTYBUF as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) | old_flags & UH_RELOAD as ::core::ffi::c_int;
    setpcmark();
    zero_fmark_additional_data(&raw mut (*curbuf.get()).b_namedm as *mut fmark_T);
    memmove(
        &raw mut namedm as *mut fmark_T as *mut ::core::ffi::c_void,
        &raw mut (*curbuf.get()).b_namedm as *mut fmark_T as *const ::core::ffi::c_void,
        ::core::mem::size_of::<fmark_T>().wrapping_mul(NMARKS as size_t),
    );
    let mut visualinfo: visualinfo_T = (*curbuf.get()).b_visual;
    (*curbuf.get()).b_op_start.lnum = (*curbuf.get()).b_ml.ml_line_count;
    (*curbuf.get()).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
    (*curbuf.get()).b_op_end.lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
    let mut uep: *mut u_entry_T = (*curhead).uh_entry;
    while !uep.is_null() {
        let mut top: linenr_T = (*uep).ue_top;
        let mut bot: linenr_T = (*uep).ue_bot;
        if bot == 0 as linenr_T {
            bot = (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T;
        }
        if top > (*curbuf.get()).b_ml.ml_line_count
            || top >= bot
            || bot > (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T
        {
            unblock_autocmds();
            iemsg(gettext(
                b"E438: u_undo: line numbers wrong\0".as_ptr() as *const ::core::ffi::c_char
            ));
            changed(curbuf.get());
            return;
        }
        let mut oldsize: linenr_T = bot - top - 1 as linenr_T;
        let mut newsize: linenr_T = (*uep).ue_size;
        let mut lnum: linenr_T = (*curhead).uh_cursor.lnum;
        if lnum >= top && lnum <= top + newsize + 1 as linenr_T {
            new_curpos = (*curhead).uh_cursor;
            newlnum = -1 as ::core::ffi::c_int as linenr_T;
        } else if top < newlnum {
            let mut i: ::core::ffi::c_int = 0;
            i = 0 as ::core::ffi::c_int;
            while (i as linenr_T) < newsize && (i as linenr_T) < oldsize {
                if strcmp(
                    *(*uep).ue_array.offset(i as isize),
                    ml_get(top + 1 as linenr_T + i as linenr_T),
                ) != 0 as ::core::ffi::c_int
                {
                    break;
                }
                i += 1;
            }
            if i as linenr_T == newsize
                && newlnum == MAXLNUM as ::core::ffi::c_int as linenr_T
                && (*uep).ue_next.is_null()
            {
                newlnum = top;
                new_curpos.lnum = newlnum + 1 as linenr_T;
            } else if (i as linenr_T) < newsize {
                newlnum = top + i as linenr_T;
                new_curpos.lnum = newlnum + 1 as linenr_T;
            }
        }
        let mut empty_buffer: bool = false_0 != 0;
        if oldsize > 0 as linenr_T {
            newarray = xmalloc(
                ::core::mem::size_of::<*mut ::core::ffi::c_char>().wrapping_mul(oldsize as size_t),
            ) as *mut *mut ::core::ffi::c_char;
            let mut i_0: ::core::ffi::c_int = 0;
            let mut lnum_0: linenr_T = 0;
            lnum_0 = bot - 1 as linenr_T;
            i_0 = oldsize as ::core::ffi::c_int;
            loop {
                i_0 -= 1;
                if i_0 < 0 as ::core::ffi::c_int {
                    break;
                }
                *newarray.offset(i_0 as isize) = u_save_line(lnum_0);
                if (*curbuf.get()).b_ml.ml_line_count == 1 as linenr_T {
                    empty_buffer = true_0 != 0;
                }
                ml_delete(lnum_0);
                lnum_0 -= 1;
            }
        } else {
            newarray = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        }
        check_cursor_lnum(curwin.get());
        if newsize != 0 {
            let mut i_1: ::core::ffi::c_int = 0;
            let mut lnum_1: linenr_T = 0;
            lnum_1 = top;
            i_1 = 0 as ::core::ffi::c_int;
            while (i_1 as linenr_T) < newsize {
                if empty_buffer as ::core::ffi::c_int != 0 && lnum_1 == 0 as linenr_T {
                    ml_replace(
                        1 as linenr_T,
                        *(*uep).ue_array.offset(i_1 as isize),
                        true_0 != 0,
                    );
                } else {
                    ml_append_flags(
                        lnum_1,
                        *(*uep).ue_array.offset(i_1 as isize),
                        0 as colnr_T,
                        0 as ::core::ffi::c_int,
                    );
                }
                xfree(*(*uep).ue_array.offset(i_1 as isize) as *mut ::core::ffi::c_void);
                i_1 += 1;
                lnum_1 += 1;
            }
            xfree((*uep).ue_array as *mut ::core::ffi::c_void);
        }
        if oldsize != newsize {
            mark_adjust(
                top + 1 as linenr_T,
                top + oldsize,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                newsize - oldsize,
                kExtmarkNOOP,
            );
            if (*curbuf.get()).b_op_start.lnum > top + oldsize {
                (*curbuf.get()).b_op_start.lnum += newsize - oldsize;
            }
            if (*curbuf.get()).b_op_end.lnum > top + oldsize {
                (*curbuf.get()).b_op_end.lnum += newsize - oldsize;
            }
        }
        if oldsize > 0 as linenr_T || newsize > 0 as linenr_T {
            changed_lines(
                curbuf.get(),
                top + 1 as linenr_T,
                0 as colnr_T,
                bot,
                newsize - oldsize,
                do_buf_event,
            );
            if spell_check_window(curwin.get()) as ::core::ffi::c_int != 0
                && bot <= (*curbuf.get()).b_ml.ml_line_count
            {
                redrawWinline(curwin.get(), bot);
            }
        }
        (*curbuf.get()).b_op_start.lnum = if (*curbuf.get()).b_op_start.lnum < top + 1 as linenr_T {
            (*curbuf.get()).b_op_start.lnum
        } else {
            top + 1 as linenr_T
        };
        if newsize == 0 as linenr_T && top + 1 as linenr_T > (*curbuf.get()).b_op_end.lnum {
            (*curbuf.get()).b_op_end.lnum = top + 1 as linenr_T;
        } else if top + newsize > (*curbuf.get()).b_op_end.lnum {
            (*curbuf.get()).b_op_end.lnum = top + newsize;
        }
        (*u_newcount.ptr()) += newsize as ::core::ffi::c_int;
        (*u_oldcount.ptr()) += oldsize as ::core::ffi::c_int;
        (*uep).ue_size = oldsize;
        (*uep).ue_array = newarray;
        (*uep).ue_bot = top + newsize + 1 as linenr_T;
        nuep = (*uep).ue_next;
        (*uep).ue_next = newlist;
        newlist = uep;
        uep = nuep;
    }
    (*curbuf.get()).b_op_start.lnum =
        if (*curbuf.get()).b_op_start.lnum < (*curbuf.get()).b_ml.ml_line_count {
            (*curbuf.get()).b_op_start.lnum
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
    (*curbuf.get()).b_op_end.lnum =
        if (*curbuf.get()).b_op_end.lnum < (*curbuf.get()).b_ml.ml_line_count {
            (*curbuf.get()).b_op_end.lnum
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
    if undo {
        let mut i_2: ::core::ffi::c_int =
            (*curhead).uh_extmark.size as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        while i_2 > -1 as ::core::ffi::c_int {
            extmark_apply_undo(*(*curhead).uh_extmark.items.offset(i_2 as isize), undo);
            i_2 -= 1;
        }
    } else {
        let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_3 < (*curhead).uh_extmark.size as ::core::ffi::c_int {
            extmark_apply_undo(*(*curhead).uh_extmark.items.offset(i_3 as isize), undo);
            i_3 += 1;
        }
    }
    if (*curhead).uh_flags & UH_RELOAD as ::core::ffi::c_int != 0 {
        buf_updates_unload(curbuf.get(), true_0 != 0);
    }
    (*curwin.get()).w_cursor = new_curpos;
    check_cursor_lnum(curwin.get());
    (*curhead).uh_entry = newlist;
    (*curhead).uh_flags = new_flags;
    if old_flags & UH_EMPTYBUF as ::core::ffi::c_int != 0
        && buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0
    {
        (*curbuf.get()).b_ml.ml_flags |= ML_EMPTY;
    }
    if old_flags & UH_CHANGED as ::core::ffi::c_int != 0 {
        changed(curbuf.get());
    } else {
        unchanged(curbuf.get(), false_0 != 0, true_0 != 0);
    }
    if do_buf_event {
        buf_updates_changedtick(curbuf.get());
    }
    let mut i_4: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_4 < NMARKS {
        if (*curhead).uh_namedm[i_4 as usize].mark.lnum != 0 as linenr_T {
            free_fmark((*curbuf.get()).b_namedm[i_4 as usize]);
            (*curbuf.get()).b_namedm[i_4 as usize] = (*curhead).uh_namedm[i_4 as usize];
        }
        if namedm[i_4 as usize].mark.lnum != 0 as linenr_T {
            (*curhead).uh_namedm[i_4 as usize] = namedm[i_4 as usize];
        } else {
            (*curhead).uh_namedm[i_4 as usize].mark.lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
        i_4 += 1;
    }
    if (*curhead).uh_visual.vi_start.lnum != 0 as linenr_T {
        (*curbuf.get()).b_visual = (*curhead).uh_visual;
        (*curhead).uh_visual = visualinfo;
    }
    if (*curhead).uh_cursor.lnum + 1 as linenr_T == (*curwin.get()).w_cursor.lnum
        && (*curwin.get()).w_cursor.lnum > 1 as linenr_T
    {
        (*curwin.get()).w_cursor.lnum -= 1;
    }
    if (*curwin.get()).w_cursor.lnum <= (*curbuf.get()).b_ml.ml_line_count {
        if (*curhead).uh_cursor.lnum == (*curwin.get()).w_cursor.lnum {
            (*curwin.get()).w_cursor.col = (*curhead).uh_cursor.col;
            if virtual_active(curwin.get()) as ::core::ffi::c_int != 0
                && (*curhead).uh_cursor_vcol >= 0 as ::core::ffi::c_int
            {
                coladvance(curwin.get(), (*curhead).uh_cursor_vcol);
            } else {
                (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
        } else {
            beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        }
    } else {
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    }
    check_cursor(curwin.get());
    (*curbuf.get()).b_u_seq_cur = (*curhead).uh_seq;
    if undo {
        (*curbuf.get()).b_u_seq_cur = if !(*curhead).uh_next.ptr.is_null() {
            (*(*curhead).uh_next.ptr).uh_seq
        } else {
            0 as ::core::ffi::c_int
        };
    }
    if (*curhead).uh_save_nr != 0 as ::core::ffi::c_int {
        if undo {
            (*curbuf.get()).b_u_save_nr_cur = (*curhead).uh_save_nr - 1 as ::core::ffi::c_int;
        } else {
            (*curbuf.get()).b_u_save_nr_cur = (*curhead).uh_save_nr;
        }
    }
    (*curbuf.get()).b_u_time_cur = (*curhead).uh_time;
    unblock_autocmds();
}
unsafe extern "C" fn u_undo_end(mut did_undo: bool, mut absolute: bool, mut quiet: bool) {
    if fdo_flags.get() & kOptFdoFlagUndo as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        && KeyTyped.get() as ::core::ffi::c_int != 0
    {
        foldOpenCursor();
    }
    if quiet as ::core::ffi::c_int != 0 || global_busy.get() != 0 || !messaging() {
        return;
    }
    if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
        (*u_newcount.ptr()) -= 1;
    }
    (*u_oldcount.ptr()) -= u_newcount.get();
    let mut msgstr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if u_oldcount.get() == -1 as ::core::ffi::c_int {
        msgstr = b"more line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else if u_oldcount.get() < 0 as ::core::ffi::c_int {
        msgstr = b"more lines\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else if u_oldcount.get() == 1 as ::core::ffi::c_int {
        msgstr = b"line less\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else if u_oldcount.get() > 1 as ::core::ffi::c_int {
        msgstr =
            b"fewer lines\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else {
        u_oldcount.set(u_newcount.get());
        if u_newcount.get() == 1 as ::core::ffi::c_int {
            msgstr = b"change\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            msgstr =
                b"changes\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
    }
    let mut uhp: *mut u_header_T = ::core::ptr::null_mut::<u_header_T>();
    if !(*curbuf.get()).b_u_curhead.is_null() {
        if absolute as ::core::ffi::c_int != 0
            && !(*(*curbuf.get()).b_u_curhead).uh_next.ptr.is_null()
        {
            uhp = (*(*curbuf.get()).b_u_curhead).uh_next.ptr;
            did_undo = false_0 != 0;
        } else if did_undo {
            uhp = (*curbuf.get()).b_u_curhead;
        } else {
            uhp = (*(*curbuf.get()).b_u_curhead).uh_next.ptr;
        }
    } else {
        uhp = (*curbuf.get()).b_u_newhead;
    }
    let mut msgbuf: [::core::ffi::c_char; 80] = [0; 80];
    if uhp.is_null() {
        *(&raw mut msgbuf as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
    } else {
        undo_fmt_time(
            &raw mut msgbuf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 80]>(),
            (*uhp).uh_time,
        );
    }
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == curbuf.get() && (*wp).w_onebuf_opt.wo_cole > 0 as OptInt {
            redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
        }
        wp = (*wp).w_next;
    }
    if VIsual_active.get() {
        check_pos(curbuf.get(), VIsual.ptr());
    }
    smsg_keep(
        0 as ::core::ffi::c_int,
        gettext(b"%ld %s; %s #%ld  %s\0".as_ptr() as *const ::core::ffi::c_char),
        if u_oldcount.get() < 0 as ::core::ffi::c_int {
            -u_oldcount.get() as int64_t
        } else {
            u_oldcount.get() as int64_t
        },
        gettext(msgstr),
        if did_undo as ::core::ffi::c_int != 0 {
            gettext(b"before\0".as_ptr() as *const ::core::ffi::c_char)
        } else {
            gettext(b"after\0".as_ptr() as *const ::core::ffi::c_char)
        },
        if uhp.is_null() {
            0 as int64_t
        } else {
            (*uhp).uh_seq as int64_t
        },
        &raw mut msgbuf as *mut ::core::ffi::c_char,
    );
}
pub unsafe extern "C" fn undo_fmt_time(
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: size_t,
    mut tt: time_t,
) {
    if time(::core::ptr::null_mut::<time_t>()) - tt >= 100 as time_t {
        let mut curtime: tm = tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        };
        os_localtime_r(&raw mut tt, &raw mut curtime);
        let mut n: size_t = 0;
        if time(::core::ptr::null_mut::<time_t>()) - tt
            < (60 as ::core::ffi::c_int * 60 as ::core::ffi::c_int * 12 as ::core::ffi::c_int)
                as time_t
        {
            n = strftime(
                buf,
                buflen,
                b"%H:%M:%S\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut curtime,
            );
        } else {
            n = strftime(
                buf,
                buflen,
                b"%Y/%m/%d %H:%M:%S\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut curtime,
            );
        }
        if n == 0 as size_t {
            *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
    } else {
        let mut seconds: int64_t =
            time(::core::ptr::null_mut::<time_t>()) as int64_t - tt as int64_t;
        vim_snprintf(
            buf,
            buflen,
            ngettext(
                b"%ld second ago\0".as_ptr() as *const ::core::ffi::c_char,
                b"%ld seconds ago\0".as_ptr() as *const ::core::ffi::c_char,
                seconds as uint32_t as ::core::ffi::c_ulong,
            ),
            seconds,
        );
    };
}
pub unsafe extern "C" fn u_sync(mut force: bool) {
    if (*curbuf.get()).b_u_synced as ::core::ffi::c_int != 0
        || !force && no_u_sync.get() > 0 as ::core::ffi::c_int
    {
        return;
    }
    if get_undolevel(curbuf.get()) < 0 as OptInt {
        (*curbuf.get()).b_u_synced = true_0 != 0;
    } else {
        u_getbot(curbuf.get());
        (*curbuf.get()).b_u_curhead = ::core::ptr::null_mut::<u_header_T>();
    };
}
pub unsafe extern "C" fn ex_undolist(mut _eap: *mut exarg_T) {
    let mut changes: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    (*lastmark.ptr()) += 1;
    let mut mark: ::core::ffi::c_int = lastmark.get();
    (*lastmark.ptr()) += 1;
    let mut nomark: ::core::ffi::c_int = lastmark.get();
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    let mut uhp: *mut u_header_T = (*curbuf.get()).b_u_oldhead;
    while !uhp.is_null() {
        if (*uhp).uh_prev.ptr.is_null() && (*uhp).uh_walk != nomark && (*uhp).uh_walk != mark {
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%6d %7d  \0".as_ptr() as *const ::core::ffi::c_char,
                (*uhp).uh_seq,
                changes,
            );
            undo_fmt_time(
                (IObuff.ptr() as *mut ::core::ffi::c_char)
                    .offset(strlen(IObuff.ptr() as *mut ::core::ffi::c_char) as isize),
                (IOSIZE as size_t).wrapping_sub(strlen(IObuff.ptr() as *mut ::core::ffi::c_char)),
                (*uhp).uh_time,
            );
            if (*uhp).uh_save_nr > 0 as ::core::ffi::c_int {
                while strlen(IObuff.ptr() as *mut ::core::ffi::c_char) < 33 as size_t {
                    xstrlcat(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        IOSIZE as size_t,
                    );
                }
                vim_snprintf_add(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"  %3d\0".as_ptr() as *const ::core::ffi::c_char,
                    (*uhp).uh_save_nr,
                );
            }
            ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
            *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) =
                xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
            ga.ga_len += 1;
        }
        (*uhp).uh_walk = mark;
        if !(*uhp).uh_prev.ptr.is_null()
            && (*(*uhp).uh_prev.ptr).uh_walk != nomark
            && (*(*uhp).uh_prev.ptr).uh_walk != mark
        {
            uhp = (*uhp).uh_prev.ptr;
            changes += 1;
        } else if !(*uhp).uh_alt_next.ptr.is_null()
            && (*(*uhp).uh_alt_next.ptr).uh_walk != nomark
            && (*(*uhp).uh_alt_next.ptr).uh_walk != mark
        {
            uhp = (*uhp).uh_alt_next.ptr;
        } else if !(*uhp).uh_next.ptr.is_null()
            && (*uhp).uh_alt_prev.ptr.is_null()
            && (*(*uhp).uh_next.ptr).uh_walk != nomark
            && (*(*uhp).uh_next.ptr).uh_walk != mark
        {
            uhp = (*uhp).uh_next.ptr;
            changes -= 1;
        } else {
            (*uhp).uh_walk = nomark;
            if !(*uhp).uh_alt_prev.ptr.is_null() {
                uhp = (*uhp).uh_alt_prev.ptr;
            } else {
                uhp = (*uhp).uh_next.ptr;
                changes -= 1;
            }
        }
    }
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    if ga.ga_len <= 0 as ::core::ffi::c_int {
        msg(
            gettext(b"Nothing to undo\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
    } else {
        sort_strings(ga.ga_data as *mut *mut ::core::ffi::c_char, ga.ga_len);
        msg_start();
        msg_puts_hl(
            gettext(b"number changes  when               saved\0".as_ptr()
                as *const ::core::ffi::c_char),
            HLF_T as ::core::ffi::c_int,
            false_0 != 0,
        );
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < ga.ga_len && !got_int.get() {
            msg_putchar('\n' as ::core::ffi::c_int);
            if got_int.get() {
                break;
            }
            msg_puts(*(ga.ga_data as *mut *const ::core::ffi::c_char).offset(i as isize));
            i += 1;
        }
        msg_end();
        ga_clear_strings(&raw mut ga);
    };
}
pub unsafe extern "C" fn ex_undojoin(mut _eap: *mut exarg_T) {
    if (*curbuf.get()).b_u_newhead.is_null() {
        return;
    }
    if !(*curbuf.get()).b_u_curhead.is_null() {
        emsg(gettext(
            b"E790: undojoin is not allowed after undo\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return;
    }
    if !(*curbuf.get()).b_u_synced {
        return;
    }
    if get_undolevel(curbuf.get()) < 0 as OptInt {
        return;
    }
    (*curbuf.get()).b_u_synced = false_0 != 0;
}
pub unsafe extern "C" fn u_unchanged(mut buf: *mut buf_T) {
    u_unch_branch((*buf).b_u_oldhead);
    (*buf).b_did_warn = false_0 != 0;
}
pub unsafe extern "C" fn u_find_first_changed() {
    let mut uhp: *mut u_header_T = (*curbuf.get()).b_u_newhead;
    if !(*curbuf.get()).b_u_curhead.is_null() || uhp.is_null() {
        return;
    }
    let mut uep: *mut u_entry_T = (*uhp).uh_entry;
    if (*uep).ue_top != 0 as linenr_T || (*uep).ue_bot != 0 as linenr_T {
        return;
    }
    let mut lnum: linenr_T = 0;
    lnum = 1 as ::core::ffi::c_int as linenr_T;
    while lnum < (*curbuf.get()).b_ml.ml_line_count && lnum <= (*uep).ue_size {
        if strcmp(
            ml_get_buf(curbuf.get(), lnum),
            *(*uep).ue_array.offset((lnum - 1 as linenr_T) as isize),
        ) != 0 as ::core::ffi::c_int
        {
            clearpos(&raw mut (*uhp).uh_cursor);
            (*uhp).uh_cursor.lnum = lnum;
            return;
        }
        lnum += 1;
    }
    if (*curbuf.get()).b_ml.ml_line_count != (*uep).ue_size {
        clearpos(&raw mut (*uhp).uh_cursor);
        (*uhp).uh_cursor.lnum = lnum;
    }
}
pub unsafe extern "C" fn u_update_save_nr(mut buf: *mut buf_T) {
    (*buf).b_u_save_nr_last += 1;
    (*buf).b_u_save_nr_cur = (*buf).b_u_save_nr_last;
    let mut uhp: *mut u_header_T = (*buf).b_u_curhead;
    if !uhp.is_null() {
        uhp = (*uhp).uh_next.ptr;
    } else {
        uhp = (*buf).b_u_newhead;
    }
    if !uhp.is_null() {
        (*uhp).uh_save_nr = (*buf).b_u_save_nr_last;
    }
}
unsafe extern "C" fn u_unch_branch(mut uhp: *mut u_header_T) {
    let mut uh: *mut u_header_T = uhp;
    while !uh.is_null() {
        (*uh).uh_flags |= UH_CHANGED as ::core::ffi::c_int;
        if !(*uh).uh_alt_next.ptr.is_null() {
            u_unch_branch((*uh).uh_alt_next.ptr);
        }
        uh = (*uh).uh_prev.ptr;
    }
}
unsafe extern "C" fn u_get_headentry(mut buf: *mut buf_T) -> *mut u_entry_T {
    if (*buf).b_u_newhead.is_null() || (*(*buf).b_u_newhead).uh_entry.is_null() {
        iemsg(gettext(
            (e_undo_list_corrupt.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return ::core::ptr::null_mut::<u_entry_T>();
    }
    return (*(*buf).b_u_newhead).uh_entry;
}
unsafe extern "C" fn u_getbot(mut buf: *mut buf_T) {
    let mut uep: *mut u_entry_T = u_get_headentry(buf);
    if uep.is_null() {
        return;
    }
    uep = (*(*buf).b_u_newhead).uh_getbot_entry;
    if !uep.is_null() {
        let mut extra: linenr_T = (*buf).b_ml.ml_line_count - (*uep).ue_lcount;
        (*uep).ue_bot = (*uep).ue_top + (*uep).ue_size + 1 as linenr_T + extra;
        if (*uep).ue_bot < 1 as linenr_T || (*uep).ue_bot > (*buf).b_ml.ml_line_count {
            iemsg(gettext(
                (e_undo_line_missing.ptr() as *const _) as *const ::core::ffi::c_char,
            ));
            (*uep).ue_bot = (*uep).ue_top + 1 as linenr_T;
        }
        (*(*buf).b_u_newhead).uh_getbot_entry = ::core::ptr::null_mut::<u_entry_T>();
    }
    (*buf).b_u_synced = true_0 != 0;
}
unsafe extern "C" fn u_freeheader(
    mut buf: *mut buf_T,
    mut uhp: *mut u_header_T,
    mut uhpp: *mut *mut u_header_T,
) {
    if !(*uhp).uh_alt_next.ptr.is_null() {
        u_freebranch(buf, (*uhp).uh_alt_next.ptr, uhpp);
    }
    if !(*uhp).uh_alt_prev.ptr.is_null() {
        (*(*uhp).uh_alt_prev.ptr).uh_alt_next.ptr = ::core::ptr::null_mut::<u_header_T>();
    }
    if (*uhp).uh_next.ptr.is_null() {
        (*buf).b_u_oldhead = (*uhp).uh_prev.ptr;
    } else {
        (*(*uhp).uh_next.ptr).uh_prev.ptr = (*uhp).uh_prev.ptr;
    }
    if (*uhp).uh_prev.ptr.is_null() {
        (*buf).b_u_newhead = (*uhp).uh_next.ptr;
    } else {
        let mut uhap: *mut u_header_T = (*uhp).uh_prev.ptr;
        while !uhap.is_null() {
            (*uhap).uh_next.ptr = (*uhp).uh_next.ptr;
            uhap = (*uhap).uh_alt_next.ptr;
        }
    }
    u_freeentries(buf, uhp, uhpp);
}
unsafe extern "C" fn u_freebranch(
    mut buf: *mut buf_T,
    mut uhp: *mut u_header_T,
    mut uhpp: *mut *mut u_header_T,
) {
    if uhp == (*buf).b_u_oldhead {
        while !(*buf).b_u_oldhead.is_null() {
            u_freeheader(buf, (*buf).b_u_oldhead, uhpp);
        }
        return;
    }
    if !(*uhp).uh_alt_prev.ptr.is_null() {
        (*(*uhp).uh_alt_prev.ptr).uh_alt_next.ptr = ::core::ptr::null_mut::<u_header_T>();
    }
    let mut next: *mut u_header_T = uhp;
    while !next.is_null() {
        let mut tofree: *mut u_header_T = next;
        if !(*tofree).uh_alt_next.ptr.is_null() {
            u_freebranch(buf, (*tofree).uh_alt_next.ptr, uhpp);
        }
        next = (*tofree).uh_prev.ptr;
        u_freeentries(buf, tofree, uhpp);
    }
}
unsafe extern "C" fn u_freeentries(
    mut buf: *mut buf_T,
    mut uhp: *mut u_header_T,
    mut uhpp: *mut *mut u_header_T,
) {
    if (*buf).b_u_curhead == uhp {
        (*buf).b_u_curhead = ::core::ptr::null_mut::<u_header_T>();
    }
    if (*buf).b_u_newhead == uhp {
        (*buf).b_u_newhead = ::core::ptr::null_mut::<u_header_T>();
    }
    if !uhpp.is_null() && uhp == *uhpp {
        *uhpp = ::core::ptr::null_mut::<u_header_T>();
    }
    let mut nuep: *mut u_entry_T = ::core::ptr::null_mut::<u_entry_T>();
    let mut uep: *mut u_entry_T = (*uhp).uh_entry;
    while !uep.is_null() {
        nuep = (*uep).ue_next;
        u_freeentry(uep, (*uep).ue_size as ::core::ffi::c_int);
        uep = nuep;
    }
    xfree((*uhp).uh_extmark.items as *mut ::core::ffi::c_void);
    (*uhp).uh_extmark.capacity = 0 as size_t;
    (*uhp).uh_extmark.size = (*uhp).uh_extmark.capacity;
    (*uhp).uh_extmark.items = ::core::ptr::null_mut::<ExtmarkUndoObject>();
    xfree(uhp as *mut ::core::ffi::c_void);
    (*buf).b_u_numhead -= 1;
}
unsafe extern "C" fn u_freeentry(mut uep: *mut u_entry_T, mut n: ::core::ffi::c_int) {
    while n > 0 as ::core::ffi::c_int {
        n -= 1;
        xfree(*(*uep).ue_array.offset(n as isize) as *mut ::core::ffi::c_void);
    }
    xfree((*uep).ue_array as *mut ::core::ffi::c_void);
    xfree(uep as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn u_clearall(mut buf: *mut buf_T) {
    (*buf).b_u_curhead = ::core::ptr::null_mut::<u_header_T>();
    (*buf).b_u_oldhead = (*buf).b_u_curhead;
    (*buf).b_u_newhead = (*buf).b_u_oldhead;
    (*buf).b_u_synced = true_0 != 0;
    (*buf).b_u_numhead = 0 as ::core::ffi::c_int;
    (*buf).b_u_line_ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*buf).b_u_line_lnum = 0 as ::core::ffi::c_int as linenr_T;
}
pub unsafe extern "C" fn u_blockfree(mut buf: *mut buf_T) {
    while !(*buf).b_u_oldhead.is_null() {
        let mut previous_oldhead: *mut u_header_T = (*buf).b_u_oldhead;
        u_freeheader(
            buf,
            (*buf).b_u_oldhead,
            ::core::ptr::null_mut::<*mut u_header_T>(),
        );
        '_c2rust_label: {
            if (*buf).b_u_oldhead != previous_oldhead {
            } else {
                __assert_fail(
                    b"buf->b_u_oldhead != previous_oldhead\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/undo.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3006 as ::core::ffi::c_uint,
                    b"void u_blockfree(buf_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
    }
    xfree((*buf).b_u_line_ptr as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn u_clearallandblockfree(mut buf: *mut buf_T) {
    u_blockfree(buf);
    u_clearall(buf);
}
unsafe extern "C" fn u_saveline(mut buf: *mut buf_T, mut lnum: linenr_T) {
    if lnum == (*buf).b_u_line_lnum {
        return;
    }
    if lnum < 1 as linenr_T || lnum > (*buf).b_ml.ml_line_count {
        return;
    }
    u_clearline(buf);
    (*buf).b_u_line_lnum = lnum;
    if (*curwin.get()).w_buffer == buf && (*curwin.get()).w_cursor.lnum == lnum {
        (*buf).b_u_line_colnr = (*curwin.get()).w_cursor.col;
    } else {
        (*buf).b_u_line_colnr = 0 as ::core::ffi::c_int as colnr_T;
    }
    (*buf).b_u_line_ptr = u_save_line_buf(buf, lnum);
}
pub unsafe extern "C" fn u_clearline(mut buf: *mut buf_T) {
    if (*buf).b_u_line_ptr.is_null() {
        return;
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*buf).b_u_line_ptr as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*buf).b_u_line_lnum = 0 as ::core::ffi::c_int as linenr_T;
}
pub unsafe extern "C" fn u_undoline() {
    if (*curbuf.get()).b_u_line_ptr.is_null()
        || (*curbuf.get()).b_u_line_lnum > (*curbuf.get()).b_ml.ml_line_count
    {
        beep_flush();
        return;
    }
    if u_savecommon(
        curbuf.get(),
        (*curbuf.get()).b_u_line_lnum - 1 as linenr_T,
        (*curbuf.get()).b_u_line_lnum + 1 as linenr_T,
        0 as linenr_T,
        false_0 != 0,
    ) == FAIL
    {
        return;
    }
    let mut oldp: *mut ::core::ffi::c_char = u_save_line((*curbuf.get()).b_u_line_lnum);
    ml_replace(
        (*curbuf.get()).b_u_line_lnum,
        (*curbuf.get()).b_u_line_ptr,
        true_0 != 0,
    );
    extmark_splice_cols(
        curbuf.get(),
        (*curbuf.get()).b_u_line_lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        0 as colnr_T,
        strlen(oldp) as colnr_T,
        strlen((*curbuf.get()).b_u_line_ptr) as colnr_T,
        kExtmarkUndo,
    );
    changed_bytes((*curbuf.get()).b_u_line_lnum, 0 as colnr_T);
    xfree((*curbuf.get()).b_u_line_ptr as *mut ::core::ffi::c_void);
    (*curbuf.get()).b_u_line_ptr = oldp;
    let mut t: colnr_T = (*curbuf.get()).b_u_line_colnr;
    if (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_u_line_lnum {
        (*curbuf.get()).b_u_line_colnr = (*curwin.get()).w_cursor.col;
    }
    (*curwin.get()).w_cursor.col = t;
    (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_u_line_lnum;
    check_cursor_col(curwin.get());
}
unsafe extern "C" fn u_save_line(mut lnum: linenr_T) -> *mut ::core::ffi::c_char {
    return u_save_line_buf(curbuf.get(), lnum);
}
unsafe extern "C" fn u_save_line_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
) -> *mut ::core::ffi::c_char {
    return xstrdup(ml_get_buf(buf, lnum));
}
pub unsafe extern "C" fn bufIsChanged(mut buf: *mut buf_T) -> bool {
    return if bt_prompt(buf) as ::core::ffi::c_int != 0 {
        (*buf).b_modified_was_set as ::core::ffi::c_int
    } else {
        (!bt_dontwrite(buf)
            && ((*buf).b_changed != 0
                || file_ff_differs(buf, true_0 != 0) as ::core::ffi::c_int != 0))
            as ::core::ffi::c_int
    } != 0;
}
pub unsafe extern "C" fn anyBufIsChanged() -> bool {
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if bufIsChanged(buf) {
            return true_0 != 0;
        }
        buf = (*buf).b_next;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn curbufIsChanged() -> bool {
    return bufIsChanged(curbuf.get());
}
unsafe extern "C" fn u_eval_tree(buf: *mut buf_T, first_uhp: *const u_header_T) -> *mut list_T {
    let list: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut uhp: *const u_header_T = first_uhp;
    while !uhp.is_null() {
        let dict: *mut dict_T = tv_dict_alloc();
        tv_dict_add_nr(
            dict,
            b"seq\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            (*uhp).uh_seq as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            b"time\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*uhp).uh_time as varnumber_T,
        );
        if uhp == (*buf).b_u_newhead as *const u_header_T {
            tv_dict_add_nr(
                dict,
                b"newhead\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                1 as varnumber_T,
            );
        }
        if uhp == (*buf).b_u_curhead as *const u_header_T {
            tv_dict_add_nr(
                dict,
                b"curhead\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                1 as varnumber_T,
            );
        }
        if (*uhp).uh_save_nr > 0 as ::core::ffi::c_int {
            tv_dict_add_nr(
                dict,
                b"save\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                (*uhp).uh_save_nr as varnumber_T,
            );
        }
        if !(*uhp).uh_alt_next.ptr.is_null() {
            tv_dict_add_list(
                dict,
                b"alt\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                u_eval_tree(buf, (*uhp).uh_alt_next.ptr),
            );
        }
        tv_list_append_dict(list, dict);
        uhp = (*uhp).uh_prev.ptr;
    }
    return list;
}
pub unsafe extern "C" fn f_undofile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    let fname: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    if *fname as ::core::ffi::c_int == NUL {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        let mut ffname: *mut ::core::ffi::c_char = FullName_save(fname, true_0 != 0);
        if !ffname.is_null() {
            (*rettv).vval.v_string = u_get_undo_file_name(ffname, false_0 != 0);
        }
        xfree(ffname as *mut ::core::ffi::c_void);
    };
}
pub unsafe extern "C" fn f_undotree(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let tv: *mut typval_T = argvars.offset(0 as ::core::ffi::c_int as isize);
    let buf: *mut buf_T = if (*tv).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        curbuf.get()
    } else {
        get_buf_arg(tv)
    };
    if buf.is_null() {
        return;
    }
    let mut dict: *mut dict_T = (*rettv).vval.v_dict;
    tv_dict_add_nr(
        dict,
        b"synced\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*buf).b_u_synced as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"seq_last\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*buf).b_u_seq_last as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"save_last\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (*buf).b_u_save_nr_last as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"seq_cur\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*buf).b_u_seq_cur as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"time_cur\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*buf).b_u_time_cur as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"save_cur\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*buf).b_u_save_nr_cur as varnumber_T,
    );
    tv_dict_add_list(
        dict,
        b"entries\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        u_eval_tree(buf, (*buf).b_u_oldhead),
    );
}
pub unsafe extern "C" fn u_force_get_undo_header(mut buf: *mut buf_T) -> *mut u_header_T {
    let mut uhp: *mut u_header_T = ::core::ptr::null_mut::<u_header_T>();
    if !(*buf).b_u_curhead.is_null() {
        uhp = (*buf).b_u_curhead;
    } else if !(*buf).b_u_newhead.is_null() {
        uhp = (*buf).b_u_newhead;
    }
    if uhp.is_null() {
        u_savecommon(
            buf,
            0 as linenr_T,
            1 as linenr_T,
            1 as linenr_T,
            true_0 != 0,
        );
        uhp = (*buf).b_u_curhead;
        if uhp.is_null() {
            uhp = (*buf).b_u_newhead;
            if get_undolevel(buf) > 0 as OptInt && uhp.is_null() {
                abort();
            }
        }
    }
    return uhp;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
