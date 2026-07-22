use crate::src::nvim::autocmd::apply_autocmds;
use crate::src::nvim::charset::{getdigits_int, skipwhite};
use crate::src::nvim::cursor::{check_cursor, gchar_cursor};
use crate::src::nvim::eval::typval::{
    tv_dict_add_allocated_str, tv_dict_add_bool, tv_dict_add_dict, tv_dict_add_list,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_alloc_ret, tv_get_string_chk,
    tv_list_alloc, tv_list_append_dict, tv_list_append_string,
};
use crate::src::nvim::eval::vars::del_menutrans_vars;
use crate::src::nvim::ex_docmd::{
    ends_excmd, exec_normal_cmd, restore_current_state, save_current_state,
};
use crate::src::nvim::getchar::ins_typebuf;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::replace_termcodes;
use crate::src::nvim::main::{
    curbuf, current_sctx, curwin, e_cannot_change_menus_while_listing, e_invarg, e_invarg2,
    e_menu_only_exists_in_another_mode, e_trailing_arg, ex_normal_busy, finish_op, got_int, p_cpo,
    p_sel, restart_edit, root_menu, sys_menu, State, VIsual, VIsual_active, VIsual_mode,
    VIsual_reselect, VIsual_select,
};
use crate::src::nvim::mbyte::{utf_char2bytes, utfc_ptr2len};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemdupz, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    emsg, msg_outnum, msg_outtrans, msg_outtrans_special, msg_putchar, msg_puts, msg_puts_hl,
    msg_puts_title, semsg, str2special_save,
};
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, memmove, strcasecmp, strcat, strcmp, strcpy, strlen, strncasecmp,
    strncmp,
};
use crate::src::nvim::popupmenu::pum_show_popupmenu;
use crate::src::nvim::state::get_real_state;
use crate::src::nvim::strings::{vim_strchr, xstrnsave};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CMD_index, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, Direction, EvalFuncData, ExtmarkUndoObject, FileID,
    FloatAnchor, FloatRelative, GridView, Intersection, LineGetter, ListLenSpecials, LuaRef, MTKey,
    MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MsgpackRpcRequestHandler, OptInt, RemapValues, ScopeDictDictItem,
    ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue,
    StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_11, String_0, Terminal,
    Timestamp, TriState, VarLockStatus, VarType, VimMenu, VirtLines, VirtText, VirtTextChunk,
    VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, __time_t, alist_T, auto_event,
    bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, buffblock, buffblock_T, buffheader_T, bufstate_T,
    chunksize_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_15,
    dict_T, dictvar_S, disptick_T, eslist_T, eslist_elem, event_T, exarg, exarg_T, expand_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_10, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, save_state_T,
    schar_T, scid_T, sctx_T, size_t, ssize_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_3,
    syn_time_T, synblock_T, synstate_T, taggy_T, tasave_T, terminal, time_t, typebuf_T, typval_T,
    typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_7, u_header_uh_alt_prev as C2Rust_Unnamed_6,
    u_header_uh_next as C2Rust_Unnamed_9, u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, vimmenu_T, virt_line,
    visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, xp_prefix_T, QUEUE,
};
use crate::src::nvim::ui::ui_call_update_menu;
extern "C" {
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_append_via_ptr(gap: *mut garray_T, item_size: size_t) -> *mut ::core::ffi::c_void;
}
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_11 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_11 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_11 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_11 = 0;
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
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_12 = 2147483647;
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
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_14 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_14 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_14 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_14 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_14 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_14 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_14 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_14 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_14 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_14 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_14 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_14 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_14 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_14 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_14 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_14 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_14 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_14 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_14 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_14 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_14 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_14 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_14 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_14 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_14 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_14 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_14 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_14 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_14 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_14 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_14 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_14 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_14 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_14 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_14 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_14 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_14 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_14 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_14 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_14 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_14 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_14 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_14 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_14 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_14 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_14 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_14 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_14 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_14 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_14 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_14 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_14 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_14 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_14 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_14 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_14 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_14 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_14 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_14 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_14 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_14 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_14 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_14 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_14 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_14 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_14 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_14 = -2;
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
pub const REMAP_SKIP: RemapValues = -3;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const MENU_MODES: C2Rust_Unnamed_16 = 8;
pub const MENU_INDEX_TIP: C2Rust_Unnamed_16 = 7;
pub const MENU_INDEX_TERMINAL: C2Rust_Unnamed_16 = 6;
pub const MENU_INDEX_CMDLINE: C2Rust_Unnamed_16 = 5;
pub const MENU_INDEX_INSERT: C2Rust_Unnamed_16 = 4;
pub const MENU_INDEX_OP_PENDING: C2Rust_Unnamed_16 = 3;
pub const MENU_INDEX_SELECT: C2Rust_Unnamed_16 = 2;
pub const MENU_INDEX_VISUAL: C2Rust_Unnamed_16 = 1;
pub const MENU_INDEX_NORMAL: C2Rust_Unnamed_16 = 0;
pub const MENU_INDEX_INVALID: C2Rust_Unnamed_16 = -1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const MENU_ALL_MODES: C2Rust_Unnamed_17 = 127;
pub const MENU_TIP_MODE: C2Rust_Unnamed_17 = 128;
pub const MENU_TERMINAL_MODE: C2Rust_Unnamed_17 = 64;
pub const MENU_CMDLINE_MODE: C2Rust_Unnamed_17 = 32;
pub const MENU_INSERT_MODE: C2Rust_Unnamed_17 = 16;
pub const MENU_OP_PENDING_MODE: C2Rust_Unnamed_17 = 8;
pub const MENU_SELECT_MODE: C2Rust_Unnamed_17 = 4;
pub const MENU_VISUAL_MODE: C2Rust_Unnamed_17 = 2;
pub const MENU_NORMAL_MODE: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_18 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_18 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_18 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_18 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_18 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_18 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_18 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_18 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_18 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_18 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_18 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_18 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_18 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_18 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_18 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_18 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_18 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_18 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_18 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const REPTERM_NO_SIMPLIFY: C2Rust_Unnamed_19 = 8;
pub const REPTERM_NO_SPECIAL: C2Rust_Unnamed_19 = 4;
pub const REPTERM_DO_LT: C2Rust_Unnamed_19 = 2;
pub const REPTERM_FROM_PART: C2Rust_Unnamed_19 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menutrans_T {
    pub from: *mut ::core::ffi::c_char,
    pub from_noamp: *mut ::core::ffi::c_char,
    pub to: *mut ::core::ffi::c_char,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const Ctrl_G: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const Ctrl_O: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_BSL: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_dict_len(d: *const dict_T) -> ::core::ffi::c_long {
    if d.is_null() {
        return 0 as ::core::ffi::c_long;
    }
    return (*d).dv_hashtab.ht_used as ::core::ffi::c_long;
}
pub const MNU_HIDDEN_CHAR: ::core::ffi::c_int = ']' as ::core::ffi::c_int;
pub const MENUDEPTH: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
static menus_locked: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static menu_mode_chars: GlobalCell<[*mut ::core::ffi::c_char; 8]> = GlobalCell::new([
    b"n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"v\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"s\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"o\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"i\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"c\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"tl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"t\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
]);
static e_notsubmenu: GlobalCell<[::core::ffi::c_char; 45]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"E327: Part of menu-item path is not sub-menu\0",
    )
});
static e_nomenu: GlobalCell<[::core::ffi::c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E329: No menu \"%s\"\0")
});
unsafe extern "C" fn menu_is_winbar(name: *const ::core::ffi::c_char) -> bool {
    return strncmp(
        name,
        b"WinBar\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn get_root_menu(_name: *const ::core::ffi::c_char) -> *mut *mut vimmenu_T {
    return root_menu.ptr();
}
unsafe extern "C" fn is_menus_locked() -> ::core::ffi::c_int {
    if menus_locked.get() > 0 as ::core::ffi::c_int {
        emsg(gettext(
            &raw const e_cannot_change_menus_while_listing as *const ::core::ffi::c_char,
        ));
        return true_0;
    }
    return false_0;
}
pub unsafe extern "C" fn ex_menu(mut eap: *mut exarg_T) {
    let mut map_to: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut noremap: ::core::ffi::c_int = 0;
    let mut silent: bool = false_0 != 0;
    let mut unmenu: bool = false;
    let mut map_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    let mut pri_tab: [::core::ffi::c_int; 11] = [0; 11];
    let mut enable: TriState = kNone;
    let mut menuarg: vimmenu_T = vimmenu_T {
        modes: 0,
        enabled: 0,
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        dname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        en_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        en_dname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        mnemonic: 0,
        actext: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        priority: 0,
        strings: [::core::ptr::null_mut::<::core::ffi::c_char>(); 8],
        noremap: [0; 8],
        silent: [false; 8],
        children: ::core::ptr::null_mut::<vimmenu_T>(),
        parent: ::core::ptr::null_mut::<vimmenu_T>(),
        next: ::core::ptr::null_mut::<vimmenu_T>(),
    };
    let mut modes: ::core::ffi::c_int = get_menu_cmd_modes(
        (*eap).cmd,
        (*eap).forceit != 0,
        &raw mut noremap,
        &raw mut unmenu,
    );
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    loop {
        if strncmp(
            arg,
            b"<script>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            noremap = REMAP_SCRIPT as ::core::ffi::c_int;
            arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
        } else if strncmp(
            arg,
            b"<silent>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            silent = true_0 != 0;
            arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
        } else {
            if strncmp(
                arg,
                b"<special>\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                break;
            }
            arg = skipwhite(arg.offset(9 as ::core::ffi::c_int as isize));
        }
    }
    if strncmp(
        arg,
        b"icon=\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        arg = arg.offset(5 as ::core::ffi::c_int as isize);
        while *arg as ::core::ffi::c_int != NUL
            && *arg as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
        {
            if *arg as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                memmove(
                    arg as *mut ::core::ffi::c_void,
                    arg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(arg.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            }
            arg = arg.offset(utfc_ptr2len(arg) as isize);
        }
        if *arg as ::core::ffi::c_int != NUL {
            let c2rust_fresh0 = arg;
            arg = arg.offset(1);
            *c2rust_fresh0 = NUL as ::core::ffi::c_char;
            arg = skipwhite(arg);
        }
    }
    p = arg;
    while *p != 0 {
        if !ascii_isdigit(*p as ::core::ffi::c_int)
            && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        {
            break;
        }
        p = p.offset(1);
    }
    if ascii_iswhite(*p as ::core::ffi::c_int) {
        i = 0 as ::core::ffi::c_int;
        while i < MENUDEPTH && !ascii_iswhite(*arg as ::core::ffi::c_int) {
            pri_tab[i as usize] =
                getdigits_int(&raw mut arg, false_0 != 0, 0 as ::core::ffi::c_int);
            if pri_tab[i as usize] == 0 as ::core::ffi::c_int {
                pri_tab[i as usize] = 500 as ::core::ffi::c_int;
            }
            if *arg as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                arg = arg.offset(1);
            }
            i += 1;
        }
        arg = skipwhite(arg);
    } else if (*eap).addr_count != 0 && (*eap).line2 != 0 as linenr_T {
        pri_tab[0 as ::core::ffi::c_int as usize] = (*eap).line2 as ::core::ffi::c_int;
        i = 1 as ::core::ffi::c_int;
    } else {
        i = 0 as ::core::ffi::c_int;
    }
    while i < MENUDEPTH {
        let c2rust_fresh1 = i;
        i = i + 1;
        pri_tab[c2rust_fresh1 as usize] = 500 as ::core::ffi::c_int;
    }
    pri_tab[MENUDEPTH as usize] = -1 as ::core::ffi::c_int;
    if strncmp(
        arg,
        b"enable\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
        && ascii_iswhite(*arg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        enable = kTrue;
        arg = skipwhite(arg.offset(6 as ::core::ffi::c_int as isize));
    } else if strncmp(
        arg,
        b"disable\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
        && ascii_iswhite(*arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        enable = kFalse;
        arg = skipwhite(arg.offset(7 as ::core::ffi::c_int as isize));
    }
    if *arg as ::core::ffi::c_int == NUL {
        show_menus(arg, modes);
        return;
    }
    let mut menu_path: *mut ::core::ffi::c_char = arg;
    's_573: {
        if *menu_path as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                menu_path,
            );
        } else {
            map_to = menu_translate_tab_and_shift(arg);
            if *map_to as ::core::ffi::c_int == NUL
                && !unmenu
                && enable as ::core::ffi::c_int == kNone as ::core::ffi::c_int
            {
                show_menus(menu_path, modes);
            } else if *map_to as ::core::ffi::c_int != NUL
                && (unmenu as ::core::ffi::c_int != 0
                    || enable as ::core::ffi::c_int != kNone as ::core::ffi::c_int)
            {
                semsg(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    map_to,
                );
            } else {
                let mut root_menu_ptr: *mut *mut vimmenu_T = get_root_menu(menu_path);
                if enable as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
                    if strcmp(menu_path, b"*\0".as_ptr() as *const ::core::ffi::c_char)
                        == 0 as ::core::ffi::c_int
                    {
                        menu_path = b"\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    if menu_is_popup(menu_path) {
                        i = 0 as ::core::ffi::c_int;
                        while i < MENU_INDEX_TIP as ::core::ffi::c_int {
                            if modes & (1 as ::core::ffi::c_int) << i != 0 {
                                p = popup_mode_name(menu_path, i);
                                menu_enable_recurse(
                                    *root_menu_ptr,
                                    p,
                                    MENU_ALL_MODES as ::core::ffi::c_int,
                                    enable as ::core::ffi::c_int,
                                );
                                xfree(p as *mut ::core::ffi::c_void);
                            }
                            i += 1;
                        }
                    }
                    menu_enable_recurse(
                        *root_menu_ptr,
                        menu_path,
                        modes,
                        enable as ::core::ffi::c_int,
                    );
                } else if unmenu {
                    if is_menus_locked() != 0 {
                        break 's_573;
                    } else {
                        if strcmp(menu_path, b"*\0".as_ptr() as *const ::core::ffi::c_char)
                            == 0 as ::core::ffi::c_int
                        {
                            menu_path = b"\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        if menu_is_popup(menu_path) {
                            i = 0 as ::core::ffi::c_int;
                            while i < MENU_INDEX_TIP as ::core::ffi::c_int {
                                if modes & (1 as ::core::ffi::c_int) << i != 0 {
                                    p = popup_mode_name(menu_path, i);
                                    remove_menu(
                                        root_menu_ptr,
                                        p,
                                        MENU_ALL_MODES as ::core::ffi::c_int,
                                        true_0 != 0,
                                    );
                                    xfree(p as *mut ::core::ffi::c_void);
                                }
                                i += 1;
                            }
                        }
                        remove_menu(root_menu_ptr, menu_path, modes, false_0 != 0);
                    }
                } else if is_menus_locked() != 0 {
                    break 's_573;
                } else {
                    if strcasecmp(
                        map_to,
                        b"<nop>\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        map_to = b"\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        map_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    } else if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
                        map_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    } else {
                        map_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        map_to = replace_termcodes(
                            map_to,
                            strlen(map_to),
                            &raw mut map_buf,
                            0 as scid_T,
                            REPTERM_DO_LT as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<bool>(),
                            p_cpo.get(),
                        );
                    }
                    menuarg.modes = modes;
                    menuarg.noremap[0 as ::core::ffi::c_int as usize] = noremap;
                    menuarg.silent[0 as ::core::ffi::c_int as usize] = silent;
                    add_menu_path(
                        menu_path,
                        &raw mut menuarg,
                        &raw mut pri_tab as *mut ::core::ffi::c_int,
                        map_to,
                    );
                    if menu_is_popup(menu_path) {
                        i = 0 as ::core::ffi::c_int;
                        while i < MENU_INDEX_TIP as ::core::ffi::c_int {
                            if modes & (1 as ::core::ffi::c_int) << i != 0 {
                                p = popup_mode_name(menu_path, i);
                                menuarg.modes = modes;
                                add_menu_path(
                                    p,
                                    &raw mut menuarg,
                                    &raw mut pri_tab as *mut ::core::ffi::c_int,
                                    map_to,
                                );
                                xfree(p as *mut ::core::ffi::c_void);
                            }
                            i += 1;
                        }
                    }
                    xfree(map_buf as *mut ::core::ffi::c_void);
                }
                ui_call_update_menu();
            }
        }
    };
}
unsafe extern "C" fn add_menu_path(
    menu_path: *const ::core::ffi::c_char,
    mut menuarg: *mut vimmenu_T,
    pri_tab: *const ::core::ffi::c_int,
    call_data: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut amenu: ::core::ffi::c_int = 0;
    let mut modes: ::core::ffi::c_int = (*menuarg).modes;
    let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    let mut lower_pri: *mut *mut vimmenu_T = ::core::ptr::null_mut::<*mut vimmenu_T>();
    let mut dname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pri_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut old_modes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut en_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path_name: *mut ::core::ffi::c_char = xstrdup(menu_path);
    let mut root_menu_ptr: *mut *mut vimmenu_T = get_root_menu(menu_path);
    let mut menup: *mut *mut vimmenu_T = root_menu_ptr;
    let mut parent: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    let mut name: *mut ::core::ffi::c_char = path_name;
    '_erret: {
        while *name != 0 {
            let mut next_name: *mut ::core::ffi::c_char = menu_name_skip(name);
            let mut map_to: *mut ::core::ffi::c_char =
                menutrans_lookup(name, strlen(name) as ::core::ffi::c_int);
            if !map_to.is_null() {
                en_name = name;
                name = map_to;
            } else {
                en_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            dname = menu_text(
                name,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            if *dname as ::core::ffi::c_int == NUL {
                emsg(gettext(
                    b"E792: Empty menu name\0".as_ptr() as *const ::core::ffi::c_char
                ));
                break '_erret;
            } else {
                lower_pri = menup;
                menu = *menup;
                while !menu.is_null() {
                    if menu_name_equal(name, menu) as ::core::ffi::c_int != 0
                        || menu_name_equal(dname, menu) as ::core::ffi::c_int != 0
                    {
                        if *next_name as ::core::ffi::c_int == NUL && !(*menu).children.is_null() {
                            if !sys_menu.get() {
                                emsg(gettext(
                                    b"E330: Menu path must not lead to a sub-menu\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ));
                            }
                            break '_erret;
                        } else {
                            if !(*next_name as ::core::ffi::c_int != NUL
                                && (*menu).children.is_null())
                            {
                                break;
                            }
                            if !sys_menu.get() {
                                emsg(gettext(
                                    (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                                ));
                            }
                            break '_erret;
                        }
                    } else {
                        menup = &raw mut (*menu).next;
                        if !parent.is_null()
                            || menu_is_menubar((*menu).name) as ::core::ffi::c_int != 0
                        {
                            if (*menu).priority <= *pri_tab.offset(pri_idx as isize) {
                                lower_pri = menup;
                            }
                        }
                        menu = (*menu).next;
                    }
                }
                if menu.is_null() {
                    if *next_name as ::core::ffi::c_int == NUL && parent.is_null() {
                        emsg(gettext(
                            b"E331: Must not add menu items directly to menu bar\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ));
                        break '_erret;
                    } else if menu_is_separator(dname) as ::core::ffi::c_int != 0
                        && *next_name as ::core::ffi::c_int != NUL
                    {
                        emsg(gettext(
                            b"E332: Separator cannot be part of a menu path\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ));
                        break '_erret;
                    } else {
                        menu = xcalloc(1 as size_t, ::core::mem::size_of::<vimmenu_T>())
                            as *mut vimmenu_T;
                        (*menu).modes = modes;
                        (*menu).enabled = MENU_ALL_MODES as ::core::ffi::c_int;
                        (*menu).name = xstrdup(name);
                        (*menu).dname =
                            menu_text(name, &raw mut (*menu).mnemonic, &raw mut (*menu).actext);
                        if !en_name.is_null() {
                            (*menu).en_name = xstrdup(en_name);
                            (*menu).en_dname = menu_text(
                                en_name,
                                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                            );
                        } else {
                            (*menu).en_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            (*menu).en_dname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        }
                        (*menu).priority = *pri_tab.offset(pri_idx as isize);
                        (*menu).parent = parent;
                        (*menu).next = *lower_pri;
                        *lower_pri = menu;
                        old_modes = 0 as ::core::ffi::c_int;
                    }
                } else {
                    old_modes = (*menu).modes;
                    (*menu).modes |= modes;
                    (*menu).enabled |= modes;
                }
                menup = &raw mut (*menu).children;
                parent = menu;
                name = next_name;
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut dname as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                let _ = *ptr_;
                if *pri_tab.offset((pri_idx + 1 as ::core::ffi::c_int) as isize)
                    != -1 as ::core::ffi::c_int
                {
                    pri_idx += 1;
                }
            }
        }
        xfree(path_name as *mut ::core::ffi::c_void);
        amenu = (modes
            & (MENU_NORMAL_MODE as ::core::ffi::c_int | MENU_INSERT_MODE as ::core::ffi::c_int)
            == MENU_NORMAL_MODE as ::core::ffi::c_int | MENU_INSERT_MODE as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        if sys_menu.get() {
            modes &= !old_modes;
        }
        if !menu.is_null() && modes != 0 {
            let mut p: *mut ::core::ffi::c_char = if call_data.is_null() {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                xstrdup(call_data)
            };
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < MENU_MODES as ::core::ffi::c_int {
                if modes & (1 as ::core::ffi::c_int) << i != 0 {
                    free_menu_string(menu, i);
                    let mut c: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
                    let mut d: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
                    if amenu != 0 && !call_data.is_null() && *call_data as ::core::ffi::c_int != NUL
                    {
                        match (1 as ::core::ffi::c_int) << i {
                            2 | 4 | 8 | 32 => {
                                c = Ctrl_C as ::core::ffi::c_char;
                            }
                            16 => {
                                c = Ctrl_BSL as ::core::ffi::c_char;
                                d = Ctrl_O as ::core::ffi::c_char;
                            }
                            _ => {}
                        }
                    }
                    if c as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                        (*menu).strings[i as usize] =
                            xmalloc(strlen(call_data).wrapping_add(5 as size_t))
                                as *mut ::core::ffi::c_char;
                        *(*menu).strings[i as usize].offset(0 as ::core::ffi::c_int as isize) = c;
                        if d as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                            strcpy(
                                (*menu).strings[i as usize]
                                    .offset(1 as ::core::ffi::c_int as isize),
                                call_data as *mut ::core::ffi::c_char,
                            );
                        } else {
                            *(*menu).strings[i as usize].offset(1 as ::core::ffi::c_int as isize) =
                                d;
                            strcpy(
                                (*menu).strings[i as usize]
                                    .offset(2 as ::core::ffi::c_int as isize),
                                call_data as *mut ::core::ffi::c_char,
                            );
                        }
                        if c as ::core::ffi::c_int == Ctrl_C {
                            let mut len: ::core::ffi::c_int =
                                strlen((*menu).strings[i as usize]) as ::core::ffi::c_int;
                            *(*menu).strings[i as usize].offset(len as isize) =
                                Ctrl_BSL as ::core::ffi::c_char;
                            *(*menu).strings[i as usize]
                                .offset((len + 1 as ::core::ffi::c_int) as isize) =
                                Ctrl_G as ::core::ffi::c_char;
                            *(*menu).strings[i as usize]
                                .offset((len + 2 as ::core::ffi::c_int) as isize) =
                                NUL as ::core::ffi::c_char;
                        }
                    } else {
                        (*menu).strings[i as usize] = p;
                    }
                    (*menu).noremap[i as usize] =
                        (*menuarg).noremap[0 as ::core::ffi::c_int as usize];
                    (*menu).silent[i as usize] =
                        (*menuarg).silent[0 as ::core::ffi::c_int as usize];
                }
                i += 1;
            }
        }
        return OK;
    }
    xfree(path_name as *mut ::core::ffi::c_void);
    xfree(dname as *mut ::core::ffi::c_void);
    while !parent.is_null() && (*parent).children.is_null() {
        if (*parent).parent.is_null() {
            menup = root_menu_ptr;
        } else {
            menup = &raw mut (*(*parent).parent).children;
        }
        while !(*menup).is_null() && *menup != parent {
            menup = &raw mut (**menup).next;
        }
        if (*menup).is_null() {
            break;
        }
        parent = (*parent).parent;
        free_menu(menup);
    }
    return FAIL;
}
unsafe extern "C" fn menu_enable_recurse(
    mut menu: *mut vimmenu_T,
    mut name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
    mut enable: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if menu.is_null() {
        return OK;
    }
    let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
    while !menu.is_null() {
        if *name as ::core::ffi::c_int == NUL
            || *name as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            || menu_name_equal(name, menu) as ::core::ffi::c_int != 0
        {
            if *p as ::core::ffi::c_int != NUL {
                if (*menu).children.is_null() {
                    emsg(gettext(
                        (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    return FAIL;
                }
                if menu_enable_recurse((*menu).children, p, modes, enable) == FAIL {
                    return FAIL;
                }
            } else if enable != 0 {
                (*menu).enabled |= modes;
            } else {
                (*menu).enabled &= !modes;
            }
            if *name as ::core::ffi::c_int != NUL
                && *name as ::core::ffi::c_int != '*' as ::core::ffi::c_int
            {
                break;
            }
        }
        menu = (*menu).next;
    }
    if *name as ::core::ffi::c_int != NUL
        && *name as ::core::ffi::c_int != '*' as ::core::ffi::c_int
        && menu.is_null()
    {
        semsg(
            gettext((e_nomenu.ptr() as *const _) as *const ::core::ffi::c_char),
            name,
        );
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn remove_menu(
    mut menup: *mut *mut vimmenu_T,
    mut name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
    mut silent: bool,
) -> ::core::ffi::c_int {
    let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    if (*menup).is_null() {
        return OK;
    }
    let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
    loop {
        menu = *menup;
        if menu.is_null() {
            break;
        }
        if *name as ::core::ffi::c_int == NUL
            || menu_name_equal(name, menu) as ::core::ffi::c_int != 0
        {
            if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null() {
                if !silent {
                    emsg(gettext(
                        (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                }
                return FAIL;
            }
            if (*menu).modes & modes != 0 as ::core::ffi::c_int {
                if remove_menu(&raw mut (*menu).children, p, modes, silent) == FAIL {
                    return FAIL;
                }
            } else if *name as ::core::ffi::c_int != NUL {
                if !silent {
                    emsg(gettext(
                        &raw const e_menu_only_exists_in_another_mode as *const ::core::ffi::c_char,
                    ));
                }
                return FAIL;
            }
            if *name as ::core::ffi::c_int != NUL {
                break;
            }
            (*menu).modes &= !modes;
            if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
                free_menu_string(menu, MENU_INDEX_TIP as ::core::ffi::c_int);
            }
            if (*menu).modes & MENU_ALL_MODES as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                free_menu(menup);
            } else {
                menup = &raw mut (*menu).next;
            }
        } else {
            menup = &raw mut (*menu).next;
        }
    }
    if *name as ::core::ffi::c_int != NUL {
        if menu.is_null() {
            if !silent {
                semsg(
                    gettext((e_nomenu.ptr() as *const _) as *const ::core::ffi::c_char),
                    name,
                );
            }
            return FAIL;
        }
        (*menu).modes &= !modes;
        let mut child: *mut vimmenu_T = (*menu).children;
        while !child.is_null() {
            (*menu).modes |= (*child).modes;
            child = (*child).next;
        }
        if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
            free_menu_string(menu, MENU_INDEX_TIP as ::core::ffi::c_int);
        }
        if (*menu).modes & MENU_ALL_MODES as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            *menup = menu;
            free_menu(menup);
        }
    }
    return OK;
}
unsafe extern "C" fn free_menu(mut menup: *mut *mut vimmenu_T) {
    let mut menu: *mut vimmenu_T = *menup;
    *menup = (*menu).next;
    xfree((*menu).name as *mut ::core::ffi::c_void);
    xfree((*menu).dname as *mut ::core::ffi::c_void);
    xfree((*menu).en_name as *mut ::core::ffi::c_void);
    xfree((*menu).en_dname as *mut ::core::ffi::c_void);
    xfree((*menu).actext as *mut ::core::ffi::c_void);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < MENU_MODES as ::core::ffi::c_int {
        free_menu_string(menu, i);
        i += 1;
    }
    xfree(menu as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn free_menu_string(mut menu: *mut vimmenu_T, mut idx: ::core::ffi::c_int) {
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < MENU_MODES as ::core::ffi::c_int {
        if (*menu).strings[i as usize] == (*menu).strings[idx as usize] {
            count += 1;
        }
        i += 1;
    }
    if count == 1 as ::core::ffi::c_int {
        xfree((*menu).strings[idx as usize] as *mut ::core::ffi::c_void);
    }
    (*menu).strings[idx as usize] = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn menu_get_recursive(
    mut menu: *const vimmenu_T,
    mut modes: ::core::ffi::c_int,
) -> *mut dict_T {
    if menu.is_null() || (*menu).modes & modes == 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<dict_T>();
    }
    let mut dict: *mut dict_T = tv_dict_alloc();
    tv_dict_add_str(
        dict,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*menu).dname,
    );
    tv_dict_add_nr(
        dict,
        b"priority\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*menu).priority as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"hidden\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        menu_is_hidden((*menu).dname) as varnumber_T,
    );
    if (*menu).mnemonic != 0 {
        let mut buf: [::core::ffi::c_char; 7] = [0 as ::core::ffi::c_char, 0, 0, 0, 0, 0, 0];
        utf_char2bytes((*menu).mnemonic, &raw mut buf as *mut ::core::ffi::c_char);
        tv_dict_add_str(
            dict,
            b"shortcut\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
    }
    if !(*menu).actext.is_null() {
        tv_dict_add_str(
            dict,
            b"actext\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            (*menu).actext,
        );
    }
    if (*menu).modes & MENU_TIP_MODE as ::core::ffi::c_int != 0
        && !(*menu).strings[MENU_INDEX_TIP as ::core::ffi::c_int as usize].is_null()
    {
        tv_dict_add_str(
            dict,
            b"tooltip\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            (*menu).strings[MENU_INDEX_TIP as ::core::ffi::c_int as usize],
        );
    }
    if (*menu).children.is_null() {
        let mut commands: *mut dict_T = tv_dict_alloc();
        tv_dict_add_dict(
            dict,
            b"mappings\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            commands,
        );
        let mut bit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while bit < MENU_MODES as ::core::ffi::c_int {
            if (*menu).modes & modes & (1 as ::core::ffi::c_int) << bit != 0 as ::core::ffi::c_int {
                let mut impl_0: *mut dict_T = tv_dict_alloc();
                tv_dict_add_allocated_str(
                    impl_0,
                    b"rhs\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                    str2special_save((*menu).strings[bit as usize], false_0 != 0, false_0 != 0),
                );
                tv_dict_add_nr(
                    impl_0,
                    b"silent\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                    (*menu).silent[bit as usize] as varnumber_T,
                );
                tv_dict_add_nr(
                    impl_0,
                    b"enabled\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    (if (*menu).enabled & (1 as ::core::ffi::c_int) << bit != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as varnumber_T,
                );
                tv_dict_add_nr(
                    impl_0,
                    b"noremap\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    (if (*menu).noremap[bit as usize] & REMAP_NONE as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as varnumber_T,
                );
                tv_dict_add_nr(
                    impl_0,
                    b"sid\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                    (if (*menu).noremap[bit as usize] & REMAP_SCRIPT as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as varnumber_T,
                );
                tv_dict_add_dict(
                    commands,
                    (*menu_mode_chars.ptr())[bit as usize],
                    1 as size_t,
                    impl_0,
                );
            }
            bit += 1;
        }
    } else {
        let children_list: *mut list_T =
            tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        menu = (*menu).children;
        while !menu.is_null() {
            let mut d: *mut dict_T = menu_get_recursive(menu, modes);
            if tv_dict_len(d) > 0 as ::core::ffi::c_long {
                tv_list_append_dict(children_list, d);
            }
            menu = (*menu).next;
        }
        tv_dict_add_list(
            dict,
            b"submenus\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            children_list,
        );
    }
    return dict;
}
pub unsafe extern "C" fn menu_get(
    path_name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
    mut list: *mut list_T,
) -> bool {
    let mut menu: *mut vimmenu_T = *get_root_menu(path_name);
    if *path_name as ::core::ffi::c_int != NUL {
        menu = find_menu(menu, path_name, modes);
        if menu.is_null() {
            return false_0 != 0;
        }
    }
    while !menu.is_null() {
        let mut d: *mut dict_T = menu_get_recursive(menu, modes);
        if !d.is_null() && tv_dict_len(d) > 0 as ::core::ffi::c_long {
            tv_list_append_dict(list, d);
        }
        if *path_name as ::core::ffi::c_int != NUL {
            break;
        }
        menu = (*menu).next;
    }
    return true_0 != 0;
}
unsafe extern "C" fn find_menu(
    mut menu: *mut vimmenu_T,
    mut path_name: *const ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
) -> *mut vimmenu_T {
    '_c2rust_label: {
        if *path_name != 0 {
        } else {
            __assert_fail(
                b"*path_name\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/menu.rs\0".as_ptr() as *const ::core::ffi::c_char,
                760 as ::core::ffi::c_uint,
                b"vimmenu_T *find_menu(vimmenu_T *, const char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let saved_name: *mut ::core::ffi::c_char = xstrdup(path_name);
    let mut name: *mut ::core::ffi::c_char = saved_name;
    '_theend: while *name != 0 {
        let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
        while !menu.is_null() {
            if menu_name_equal(name, menu) {
                if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null() {
                    emsg(gettext(
                        (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    menu = ::core::ptr::null_mut::<vimmenu_T>();
                    break '_theend;
                } else if (*menu).modes & modes == 0 as ::core::ffi::c_int {
                    emsg(gettext(
                        &raw const e_menu_only_exists_in_another_mode as *const ::core::ffi::c_char,
                    ));
                    menu = ::core::ptr::null_mut::<vimmenu_T>();
                    break '_theend;
                } else if *p as ::core::ffi::c_int == NUL {
                    break '_theend;
                } else {
                    break;
                }
            } else {
                menu = (*menu).next;
            }
        }
        if menu.is_null() {
            semsg(
                gettext((e_nomenu.ptr() as *const _) as *const ::core::ffi::c_char),
                name,
            );
            break;
        } else {
            name = p;
            '_c2rust_label_0: {
                if *name != 0 {
                } else {
                    __assert_fail(
                        b"*name\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/menu.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        792 as ::core::ffi::c_uint,
                        b"vimmenu_T *find_menu(vimmenu_T *, const char *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            menu = (*menu).children;
        }
    }
    xfree(saved_name as *mut ::core::ffi::c_void);
    return menu;
}
unsafe extern "C" fn show_menus(
    path_name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    if *path_name as ::core::ffi::c_int != NUL {
        menu = find_menu(*get_root_menu(path_name), path_name, modes);
        if menu.is_null() {
            return FAIL;
        }
    }
    (*menus_locked.ptr()) += 1;
    msg_puts_title(gettext(
        b"\n--- Menus ---\0".as_ptr() as *const ::core::ffi::c_char
    ));
    show_menus_recursive(menu, modes, 0 as ::core::ffi::c_int);
    (*menus_locked.ptr()) -= 1;
    return OK;
}
unsafe extern "C" fn show_menus_recursive(
    mut menu: *mut vimmenu_T,
    mut modes: ::core::ffi::c_int,
    mut depth: ::core::ffi::c_int,
) {
    if !menu.is_null() && (*menu).modes & modes == 0 as ::core::ffi::c_int {
        return;
    }
    if !menu.is_null() {
        msg_putchar('\n' as ::core::ffi::c_int);
        if got_int.get() {
            return;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < depth {
            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
            i += 1;
        }
        if (*menu).priority != 0 {
            msg_outnum((*menu).priority);
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_outtrans((*menu).name, HLF_D as ::core::ffi::c_int, false_0 != 0);
    }
    if !menu.is_null() && (*menu).children.is_null() {
        let mut bit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while bit < MENU_MODES as ::core::ffi::c_int {
            if (*menu).modes & modes & (1 as ::core::ffi::c_int) << bit != 0 as ::core::ffi::c_int {
                msg_putchar('\n' as ::core::ffi::c_int);
                if got_int.get() {
                    return;
                }
                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_0 < depth + 2 as ::core::ffi::c_int {
                    msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                    i_0 += 1;
                }
                msg_puts((*menu_mode_chars.ptr())[bit as usize]);
                if (*menu).noremap[bit as usize] == REMAP_NONE as ::core::ffi::c_int {
                    msg_putchar('*' as ::core::ffi::c_int);
                } else if (*menu).noremap[bit as usize] == REMAP_SCRIPT as ::core::ffi::c_int {
                    msg_putchar('&' as ::core::ffi::c_int);
                } else {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                if (*menu).silent[bit as usize] {
                    msg_putchar('s' as ::core::ffi::c_int);
                } else {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                if (*menu).modes & (*menu).enabled & (1 as ::core::ffi::c_int) << bit
                    == 0 as ::core::ffi::c_int
                {
                    msg_putchar('-' as ::core::ffi::c_int);
                } else {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
                if *(*menu).strings[bit as usize] as ::core::ffi::c_int == NUL {
                    msg_puts_hl(
                        b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char,
                        HLF_8 as ::core::ffi::c_int,
                        false_0 != 0,
                    );
                } else {
                    msg_outtrans_special(
                        (*menu).strings[bit as usize],
                        false_0 != 0,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            bit += 1;
        }
    } else {
        if menu.is_null() {
            menu = root_menu.get();
            depth -= 1;
        } else {
            menu = (*menu).children;
        }
        while !menu.is_null() && !got_int.get() {
            if !menu_is_hidden((*menu).dname) {
                show_menus_recursive(menu, modes, depth + 1 as ::core::ffi::c_int);
            }
            menu = (*menu).next;
        }
    };
}
static expand_menu: GlobalCell<*mut vimmenu_T> =
    GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
static expand_modes: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static expand_emenu: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub unsafe extern "C" fn set_context_in_menu_cmd(
    mut xp: *mut expand_T,
    mut cmd: *const ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_char,
    mut forceit: bool,
) -> *mut ::core::ffi::c_char {
    let mut after_dot: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut unmenu: bool = false;
    let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    (*xp).xp_context = EXPAND_UNSUCCESSFUL as ::core::ffi::c_int;
    p = arg;
    while *p != 0 {
        if !ascii_isdigit(*p as ::core::ffi::c_int)
            && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        {
            break;
        }
        p = p.offset(1);
    }
    if !ascii_iswhite(*p as ::core::ffi::c_int) {
        if strncmp(
            arg,
            b"enable\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
            && (*arg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || ascii_iswhite(*arg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
        {
            p = arg.offset(6 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"disable\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
            && (*arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || ascii_iswhite(*arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
        {
            p = arg.offset(7 as ::core::ffi::c_int as isize);
        } else {
            p = arg;
        }
    }
    while *p as ::core::ffi::c_int != NUL
        && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        p = p.offset(1);
    }
    after_dot = p;
    arg = after_dot;
    while *p as ::core::ffi::c_int != 0 && !ascii_iswhite(*p as ::core::ffi::c_int) {
        if (*p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == Ctrl_V)
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        } else if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            after_dot = p.offset(1 as ::core::ffi::c_int as isize);
        }
        p = p.offset(1);
    }
    let mut expand_menus: ::core::ffi::c_int = !(*cmd as ::core::ffi::c_int
        == 't' as ::core::ffi::c_int
        && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'e' as ::core::ffi::c_int
        || *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    expand_emenu
        .set((*cmd as ::core::ffi::c_int == 'e' as ::core::ffi::c_int) as ::core::ffi::c_int);
    if expand_menus != 0 && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if *p as ::core::ffi::c_int == NUL {
        expand_modes.set(get_menu_cmd_modes(
            cmd,
            forceit,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut unmenu,
        ));
        if !unmenu {
            expand_modes.set(MENU_ALL_MODES as ::core::ffi::c_int);
        }
        menu = root_menu.get();
        if after_dot > arg {
            let mut path_len: size_t = after_dot.offset_from(arg) as size_t;
            path_name = xmalloc(path_len) as *mut ::core::ffi::c_char;
            xstrlcpy(path_name, arg, path_len);
        }
        let mut name: *mut ::core::ffi::c_char = path_name;
        while !name.is_null() && *name as ::core::ffi::c_int != 0 {
            p = menu_name_skip(name);
            while !menu.is_null() {
                if menu_name_equal(name, menu) {
                    if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null()
                        || (*menu).modes & expand_modes.get() == 0 as ::core::ffi::c_int
                    {
                        xfree(path_name as *mut ::core::ffi::c_void);
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    break;
                } else {
                    menu = (*menu).next;
                }
            }
            if menu.is_null() {
                xfree(path_name as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            name = p;
            menu = (*menu).children;
        }
        xfree(path_name as *mut ::core::ffi::c_void);
        (*xp).xp_context = if expand_menus != 0 {
            EXPAND_MENUNAMES as ::core::ffi::c_int
        } else {
            EXPAND_MENUS as ::core::ffi::c_int
        };
        (*xp).xp_pattern = after_dot;
        expand_menu.set(menu);
    } else {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn get_menu_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static menu: GlobalCell<*mut vimmenu_T> = GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static should_advance: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if idx == 0 as ::core::ffi::c_int {
        menu.set(expand_menu.get());
        should_advance.set(false_0 != 0);
    }
    while !(*menu.ptr()).is_null()
        && (menu_is_hidden((*menu.get()).dname) as ::core::ffi::c_int != 0
            || menu_is_separator((*menu.get()).dname) as ::core::ffi::c_int != 0
            || (*menu.get()).children.is_null())
    {
        menu.set((*menu.get()).next);
    }
    if (*menu.ptr()).is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*menu.get()).modes & expand_modes.get() != 0 {
        if should_advance.get() {
            str = (*menu.get()).en_dname;
        } else {
            str = (*menu.get()).dname;
            if (*menu.get()).en_dname.is_null() {
                should_advance.set(true_0 != 0);
            }
        }
    } else {
        str = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if should_advance.get() {
        menu.set((*menu.get()).next);
    }
    should_advance.set(!should_advance.get());
    return str;
}
pub unsafe extern "C" fn get_menu_names(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static menu: GlobalCell<*mut vimmenu_T> = GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
    static tbuffer: GlobalCell<[::core::ffi::c_char; 256]> = GlobalCell::new([0; 256]);
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static should_advance: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if idx == 0 as ::core::ffi::c_int {
        menu.set(expand_menu.get());
        should_advance.set(false_0 != 0);
    }
    while !(*menu.ptr()).is_null()
        && (menu_is_hidden((*menu.get()).dname) as ::core::ffi::c_int != 0
            || expand_emenu.get() != 0
                && menu_is_separator((*menu.get()).dname) as ::core::ffi::c_int != 0
            || *(*menu.get())
                .dname
                .offset(strlen((*menu.get()).dname).wrapping_sub(1 as size_t) as isize)
                as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int)
    {
        menu.set((*menu.get()).next);
    }
    if (*menu.ptr()).is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*menu.get()).modes & expand_modes.get() != 0 {
        if !(*menu.get()).children.is_null() {
            if should_advance.get() {
                xstrlcpy(
                    tbuffer.ptr() as *mut ::core::ffi::c_char,
                    (*menu.get()).en_dname,
                    TBUFFER_LEN as size_t,
                );
            } else {
                xstrlcpy(
                    tbuffer.ptr() as *mut ::core::ffi::c_char,
                    (*menu.get()).dname,
                    TBUFFER_LEN as size_t,
                );
                if (*menu.get()).en_dname.is_null() {
                    should_advance.set(true_0 != 0);
                }
            }
            strcat(
                tbuffer.ptr() as *mut ::core::ffi::c_char,
                b"\x01\0".as_ptr() as *const ::core::ffi::c_char,
            );
            str = tbuffer.ptr() as *mut ::core::ffi::c_char;
        } else if should_advance.get() {
            str = (*menu.get()).en_dname;
        } else {
            str = (*menu.get()).dname;
            if (*menu.get()).en_dname.is_null() {
                should_advance.set(true_0 != 0);
            }
        }
    } else {
        str = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if should_advance.get() {
        menu.set((*menu.get()).next);
    }
    should_advance.set(!should_advance.get());
    return str;
}
pub const TBUFFER_LEN: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
unsafe extern "C" fn menu_name_skip(name: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    p = name;
    while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == Ctrl_V
        {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if *p != 0 {
        let c2rust_fresh2 = p;
        p = p.offset(1);
        *c2rust_fresh2 = NUL as ::core::ffi::c_char;
    }
    return p;
}
unsafe extern "C" fn menu_name_equal(
    name: *const ::core::ffi::c_char,
    menu: *const vimmenu_T,
) -> bool {
    if !(*menu).en_name.is_null()
        && (menu_namecmp(name, (*menu).en_name) as ::core::ffi::c_int != 0
            || menu_namecmp(name, (*menu).en_dname) as ::core::ffi::c_int != 0)
    {
        return true_0 != 0;
    }
    return menu_namecmp(name, (*menu).name) as ::core::ffi::c_int != 0
        || menu_namecmp(name, (*menu).dname) as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn menu_namecmp(
    name: *const ::core::ffi::c_char,
    mname: *const ::core::ffi::c_char,
) -> bool {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while *name.offset(i as isize) as ::core::ffi::c_int != NUL
        && *name.offset(i as isize) as ::core::ffi::c_int != TAB
    {
        if *name.offset(i as isize) as ::core::ffi::c_int
            != *mname.offset(i as isize) as ::core::ffi::c_int
        {
            break;
        }
        i += 1;
    }
    return (*name.offset(i as isize) as ::core::ffi::c_int == NUL
        || *name.offset(i as isize) as ::core::ffi::c_int == TAB)
        && (*mname.offset(i as isize) as ::core::ffi::c_int == NUL
            || *mname.offset(i as isize) as ::core::ffi::c_int == TAB);
}
pub unsafe extern "C" fn get_menu_cmd_modes(
    mut cmd: *const ::core::ffi::c_char,
    mut forceit: bool,
    mut noremap: *mut ::core::ffi::c_int,
    mut unmenu: *mut bool,
) -> ::core::ffi::c_int {
    let mut modes: ::core::ffi::c_int = 0;
    's_121: {
        let c2rust_fresh3 = cmd;
        cmd = cmd.offset(1);
        match *c2rust_fresh3 as ::core::ffi::c_int {
            118 => {
                modes =
                    MENU_VISUAL_MODE as ::core::ffi::c_int | MENU_SELECT_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            120 => {
                modes = MENU_VISUAL_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            115 => {
                modes = MENU_SELECT_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            111 => {
                modes = MENU_OP_PENDING_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            105 => {
                modes = MENU_INSERT_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            116 => {
                if *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                    modes = MENU_TERMINAL_MODE as ::core::ffi::c_int;
                    cmd = cmd.offset(1);
                    break 's_121;
                } else {
                    modes = MENU_TIP_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
            }
            99 => {
                modes = MENU_CMDLINE_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            97 => {
                modes = MENU_INSERT_MODE as ::core::ffi::c_int
                    | MENU_CMDLINE_MODE as ::core::ffi::c_int
                    | MENU_NORMAL_MODE as ::core::ffi::c_int
                    | MENU_VISUAL_MODE as ::core::ffi::c_int
                    | MENU_SELECT_MODE as ::core::ffi::c_int
                    | MENU_OP_PENDING_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            110 => {
                if *cmd as ::core::ffi::c_int != 'o' as ::core::ffi::c_int {
                    modes = MENU_NORMAL_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
            }
            _ => {}
        }
        cmd = cmd.offset(-1);
        if forceit {
            modes =
                MENU_INSERT_MODE as ::core::ffi::c_int | MENU_CMDLINE_MODE as ::core::ffi::c_int;
        } else {
            modes = MENU_NORMAL_MODE as ::core::ffi::c_int
                | MENU_VISUAL_MODE as ::core::ffi::c_int
                | MENU_SELECT_MODE as ::core::ffi::c_int
                | MENU_OP_PENDING_MODE as ::core::ffi::c_int;
        }
    }
    if !noremap.is_null() {
        *noremap = if *cmd as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
            REMAP_NONE as ::core::ffi::c_int
        } else {
            REMAP_YES as ::core::ffi::c_int
        };
    }
    if !unmenu.is_null() {
        *unmenu = *cmd as ::core::ffi::c_int == 'u' as ::core::ffi::c_int;
    }
    return modes;
}
unsafe extern "C" fn get_menu_mode_str(mut modes: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    if modes
        & (MENU_INSERT_MODE as ::core::ffi::c_int
            | MENU_CMDLINE_MODE as ::core::ffi::c_int
            | MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int)
        == MENU_INSERT_MODE as ::core::ffi::c_int
            | MENU_CMDLINE_MODE as ::core::ffi::c_int
            | MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int
    {
        return b"a\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes
        & (MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int)
        == MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int
    {
        return b" \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & (MENU_INSERT_MODE as ::core::ffi::c_int | MENU_CMDLINE_MODE as ::core::ffi::c_int)
        == MENU_INSERT_MODE as ::core::ffi::c_int | MENU_CMDLINE_MODE as ::core::ffi::c_int
    {
        return b"!\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & (MENU_VISUAL_MODE as ::core::ffi::c_int | MENU_SELECT_MODE as ::core::ffi::c_int)
        == MENU_VISUAL_MODE as ::core::ffi::c_int | MENU_SELECT_MODE as ::core::ffi::c_int
    {
        return b"v\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_VISUAL_MODE as ::core::ffi::c_int != 0 {
        return b"x\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_SELECT_MODE as ::core::ffi::c_int != 0 {
        return b"s\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_OP_PENDING_MODE as ::core::ffi::c_int != 0 {
        return b"o\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_INSERT_MODE as ::core::ffi::c_int != 0 {
        return b"i\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_TERMINAL_MODE as ::core::ffi::c_int != 0 {
        return b"tl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_CMDLINE_MODE as ::core::ffi::c_int != 0 {
        return b"c\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_NORMAL_MODE as ::core::ffi::c_int != 0 {
        return b"n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
        return b"t\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn popup_mode_name(
    mut name: *mut ::core::ffi::c_char,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = strlen(name);
    '_c2rust_label: {
        if len >= 4 as size_t {
        } else {
            __assert_fail(
                b"len >= 4\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/menu.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1296 as ::core::ffi::c_uint,
                b"char *popup_mode_name(char *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut mode_chars: *mut ::core::ffi::c_char = (*menu_mode_chars.ptr())[idx as usize];
    let mut mode_chars_len: size_t = strlen(mode_chars);
    let mut p: *mut ::core::ffi::c_char = xstrnsave(name, len.wrapping_add(mode_chars_len));
    memmove(
        p.offset(5 as ::core::ffi::c_int as isize)
            .offset(mode_chars_len as isize) as *mut ::core::ffi::c_void,
        p.offset(5 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        len.wrapping_sub(4 as size_t),
    );
    let mut i: size_t = 0 as size_t;
    while i < mode_chars_len {
        *p.offset((5 as size_t).wrapping_add(i) as isize) =
            *(*menu_mode_chars.ptr())[idx as usize].offset(i as isize);
        i = i.wrapping_add(1);
    }
    return p;
}
unsafe extern "C" fn menu_text(
    mut str: *const ::core::ffi::c_char,
    mut mnemonic: *mut ::core::ffi::c_int,
    mut actext: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = vim_strchr(str, TAB);
    if !p.is_null() {
        if !actext.is_null() {
            *actext = xstrdup(p.offset(1 as ::core::ffi::c_int as isize));
        }
        '_c2rust_label: {
            if p >= str as *mut ::core::ffi::c_char {
            } else {
                __assert_fail(
                    b"p >= str\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/menu.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1332 as ::core::ffi::c_uint,
                    b"char *menu_text(const char *, int *, char **)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        text = xmemdupz(
            str as *const ::core::ffi::c_void,
            p.offset_from(str) as size_t,
        ) as *mut ::core::ffi::c_char;
    } else {
        text = xstrdup(str);
    }
    p = text;
    while !p.is_null() {
        p = vim_strchr(p, '&' as ::core::ffi::c_int);
        if p.is_null() {
            continue;
        }
        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            break;
        }
        if !mnemonic.is_null()
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '&' as ::core::ffi::c_int
        {
            *mnemonic =
                *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int;
        }
        memmove(
            p as *mut ::core::ffi::c_void,
            p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
        );
        p = p.offset(1 as ::core::ffi::c_int as isize);
    }
    return text;
}
pub unsafe extern "C" fn menu_is_menubar(name: *const ::core::ffi::c_char) -> bool {
    return !menu_is_popup(name)
        && !menu_is_toolbar(name)
        && !menu_is_winbar(name)
        && *name as ::core::ffi::c_int != MNU_HIDDEN_CHAR;
}
pub unsafe extern "C" fn menu_is_popup(name: *const ::core::ffi::c_char) -> bool {
    return strncmp(
        name,
        b"PopUp\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn menu_is_toolbar(name: *const ::core::ffi::c_char) -> bool {
    return strncmp(
        name,
        b"ToolBar\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn menu_is_separator(mut name: *mut ::core::ffi::c_char) -> bool {
    return *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '-' as ::core::ffi::c_int
        && *name.offset(strlen(name).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int;
}
unsafe extern "C" fn menu_is_hidden(mut name: *mut ::core::ffi::c_char) -> bool {
    return *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == MNU_HIDDEN_CHAR
        || menu_is_popup(name) as ::core::ffi::c_int != 0
            && *name.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL;
}
unsafe extern "C" fn get_menu_mode() -> ::core::ffi::c_int {
    if State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0 {
        return MENU_INDEX_TERMINAL as ::core::ffi::c_int;
    }
    if VIsual_active.get() {
        if VIsual_select.get() {
            return MENU_INDEX_SELECT as ::core::ffi::c_int;
        }
        return MENU_INDEX_VISUAL as ::core::ffi::c_int;
    }
    if State.get() & MODE_INSERT as ::core::ffi::c_int != 0 {
        return MENU_INDEX_INSERT as ::core::ffi::c_int;
    }
    if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0
        || State.get() == MODE_ASKMORE as ::core::ffi::c_int
        || State.get() == MODE_HITRETURN as ::core::ffi::c_int
    {
        return MENU_INDEX_CMDLINE as ::core::ffi::c_int;
    }
    if finish_op.get() {
        return MENU_INDEX_OP_PENDING as ::core::ffi::c_int;
    }
    if State.get() & MODE_NORMAL as ::core::ffi::c_int != 0 {
        return MENU_INDEX_NORMAL as ::core::ffi::c_int;
    }
    if State.get() & MODE_LANGMAP as ::core::ffi::c_int != 0 {
        return MENU_INDEX_INSERT as ::core::ffi::c_int;
    }
    return MENU_INDEX_INVALID as ::core::ffi::c_int;
}
pub unsafe extern "C" fn get_menu_mode_flag() -> ::core::ffi::c_int {
    let mut mode: ::core::ffi::c_int = get_menu_mode();
    if mode == MENU_INDEX_INVALID as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return (1 as ::core::ffi::c_int) << mode;
}
pub unsafe extern "C" fn show_popupmenu() {
    let mut menu_mode: ::core::ffi::c_int = get_menu_mode();
    if menu_mode == MENU_INDEX_INVALID as ::core::ffi::c_int {
        return;
    }
    let mut mode: *mut ::core::ffi::c_char = (*menu_mode_chars.ptr())[menu_mode as usize];
    let mut mode_len: size_t = strlen(mode);
    apply_autocmds(
        EVENT_MENUPOPUP,
        mode,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    menu = root_menu.get();
    while !menu.is_null() {
        if strncmp(
            b"PopUp\0".as_ptr() as *const ::core::ffi::c_char,
            (*menu).name,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
            && strncmp(
                (*menu).name.offset(5 as ::core::ffi::c_int as isize),
                mode,
                mode_len,
            ) == 0 as ::core::ffi::c_int
        {
            break;
        }
        menu = (*menu).next;
    }
    if menu.is_null() || (*menu).children.is_null() {
        return;
    }
    pum_show_popupmenu(menu);
}
pub unsafe extern "C" fn execute_menu(
    mut eap: *const exarg_T,
    mut menu: *mut vimmenu_T,
    mut mode_idx: ::core::ffi::c_int,
) {
    let mut idx: ::core::ffi::c_int = mode_idx;
    if idx < 0 as ::core::ffi::c_int {
        if State.get() & MODE_TERMINAL as ::core::ffi::c_int != 0 {
            idx = MENU_INDEX_TERMINAL as ::core::ffi::c_int;
        } else if State.get() & MODE_CMDLINE as ::core::ffi::c_int != 0 {
            idx = MENU_INDEX_CMDLINE as ::core::ffi::c_int;
        } else if get_real_state() & MODE_VISUAL as ::core::ffi::c_int != 0 {
            idx = MENU_INDEX_VISUAL as ::core::ffi::c_int;
        } else if (State.get() & MODE_INSERT as ::core::ffi::c_int != 0 || restart_edit.get() != 0)
            && (*current_sctx.ptr()).sc_sid == 0 as ::core::ffi::c_int
        {
            idx = MENU_INDEX_INSERT as ::core::ffi::c_int;
        } else if !eap.is_null() && (*eap).addr_count != 0 {
            let mut tpos: pos_T = pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            };
            idx = MENU_INDEX_VISUAL as ::core::ffi::c_int;
            if (*curbuf.get()).b_visual.vi_start.lnum == (*eap).line1
                && (*curbuf.get()).b_visual.vi_end.lnum == (*eap).line2
            {
                VIsual_mode.set((*curbuf.get()).b_visual.vi_mode);
                tpos = (*curbuf.get()).b_visual.vi_end;
                (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_start;
                (*curwin.get()).w_curswant = (*curbuf.get()).b_visual.vi_curswant;
            } else {
                VIsual_mode.set('V' as ::core::ffi::c_int);
                (*curwin.get()).w_cursor.lnum = (*eap).line1;
                (*curwin.get()).w_cursor.col = 1 as ::core::ffi::c_int as colnr_T;
                tpos.lnum = (*eap).line2;
                tpos.col = MAXCOL as ::core::ffi::c_int as colnr_T;
                tpos.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
            VIsual_active.set(true_0 != 0);
            VIsual_reselect.set(true_0);
            check_cursor(curwin.get());
            VIsual.set((*curwin.get()).w_cursor);
            (*curwin.get()).w_cursor = tpos;
            check_cursor(curwin.get());
            if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                && gchar_cursor() != NUL
            {
                (*curwin.get()).w_cursor.col += 1;
            }
        }
    }
    if idx == MENU_INDEX_INVALID as ::core::ffi::c_int || eap.is_null() {
        idx = MENU_INDEX_NORMAL as ::core::ffi::c_int;
    }
    if !(*menu).strings[idx as usize].is_null()
        && (*menu).modes & (1 as ::core::ffi::c_int) << idx != 0
    {
        if eap.is_null() || (*current_sctx.ptr()).sc_sid != 0 as ::core::ffi::c_int {
            let mut save_state: save_state_T = save_state_T {
                save_msg_scroll: 0,
                save_restart_edit: 0,
                save_msg_didout: false,
                save_State: 0,
                save_finish_op: false,
                save_opcount: 0,
                save_reg_executing: 0,
                save_pending_end_reg_executing: false,
                tabuf: tasave_T {
                    save_typebuf: typebuf_T {
                        tb_buf: ::core::ptr::null_mut::<uint8_t>(),
                        tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
                        tb_buflen: 0,
                        tb_off: 0,
                        tb_len: 0,
                        tb_maplen: 0,
                        tb_silent: 0,
                        tb_no_abbr_cnt: 0,
                        tb_change_cnt: 0,
                    },
                    typebuf_valid: false,
                    old_char: 0,
                    old_mod_mask: 0,
                    save_readbuf1: buffheader_T {
                        bh_first: buffblock_T {
                            b_next: ::core::ptr::null_mut::<buffblock>(),
                            b_strlen: 0,
                            b_str: [0; 1],
                        },
                        bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                        bh_index: 0,
                        bh_space: 0,
                        bh_create_newblock: false,
                    },
                    save_readbuf2: buffheader_T {
                        bh_first: buffblock_T {
                            b_next: ::core::ptr::null_mut::<buffblock>(),
                            b_strlen: 0,
                            b_str: [0; 1],
                        },
                        bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                        bh_index: 0,
                        bh_space: 0,
                        bh_create_newblock: false,
                    },
                    save_inputbuf: String_0 {
                        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        size: 0,
                    },
                },
            };
            (*ex_normal_busy.ptr()) += 1;
            if save_current_state(&raw mut save_state) {
                exec_normal_cmd(
                    (*menu).strings[idx as usize],
                    (*menu).noremap[idx as usize],
                    (*menu).silent[idx as usize],
                );
            }
            restore_current_state(&raw mut save_state);
            (*ex_normal_busy.ptr()) -= 1;
        } else {
            ins_typebuf(
                (*menu).strings[idx as usize],
                (*menu).noremap[idx as usize],
                0 as ::core::ffi::c_int,
                true_0 != 0,
                (*menu).silent[idx as usize],
            );
        }
    } else if !eap.is_null() {
        let mut mode: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        match idx {
            1 => {
                mode =
                    b"Visual\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            2 => {
                mode =
                    b"Select\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            3 => {
                mode = b"Op-pending\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            6 => {
                mode = b"Terminal\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            4 => {
                mode =
                    b"Insert\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            5 => {
                mode =
                    b"Cmdline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            _ => {
                mode =
                    b"Normal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
        }
        semsg(
            gettext(b"E335: Menu not defined for %s mode\0".as_ptr() as *const ::core::ffi::c_char),
            mode,
        );
    }
}
unsafe extern "C" fn menu_getbyname(mut name_arg: *mut ::core::ffi::c_char) -> *mut vimmenu_T {
    let mut saved_name: *mut ::core::ffi::c_char = xstrdup(name_arg);
    let mut menu: *mut vimmenu_T = *get_root_menu(saved_name);
    let mut name: *mut ::core::ffi::c_char = saved_name;
    let mut gave_emsg: bool = false_0 != 0;
    while *name != 0 {
        let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
        while !menu.is_null() {
            if menu_name_equal(name, menu) {
                if *p as ::core::ffi::c_int == NUL && !(*menu).children.is_null() {
                    emsg(gettext(
                        b"E333: Menu path must lead to a menu item\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    gave_emsg = true_0 != 0;
                    menu = ::core::ptr::null_mut::<vimmenu_T>();
                } else if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null() {
                    emsg(gettext(
                        (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    menu = ::core::ptr::null_mut::<vimmenu_T>();
                }
                break;
            } else {
                menu = (*menu).next;
            }
        }
        if menu.is_null() || *p as ::core::ffi::c_int == NUL {
            break;
        }
        menu = (*menu).children;
        name = p;
    }
    xfree(saved_name as *mut ::core::ffi::c_void);
    if menu.is_null() {
        if !gave_emsg {
            semsg(
                gettext(b"E334: Menu not found: %s\0".as_ptr() as *const ::core::ffi::c_char),
                name_arg,
            );
        }
        return ::core::ptr::null_mut::<vimmenu_T>();
    }
    return menu;
}
pub unsafe extern "C" fn ex_emenu(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut mode_idx: ::core::ffi::c_int = MENU_INDEX_INVALID as ::core::ffi::c_int;
    if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
        && ascii_iswhite(*arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        match *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            110 => {
                mode_idx = MENU_INDEX_NORMAL as ::core::ffi::c_int;
            }
            118 => {
                mode_idx = MENU_INDEX_VISUAL as ::core::ffi::c_int;
            }
            115 => {
                mode_idx = MENU_INDEX_SELECT as ::core::ffi::c_int;
            }
            111 => {
                mode_idx = MENU_INDEX_OP_PENDING as ::core::ffi::c_int;
            }
            116 => {
                mode_idx = MENU_INDEX_TERMINAL as ::core::ffi::c_int;
            }
            105 => {
                mode_idx = MENU_INDEX_INSERT as ::core::ffi::c_int;
            }
            99 => {
                mode_idx = MENU_INDEX_CMDLINE as ::core::ffi::c_int;
            }
            _ => {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    arg,
                );
                return;
            }
        }
        arg = skipwhite(arg.offset(2 as ::core::ffi::c_int as isize));
    }
    let mut menu: *mut vimmenu_T = menu_getbyname(arg);
    if menu.is_null() {
        return;
    }
    execute_menu(eap, menu, mode_idx);
}
pub unsafe extern "C" fn menu_find(mut path_name: *const ::core::ffi::c_char) -> *mut vimmenu_T {
    let mut menu: *mut vimmenu_T = *get_root_menu(path_name);
    let mut saved_name: *mut ::core::ffi::c_char = xstrdup(path_name);
    let mut name: *mut ::core::ffi::c_char = saved_name;
    '_theend: {
        while *name != 0 {
            let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
            while !menu.is_null() {
                if menu_name_equal(name, menu) {
                    if (*menu).children.is_null() {
                        if *p as ::core::ffi::c_int == NUL {
                            emsg(gettext(
                                b"E336: Menu path must lead to a sub-menu\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                        } else {
                            emsg(gettext(
                                (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                            ));
                        }
                        menu = ::core::ptr::null_mut::<vimmenu_T>();
                        break '_theend;
                    } else if *p as ::core::ffi::c_int == NUL {
                        break '_theend;
                    } else {
                        break;
                    }
                } else {
                    menu = (*menu).next;
                }
            }
            if menu.is_null() {
                break;
            }
            menu = (*menu).children;
            name = p;
        }
        if menu.is_null() {
            emsg(gettext(
                b"E337: Menu not found - check menu names\0".as_ptr() as *const ::core::ffi::c_char,
            ));
        }
    }
    xfree(saved_name as *mut ::core::ffi::c_void);
    return menu;
}
static menutrans_ga: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
pub unsafe extern "C" fn ex_menutranslate(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    if (*menutrans_ga.ptr()).ga_itemsize == 0 as ::core::ffi::c_int {
        ga_init(
            menutrans_ga.ptr(),
            ::core::mem::size_of::<menutrans_T>() as ::core::ffi::c_int,
            5 as ::core::ffi::c_int,
        );
    }
    if strncmp(
        arg,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
        && ends_excmd(*skipwhite(arg.offset(5 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int)
            != 0
    {
        let mut _gap: *mut garray_T = menutrans_ga.ptr();
        if !(*_gap).ga_data.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*_gap).ga_len {
                let mut _item: *mut menutrans_T =
                    ((*_gap).ga_data as *mut menutrans_T).offset(i as isize);
                let mut _mt: *mut menutrans_T = _item;
                xfree((*_mt).from as *mut ::core::ffi::c_void);
                xfree((*_mt).from_noamp as *mut ::core::ffi::c_void);
                xfree((*_mt).to as *mut ::core::ffi::c_void);
                i += 1;
            }
        }
        ga_clear(_gap);
        del_menutrans_vars();
    } else {
        let mut from: *mut ::core::ffi::c_char = arg;
        arg = menu_skip_part(arg);
        let mut to: *mut ::core::ffi::c_char = skipwhite(arg);
        *arg = NUL as ::core::ffi::c_char;
        arg = menu_skip_part(to);
        if arg == to {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else {
            from = xstrdup(from);
            let mut from_noamp: *mut ::core::ffi::c_char = menu_text(
                from,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            '_c2rust_label: {
                if arg >= to {
                } else {
                    __assert_fail(
                        b"arg >= to\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/menu.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1754 as ::core::ffi::c_uint,
                        b"void ex_menutranslate(exarg_T *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            to = xmemdupz(
                to as *const ::core::ffi::c_void,
                arg.offset_from(to) as size_t,
            ) as *mut ::core::ffi::c_char;
            menu_translate_tab_and_shift(from);
            menu_translate_tab_and_shift(to);
            menu_unescape_name(from);
            menu_unescape_name(to);
            let mut tp: *mut menutrans_T =
                ga_append_via_ptr(menutrans_ga.ptr(), ::core::mem::size_of::<menutrans_T>())
                    as *mut menutrans_T;
            (*tp).from = from;
            (*tp).from_noamp = from_noamp;
            (*tp).to = to;
        }
    };
}
unsafe extern "C" fn menu_skip_part(mut p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    while *p as ::core::ffi::c_int != NUL
        && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        && !ascii_iswhite(*p as ::core::ffi::c_int)
    {
        if (*p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == Ctrl_V)
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        }
        p = p.offset(1);
    }
    return p;
}
unsafe extern "C" fn menutrans_lookup(
    mut name: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut tp: *mut menutrans_T = (*menutrans_ga.ptr()).ga_data as *mut menutrans_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*menutrans_ga.ptr()).ga_len {
        if strncasecmp(name, (*tp.offset(i as isize)).from, len as size_t)
            == 0 as ::core::ffi::c_int
            && *(*tp.offset(i as isize)).from.offset(len as isize) as ::core::ffi::c_int == NUL
        {
            return (*tp.offset(i as isize)).to;
        }
        i += 1;
    }
    let mut c: ::core::ffi::c_char = *name.offset(len as isize);
    *name.offset(len as isize) = NUL as ::core::ffi::c_char;
    let mut dname: *mut ::core::ffi::c_char = menu_text(
        name,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    *name.offset(len as isize) = c;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*menutrans_ga.ptr()).ga_len {
        if strcasecmp(dname, (*tp.offset(i_0 as isize)).from_noamp) == 0 as ::core::ffi::c_int {
            xfree(dname as *mut ::core::ffi::c_void);
            return (*tp.offset(i_0 as isize)).to;
        }
        i_0 += 1;
    }
    xfree(dname as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn menu_unescape_name(mut name: *mut ::core::ffi::c_char) {
    let mut p: *mut ::core::ffi::c_char = name;
    while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
}
unsafe extern "C" fn menu_translate_tab_and_shift(
    mut arg_start: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut arg: *mut ::core::ffi::c_char = arg_start;
    while *arg as ::core::ffi::c_int != 0 && !ascii_iswhite(*arg as ::core::ffi::c_int) {
        if (*arg as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            || *arg as ::core::ffi::c_int == Ctrl_V)
            && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            arg = arg.offset(1);
        } else if strncasecmp(
            arg,
            b"<TAB>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            5 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            *arg = TAB as ::core::ffi::c_char;
            memmove(
                arg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                arg.offset(5 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(arg.offset(5 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
        }
        arg = arg.offset(1);
    }
    if *arg as ::core::ffi::c_int != NUL {
        let c2rust_fresh4 = arg;
        arg = arg.offset(1);
        *c2rust_fresh4 = NUL as ::core::ffi::c_char;
    }
    arg = skipwhite(arg);
    return arg;
}
unsafe extern "C" fn menuitem_getinfo(
    mut menu_name: *const ::core::ffi::c_char,
    mut menu: *const vimmenu_T,
    mut modes: ::core::ffi::c_int,
    mut dict: *mut dict_T,
) {
    if *menu_name as ::core::ffi::c_int == NUL {
        let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        tv_dict_add_list(
            dict,
            b"submenus\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            l,
        );
        let mut topmenu: *const vimmenu_T = menu;
        while !topmenu.is_null() {
            if !menu_is_hidden((*topmenu).dname) {
                tv_list_append_string(l, (*topmenu).dname, -1 as ssize_t);
            }
            topmenu = (*topmenu).next;
        }
        return;
    }
    tv_dict_add_str(
        dict,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*menu).name,
    );
    tv_dict_add_str(
        dict,
        b"display\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*menu).dname,
    );
    if !(*menu).actext.is_null() {
        tv_dict_add_str(
            dict,
            b"accel\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            (*menu).actext,
        );
    }
    tv_dict_add_nr(
        dict,
        b"priority\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*menu).priority as varnumber_T,
    );
    tv_dict_add_str(
        dict,
        b"modes\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        get_menu_mode_str((*menu).modes),
    );
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    buf[utf_char2bytes((*menu).mnemonic, &raw mut buf as *mut ::core::ffi::c_char) as usize] =
        NUL as ::core::ffi::c_char;
    tv_dict_add_str(
        dict,
        b"shortcut\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    if (*menu).children.is_null() {
        let mut bit: ::core::ffi::c_int = 0;
        bit = 0 as ::core::ffi::c_int;
        while bit < MENU_MODES as ::core::ffi::c_int
            && (1 as ::core::ffi::c_int) << bit & modes == 0
        {
            bit += 1;
        }
        if bit < MENU_MODES as ::core::ffi::c_int {
            if !(*menu).strings[bit as usize].is_null() {
                tv_dict_add_allocated_str(
                    dict,
                    b"rhs\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                    if *(*menu).strings[bit as usize] as ::core::ffi::c_int == NUL {
                        xstrdup(b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char)
                    } else {
                        str2special_save((*menu).strings[bit as usize], false_0 != 0, false_0 != 0)
                    },
                );
            }
            tv_dict_add_bool(
                dict,
                b"noremenu\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                ((*menu).noremap[bit as usize] == REMAP_NONE as ::core::ffi::c_int)
                    as ::core::ffi::c_int as BoolVarValue,
            );
            tv_dict_add_bool(
                dict,
                b"script\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                ((*menu).noremap[bit as usize] == REMAP_SCRIPT as ::core::ffi::c_int)
                    as ::core::ffi::c_int as BoolVarValue,
            );
            tv_dict_add_bool(
                dict,
                b"silent\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                (*menu).silent[bit as usize] as BoolVarValue,
            );
            tv_dict_add_bool(
                dict,
                b"enabled\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                ((*menu).enabled & (1 as ::core::ffi::c_int) << bit != 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int as BoolVarValue,
            );
        }
    } else {
        let l_0: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        tv_dict_add_list(
            dict,
            b"submenus\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            l_0,
        );
        let mut child: *const vimmenu_T = (*menu).children;
        while !child.is_null() {
            tv_list_append_string(l_0, (*child).dname, -1 as ssize_t);
            child = (*child).next;
        }
    };
}
pub unsafe extern "C" fn f_menu_info(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let retdict: *mut dict_T = (*rettv).vval.v_dict;
    let menu_name: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if menu_name.is_null() {
        return;
    }
    let mut which: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        which = tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    } else {
        which = b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if which.is_null() {
        return;
    }
    let modes: ::core::ffi::c_int = get_menu_cmd_modes(
        which,
        *which as ::core::ffi::c_int == '!' as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<bool>(),
    );
    let mut menu: *const vimmenu_T = *get_root_menu(menu_name);
    let saved_name: *mut ::core::ffi::c_char = xstrdup(menu_name);
    if *saved_name as ::core::ffi::c_int != NUL {
        let mut name: *mut ::core::ffi::c_char = saved_name;
        while *name != 0 {
            let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
            while !menu.is_null() {
                if menu_name_equal(name, menu) {
                    break;
                }
                menu = (*menu).next;
            }
            if menu.is_null() || *p as ::core::ffi::c_int == NUL {
                break;
            }
            menu = (*menu).children;
            name = p;
        }
    }
    xfree(saved_name as *mut ::core::ffi::c_void);
    if menu.is_null() {
        return;
    }
    if (*menu).modes & modes != 0 {
        menuitem_getinfo(menu_name, menu, modes, retdict);
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
