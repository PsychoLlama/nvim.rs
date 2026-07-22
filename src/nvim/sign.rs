use crate::src::nvim::api::extmark::{describe_ns, nvim_create_namespace};
use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::buffer::{buflist_findname_exp, buflist_findnr};
use crate::src::nvim::charset::{
    backslash_halve, getdigits_int, skiptowhite, skiptowhite_esc, skipwhite, vim_isprintc,
};
use crate::src::nvim::cursor::check_cursor_lnum;
use crate::src::nvim::decoration::{decor_find_sign, decor_put_sh, sign_item_cmp};
use crate::src::nvim::drawscreen::redraw_buf_later;
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::funcs::get_buf_arg;
use crate::src::nvim::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_check_for_opt_dict_arg, tv_check_for_string_arg,
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find,
    tv_dict_get_number, tv_dict_get_number_def, tv_dict_get_string, tv_get_lnum, tv_get_number_chk,
    tv_get_string, tv_get_string_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict,
    tv_list_append_number,
};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::extmark::{extmark_del, extmark_del_id, extmark_set};
use crate::src::nvim::fold::foldOpenCursor;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::schar_get;
use crate::src::nvim::highlight_group::{get_highlight_name_ext, syn_check_group};
use crate::src::nvim::main::{
    curtab, curwin, e_argreq, e_dictreq, e_invalid_buffer_name_str, e_invarg, e_invarg2, e_listreq,
    e_trailing_arg, firstbuf, firstwin, got_int, namespace_ids,
};
use crate::src::nvim::map::{
    map_del_cstr_t_ptr_t, map_put_ref_cstr_t_ptr_t, mh_get_String, mh_get_cstr_t,
};
use crate::src::nvim::marktree::{
    marktree_itr_current, marktree_itr_get, marktree_itr_get_overlap, marktree_itr_next,
    marktree_itr_step_overlap, marktree_lookup_ns,
};
use crate::src::nvim::mbyte::{utf_ptr2cells, utfc_ptr2len, utfc_ptr2schar};
use crate::src::nvim::memory::{xcalloc, xfree, xmallocz, xrealloc, xstrdup};
use crate::src::nvim::message::{
    emsg, msg_outtrans, msg_putchar, msg_puts, msg_puts_hl, msg_puts_title, semsg, smsg,
};
use crate::src::nvim::os::libc::{
    __assert_fail, atoi, gettext, memcpy, memmove, qsort, snprintf, strcmp, strlen, strncmp,
};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CMD_index, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInline, DecorInlineData, DecorPriority, DecorSignHighlight,
    DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Direction, Error, ErrorType,
    EvalFuncData, ExtmarkMove, ExtmarkSavePos, ExtmarkSplice, ExtmarkUndoObject, FileID,
    FloatAnchor, FloatRelative, GridView, Integer, Intersection, LineGetter, ListLenSpecials,
    LuaRef, MTKey, MTNode, MTPair, MTPos, MapHash, Map_String_int, Map_cstr_t_ptr_t,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_16, MetaIndex, MsgpackRpcRequestHandler, OptInt,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_String, Set_cstr_t, Set_int64_t, Set_uint32_t,
    Set_uint64_t, SignItem, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_13, String_0, Terminal, Timestamp, UndoObjectType,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, __compar_fn_t, __time_t, alist_T, bcount_t, bhdr_T, blob_T,
    blobvar_S, blocknr_T, buf_T, bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_19, cstr_t, dict_T, dictitem_T, dictvar_S, diff_T,
    diffblock_S, disptick_T, eslist_T, eslist_elem, exarg, exarg_T, expand_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_12, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, sign_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T,
    synblock_T, synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T,
    typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_9, u_header_uh_alt_prev as C2Rust_Unnamed_8,
    u_header_uh_next as C2Rust_Unnamed_11, u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, undo_object_data as C2Rust_Unnamed_7,
    varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T,
    xp_prefix_T, NS, QUEUE,
};
use crate::src::nvim::window::buf_jump_open_win;
extern "C" {
    static decor_items: GlobalCell<C2Rust_Unnamed_20>;
}
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
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
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
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kSHConcealLines: C2Rust_Unnamed_14 = 128;
pub const kSHConceal: C2Rust_Unnamed_14 = 64;
pub const kSHSpellOff: C2Rust_Unnamed_14 = 32;
pub const kSHSpellOn: C2Rust_Unnamed_14 = 16;
pub const kSHUIWatchedOverlay: C2Rust_Unnamed_14 = 8;
pub const kSHUIWatched: C2Rust_Unnamed_14 = 4;
pub const kSHHlEol: C2Rust_Unnamed_14 = 2;
pub const kSHIsSign: C2Rust_Unnamed_14 = 1;
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
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_17 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_17 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_17 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_17 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_17 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_17 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_17 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_17 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_17 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_17 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_17 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_17 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_17 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_17 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_17 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_17 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_17 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_17 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_17 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_17 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_17 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_17 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_17 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_17 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_17 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_17 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_17 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_17 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_17 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_17 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_17 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_17 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_17 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_17 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_17 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_17 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_17 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_17 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_17 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_17 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_17 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_17 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_17 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_17 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_17 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_17 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_17 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_17 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_17 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_17 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_17 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_17 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_17 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_17 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_17 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_17 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_17 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_17 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_17 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_17 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_17 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_17 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_17 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_17 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_17 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_17 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_17 = -2;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const SIGN_DEF_PRIO: C2Rust_Unnamed_18 = 10;
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
pub struct C2Rust_Unnamed_20 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorSignHighlight,
}
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_21 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_21 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_21 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_21 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_21 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_21 = 20;
pub const UPD_VALID: C2Rust_Unnamed_21 = 10;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_22 = 4;
pub const BL_SOL: C2Rust_Unnamed_22 = 2;
pub const BL_WHITE: C2Rust_Unnamed_22 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_23 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTKey,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_24 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_25 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTKey,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut cstr_t,
}
pub const EXP_SIGN_GROUPS: C2Rust_Unnamed_27 = 6;
pub const EXP_SIGN_NAMES: C2Rust_Unnamed_27 = 5;
pub const EXP_UNPLACE: C2Rust_Unnamed_27 = 4;
pub const EXP_LIST: C2Rust_Unnamed_27 = 3;
pub const EXP_PLACE: C2Rust_Unnamed_27 = 2;
pub const EXP_DEFINE: C2Rust_Unnamed_27 = 1;
pub const EXP_SUBCMD: C2Rust_Unnamed_27 = 0;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_28 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTKey,
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"int sign_row_cmp(const void *, const void *)\0",
    )
};
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_cstr_t = Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
};
pub const MAP_INIT: Map_cstr_t_ptr_t = Map_cstr_t_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_cstr_t(mut set: *mut Set_cstr_t, mut key: cstr_t) -> bool {
    return mh_get_cstr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn map_get_cstr_t_ptr_t(
    mut map: *mut Map_cstr_t_ptr_t,
    mut key: cstr_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_cstr_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_get_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_String(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_int.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_SIGN_HIGHLIGHT_INIT: DecorSignHighlight = DecorSignHighlight {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    text: [0 as schar_T, 0 as schar_T],
    sign_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    sign_add_id: 0 as ::core::ffi::c_int,
    number_hl_id: 0 as ::core::ffi::c_int,
    line_hl_id: 0 as ::core::ffi::c_int,
    cursorline_hl_id: 0 as ::core::ffi::c_int,
    next: DECOR_ID_INVALID as uint32_t,
    url: ::core::ptr::null::<::core::ffi::c_char>(),
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn buf_meta_total(mut b: *const buf_T, mut m: MetaIndex) -> uint32_t {
    return (*(&raw const (*b).b_marktree as *const MarkTree)).meta_root[m as usize];
}
pub const MSG_BUF_LEN: ::core::ffi::c_int = 480 as ::core::ffi::c_int;
pub const MT_FLAG_END: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_EXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNTEXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 9 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNHL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 10 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mt_end(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_END != 0;
}
#[inline]
unsafe extern "C" fn mt_decor_sign(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & (MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL) != 0;
}
#[inline]
unsafe extern "C" fn mt_decor(mut key: MTKey) -> DecorInline {
    return DecorInline {
        ext: key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0,
        data: key.decor_data,
    };
}
static sign_map: GlobalCell<Map_cstr_t_ptr_t> = GlobalCell::new(MAP_INIT);
static sign_ns: GlobalCell<C2Rust_Unnamed_24> = GlobalCell::new(C2Rust_Unnamed_24 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Integer>(),
});
static cmds: GlobalCell<[*mut ::core::ffi::c_char; 7]> = GlobalCell::new([
    b"define\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"undefine\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"list\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"place\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"unplace\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"jump\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
]);
pub const SIGNCMD_DEFINE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SIGNCMD_UNDEFINE: ::core::ffi::c_int = 1;
pub const SIGNCMD_LIST: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SIGNCMD_PLACE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SIGNCMD_UNPLACE: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const SIGNCMD_JUMP: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const SIGNCMD_LAST: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
unsafe extern "C" fn group_get_ns(mut group: *const ::core::ffi::c_char) -> int64_t {
    if group.is_null() {
        return 0 as int64_t;
    } else if strcmp(group, b"*\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        return UINT32_MAX as int64_t;
    }
    let mut ns: ::core::ffi::c_int = map_get_String_int(namespace_ids.ptr(), cstr_as_string(group));
    return (if ns != 0 {
        ns
    } else {
        -1 as ::core::ffi::c_int
    }) as int64_t;
}
unsafe extern "C" fn sign_get_name(mut sh: *mut DecorSignHighlight) -> *const ::core::ffi::c_char {
    let mut name: *mut ::core::ffi::c_char = (*sh).sign_name;
    return if name.is_null() {
        b"\0".as_ptr() as *const ::core::ffi::c_char
    } else if set_has_cstr_t(&raw mut (*sign_map.ptr()).set, name as cstr_t) as ::core::ffi::c_int
        != 0
    {
        name as *const ::core::ffi::c_char
    } else {
        b"[Deleted]\0".as_ptr() as *const ::core::ffi::c_char
    };
}
unsafe extern "C" fn buf_set_sign(
    mut buf: *mut buf_T,
    mut id: *mut uint32_t,
    mut group: *mut ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
    mut lnum: linenr_T,
    mut sp: *mut sign_T,
) {
    if !group.is_null() && map_get_String_int(namespace_ids.ptr(), cstr_as_string(group)) == 0 {
        if (*sign_ns.ptr()).size == (*sign_ns.ptr()).capacity {
            (*sign_ns.ptr()).capacity = if (*sign_ns.ptr()).capacity != 0 {
                (*sign_ns.ptr()).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*sign_ns.ptr()).items = xrealloc(
                (*sign_ns.ptr()).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Integer>().wrapping_mul((*sign_ns.ptr()).capacity),
            ) as *mut Integer;
        } else {
        };
        let c2rust_fresh3 = (*sign_ns.ptr()).size;
        (*sign_ns.ptr()).size = (*sign_ns.ptr()).size.wrapping_add(1);
        *(*sign_ns.ptr()).items.offset(c2rust_fresh3 as isize) =
            nvim_create_namespace(cstr_as_string(group));
    }
    let mut ns: uint32_t = if !group.is_null() {
        nvim_create_namespace(cstr_as_string(group)) as uint32_t
    } else {
        0 as uint32_t
    };
    let mut sign: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    sign.flags = (sign.flags as ::core::ffi::c_int | kSHIsSign as ::core::ffi::c_int) as uint16_t;
    memcpy(
        &raw mut sign.text as *mut schar_T as *mut ::core::ffi::c_void,
        &raw mut (*sp).sn_text as *mut schar_T as *const ::core::ffi::c_void,
        (SIGN_WIDTH as ::core::ffi::c_int as size_t)
            .wrapping_mul(::core::mem::size_of::<schar_T>()),
    );
    sign.sign_name = xstrdup((*sp).sn_name);
    sign.hl_id = (*sp).sn_text_hl;
    sign.line_hl_id = (*sp).sn_line_hl;
    sign.number_hl_id = (*sp).sn_num_hl;
    sign.cursorline_hl_id = (*sp).sn_cul_hl;
    sign.priority = prio as DecorPriority;
    let mut has_hl: bool = (*sp).sn_line_hl != 0 || (*sp).sn_num_hl != 0 || (*sp).sn_cul_hl != 0;
    let mut decor_flags: uint16_t = ((if (*sp).sn_text[0 as ::core::ffi::c_int as usize] != 0 {
        MT_FLAG_DECOR_SIGNTEXT
    } else {
        0 as ::core::ffi::c_int
    }) | (if has_hl as ::core::ffi::c_int != 0 {
        MT_FLAG_DECOR_SIGNHL
    } else {
        0 as ::core::ffi::c_int
    })) as uint16_t;
    let mut decor: DecorInline = DecorInline {
        ext: true_0 != 0,
        data: DecorInlineData {
            ext: DecorExt {
                sh_idx: decor_put_sh(sign),
                vt: ::core::ptr::null_mut::<DecorVirtText>(),
            },
        },
    };
    extmark_set(
        buf,
        ns,
        id,
        (if (*buf).b_ml.ml_line_count < lnum {
            (*buf).b_ml.ml_line_count as ::core::ffi::c_int
        } else {
            lnum as ::core::ffi::c_int
        }) - 1 as ::core::ffi::c_int,
        0 as colnr_T,
        -1 as ::core::ffi::c_int,
        -1 as colnr_T,
        decor,
        decor_flags,
        true_0 != 0,
        false_0 != 0,
        true_0 != 0,
        true_0 != 0,
        ::core::ptr::null_mut::<Error>(),
    );
}
unsafe extern "C" fn buf_mod_sign(
    mut buf: *mut buf_T,
    mut id: *mut uint32_t,
    mut group: *mut ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
    mut sp: *mut sign_T,
) -> linenr_T {
    let mut ns: int64_t = group_get_ns(group);
    if ns < 0 as int64_t || !group.is_null() && ns == 0 as int64_t {
        return 0 as linenr_T;
    }
    let mut mark: MTKey = marktree_lookup_ns(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        ns as uint32_t,
        *id,
        false_0 != 0,
        ::core::ptr::null_mut::<MarkTreeIter>(),
    );
    if mark.pos.row >= 0 as int32_t {
        buf_set_sign(
            buf,
            id,
            group,
            prio,
            mark.pos.row as linenr_T + 1 as linenr_T,
            sp,
        );
    }
    return mark.pos.row as linenr_T + 1 as linenr_T;
}
unsafe extern "C" fn buf_findsign(
    mut buf: *mut buf_T,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ns: int64_t = group_get_ns(group);
    if ns < 0 as int64_t || !group.is_null() && ns == 0 as int64_t {
        return 0 as ::core::ffi::c_int;
    }
    return marktree_lookup_ns(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        ns as uint32_t,
        id as uint32_t,
        false_0 != 0,
        ::core::ptr::null_mut::<MarkTreeIter>(),
    )
    .pos
    .row as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn sign_row_cmp(
    mut p1: *const ::core::ffi::c_void,
    mut p2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut s1: *const MTKey = p1 as *mut MTKey;
    let mut s2: *const MTKey = p2 as *mut MTKey;
    if (*s1).pos.row != (*s2).pos.row {
        return if (*s1).pos.row > (*s2).pos.row {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    let mut sh1: *mut DecorSignHighlight = decor_find_sign(mt_decor(*s1));
    let mut sh2: *mut DecorSignHighlight = decor_find_sign(mt_decor(*s2));
    '_c2rust_label: {
        if !sh1.is_null() && !sh2.is_null() {
        } else {
            __assert_fail(
                b"sh1 && sh2\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/sign.rs\0".as_ptr() as *const ::core::ffi::c_char,
                178 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    let mut si1: SignItem = SignItem {
        sh: sh1,
        id: (*s1).id,
    };
    let mut si2: SignItem = SignItem {
        sh: sh2,
        id: (*s2).id,
    };
    return sign_item_cmp(
        &raw mut si1 as *const ::core::ffi::c_void,
        &raw mut si2 as *const ::core::ffi::c_void,
    );
}
unsafe extern "C" fn buf_delete_signs(
    mut buf: *mut buf_T,
    mut group: *mut ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut atlnum: linenr_T,
) -> ::core::ffi::c_int {
    let mut ns: int64_t = group_get_ns(group);
    if ns < 0 as int64_t {
        return FAIL;
    }
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    let mut row: ::core::ffi::c_int = if atlnum > 0 as linenr_T {
        atlnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut signs: C2Rust_Unnamed_23 = C2Rust_Unnamed_23 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<MTKey>(),
    };
    if atlnum > 0 as linenr_T {
        if !marktree_itr_get_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            row,
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        ) {
            return FAIL;
        }
        let mut pair: MTPair = MTPair {
            start: MTKey {
                pos: MTPos { row: 0, col: 0 },
                ns: 0,
                id: 0,
                flags: 0,
                decor_data: DecorInlineData {
                    hl: DecorHighlightInline {
                        flags: 0,
                        priority: 0,
                        hl_id: 0,
                        conceal_char: 0,
                    },
                },
            },
            end_pos: MTPos { row: 0, col: 0 },
            end_right_gravity: false,
        };
        while marktree_itr_step_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            &raw mut pair,
        ) {
            if (ns == UINT32_MAX as int64_t || ns == pair.start.ns as int64_t)
                && mt_decor_sign(pair.start) as ::core::ffi::c_int != 0
            {
                if signs.size == signs.capacity {
                    signs.capacity = if signs.capacity != 0 {
                        signs.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    signs.items = xrealloc(
                        signs.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<MTKey>().wrapping_mul(signs.capacity),
                    ) as *mut MTKey;
                } else {
                };
                let c2rust_fresh1 = signs.size;
                signs.size = signs.size.wrapping_add(1);
                *signs.items.offset(c2rust_fresh1 as isize) = pair.start;
            }
        }
    } else {
        marktree_itr_get(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            0 as int32_t,
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if row != 0 && mark.pos.row > row as int32_t {
            break;
        }
        if !mt_end(mark)
            && mt_decor_sign(mark) as ::core::ffi::c_int != 0
            && (id == 0 as ::core::ffi::c_int || mark.id as ::core::ffi::c_int == id)
            && (ns == UINT32_MAX as int64_t || ns == mark.ns as int64_t)
        {
            if atlnum > 0 as linenr_T {
                if signs.size == signs.capacity {
                    signs.capacity = if signs.capacity != 0 {
                        signs.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    signs.items = xrealloc(
                        signs.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<MTKey>().wrapping_mul(signs.capacity),
                    ) as *mut MTKey;
                } else {
                };
                let c2rust_fresh2 = signs.size;
                signs.size = signs.size.wrapping_add(1);
                *signs.items.offset(c2rust_fresh2 as isize) = mark;
                marktree_itr_next(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    &raw mut itr as *mut MarkTreeIter,
                );
            } else {
                extmark_del(buf, &raw mut itr as *mut MarkTreeIter, mark, true_0 != 0);
            }
        } else {
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
            );
        }
    }
    if signs.size != 0 {
        qsort(
            signs.items.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            signs.size,
            ::core::mem::size_of::<MTKey>(),
            Some(
                sign_row_cmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        extmark_del_id(
            buf,
            (*signs.items.offset(0 as ::core::ffi::c_int as isize)).ns,
            (*signs.items.offset(0 as ::core::ffi::c_int as isize)).id,
        );
        xfree(signs.items as *mut ::core::ffi::c_void);
        signs.capacity = 0 as size_t;
        signs.size = signs.capacity;
        signs.items = ::core::ptr::null_mut::<MTKey>();
    } else if atlnum > 0 as linenr_T {
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn buf_has_signs(mut buf: *const buf_T) -> bool {
    return buf_meta_total(buf, kMTMetaSignHL).wrapping_add(buf_meta_total(buf, kMTMetaSignText))
        != 0;
}
unsafe extern "C" fn sign_list_placed(mut rbuf: *mut buf_T, mut group: *mut ::core::ffi::c_char) {
    let mut lbuf: [::core::ffi::c_char; 480] = [0; 480];
    let mut namebuf: [::core::ffi::c_char; 480] = [0; 480];
    let mut groupbuf: [::core::ffi::c_char; 480] = [0; 480];
    let mut buf: *mut buf_T = if !rbuf.is_null() {
        rbuf
    } else {
        firstbuf.get()
    };
    let mut ns: int64_t = group_get_ns(group);
    msg_puts_title(gettext(
        b"\n--- Signs ---\0".as_ptr() as *const ::core::ffi::c_char
    ));
    while !buf.is_null() && !got_int.get() {
        if buf_has_signs(buf) {
            msg_putchar('\n' as ::core::ffi::c_int);
            vim_snprintf(
                &raw mut lbuf as *mut ::core::ffi::c_char,
                MSG_BUF_LEN as size_t,
                gettext(b"Signs for %s:\0".as_ptr() as *const ::core::ffi::c_char),
                (*buf).b_fname,
            );
            msg_puts_hl(
                &raw mut lbuf as *mut ::core::ffi::c_char,
                HLF_D as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
        if ns >= 0 as int64_t {
            let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
                pos: MTPos { row: 0, col: 0 },
                lvl: 0,
                x: ::core::ptr::null_mut::<MTNode>(),
                i: 0,
                s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
                intersect_idx: 0,
                intersect_pos: MTPos { row: 0, col: 0 },
                intersect_pos_x: MTPos { row: 0, col: 0 },
            }; 1];
            let mut signs: C2Rust_Unnamed_25 = C2Rust_Unnamed_25 {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<MTKey>(),
            };
            marktree_itr_get(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                0 as int32_t,
                0 as ::core::ffi::c_int,
                &raw mut itr as *mut MarkTreeIter,
            );
            while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
                let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
                if !mt_end(mark)
                    && mt_decor_sign(mark) as ::core::ffi::c_int != 0
                    && (ns == UINT32_MAX as int64_t || ns == mark.ns as int64_t)
                {
                    if signs.size == signs.capacity {
                        signs.capacity = if signs.capacity != 0 {
                            signs.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        signs.items = xrealloc(
                            signs.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<MTKey>().wrapping_mul(signs.capacity),
                        ) as *mut MTKey;
                    } else {
                    };
                    let c2rust_fresh4 = signs.size;
                    signs.size = signs.size.wrapping_add(1);
                    *signs.items.offset(c2rust_fresh4 as isize) = mark;
                }
                marktree_itr_next(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    &raw mut itr as *mut MarkTreeIter,
                );
            }
            if signs.size != 0 {
                qsort(
                    signs.items.offset(0 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    signs.size,
                    ::core::mem::size_of::<MTKey>(),
                    Some(
                        sign_row_cmp
                            as unsafe extern "C" fn(
                                *const ::core::ffi::c_void,
                                *const ::core::ffi::c_void,
                            )
                                -> ::core::ffi::c_int,
                    ),
                );
                msg_putchar('\n' as ::core::ffi::c_int);
                let mut i: size_t = 0 as size_t;
                while i < signs.size {
                    namebuf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                    groupbuf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                    let mut mark_0: MTKey = *signs.items.offset(i as isize);
                    let mut sh: *mut DecorSignHighlight = decor_find_sign(mt_decor(mark_0));
                    if !(*sh).sign_name.is_null() {
                        vim_snprintf(
                            &raw mut namebuf as *mut ::core::ffi::c_char,
                            MSG_BUF_LEN as size_t,
                            gettext(b"  name=%s\0".as_ptr() as *const ::core::ffi::c_char),
                            sign_get_name(sh),
                        );
                    }
                    if mark_0.ns != 0 as uint32_t {
                        vim_snprintf(
                            &raw mut groupbuf as *mut ::core::ffi::c_char,
                            MSG_BUF_LEN as size_t,
                            gettext(b"  group=%s\0".as_ptr() as *const ::core::ffi::c_char),
                            describe_ns(
                                mark_0.ns as NS,
                                b"\0".as_ptr() as *const ::core::ffi::c_char,
                            ),
                        );
                    }
                    vim_snprintf(
                        &raw mut lbuf as *mut ::core::ffi::c_char,
                        MSG_BUF_LEN as size_t,
                        gettext(b"    line=%d  id=%u%s%s  priority=%d\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        mark_0.pos.row + 1 as int32_t,
                        mark_0.id,
                        &raw mut groupbuf as *mut ::core::ffi::c_char,
                        &raw mut namebuf as *mut ::core::ffi::c_char,
                        (*sh).priority as ::core::ffi::c_int,
                    );
                    msg_puts(&raw mut lbuf as *mut ::core::ffi::c_char);
                    if i < signs.size.wrapping_sub(1 as size_t) {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    }
                    i = i.wrapping_add(1);
                }
                xfree(signs.items as *mut ::core::ffi::c_void);
                signs.capacity = 0 as size_t;
                signs.size = signs.capacity;
                signs.items = ::core::ptr::null_mut::<MTKey>();
            }
        }
        if !rbuf.is_null() {
            return;
        }
        buf = (*buf).b_next;
    }
}
unsafe extern "C" fn sign_cmd_idx(
    mut begin_cmd: *mut ::core::ffi::c_char,
    mut end_cmd: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut idx: ::core::ffi::c_int = 0;
    let mut save: ::core::ffi::c_char = *end_cmd;
    *end_cmd = NUL as ::core::ffi::c_char;
    idx = 0 as ::core::ffi::c_int;
    while !((*cmds.ptr())[idx as usize].is_null()
        || strcmp(begin_cmd, (*cmds.ptr())[idx as usize]) == 0 as ::core::ffi::c_int)
    {
        idx += 1;
    }
    *end_cmd = save;
    return idx;
}
#[no_mangle]
pub unsafe extern "C" fn describe_sign_text(
    mut buf: *mut ::core::ffi::c_char,
    mut sign_text: *mut schar_T,
) -> size_t {
    let mut p: size_t = 0 as size_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < SIGN_WIDTH as ::core::ffi::c_int {
        schar_get(buf.offset(p as isize), *sign_text.offset(i as isize));
        let mut len: size_t = strlen(buf.offset(p as isize));
        if len == 0 as size_t {
            break;
        }
        p = p.wrapping_add(len);
        i += 1;
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn init_sign_text(
    mut sp: *mut sign_T,
    mut sign_text: *mut schar_T,
    mut text: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut endp: *mut ::core::ffi::c_char =
        text.offset(strlen(text) as ::core::ffi::c_int as isize);
    s = if !sp.is_null() { text } else { endp };
    while s.offset(1 as ::core::ffi::c_int as isize) < endp {
        if *s as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            memmove(
                s as *mut ::core::ffi::c_void,
                s.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(s.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
            endp = endp.offset(-1);
        }
        s = s.offset(1);
    }
    let mut cells: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    s = text;
    while s < endp {
        let mut c: ::core::ffi::c_int = 0;
        *sign_text.offset(cells as isize) = utfc_ptr2schar(s, &raw mut c);
        if !vim_isprintc(c) {
            break;
        }
        let mut width: ::core::ffi::c_int = utf_ptr2cells(s);
        if width == 2 as ::core::ffi::c_int {
            *sign_text.offset((cells + 1 as ::core::ffi::c_int) as isize) = 0 as schar_T;
        }
        cells += width;
        s = s.offset(utfc_ptr2len(s) as isize);
    }
    if s != endp || cells > SIGN_WIDTH as ::core::ffi::c_int {
        if !sp.is_null() {
            semsg(
                gettext(b"E239: Invalid sign text: %s\0".as_ptr() as *const ::core::ffi::c_char),
                text,
            );
        }
        return FAIL;
    }
    if cells < 1 as ::core::ffi::c_int {
        *sign_text.offset(0 as ::core::ffi::c_int as isize) = 0 as schar_T;
    } else if cells == 1 as ::core::ffi::c_int {
        *sign_text.offset(1 as ::core::ffi::c_int as isize) = ' ' as ::core::ffi::c_int as schar_T;
    }
    return OK;
}
unsafe extern "C" fn sign_define_by_name(
    mut name: *mut ::core::ffi::c_char,
    mut icon: *mut ::core::ffi::c_char,
    mut text: *mut ::core::ffi::c_char,
    mut linehl: *mut ::core::ffi::c_char,
    mut texthl: *mut ::core::ffi::c_char,
    mut culhl: *mut ::core::ffi::c_char,
    mut numhl: *mut ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut key: *mut cstr_t = ::core::ptr::null_mut::<cstr_t>();
    let mut new_sign: bool = false_0 != 0;
    let mut sp: *mut *mut sign_T = map_put_ref_cstr_t_ptr_t(
        sign_map.ptr(),
        name as cstr_t,
        &raw mut key,
        &raw mut new_sign,
    ) as *mut *mut sign_T;
    if new_sign {
        *key = xstrdup(name) as cstr_t;
        *sp = xcalloc(1 as size_t, ::core::mem::size_of::<sign_T>()) as *mut sign_T;
        (**sp).sn_name = *key as *mut ::core::ffi::c_char;
    }
    if !icon.is_null() {
        xfree((**sp).sn_icon as *mut ::core::ffi::c_void);
        (**sp).sn_icon = xstrdup(icon);
        backslash_halve((**sp).sn_icon);
    }
    if !text.is_null() && init_sign_text(*sp, &raw mut (**sp).sn_text as *mut schar_T, text) == FAIL
    {
        return FAIL;
    }
    (**sp).sn_priority = prio;
    let mut arg: [*mut ::core::ffi::c_char; 4] = [linehl, texthl, culhl, numhl];
    let mut hl: [*mut ::core::ffi::c_int; 4] = [
        &raw mut (**sp).sn_line_hl,
        &raw mut (**sp).sn_text_hl,
        &raw mut (**sp).sn_cul_hl,
        &raw mut (**sp).sn_num_hl,
    ];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        if !arg[i as usize].is_null() {
            *hl[i as usize] = if *arg[i as usize] as ::core::ffi::c_int != 0 {
                syn_check_group(arg[i as usize], strlen(arg[i as usize]))
            } else {
                0 as ::core::ffi::c_int
            };
        }
        i += 1;
    }
    if !new_sign {
        let mut did_redraw: bool = false_0 != 0;
        let mut i_0: size_t = 0 as size_t;
        while i_0 < (*decor_items.ptr()).size {
            let mut sh: *mut DecorSignHighlight = (*decor_items.ptr()).items.offset(i_0 as isize);
            if !(*sh).sign_name.is_null()
                && strcmp((*sh).sign_name, name) == 0 as ::core::ffi::c_int
            {
                memcpy(
                    &raw mut (*sh).text as *mut schar_T as *mut ::core::ffi::c_void,
                    &raw mut (**sp).sn_text as *mut schar_T as *const ::core::ffi::c_void,
                    (SIGN_WIDTH as ::core::ffi::c_int as size_t)
                        .wrapping_mul(::core::mem::size_of::<schar_T>()),
                );
                (*sh).hl_id = (**sp).sn_text_hl;
                (*sh).line_hl_id = (**sp).sn_line_hl;
                (*sh).number_hl_id = (**sp).sn_num_hl;
                (*sh).cursorline_hl_id = (**sp).sn_cul_hl;
                if !did_redraw {
                    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                        firstwin.get()
                    } else {
                        (*curtab.get()).tp_firstwin
                    };
                    while !wp.is_null() {
                        if buf_has_signs((*wp).w_buffer) {
                            redraw_buf_later((*wp).w_buffer, UPD_NOT_VALID as ::core::ffi::c_int);
                        }
                        wp = (*wp).w_next;
                    }
                    did_redraw = true_0 != 0;
                }
            }
            i_0 = i_0.wrapping_add(1);
        }
    }
    return OK;
}
unsafe extern "C" fn sign_undefine_by_name(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut sp: *mut sign_T = map_del_cstr_t_ptr_t(
        sign_map.ptr(),
        name as cstr_t,
        ::core::ptr::null_mut::<cstr_t>(),
    ) as *mut sign_T;
    if sp.is_null() {
        semsg(
            gettext(b"E155: Unknown sign: %s\0".as_ptr() as *const ::core::ffi::c_char),
            name,
        );
        return FAIL;
    }
    xfree((*sp).sn_name as *mut ::core::ffi::c_void);
    xfree((*sp).sn_icon as *mut ::core::ffi::c_void);
    xfree(sp as *mut ::core::ffi::c_void);
    return OK;
}
unsafe extern "C" fn sign_list_defined(mut sp: *mut sign_T) {
    smsg(
        0 as ::core::ffi::c_int,
        b"sign %s\0".as_ptr() as *const ::core::ffi::c_char,
        (*sp).sn_name,
    );
    if !(*sp).sn_icon.is_null() {
        msg_puts(b" icon=\0".as_ptr() as *const ::core::ffi::c_char);
        msg_outtrans((*sp).sn_icon, 0 as ::core::ffi::c_int, false_0 != 0);
        msg_puts(gettext(
            b" (not supported)\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
    if (*sp).sn_text[0 as ::core::ffi::c_int as usize] != 0 {
        msg_puts(b" text=\0".as_ptr() as *const ::core::ffi::c_char);
        let mut buf: [::core::ffi::c_char; 64] = [0; 64];
        describe_sign_text(
            &raw mut buf as *mut ::core::ffi::c_char,
            &raw mut (*sp).sn_text as *mut schar_T,
        );
        msg_outtrans(
            &raw mut buf as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    if (*sp).sn_priority > 0 as ::core::ffi::c_int {
        let mut lbuf: [::core::ffi::c_char; 480] = [0; 480];
        vim_snprintf(
            &raw mut lbuf as *mut ::core::ffi::c_char,
            MSG_BUF_LEN as size_t,
            b" priority=%d\0".as_ptr() as *const ::core::ffi::c_char,
            (*sp).sn_priority,
        );
        msg_puts(&raw mut lbuf as *mut ::core::ffi::c_char);
    }
    static arg: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
        b" linehl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b" texthl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b" culhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b" numhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    let mut hl: [::core::ffi::c_int; 4] = [
        (*sp).sn_line_hl,
        (*sp).sn_text_hl,
        (*sp).sn_cul_hl,
        (*sp).sn_num_hl,
    ];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        if hl[i as usize] > 0 as ::core::ffi::c_int {
            msg_puts((*arg.ptr())[i as usize]);
            let mut p: *const ::core::ffi::c_char = get_highlight_name_ext(
                ::core::ptr::null_mut::<expand_T>(),
                hl[i as usize] - 1 as ::core::ffi::c_int,
                false_0 != 0,
            );
            msg_puts(if !p.is_null() {
                p
            } else {
                b"NONE\0".as_ptr() as *const ::core::ffi::c_char
            });
        }
        i += 1;
    }
}
unsafe extern "C" fn sign_list_by_name(mut name: *mut ::core::ffi::c_char) {
    let mut sp: *mut sign_T = map_get_cstr_t_ptr_t(sign_map.ptr(), name as cstr_t) as *mut sign_T;
    if !sp.is_null() {
        sign_list_defined(sp);
    } else {
        semsg(
            gettext(b"E155: Unknown sign: %s\0".as_ptr() as *const ::core::ffi::c_char),
            name,
        );
    };
}
unsafe extern "C" fn sign_place(
    mut id: *mut uint32_t,
    mut group: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut prio: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if !group.is_null()
        && (*group as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            || *group as ::core::ffi::c_int == NUL)
    {
        return FAIL;
    }
    let mut sp: *mut sign_T = map_get_cstr_t_ptr_t(sign_map.ptr(), name as cstr_t) as *mut sign_T;
    if sp.is_null() {
        semsg(
            gettext(b"E155: Unknown sign: %s\0".as_ptr() as *const ::core::ffi::c_char),
            name,
        );
        return FAIL;
    }
    if prio == -1 as ::core::ffi::c_int {
        prio = if (*sp).sn_priority != -1 as ::core::ffi::c_int {
            (*sp).sn_priority
        } else {
            SIGN_DEF_PRIO as ::core::ffi::c_int
        };
    }
    if lnum > 0 as linenr_T {
        buf_set_sign(buf, id, group, prio, lnum, sp);
    } else {
        lnum = buf_mod_sign(buf, id, group, prio, sp);
    }
    if lnum <= 0 as linenr_T {
        semsg(
            gettext(
                b"E885: Not possible to change sign %s\0".as_ptr() as *const ::core::ffi::c_char
            ),
            name,
        );
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn sign_unplace_inner(
    mut buf: *mut buf_T,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
    mut atlnum: linenr_T,
) -> ::core::ffi::c_int {
    if !buf_has_signs(buf) {
        return FAIL;
    }
    if id == 0 as ::core::ffi::c_int
        || atlnum > 0 as linenr_T
        || !group.is_null() && *group as ::core::ffi::c_int == '*' as ::core::ffi::c_int
    {
        if buf_delete_signs(buf, group, id, atlnum) == 0 {
            return FAIL;
        }
    } else {
        let mut ns: int64_t = group_get_ns(group);
        if ns < 0 as int64_t || !extmark_del_id(buf, ns as uint32_t, id as uint32_t) {
            return FAIL;
        }
    }
    return OK;
}
unsafe extern "C" fn sign_unplace(
    mut buf: *mut buf_T,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
    mut atlnum: linenr_T,
) -> ::core::ffi::c_int {
    if !buf.is_null() {
        return sign_unplace_inner(buf, id, group, atlnum);
    } else {
        let mut retval: ::core::ffi::c_int = OK;
        let mut cbuf: *mut buf_T = firstbuf.get();
        while !cbuf.is_null() {
            if sign_unplace_inner(cbuf, id, group, atlnum) == 0 {
                retval = FAIL;
            }
            cbuf = (*cbuf).b_next;
        }
        return retval;
    };
}
unsafe extern "C" fn sign_jump(
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
    mut buf: *mut buf_T,
) -> linenr_T {
    let mut lnum: linenr_T = buf_findsign(buf, id, group) as linenr_T;
    if lnum <= 0 as linenr_T {
        semsg(
            gettext(b"E157: Invalid sign ID: %d\0".as_ptr() as *const ::core::ffi::c_char),
            id,
        );
        return -1 as linenr_T;
    }
    if !buf_jump_open_win(buf).is_null() {
        (*curwin.get()).w_cursor.lnum = lnum;
        check_cursor_lnum(curwin.get());
        beginline(BL_WHITE as ::core::ffi::c_int);
    } else {
        if (*buf).b_fname.is_null() {
            emsg(gettext(
                b"E934: Cannot jump to a buffer that does not have a name\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return -1 as linenr_T;
        }
        let mut cmdlen: size_t = strlen((*buf).b_fname).wrapping_add(24 as size_t);
        let mut cmd: *mut ::core::ffi::c_char = xmallocz(cmdlen) as *mut ::core::ffi::c_char;
        snprintf(
            cmd,
            cmdlen,
            b"e +%ld %s\0".as_ptr() as *const ::core::ffi::c_char,
            lnum as int64_t,
            (*buf).b_fname,
        );
        do_cmdline_cmd(cmd);
        xfree(cmd as *mut ::core::ffi::c_void);
    }
    foldOpenCursor();
    return lnum;
}
unsafe extern "C" fn sign_define_cmd(
    mut name: *mut ::core::ffi::c_char,
    mut cmdline: *mut ::core::ffi::c_char,
) {
    let mut icon: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut linehl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut texthl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut culhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut numhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    loop {
        let mut arg: *mut ::core::ffi::c_char = skipwhite(cmdline);
        if *arg as ::core::ffi::c_int == NUL {
            break;
        }
        cmdline = skiptowhite_esc(arg);
        if strncmp(
            arg,
            b"icon=\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            icon = arg.offset(5 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"text=\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            text = arg.offset(5 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"linehl=\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            linehl = arg.offset(7 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"texthl=\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            texthl = arg.offset(7 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"culhl=\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            culhl = arg.offset(6 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"numhl=\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            numhl = arg.offset(6 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"priority=\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            prio = atoi(arg.offset(9 as ::core::ffi::c_int as isize));
        } else {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                arg,
            );
            return;
        }
        if *cmdline as ::core::ffi::c_int == NUL {
            break;
        }
        let c2rust_fresh7 = cmdline;
        cmdline = cmdline.offset(1);
        *c2rust_fresh7 = NUL as ::core::ffi::c_char;
    }
    sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio);
}
unsafe extern "C" fn sign_place_cmd(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut name: *mut ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
) {
    if id <= 0 as ::core::ffi::c_int {
        if lnum >= 0 as linenr_T
            || !name.is_null()
            || !group.is_null() && *group as ::core::ffi::c_int == NUL
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else {
            sign_list_placed(buf, group);
        }
    } else {
        if name.is_null()
            || buf.is_null()
            || !group.is_null() && *group as ::core::ffi::c_int == NUL
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        let mut uid: uint32_t = id as uint32_t;
        sign_place(&raw mut uid, group, name, buf, lnum, prio);
    };
}
unsafe extern "C" fn sign_unplace_cmd(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut name: *const ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
) {
    if lnum >= 0 as linenr_T
        || !name.is_null()
        || !group.is_null() && *group as ::core::ffi::c_int == NUL
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    if id == -1 as ::core::ffi::c_int {
        lnum = (*curwin.get()).w_cursor.lnum;
        buf = (*curwin.get()).w_buffer;
    }
    if sign_unplace(
        buf,
        if 0 as ::core::ffi::c_int > id {
            0 as ::core::ffi::c_int
        } else {
            id
        },
        group,
        lnum,
    ) == 0
        && lnum > 0 as linenr_T
    {
        emsg(gettext(
            b"E159: Missing sign number\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
}
unsafe extern "C" fn sign_jump_cmd(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut name: *const ::core::ffi::c_char,
    mut id: ::core::ffi::c_int,
    mut group: *mut ::core::ffi::c_char,
) {
    if name.is_null() && group.is_null() && id == -1 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
        return;
    }
    if buf.is_null()
        || !group.is_null() && *group as ::core::ffi::c_int == NUL
        || lnum >= 0 as linenr_T
        || !name.is_null()
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    sign_jump(id, group, buf);
}
unsafe extern "C" fn parse_sign_cmd_args(
    mut cmd: ::core::ffi::c_int,
    mut arg: *mut ::core::ffi::c_char,
    mut name: *mut *mut ::core::ffi::c_char,
    mut id: *mut ::core::ffi::c_int,
    mut group: *mut *mut ::core::ffi::c_char,
    mut prio: *mut ::core::ffi::c_int,
    mut buf: *mut *mut buf_T,
    mut lnum: *mut linenr_T,
) -> ::core::ffi::c_int {
    let mut arg1: *mut ::core::ffi::c_char = arg;
    let mut filename: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lnum_arg: bool = false_0 != 0;
    if ascii_isdigit(*arg as ::core::ffi::c_int) {
        *id = getdigits_int(&raw mut arg, true_0 != 0, 0 as ::core::ffi::c_int);
        if !ascii_iswhite(*arg as ::core::ffi::c_int) && *arg as ::core::ffi::c_int != NUL {
            *id = -1 as ::core::ffi::c_int;
            arg = arg1;
        } else {
            arg = skipwhite(arg);
        }
    }
    while *arg as ::core::ffi::c_int != NUL {
        if strncmp(
            arg,
            b"line=\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(5 as ::core::ffi::c_int as isize);
            *lnum = atoi(arg) as linenr_T;
            arg = skiptowhite(arg);
            lnum_arg = true_0 != 0;
        } else if strncmp(
            arg,
            b"*\0".as_ptr() as *const ::core::ffi::c_char,
            1 as size_t,
        ) == 0 as ::core::ffi::c_int
            && cmd == SIGNCMD_UNPLACE
        {
            if *id != -1 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return FAIL;
            }
            *id = -2 as ::core::ffi::c_int;
            arg = skiptowhite(arg.offset(1 as ::core::ffi::c_int as isize));
        } else if strncmp(
            arg,
            b"name=\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(5 as ::core::ffi::c_int as isize);
            let mut namep: *mut ::core::ffi::c_char = arg;
            arg = skiptowhite(arg);
            if *arg as ::core::ffi::c_int != NUL {
                let c2rust_fresh5 = arg;
                arg = arg.offset(1);
                *c2rust_fresh5 = NUL as ::core::ffi::c_char;
            }
            while *namep.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '0' as ::core::ffi::c_int
                && *namep.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                namep = namep.offset(1);
            }
            *name = namep;
        } else if strncmp(
            arg,
            b"group=\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(6 as ::core::ffi::c_int as isize);
            *group = arg;
            arg = skiptowhite(arg);
            if *arg as ::core::ffi::c_int != NUL {
                let c2rust_fresh6 = arg;
                arg = arg.offset(1);
                *c2rust_fresh6 = NUL as ::core::ffi::c_char;
            }
        } else if strncmp(
            arg,
            b"priority=\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(9 as ::core::ffi::c_int as isize);
            *prio = atoi(arg);
            arg = skiptowhite(arg);
        } else if strncmp(
            arg,
            b"file=\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(5 as ::core::ffi::c_int as isize);
            filename = arg;
            *buf = buflist_findname_exp(arg);
            break;
        } else if strncmp(
            arg,
            b"buffer=\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            arg = arg.offset(7 as ::core::ffi::c_int as isize);
            filename = arg;
            *buf = buflist_findnr(getdigits_int(
                &raw mut arg,
                true_0 != 0,
                0 as ::core::ffi::c_int,
            ));
            if *skipwhite(arg) as ::core::ffi::c_int != NUL {
                semsg(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    arg,
                );
            }
            break;
        } else {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return FAIL;
        }
        arg = skipwhite(arg);
    }
    if !filename.is_null() && (*buf).is_null() {
        semsg(
            gettext(&raw const e_invalid_buffer_name_str as *const ::core::ffi::c_char),
            filename,
        );
        return FAIL;
    }
    if filename.is_null()
        && (cmd == SIGNCMD_PLACE && lnum_arg as ::core::ffi::c_int != 0 || cmd == SIGNCMD_JUMP)
    {
        *buf = (*curwin.get()).w_buffer;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn ex_sign(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut p: *mut ::core::ffi::c_char = skiptowhite(arg);
    let mut idx: ::core::ffi::c_int = sign_cmd_idx(arg, p);
    if idx == SIGNCMD_LAST {
        semsg(
            gettext(b"E160: Unknown sign command: %s\0".as_ptr() as *const ::core::ffi::c_char),
            arg,
        );
        return;
    }
    arg = skipwhite(p);
    if idx <= SIGNCMD_LIST {
        if idx == SIGNCMD_LIST && *arg as ::core::ffi::c_int == NUL {
            let mut sp: *mut sign_T = ::core::ptr::null_mut::<sign_T>();
            let mut __i: uint32_t = 0;
            __i = 0 as uint32_t;
            while __i < (*sign_map.ptr()).set.h.n_keys {
                sp = *(*sign_map.ptr()).values.offset(__i as isize) as *mut sign_T;
                sign_list_defined(sp);
                __i = __i.wrapping_add(1);
            }
        } else if *arg as ::core::ffi::c_int == NUL {
            emsg(gettext(
                b"E156: Missing sign name\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            p = skiptowhite(arg);
            if *p as ::core::ffi::c_int != NUL {
                let c2rust_fresh0 = p;
                p = p.offset(1);
                *c2rust_fresh0 = NUL as ::core::ffi::c_char;
            }
            while *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '0' as ::core::ffi::c_int
                && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                arg = arg.offset(1);
            }
            if idx == SIGNCMD_DEFINE {
                sign_define_cmd(arg, p);
            } else if idx == SIGNCMD_LIST {
                sign_list_by_name(arg);
            } else {
                sign_undefine_by_name(arg);
            }
            return;
        }
    } else {
        let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut lnum: linenr_T = -1 as linenr_T;
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut group: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if parse_sign_cmd_args(
            idx,
            arg,
            &raw mut name,
            &raw mut id,
            &raw mut group,
            &raw mut prio,
            &raw mut buf,
            &raw mut lnum,
        ) == FAIL
        {
            return;
        }
        if idx == SIGNCMD_PLACE {
            sign_place_cmd(buf, lnum, name, id, group, prio);
        } else if idx == SIGNCMD_UNPLACE {
            sign_unplace_cmd(buf, lnum, name, id, group);
        } else if idx == SIGNCMD_JUMP {
            sign_jump_cmd(buf, lnum, name, id, group);
        }
    };
}
unsafe extern "C" fn sign_get_info_dict(mut sp: *mut sign_T) -> *mut dict_T {
    let mut d: *mut dict_T = tv_dict_alloc();
    tv_dict_add_str(
        d,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*sp).sn_name,
    );
    if !(*sp).sn_icon.is_null() {
        tv_dict_add_str(
            d,
            b"icon\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*sp).sn_icon,
        );
    }
    if (*sp).sn_text[0 as ::core::ffi::c_int as usize] != 0 {
        let mut buf: [::core::ffi::c_char; 64] = [0; 64];
        describe_sign_text(
            &raw mut buf as *mut ::core::ffi::c_char,
            &raw mut (*sp).sn_text as *mut schar_T,
        );
        tv_dict_add_str(
            d,
            b"text\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
    }
    if (*sp).sn_priority > 0 as ::core::ffi::c_int {
        tv_dict_add_nr(
            d,
            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            (*sp).sn_priority as varnumber_T,
        );
    }
    static arg: GlobalCell<[*mut ::core::ffi::c_char; 4]> = GlobalCell::new([
        b"linehl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"texthl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"culhl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"numhl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    let mut hl: [::core::ffi::c_int; 4] = [
        (*sp).sn_line_hl,
        (*sp).sn_text_hl,
        (*sp).sn_cul_hl,
        (*sp).sn_num_hl,
    ];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        if hl[i as usize] > 0 as ::core::ffi::c_int {
            let mut p: *const ::core::ffi::c_char = get_highlight_name_ext(
                ::core::ptr::null_mut::<expand_T>(),
                hl[i as usize] - 1 as ::core::ffi::c_int,
                false_0 != 0,
            );
            tv_dict_add_str(
                d,
                (*arg.ptr())[i as usize],
                strlen((*arg.ptr())[i as usize]),
                if !p.is_null() {
                    p
                } else {
                    b"NONE\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
        }
        i += 1;
    }
    return d;
}
unsafe extern "C" fn sign_get_placed_info_dict(mut mark: MTKey) -> *mut dict_T {
    let mut d: *mut dict_T = tv_dict_alloc();
    let mut sh: *mut DecorSignHighlight = decor_find_sign(mt_decor(mark));
    tv_dict_add_str(
        d,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        sign_get_name(sh),
    );
    tv_dict_add_nr(
        d,
        b"id\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
        mark.id as ::core::ffi::c_int as varnumber_T,
    );
    tv_dict_add_str(
        d,
        b"group\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        describe_ns(mark.ns as NS, b"\0".as_ptr() as *const ::core::ffi::c_char),
    );
    tv_dict_add_nr(
        d,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (mark.pos.row + 1 as int32_t) as varnumber_T,
    );
    tv_dict_add_nr(
        d,
        b"priority\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*sh).priority as varnumber_T,
    );
    return d;
}
#[no_mangle]
pub unsafe extern "C" fn get_buffer_signs(mut buf: *mut buf_T) -> *mut list_T {
    let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    marktree_itr_get(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        0 as int32_t,
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if !mt_end(mark) && mt_decor_sign(mark) as ::core::ffi::c_int != 0 {
            tv_list_append_dict(l, sign_get_placed_info_dict(mark));
        }
        marktree_itr_next(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
    return l;
}
unsafe extern "C" fn sign_get_placed_in_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut sign_id: ::core::ffi::c_int,
    mut group: *const ::core::ffi::c_char,
    mut retlist: *mut list_T,
) {
    let mut d: *mut dict_T = tv_dict_alloc();
    tv_list_append_dict(retlist, d);
    tv_dict_add_nr(
        d,
        b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*buf).handle as varnumber_T,
    );
    let mut l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    tv_dict_add_list(
        d,
        b"signs\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        l,
    );
    let mut ns: int64_t = group_get_ns(group);
    if !buf_has_signs(buf) || ns < 0 as int64_t {
        return;
    }
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_16 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    let mut signs: C2Rust_Unnamed_28 = C2Rust_Unnamed_28 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<MTKey>(),
    };
    marktree_itr_get(
        &raw mut (*buf).b_marktree as *mut MarkTree,
        if lnum != 0 {
            lnum as int32_t - 1 as int32_t
        } else {
            0 as int32_t
        },
        0 as ::core::ffi::c_int,
        &raw mut itr as *mut MarkTreeIter,
    );
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if lnum != 0 && mark.pos.row >= lnum {
            break;
        }
        if !mt_end(mark)
            && (ns == UINT32_MAX as int64_t || ns == mark.ns as int64_t)
            && (lnum == 0 as linenr_T && sign_id == 0 as ::core::ffi::c_int
                || sign_id == 0 as ::core::ffi::c_int && lnum == mark.pos.row + 1 as int32_t
                || lnum == 0 as linenr_T && sign_id == mark.id as ::core::ffi::c_int
                || lnum == mark.pos.row + 1 as int32_t && sign_id == mark.id as ::core::ffi::c_int)
        {
            if mt_decor_sign(mark) {
                if signs.size == signs.capacity {
                    signs.capacity = if signs.capacity != 0 {
                        signs.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    signs.items = xrealloc(
                        signs.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<MTKey>().wrapping_mul(signs.capacity),
                    ) as *mut MTKey;
                } else {
                };
                let c2rust_fresh10 = signs.size;
                signs.size = signs.size.wrapping_add(1);
                *signs.items.offset(c2rust_fresh10 as isize) = mark;
            }
        }
        marktree_itr_next(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
        );
    }
    if signs.size != 0 {
        qsort(
            signs.items.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            signs.size,
            ::core::mem::size_of::<MTKey>(),
            Some(
                sign_row_cmp
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        let mut i: size_t = 0 as size_t;
        while i < signs.size {
            tv_list_append_dict(
                l,
                sign_get_placed_info_dict(*signs.items.offset(i as isize)),
            );
            i = i.wrapping_add(1);
        }
        xfree(signs.items as *mut ::core::ffi::c_void);
        signs.capacity = 0 as size_t;
        signs.size = signs.capacity;
        signs.items = ::core::ptr::null_mut::<MTKey>();
    }
}
unsafe extern "C" fn sign_get_placed(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut id: ::core::ffi::c_int,
    mut group: *const ::core::ffi::c_char,
    mut retlist: *mut list_T,
) {
    if !buf.is_null() {
        sign_get_placed_in_buf(buf, lnum, id, group, retlist);
    } else {
        let mut cbuf: *mut buf_T = firstbuf.get();
        while !cbuf.is_null() {
            if buf_has_signs(cbuf) {
                sign_get_placed_in_buf(cbuf, 0 as linenr_T, id, group, retlist);
            }
            cbuf = (*cbuf).b_next;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn free_signs() {
    let mut name: cstr_t = ::core::ptr::null::<::core::ffi::c_char>();
    let mut names: C2Rust_Unnamed_26 = C2Rust_Unnamed_26 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<cstr_t>(),
    };
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*sign_map.ptr()).set.h.n_keys {
        name = *(*sign_map.ptr()).set.keys.offset(__i as isize);
        if names.size == names.capacity {
            names.capacity = if names.capacity != 0 {
                names.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            names.items = xrealloc(
                names.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<cstr_t>().wrapping_mul(names.capacity),
            ) as *mut cstr_t;
        } else {
        };
        let c2rust_fresh8 = names.size;
        names.size = names.size.wrapping_add(1);
        let c2rust_lvalue_ptr = &raw mut *names.items.offset(c2rust_fresh8 as isize);
        *c2rust_lvalue_ptr = name;
        __i = __i.wrapping_add(1);
    }
    let mut i: size_t = 0 as size_t;
    while i < names.size {
        sign_undefine_by_name(*names.items.offset(i as isize) as *const ::core::ffi::c_char);
        i = i.wrapping_add(1);
    }
    xfree(names.items as *mut ::core::ffi::c_void);
    names.capacity = 0 as size_t;
    names.size = names.capacity;
    names.items = ::core::ptr::null_mut::<cstr_t>();
}
static expand_what: GlobalCell<C2Rust_Unnamed_27> = GlobalCell::new(EXP_SUBCMD);
unsafe extern "C" fn get_nth_sign_name(mut idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut name: cstr_t = ::core::ptr::null::<::core::ffi::c_char>();
    let mut current_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*sign_map.ptr()).set.h.n_keys {
        name = *(*sign_map.ptr()).set.keys.offset(__i as isize);
        let c2rust_fresh9 = current_idx;
        current_idx = current_idx + 1;
        if c2rust_fresh9 == idx {
            return name as *mut ::core::ffi::c_char;
        }
        __i = __i.wrapping_add(1);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn get_nth_sign_group_name(
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx < (*sign_ns.ptr()).size as ::core::ffi::c_int {
        return describe_ns(
            *(*sign_ns.ptr()).items.offset(idx as isize) as NS,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
        ) as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn get_sign_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match expand_what.get() as ::core::ffi::c_uint {
        0 => return (*cmds.ptr())[idx as usize],
        1 => {
            let mut define_arg: [*mut ::core::ffi::c_char; 8] = [
                b"culhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"icon=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"linehl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"numhl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"text=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"texthl=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"priority=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return define_arg[idx as usize];
        }
        2 => {
            let mut place_arg: [*mut ::core::ffi::c_char; 7] = [
                b"line=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"name=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"group=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"priority=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"file=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"buffer=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return place_arg[idx as usize];
        }
        3 => {
            let mut list_arg: [*mut ::core::ffi::c_char; 4] = [
                b"group=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"file=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"buffer=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return list_arg[idx as usize];
        }
        4 => {
            let mut unplace_arg: [*mut ::core::ffi::c_char; 4] = [
                b"group=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"file=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                b"buffer=\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ];
            return unplace_arg[idx as usize];
        }
        5 => return get_nth_sign_name(idx),
        6 => return get_nth_sign_group_name(idx),
        _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
}
#[no_mangle]
pub unsafe extern "C" fn set_context_in_sign_cmd(
    mut xp: *mut expand_T,
    mut arg: *mut ::core::ffi::c_char,
) {
    (*xp).xp_context = EXPAND_SIGN as ::core::ffi::c_int;
    expand_what.set(EXP_SUBCMD);
    (*xp).xp_pattern = arg;
    let mut end_subcmd: *mut ::core::ffi::c_char = skiptowhite(arg);
    if *end_subcmd as ::core::ffi::c_int == NUL {
        return;
    }
    let mut cmd_idx: ::core::ffi::c_int = sign_cmd_idx(arg, end_subcmd);
    let mut begin_subcmd_args: *mut ::core::ffi::c_char = skipwhite(end_subcmd);
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = begin_subcmd_args;
    loop {
        p = skipwhite(p);
        last = p;
        p = skiptowhite(p);
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
    }
    p = vim_strchr(last, '=' as ::core::ffi::c_int);
    if p.is_null() {
        (*xp).xp_pattern = last;
        match cmd_idx {
            SIGNCMD_DEFINE => {
                expand_what.set(EXP_DEFINE);
            }
            SIGNCMD_PLACE => {
                if ascii_isdigit(*begin_subcmd_args as ::core::ffi::c_int) {
                    expand_what.set(EXP_PLACE);
                } else {
                    expand_what.set(EXP_LIST);
                }
            }
            SIGNCMD_LIST | SIGNCMD_UNDEFINE => {
                expand_what.set(EXP_SIGN_NAMES);
            }
            SIGNCMD_JUMP | SIGNCMD_UNPLACE => {
                expand_what.set(EXP_UNPLACE);
            }
            _ => {
                (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            }
        }
    } else {
        (*xp).xp_pattern = p.offset(1 as ::core::ffi::c_int as isize);
        match cmd_idx {
            SIGNCMD_DEFINE => {
                if strncmp(
                    last,
                    b"texthl\0".as_ptr() as *const ::core::ffi::c_char,
                    6 as size_t,
                ) == 0 as ::core::ffi::c_int
                    || strncmp(
                        last,
                        b"linehl\0".as_ptr() as *const ::core::ffi::c_char,
                        6 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    || strncmp(
                        last,
                        b"culhl\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    || strncmp(
                        last,
                        b"numhl\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    (*xp).xp_context = EXPAND_HIGHLIGHT as ::core::ffi::c_int;
                } else if strncmp(
                    last,
                    b"icon\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*xp).xp_context = EXPAND_FILES as ::core::ffi::c_int;
                } else {
                    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                }
            }
            SIGNCMD_PLACE => {
                if strncmp(
                    last,
                    b"name\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    expand_what.set(EXP_SIGN_NAMES);
                } else if strncmp(
                    last,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    expand_what.set(EXP_SIGN_GROUPS);
                } else if strncmp(
                    last,
                    b"file\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*xp).xp_context = EXPAND_BUFFERS as ::core::ffi::c_int;
                } else {
                    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                }
            }
            SIGNCMD_UNPLACE | SIGNCMD_JUMP => {
                if strncmp(
                    last,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    expand_what.set(EXP_SIGN_GROUPS);
                } else if strncmp(
                    last,
                    b"file\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    (*xp).xp_context = EXPAND_BUFFERS as ::core::ffi::c_int;
                } else {
                    (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
                }
            }
            _ => {
                (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
            }
        }
    };
}
unsafe extern "C" fn sign_define_from_dict(
    mut name: *mut ::core::ffi::c_char,
    mut dict: *mut dict_T,
) -> ::core::ffi::c_int {
    if name.is_null() {
        name = tv_dict_get_string(
            dict,
            b"name\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if name.is_null()
            || *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            return -1 as ::core::ffi::c_int;
        }
    }
    let mut icon: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut linehl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut texthl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut culhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut numhl: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if !dict.is_null() {
        icon = tv_dict_get_string(
            dict,
            b"icon\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        linehl = tv_dict_get_string(
            dict,
            b"linehl\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        text = tv_dict_get_string(
            dict,
            b"text\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        texthl = tv_dict_get_string(
            dict,
            b"texthl\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        culhl = tv_dict_get_string(
            dict,
            b"culhl\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        numhl = tv_dict_get_string(
            dict,
            b"numhl\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        prio = tv_dict_get_number_def(
            dict,
            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int;
    }
    return sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio)
        - 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn sign_define_multiple(mut l: *mut list_T, mut retlist: *mut list_T) {
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                retval = sign_define_from_dict(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    (*li).li_tv.vval.v_dict,
                );
            } else {
                emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
            }
            tv_list_append_number(retlist, retval as varnumber_T);
            li = (*li).li_next;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_sign_define(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        sign_define_multiple(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            (*rettv).vval.v_list,
        );
        return;
    }
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut name: *mut ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize))
            as *mut ::core::ffi::c_char;
    if name.is_null() {
        return;
    }
    if tv_check_for_opt_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut d: *mut dict_T = if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type
        as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict
    } else {
        ::core::ptr::null_mut::<dict_T>()
    };
    (*rettv).vval.v_number = sign_define_from_dict(name, d) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_sign_getdefined(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut sp: *mut sign_T = ::core::ptr::null_mut::<sign_T>();
        let mut __i: uint32_t = 0;
        __i = 0 as uint32_t;
        while __i < (*sign_map.ptr()).set.h.n_keys {
            sp = *(*sign_map.ptr()).values.offset(__i as isize) as *mut sign_T;
            tv_list_append_dict((*rettv).vval.v_list, sign_get_info_dict(sp));
            __i = __i.wrapping_add(1);
        }
    } else {
        let mut sp_0: *mut sign_T = map_get_cstr_t_ptr_t(
            sign_map.ptr(),
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
        ) as *mut sign_T;
        if !sp_0.is_null() {
            tv_list_append_dict((*rettv).vval.v_list, sign_get_info_dict(sp_0));
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_sign_getplaced(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut sign_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut group: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut notanum: bool = false_0 != 0;
    tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = get_buf_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
        if buf.is_null() {
            return;
        }
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_check_for_nonnull_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
                return;
            }
            let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
            let mut dict: *mut dict_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
            di = tv_dict_find(
                dict,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                lnum = tv_get_lnum(&raw mut (*di).di_tv);
                if lnum <= 0 as linenr_T {
                    return;
                }
            }
            di = tv_dict_find(
                dict,
                b"id\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                sign_id =
                    tv_get_number_chk(&raw mut (*di).di_tv, &raw mut notanum) as ::core::ffi::c_int;
                if notanum {
                    return;
                }
            }
            di = tv_dict_find(
                dict,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if !di.is_null() {
                group = tv_get_string_chk(&raw mut (*di).di_tv);
                if group.is_null() {
                    return;
                }
                if *group as ::core::ffi::c_int == NUL {
                    group = ::core::ptr::null::<::core::ffi::c_char>();
                }
            }
        }
    }
    sign_get_placed(buf, lnum, sign_id, group, (*rettv).vval.v_list);
}
#[no_mangle]
pub unsafe extern "C" fn f_sign_jump(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut notanum: bool = false_0 != 0;
    let mut id: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut notanum,
    ) as ::core::ffi::c_int;
    if notanum {
        return;
    }
    if id <= 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let mut group: *mut ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize))
            as *mut ::core::ffi::c_char;
    if group.is_null() {
        return;
    }
    if *group.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        group = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buf: *mut buf_T = get_buf_arg(argvars.offset(2 as ::core::ffi::c_int as isize));
    if buf.is_null() {
        return;
    }
    (*rettv).vval.v_number = sign_jump(id, group, buf) as varnumber_T;
}
unsafe extern "C" fn sign_place_from_dict(
    mut id_tv: *mut typval_T,
    mut group_tv: *mut typval_T,
    mut name_tv: *mut typval_T,
    mut buf_tv: *mut typval_T,
    mut dict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut notanum: bool = false_0 != 0;
    if id_tv.is_null() {
        di = tv_dict_find(
            dict,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            id_tv = &raw mut (*di).di_tv;
        }
    }
    if !id_tv.is_null() {
        id = tv_get_number_chk(id_tv, &raw mut notanum) as ::core::ffi::c_int;
        if notanum {
            return -1 as ::core::ffi::c_int;
        }
        if id < 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return -1 as ::core::ffi::c_int;
        }
    }
    let mut group: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if group_tv.is_null() {
        di = tv_dict_find(
            dict,
            b"group\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            group_tv = &raw mut (*di).di_tv;
        }
    }
    if !group_tv.is_null() {
        group = tv_get_string_chk(group_tv) as *mut ::core::ffi::c_char;
        if group.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        if *group.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            group = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if name_tv.is_null() {
        di = tv_dict_find(
            dict,
            b"name\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            name_tv = &raw mut (*di).di_tv;
        }
    }
    if name_tv.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    name = tv_get_string_chk(name_tv) as *mut ::core::ffi::c_char;
    if name.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    if buf_tv.is_null() {
        di = tv_dict_find(
            dict,
            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            buf_tv = &raw mut (*di).di_tv;
        }
    }
    if buf_tv.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    let mut buf: *mut buf_T = get_buf_arg(buf_tv);
    if buf.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    let mut lnum: linenr_T = 0 as linenr_T;
    di = tv_dict_find(
        dict,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        -1 as ptrdiff_t,
    );
    if !di.is_null() {
        lnum = tv_get_lnum(&raw mut (*di).di_tv);
        if lnum <= 0 as linenr_T {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return -1 as ::core::ffi::c_int;
        }
    }
    let mut prio: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    di = tv_dict_find(
        dict,
        b"priority\0".as_ptr() as *const ::core::ffi::c_char,
        -1 as ptrdiff_t,
    );
    if !di.is_null() {
        prio = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut notanum) as ::core::ffi::c_int;
        if notanum {
            return -1 as ::core::ffi::c_int;
        }
    }
    let mut uid: uint32_t = id as uint32_t;
    if sign_place(&raw mut uid, group, name, buf, lnum, prio) == OK {
        return uid as ::core::ffi::c_int;
    }
    return -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn f_sign_place(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_check_for_nonnull_dict_arg(argvars, 4 as ::core::ffi::c_int) == FAIL {
            return;
        }
        dict = (*argvars.offset(4 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
    }
    (*rettv).vval.v_number = sign_place_from_dict(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        argvars.offset(1 as ::core::ffi::c_int as isize),
        argvars.offset(2 as ::core::ffi::c_int as isize),
        argvars.offset(3 as ::core::ffi::c_int as isize),
        dict,
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_sign_placelist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    }
    let l_: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut sign_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                sign_id = sign_place_from_dict(
                    ::core::ptr::null_mut::<typval_T>(),
                    ::core::ptr::null_mut::<typval_T>(),
                    ::core::ptr::null_mut::<typval_T>(),
                    ::core::ptr::null_mut::<typval_T>(),
                    (*li).li_tv.vval.v_dict,
                );
            } else {
                emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
            }
            tv_list_append_number((*rettv).vval.v_list, sign_id as varnumber_T);
            li = (*li).li_next;
        }
    }
}
unsafe extern "C" fn sign_undefine_multiple(mut l: *mut list_T, mut retlist: *mut list_T) {
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut name: *mut ::core::ffi::c_char =
                tv_get_string_chk(&raw const (*li).li_tv) as *mut ::core::ffi::c_char;
            if !name.is_null() && sign_undefine_by_name(name) == 1 as ::core::ffi::c_int {
                retval = 0 as ::core::ffi::c_int;
            }
            tv_list_append_number(retlist, retval as varnumber_T);
            li = (*li).li_next;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_sign_undefine(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        sign_undefine_multiple(
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            (*rettv).vval.v_list,
        );
        return;
    }
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        free_signs();
        (*rettv).vval.v_number = 0 as varnumber_T;
    } else {
        let mut name: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        if name.is_null() {
            return;
        }
        if sign_undefine_by_name(name) == OK {
            (*rettv).vval.v_number = 0 as varnumber_T;
        }
    };
}
unsafe extern "C" fn sign_unplace_from_dict(
    mut group_tv: *mut typval_T,
    mut dict: *mut dict_T,
) -> ::core::ffi::c_int {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut group: *mut ::core::ffi::c_char = if !group_tv.is_null() {
        tv_get_string(group_tv) as *mut ::core::ffi::c_char
    } else {
        tv_dict_get_string(
            dict,
            b"group\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        )
    };
    if !group.is_null()
        && *group.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        group = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !dict.is_null() {
        di = tv_dict_find(
            dict,
            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            buf = get_buf_arg(&raw mut (*di).di_tv);
            if buf.is_null() {
                return -1 as ::core::ffi::c_int;
            }
        }
        if !tv_dict_find(
            dict,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        )
        .is_null()
        {
            id = tv_dict_get_number(dict, b"id\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int;
            if id <= 0 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return -1 as ::core::ffi::c_int;
            }
        }
    }
    return sign_unplace(buf, id, group, 0 as linenr_T) - 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn f_sign_unplace(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut dict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    (*rettv).vval.v_number = -1 as varnumber_T;
    if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_dict_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        dict = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
    }
    (*rettv).vval.v_number =
        sign_unplace_from_dict(argvars.offset(0 as ::core::ffi::c_int as isize), dict)
            as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_sign_unplacelist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    }
    let l_: *const list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut retval: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                retval = sign_unplace_from_dict(
                    ::core::ptr::null_mut::<typval_T>(),
                    (*li).li_tv.vval.v_dict,
                );
            } else {
                emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
            }
            tv_list_append_number((*rettv).vval.v_list, retval as varnumber_T);
            li = (*li).li_next;
        }
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
