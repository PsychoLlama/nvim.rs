use crate::src::nvim::change::inserted_bytes;
use crate::src::nvim::charset::{getdigits_int, rl_mirror_ascii, skiptowhite, skipwhite};
use crate::src::nvim::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::src::nvim::eval::typval::tv_list_unref;
use crate::src::nvim::eval::vars::{eval_spell_expr, get_spellword};
use crate::src::nvim::fileio::vim_fgets;
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_grow, ga_init};
use crate::src::nvim::getchar::{
    beep_flush, vgetc, AppendCharToRedobuff, AppendToRedobuff, AppendToRedobuffLit, ResetRedobuff,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::input::prompt_for_input;
use crate::src::nvim::main::{
    cmdline_row, cmdmsg_rl, curbuf, curwin, e_no_spell, e_notopen, got_int, lines_left, mouse_row,
    msg_col, msg_row, msg_scroll, p_sps, p_verbose, IObuff, Rows, VIsual, VIsual_active,
};
use crate::src::nvim::mbyte::{
    mb_charlen, mb_cptr2char_adv, mb_isupper, utf8len_tab, utf_char2bytes, utf_char2len, utf_fold,
    utf_head_off, utf_iscomposing_legacy, utf_ptr2char, utf_ptr2len, utfc_ptr2len,
};
use crate::src::nvim::memline::{ml_get_buf, ml_replace};
use crate::src::nvim::memory::{xfree, xmalloc, xmemcpyz, xmemdupz, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    emsg, internal_error, msg, msg_advance, msg_clr_eos, msg_ext_set_kind, msg_putchar, msg_puts,
    msg_start, semsg, smsg,
};
use crate::src::nvim::normal::end_visual_mode;
use crate::src::nvim::option::copy_option_part;
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::input::{line_breakcheck, os_breakcheck};
use crate::src::nvim::os::libc::{
    __assert_fail, atoi, fclose, gettext, memcpy, memmove, memset, qsort, strcasecmp, strcat,
    strcmp, strcpy, strlen, strncmp, strncpy,
};
use crate::src::nvim::profile::{profile_passed_limit, profile_setlimit};
use crate::src::nvim::spell::{
    allcap_copy, byte_in_str, can_compound, captype, check_need_cap, make_case_word,
    match_checkcompoundpattern, match_compoundrule, nofold_len, parse_spelllang, repl_from,
    repl_to, spell_casefold, spell_check, spell_iswordp, spell_iswordp_nmw, spell_move_to,
    spell_soundfold, spell_valid_case, spelltab, valid_word_prefix,
};
use crate::src::nvim::spellfile::suggest_load_files;
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, xstrnsave};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_1,
    ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection, LuaRef, MTKey,
    MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_11, Terminal, Timestamp, UIExtension,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, __compar_fn_t,
    __off64_t, __off_t, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T,
    chunksize_T, colnr_T, dict_T, dictvar_S, disptick_T, extmark_undo_vec_t, fcs_chars_T,
    file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_2,
    file_buffer_b_wininfo as C2Rust_Unnamed_10, file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    fromto_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T, garray_T, handle_T,
    hash_T, hashitem_T, hashtab_T, hlf_T, idx_T, infoptr_T, int16_t, int32_t, int64_t, langp_T,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T,
    ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T,
    salfirst_T, sattr_T, schar_T, scid_T, sctx_T, size_t, slang_S, slang_T, smt_T, spelltab_T,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T,
    taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header,
    u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7, u_header_uh_alt_prev as C2Rust_Unnamed_6,
    u_header_uh_next as C2Rust_Unnamed_9, u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, wordcount_T, xfmark_T, FILE, QUEUE, _IO_FILE,
};
use crate::src::nvim::ui::{ui_has, vim_beep};
use crate::src::nvim::undo::u_save_cursor;
extern "C" {
    fn hash_init(ht: *mut hashtab_T);
    fn hash_clear(ht: *mut hashtab_T);
    fn hash_clear_all(ht: *mut hashtab_T, off: ::core::ffi::c_uint);
    fn hash_find(ht: *const hashtab_T, key: *const ::core::ffi::c_char) -> *mut hashitem_T;
    fn hash_lookup(
        ht: *const hashtab_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        hash: hash_T,
    ) -> *mut hashitem_T;
    fn hash_add_item(
        ht: *mut hashtab_T,
        hi: *mut hashitem_T,
        key: *mut ::core::ffi::c_char,
        hash: hash_T,
    );
    fn hash_hash(key: *const ::core::ffi::c_char) -> hash_T;
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
pub type C2Rust_Unnamed_12 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_12 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_12 = 3;
pub const BACKWARD: C2Rust_Unnamed_12 = -1;
pub const FORWARD: C2Rust_Unnamed_12 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_12 = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_13 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_13 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_13 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_13 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_13 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_13 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_13 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_13 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_13 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_13 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_13 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_13 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_13 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_13 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_13 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_13 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_13 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_13 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_13 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_13 = 1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXWLEN: C2Rust_Unnamed_14 = 254;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const WF_CAPMASK: C2Rust_Unnamed_15 = 198;
pub const WF_KEEPCAP: C2Rust_Unnamed_15 = 128;
pub const WF_FIXCAP: C2Rust_Unnamed_15 = 64;
pub const WF_AFX: C2Rust_Unnamed_15 = 32;
pub const WF_BANNED: C2Rust_Unnamed_15 = 16;
pub const WF_RARE: C2Rust_Unnamed_15 = 8;
pub const WF_ALLCAP: C2Rust_Unnamed_15 = 4;
pub const WF_ONECAP: C2Rust_Unnamed_15 = 2;
pub const WF_REGION: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const WF_NOCOMPAFT: C2Rust_Unnamed_16 = 8192;
pub const WF_NOCOMPBEF: C2Rust_Unnamed_16 = 4096;
pub const WF_COMPROOT: C2Rust_Unnamed_16 = 2048;
pub const WF_NOSUGGEST: C2Rust_Unnamed_16 = 1024;
pub const WF_NEEDCOMP: C2Rust_Unnamed_16 = 512;
pub const WF_HAS_AFF: C2Rust_Unnamed_16 = 256;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const WF_PFX_COMPFORBID: C2Rust_Unnamed_17 = 268435456;
pub const WF_PFX_COMPPERMIT: C2Rust_Unnamed_17 = 134217728;
pub const WF_PFX_UP: C2Rust_Unnamed_17 = 67108864;
pub const WF_PFX_NC: C2Rust_Unnamed_17 = 33554432;
pub const WF_RAREPFX: C2Rust_Unnamed_17 = 16777216;
pub const SMT_RARE: smt_T = 2;
pub const SMT_BAD: smt_T = 1;
pub const SMT_ALL: smt_T = 0;
pub const SPS_BEST: C2Rust_Unnamed_26 = 1;
pub const SPS_DOUBLE: C2Rust_Unnamed_26 = 4;
pub const SPS_FAST: C2Rust_Unnamed_26 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct suginfo_T {
    pub su_ga: garray_T,
    pub su_maxcount: ::core::ffi::c_int,
    pub su_maxscore: ::core::ffi::c_int,
    pub su_sfmaxscore: ::core::ffi::c_int,
    pub su_sga: garray_T,
    pub su_badptr: *mut ::core::ffi::c_char,
    pub su_badlen: ::core::ffi::c_int,
    pub su_badflags: ::core::ffi::c_int,
    pub su_badword: [::core::ffi::c_char; 254],
    pub su_fbadword: [::core::ffi::c_char; 254],
    pub su_sal_badword: [::core::ffi::c_char; 254],
    pub su_banned: hashtab_T,
    pub su_sallang: *mut slang_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct suggest_T {
    pub st_word: *mut ::core::ffi::c_char,
    pub st_wordlen: ::core::ffi::c_int,
    pub st_orglen: ::core::ffi::c_int,
    pub st_score: ::core::ffi::c_int,
    pub st_altscore: ::core::ffi::c_int,
    pub st_salscore: bool,
    pub st_had_bonus: bool,
    pub st_slang: *mut slang_T,
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
pub const SCORE_INS: C2Rust_Unnamed_18 = 96;
pub const SCORE_MAXMAX: C2Rust_Unnamed_22 = 999999;
pub const SCORE_DEL: C2Rust_Unnamed_18 = 94;
pub const SCORE_SWAP: C2Rust_Unnamed_18 = 75;
pub const SCORE_SUBST: C2Rust_Unnamed_18 = 93;
pub const SCORE_SIMILAR: C2Rust_Unnamed_18 = 33;
pub const SCORE_ICASE: C2Rust_Unnamed_18 = 52;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sftword_T {
    pub sft_score: int16_t,
    pub sft_word: [uint8_t; 0],
}
pub const PFD_PREFIXTREE: C2Rust_Unnamed_25 = 254;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct trystate_T {
    pub ts_state: state_T,
    pub ts_score: ::core::ffi::c_int,
    pub ts_arridx: idx_T,
    pub ts_curi: int16_t,
    pub ts_fidx: uint8_t,
    pub ts_fidxtry: uint8_t,
    pub ts_twordlen: uint8_t,
    pub ts_prefixdepth: uint8_t,
    pub ts_flags: uint8_t,
    pub ts_tcharlen: uint8_t,
    pub ts_tcharidx: uint8_t,
    pub ts_isdiff: uint8_t,
    pub ts_fcharstart: uint8_t,
    pub ts_prewordlen: uint8_t,
    pub ts_splitoff: uint8_t,
    pub ts_splitfidx: uint8_t,
    pub ts_complen: uint8_t,
    pub ts_compsplit: uint8_t,
    pub ts_save_badflags: uint8_t,
    pub ts_delidx: uint8_t,
}
pub type state_T = ::core::ffi::c_uint;
pub const STATE_FINAL: state_T = 17;
pub const STATE_REP_UNDO: state_T = 16;
pub const STATE_REP: state_T = 15;
pub const STATE_REP_INI: state_T = 14;
pub const STATE_UNROT3R: state_T = 13;
pub const STATE_UNROT3L: state_T = 12;
pub const STATE_UNSWAP3: state_T = 11;
pub const STATE_SWAP3: state_T = 10;
pub const STATE_UNSWAP: state_T = 9;
pub const STATE_SWAP: state_T = 8;
pub const STATE_INS: state_T = 7;
pub const STATE_INS_PREP: state_T = 6;
pub const STATE_DEL: state_T = 5;
pub const STATE_PLAIN: state_T = 4;
pub const STATE_ENDNUL: state_T = 3;
pub const STATE_SPLITUNDO: state_T = 2;
pub const STATE_NOPREFIX: state_T = 1;
pub const STATE_START: state_T = 0;
pub const SCORE_REP: C2Rust_Unnamed_18 = 65;
pub const SCORE_SWAP3: C2Rust_Unnamed_18 = 110;
pub const SCORE_INSDUP: C2Rust_Unnamed_18 = 67;
pub const DIFF_INSERT: C2Rust_Unnamed_23 = 2;
pub const TSF_DIDDEL: C2Rust_Unnamed_24 = 4;
pub const SCORE_DELDUP: C2Rust_Unnamed_18 = 66;
pub const SCORE_DELCOMP: C2Rust_Unnamed_18 = 28;
pub const SCORE_INSCOMP: C2Rust_Unnamed_18 = 30;
pub const SCORE_SUBCOMP: C2Rust_Unnamed_18 = 33;
pub const DIFF_YES: C2Rust_Unnamed_23 = 1;
pub const DIFF_NONE: C2Rust_Unnamed_23 = 0;
pub const PFD_NOPREFIX: C2Rust_Unnamed_25 = 255;
pub const SCORE_SPLIT: C2Rust_Unnamed_18 = 149;
pub const SCORE_COMMON3: C2Rust_Unnamed_20 = 50;
pub const SCORE_COMMON2: C2Rust_Unnamed_20 = 40;
pub const SCORE_THRES3: C2Rust_Unnamed_20 = 100;
pub const SCORE_COMMON1: C2Rust_Unnamed_20 = 30;
pub const SCORE_THRES2: C2Rust_Unnamed_20 = 10;
pub const SCORE_SPLIT_NO: C2Rust_Unnamed_18 = 249;
pub const TSF_DIDSPLIT: C2Rust_Unnamed_24 = 2;
pub const SCORE_NONWORD: C2Rust_Unnamed_18 = 103;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct limitscore_T {
    pub badi: ::core::ffi::c_int,
    pub goodi: ::core::ffi::c_int,
    pub score: ::core::ffi::c_int,
}
pub const SCORE_LIMITMAX: C2Rust_Unnamed_22 = 350;
pub const SCORE_REGION: C2Rust_Unnamed_18 = 200;
pub const SCORE_RARE: C2Rust_Unnamed_18 = 180;
pub const TSF_PREFIXOK: C2Rust_Unnamed_24 = 1;
pub const PFD_NOTSPECIAL: C2Rust_Unnamed_25 = 253;
pub const SCORE_SFMAX3: C2Rust_Unnamed_21 = 400;
pub const SCORE_SFMAX2: C2Rust_Unnamed_21 = 300;
pub const SCORE_MAXINIT: C2Rust_Unnamed_19 = 350;
pub const SCORE_SFMAX1: C2Rust_Unnamed_21 = 200;
pub const SCORE_FILE: C2Rust_Unnamed_19 = 30;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const WC_KEY_OFF: ::core::ffi::c_ulong = 2 as ::core::ffi::c_ulong;
pub const WF_MIXCAP: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
static spell_suggest_timeout: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(5000 as ::core::ffi::c_int);
unsafe extern "C" fn can_be_compound(
    mut sp: *mut trystate_T,
    mut slang: *mut slang_T,
    mut compflags: *mut uint8_t,
    mut flag: ::core::ffi::c_int,
) -> bool {
    if !byte_in_str(
        if (*sp).ts_complen as ::core::ffi::c_int == (*sp).ts_compsplit as ::core::ffi::c_int {
            (*slang).sl_compstartflags
        } else {
            (*slang).sl_compallflags
        },
        flag,
    ) {
        return false_0 != 0;
    }
    if !(*slang).sl_comprules.is_null()
        && (*sp).ts_complen as ::core::ffi::c_int > (*sp).ts_compsplit as ::core::ffi::c_int
    {
        *compflags.offset((*sp).ts_complen as isize) = flag as uint8_t;
        *compflags
            .offset(((*sp).ts_complen as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
            NUL as uint8_t;
        let mut v: bool = match_compoundrule(
            slang,
            compflags.offset((*sp).ts_compsplit as ::core::ffi::c_int as isize),
        );
        *compflags.offset((*sp).ts_complen as isize) = NUL as uint8_t;
        return v;
    }
    return true_0 != 0;
}
unsafe extern "C" fn score_wordcount_adj(
    mut slang: *mut slang_T,
    mut score: ::core::ffi::c_int,
    mut word: *mut ::core::ffi::c_char,
    mut split: bool,
) -> ::core::ffi::c_int {
    let mut bonus: ::core::ffi::c_int = 0;
    let mut hi: *mut hashitem_T = hash_find(&raw mut (*slang).sl_wordcount, word);
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
    {
        return score;
    }
    let mut wc: *mut wordcount_T = (*hi).hi_key.offset(-(WC_KEY_OFF as isize)) as *mut wordcount_T;
    if ((*wc).wc_count as ::core::ffi::c_int) < SCORE_THRES2 as ::core::ffi::c_int {
        bonus = SCORE_COMMON1 as ::core::ffi::c_int;
    } else if ((*wc).wc_count as ::core::ffi::c_int) < SCORE_THRES3 as ::core::ffi::c_int {
        bonus = SCORE_COMMON2 as ::core::ffi::c_int;
    } else {
        bonus = SCORE_COMMON3 as ::core::ffi::c_int;
    }
    let mut newscore: ::core::ffi::c_int = if split as ::core::ffi::c_int != 0 {
        score - bonus / 2 as ::core::ffi::c_int
    } else {
        score - bonus
    };
    if newscore < 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return newscore;
}
unsafe extern "C" fn badword_captype(
    mut word: *mut ::core::ffi::c_char,
    mut end: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = captype(word, end);
    if flags & WF_KEEPCAP as ::core::ffi::c_int == 0 {
        return flags;
    }
    let mut l: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut u: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut first: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = word;
    while p < end {
        let mut c: ::core::ffi::c_int = utf_ptr2char(p);
        if if c >= 128 as ::core::ffi::c_int {
            mb_isupper(c) as ::core::ffi::c_int
        } else {
            (*spelltab.ptr()).st_isu[c as usize] as ::core::ffi::c_int
        } != 0
        {
            u += 1;
            if p == word {
                first = true_0 != 0;
            }
        } else {
            l += 1;
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if u > l && u > 2 as ::core::ffi::c_int {
        flags |= WF_ALLCAP as ::core::ffi::c_int;
    } else if first {
        flags |= WF_ONECAP as ::core::ffi::c_int;
    }
    if u >= 2 as ::core::ffi::c_int && l >= 2 as ::core::ffi::c_int {
        flags |= WF_MIXCAP;
    }
    return flags;
}
unsafe extern "C" fn bytes2offset(mut pp: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *mut uint8_t = *pp as *mut uint8_t;
    let mut nr: ::core::ffi::c_int = 0;
    let c2rust_fresh18 = p;
    p = p.offset(1);
    let mut c: ::core::ffi::c_int = *c2rust_fresh18 as ::core::ffi::c_int;
    if c & 0x80 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        nr = c - 1 as ::core::ffi::c_int;
    } else if c & 0xc0 as ::core::ffi::c_int == 0x80 as ::core::ffi::c_int {
        nr = (c & 0x3f as ::core::ffi::c_int) - 1 as ::core::ffi::c_int;
        let c2rust_fresh19 = p;
        p = p.offset(1);
        nr = nr * 255 as ::core::ffi::c_int
            + (*c2rust_fresh19 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    } else if c & 0xe0 as ::core::ffi::c_int == 0xc0 as ::core::ffi::c_int {
        nr = (c & 0x1f as ::core::ffi::c_int) - 1 as ::core::ffi::c_int;
        let c2rust_fresh20 = p;
        p = p.offset(1);
        nr = nr * 255 as ::core::ffi::c_int
            + (*c2rust_fresh20 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        let c2rust_fresh21 = p;
        p = p.offset(1);
        nr = nr * 255 as ::core::ffi::c_int
            + (*c2rust_fresh21 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    } else {
        nr = (c & 0xf as ::core::ffi::c_int) - 1 as ::core::ffi::c_int;
        let c2rust_fresh22 = p;
        p = p.offset(1);
        nr = nr * 255 as ::core::ffi::c_int
            + (*c2rust_fresh22 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        let c2rust_fresh23 = p;
        p = p.offset(1);
        nr = nr * 255 as ::core::ffi::c_int
            + (*c2rust_fresh23 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        let c2rust_fresh24 = p;
        p = p.offset(1);
        nr = nr * 255 as ::core::ffi::c_int
            + (*c2rust_fresh24 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
    *pp = p as *mut ::core::ffi::c_char;
    return nr;
}
static sps_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(SPS_BEST as ::core::ffi::c_int);
static sps_limit: GlobalCell<::core::ffi::c_int> = GlobalCell::new(9999 as ::core::ffi::c_int);
pub unsafe extern "C" fn spell_check_sps() -> ::core::ffi::c_int {
    let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    sps_flags.set(0 as ::core::ffi::c_int);
    sps_limit.set(9999 as ::core::ffi::c_int);
    let mut p: *mut ::core::ffi::c_char = p_sps.get();
    while *p as ::core::ffi::c_int != NUL {
        copy_option_part(
            &raw mut p,
            &raw mut buf as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        let mut f: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if ascii_isdigit(*(&raw mut buf as *mut ::core::ffi::c_char) as ::core::ffi::c_int) {
            let mut s: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
            sps_limit.set(getdigits_int(
                &raw mut s,
                true_0 != 0,
                0 as ::core::ffi::c_int,
            ));
            if *s as ::core::ffi::c_int != NUL && !ascii_isdigit(*s as ::core::ffi::c_int) {
                f = -1 as ::core::ffi::c_int;
            }
        } else if strcmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"best\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            f = SPS_BEST as ::core::ffi::c_int;
        } else if strcmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"fast\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            f = SPS_FAST as ::core::ffi::c_int;
        } else if strcmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"double\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            f = SPS_DOUBLE as ::core::ffi::c_int;
        } else if strncmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"expr:\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) != 0 as ::core::ffi::c_int
            && strncmp(
                &raw mut buf as *mut ::core::ffi::c_char,
                b"file:\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) != 0 as ::core::ffi::c_int
            && (strncmp(
                &raw mut buf as *mut ::core::ffi::c_char,
                b"timeout:\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) != 0 as ::core::ffi::c_int
                || !ascii_isdigit(buf[8 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
                    && !(buf[8 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                        == '-' as ::core::ffi::c_int
                        && ascii_isdigit(
                            buf[9 as ::core::ffi::c_int as usize] as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                            != 0))
        {
            f = -1 as ::core::ffi::c_int;
        }
        if f == -1 as ::core::ffi::c_int
            || sps_flags.get() != 0 as ::core::ffi::c_int && f != 0 as ::core::ffi::c_int
        {
            sps_flags.set(SPS_BEST as ::core::ffi::c_int);
            sps_limit.set(9999 as ::core::ffi::c_int);
            return FAIL;
        }
        if f != 0 as ::core::ffi::c_int {
            sps_flags.set(f);
        }
    }
    if sps_flags.get() == 0 as ::core::ffi::c_int {
        sps_flags.set(SPS_BEST as ::core::ffi::c_int);
    }
    return OK;
}
pub unsafe extern "C" fn spell_suggest(mut count: ::core::ffi::c_int) {
    let mut need_cap: ::core::ffi::c_int = 0;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sug: suginfo_T = suginfo_T {
        su_ga: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        su_maxcount: 0,
        su_maxscore: 0,
        su_sfmaxscore: 0,
        su_sga: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        su_badptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        su_badlen: 0,
        su_badflags: 0,
        su_badword: [0; 254],
        su_fbadword: [0; 254],
        su_sal_badword: [0; 254],
        su_banned: hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            }; 16],
        },
        su_sallang: ::core::ptr::null_mut::<slang_T>(),
    };
    let mut limit: ::core::ffi::c_int = 0;
    let mut selected: ::core::ffi::c_int = 0;
    let mut prev_cursor: pos_T = (*curwin.get()).w_cursor;
    let mut mouse_used: bool = false_0 != 0;
    let mut badlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut msg_scroll_save: ::core::ffi::c_int = msg_scroll.get();
    let wo_spell_save: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_spell;
    if (*curwin.get()).w_onebuf_opt.wo_spell == 0 {
        parse_spelllang(curwin.get());
        (*curwin.get()).w_onebuf_opt.wo_spell = true_0;
    }
    '_skip: {
        if *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int == NUL {
            emsg(gettext(&raw const e_no_spell as *const ::core::ffi::c_char));
        } else {
            if VIsual_active.get() {
                if (*curwin.get()).w_cursor.lnum != (*VIsual.ptr()).lnum {
                    vim_beep(kOptBoFlagSpell as ::core::ffi::c_int as ::core::ffi::c_uint);
                    break '_skip;
                } else {
                    badlen = (*curwin.get()).w_cursor.col - (*VIsual.ptr()).col;
                    if badlen < 0 as ::core::ffi::c_int {
                        badlen = -badlen;
                    } else {
                        (*curwin.get()).w_cursor.col = (*VIsual.ptr()).col;
                    }
                    badlen += 1;
                    end_visual_mode();
                    badlen = if badlen < get_cursor_line_len() - (*curwin.get()).w_cursor.col {
                        badlen
                    } else {
                        get_cursor_line_len() - (*curwin.get()).w_cursor.col as ::core::ffi::c_int
                    };
                }
            } else if spell_move_to(
                curwin.get(),
                FORWARD as ::core::ffi::c_int,
                SMT_ALL,
                true_0 != 0,
                ::core::ptr::null_mut::<hlf_T>(),
            ) == 0 as size_t
                || (*curwin.get()).w_cursor.col > prev_cursor.col
            {
                (*curwin.get()).w_cursor = prev_cursor;
                let mut curline: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                let mut p: *mut ::core::ffi::c_char =
                    curline.offset((*curwin.get()).w_cursor.col as isize);
                while p > curline && spell_iswordp_nmw(p, curwin.get()) as ::core::ffi::c_int != 0 {
                    p = p.offset(
                        -((utf_head_off(curline, p.offset(-(1 as ::core::ffi::c_int as isize)))
                            + 1 as ::core::ffi::c_int) as isize),
                    );
                }
                while *p as ::core::ffi::c_int != NUL && !spell_iswordp_nmw(p, curwin.get()) {
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
                if !spell_iswordp_nmw(p, curwin.get()) {
                    beep_flush();
                    break '_skip;
                } else {
                    (*curwin.get()).w_cursor.col = p.offset_from(curline) as colnr_T;
                }
            }
            need_cap = check_need_cap(
                curwin.get(),
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.col,
            ) as ::core::ffi::c_int;
            line = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
            spell_suggest_timeout.set(5000 as ::core::ffi::c_int);
            sug = suginfo_T {
                su_ga: garray_T {
                    ga_len: 0,
                    ga_maxlen: 0,
                    ga_itemsize: 0,
                    ga_growsize: 0,
                    ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                },
                su_maxcount: 0,
                su_maxscore: 0,
                su_sfmaxscore: 0,
                su_sga: garray_T {
                    ga_len: 0,
                    ga_maxlen: 0,
                    ga_itemsize: 0,
                    ga_growsize: 0,
                    ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                },
                su_badptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                su_badlen: 0,
                su_badflags: 0,
                su_badword: [0; 254],
                su_fbadword: [0; 254],
                su_sal_badword: [0; 254],
                su_banned: hashtab_T {
                    ht_mask: 0,
                    ht_used: 0,
                    ht_filled: 0,
                    ht_changed: 0,
                    ht_locked: 0,
                    ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                    ht_smallarray: [hashitem_T {
                        hi_hash: 0,
                        hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    }; 16],
                },
                su_sallang: ::core::ptr::null_mut::<slang_T>(),
            };
            limit = if sps_limit.get() < Rows.get() - 2 as ::core::ffi::c_int {
                sps_limit.get()
            } else {
                Rows.get() - 2 as ::core::ffi::c_int
            };
            spell_find_suggest(
                line.offset((*curwin.get()).w_cursor.col as isize),
                badlen,
                &raw mut sug,
                limit,
                true_0 != 0,
                need_cap != 0,
                true_0 != 0,
            );
            selected = count;
            msg_ext_set_kind(b"confirm\0".as_ptr() as *const ::core::ffi::c_char);
            if sug.su_ga.ga_len <= 0 as ::core::ffi::c_int {
                msg(
                    gettext(b"No suggestions\0".as_ptr() as *const ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                );
            } else if count > 0 as ::core::ffi::c_int {
                if count > sug.su_ga.ga_len {
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"Only %ld suggestions\0".as_ptr() as *const ::core::ffi::c_char),
                        sug.su_ga.ga_len as int64_t,
                    );
                }
            } else {
                cmdmsg_rl.set((*curwin.get()).w_onebuf_opt.wo_rl != 0);
                msg_start();
                msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
                lines_left.set(Rows.get());
                let mut fmt: *mut ::core::ffi::c_char =
                    gettext(b"Change \"%.*s\" to:\0".as_ptr() as *const ::core::ffi::c_char);
                if cmdmsg_rl.get() as ::core::ffi::c_int != 0
                    && strncmp(
                        fmt,
                        b"Change\0".as_ptr() as *const ::core::ffi::c_char,
                        6 as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    fmt = b":ot \"%.*s\" egnahC\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    fmt,
                    sug.su_badlen,
                    sug.su_badptr,
                );
                msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
                msg_clr_eos();
                msg_putchar('\n' as ::core::ffi::c_int);
                msg_scroll.set(true_0);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < sug.su_ga.ga_len {
                    let mut stp: *mut suggest_T =
                        (sug.su_ga.ga_data as *mut suggest_T).offset(i as isize);
                    let mut wcopy: [::core::ffi::c_char; 256] = [0; 256];
                    xstrlcpy(
                        &raw mut wcopy as *mut ::core::ffi::c_char,
                        (*stp).st_word,
                        (MAXWLEN as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
                    );
                    let mut el: ::core::ffi::c_int = sug.su_badlen - (*stp).st_orglen;
                    if el > 0 as ::core::ffi::c_int
                        && (*stp).st_wordlen + el <= MAXWLEN as ::core::ffi::c_int
                    {
                        '_c2rust_label: {
                            if !sug.su_badptr.is_null() {
                            } else {
                                __assert_fail(
                                    b"sug.su_badptr != NULL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/spellsuggest.rs\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    552 as ::core::ffi::c_uint,
                                    b"void spell_suggest(int)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        xmemcpyz(
                            (&raw mut wcopy as *mut ::core::ffi::c_char)
                                .offset((*stp).st_wordlen as isize)
                                as *mut ::core::ffi::c_void,
                            sug.su_badptr.offset((*stp).st_orglen as isize)
                                as *const ::core::ffi::c_void,
                            el as size_t,
                        );
                    }
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        b"%2d\0".as_ptr() as *const ::core::ffi::c_char,
                        i + 1 as ::core::ffi::c_int,
                    );
                    if cmdmsg_rl.get() {
                        rl_mirror_ascii(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        );
                    }
                    msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        b" \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                        &raw mut wcopy as *mut ::core::ffi::c_char,
                    );
                    msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
                    if sug.su_badlen < (*stp).st_orglen {
                        vim_snprintf(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            IOSIZE as size_t,
                            gettext(b" < \"%.*s\"\0".as_ptr() as *const ::core::ffi::c_char),
                            (*stp).st_orglen,
                            sug.su_badptr,
                        );
                        msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
                    }
                    if p_verbose.get() > 0 as OptInt {
                        if sps_flags.get()
                            & (SPS_DOUBLE as ::core::ffi::c_int | SPS_BEST as ::core::ffi::c_int)
                            != 0
                        {
                            vim_snprintf(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                IOSIZE as size_t,
                                b" (%s%d - %d)\0".as_ptr() as *const ::core::ffi::c_char,
                                if (*stp).st_salscore as ::core::ffi::c_int != 0 {
                                    b"s \0".as_ptr() as *const ::core::ffi::c_char
                                } else {
                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                },
                                (*stp).st_score,
                                (*stp).st_altscore,
                            );
                        } else {
                            vim_snprintf(
                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                IOSIZE as size_t,
                                b" (%d)\0".as_ptr() as *const ::core::ffi::c_char,
                                (*stp).st_score,
                            );
                        }
                        if cmdmsg_rl.get() {
                            rl_mirror_ascii(
                                (IObuff.ptr() as *mut ::core::ffi::c_char)
                                    .offset(1 as ::core::ffi::c_int as isize),
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            );
                        }
                        msg_advance(30 as ::core::ffi::c_int);
                        msg_puts(IObuff.ptr() as *mut ::core::ffi::c_char);
                    }
                    if !ui_has(kUIMessages) || i < sug.su_ga.ga_len - 1 as ::core::ffi::c_int {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    }
                    i += 1;
                }
                cmdmsg_rl.set(false_0 != 0);
                msg_col.set(0 as ::core::ffi::c_int);
                selected = prompt_for_input(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                    &raw mut mouse_used,
                );
                if mouse_used {
                    selected = sug.su_ga.ga_len + 1 as ::core::ffi::c_int
                        - (cmdline_row.get() - mouse_row.get());
                }
                lines_left.set(Rows.get());
                msg_scroll.set(msg_scroll_save);
            }
            if selected > 0 as ::core::ffi::c_int
                && selected <= sug.su_ga.ga_len
                && u_save_cursor() == OK
            {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    repl_from.ptr() as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                let _ = *ptr_;
                let mut ptr__0: *mut *mut ::core::ffi::c_void =
                    repl_to.ptr() as *mut *mut ::core::ffi::c_void;
                xfree(*ptr__0);
                *ptr__0 = NULL;
                let _ = *ptr__0;
                let mut stp_0: *mut suggest_T = (sug.su_ga.ga_data as *mut suggest_T)
                    .offset((selected - 1 as ::core::ffi::c_int) as isize);
                if sug.su_badlen > (*stp_0).st_orglen {
                    repl_from.set(xstrnsave(sug.su_badptr, sug.su_badlen as size_t));
                    vim_snprintf(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        IOSIZE as size_t,
                        b"%s%.*s\0".as_ptr() as *const ::core::ffi::c_char,
                        (*stp_0).st_word,
                        sug.su_badlen - (*stp_0).st_orglen,
                        sug.su_badptr.offset((*stp_0).st_orglen as isize),
                    );
                    repl_to.set(xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char));
                } else {
                    repl_from.set(xstrnsave(sug.su_badptr, (*stp_0).st_orglen as size_t));
                    repl_to.set(xstrdup((*stp_0).st_word));
                }
                let mut p_0: *mut ::core::ffi::c_char = xmalloc(
                    strlen(line)
                        .wrapping_sub((*stp_0).st_orglen as size_t)
                        .wrapping_add((*stp_0).st_wordlen as size_t)
                        .wrapping_add(1 as size_t),
                )
                    as *mut ::core::ffi::c_char;
                let mut c: ::core::ffi::c_int =
                    sug.su_badptr.offset_from(line) as ::core::ffi::c_int;
                memmove(
                    p_0 as *mut ::core::ffi::c_void,
                    line as *const ::core::ffi::c_void,
                    c as size_t,
                );
                strcpy(p_0.offset(c as isize), (*stp_0).st_word);
                strcat(p_0, sug.su_badptr.offset((*stp_0).st_orglen as isize));
                ResetRedobuff();
                AppendToRedobuff(b"ciw\0".as_ptr() as *const ::core::ffi::c_char);
                AppendToRedobuffLit(
                    p_0.offset(c as isize),
                    (*stp_0).st_wordlen + sug.su_badlen - (*stp_0).st_orglen,
                );
                AppendCharToRedobuff(ESC);
                ml_replace((*curwin.get()).w_cursor.lnum, p_0, false_0 != 0);
                (*curwin.get()).w_cursor.col = c as colnr_T;
                inserted_bytes(
                    (*curwin.get()).w_cursor.lnum,
                    c as colnr_T,
                    (*stp_0).st_orglen,
                    (*stp_0).st_wordlen,
                );
            } else {
                (*curwin.get()).w_cursor = prev_cursor;
            }
            spell_find_cleanup(&raw mut sug);
            xfree(line as *mut ::core::ffi::c_void);
        }
    }
    (*curwin.get()).w_onebuf_opt.wo_spell = wo_spell_save;
}
pub unsafe extern "C" fn spell_suggest_list(
    mut gap: *mut garray_T,
    mut word: *mut ::core::ffi::c_char,
    mut maxcount: ::core::ffi::c_int,
    mut need_cap: bool,
    mut interactive: bool,
) {
    let mut sug: suginfo_T = suginfo_T {
        su_ga: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        su_maxcount: 0,
        su_maxscore: 0,
        su_sfmaxscore: 0,
        su_sga: garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        su_badptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        su_badlen: 0,
        su_badflags: 0,
        su_badword: [0; 254],
        su_fbadword: [0; 254],
        su_sal_badword: [0; 254],
        su_banned: hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            }; 16],
        },
        su_sallang: ::core::ptr::null_mut::<slang_T>(),
    };
    spell_find_suggest(
        word,
        0 as ::core::ffi::c_int,
        &raw mut sug,
        maxcount,
        false_0 != 0,
        need_cap,
        interactive,
    );
    ga_init(
        gap,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        sug.su_ga.ga_len + 1 as ::core::ffi::c_int,
    );
    ga_grow(gap, sug.su_ga.ga_len);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < sug.su_ga.ga_len {
        let mut stp: *mut suggest_T = (sug.su_ga.ga_data as *mut suggest_T).offset(i as isize);
        let mut wcopy: *mut ::core::ffi::c_char = xmalloc(
            ((*stp).st_wordlen as size_t)
                .wrapping_add(strlen(sug.su_badptr.offset((*stp).st_orglen as isize)))
                .wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        strcpy(wcopy, (*stp).st_word);
        strcpy(
            wcopy.offset((*stp).st_wordlen as isize),
            sug.su_badptr.offset((*stp).st_orglen as isize),
        );
        let c2rust_fresh26 = (*gap).ga_len;
        (*gap).ga_len = (*gap).ga_len + 1;
        let c2rust_lvalue_ptr = &raw mut *((*gap).ga_data as *mut *mut ::core::ffi::c_char)
            .offset(c2rust_fresh26 as isize);
        *c2rust_lvalue_ptr = wcopy;
        i += 1;
    }
    spell_find_cleanup(&raw mut sug);
}
unsafe extern "C" fn spell_find_suggest(
    mut badptr: *mut ::core::ffi::c_char,
    mut badlen: ::core::ffi::c_int,
    mut su: *mut suginfo_T,
    mut maxcount: ::core::ffi::c_int,
    mut banbadword: bool,
    mut need_cap: bool,
    mut interactive: bool,
) {
    let mut attr: hlf_T = HLF_COUNT;
    let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut do_combine: bool = false_0 != 0;
    static expr_busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut did_intern: bool = false_0 != 0;
    memset(
        su as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<suginfo_T>(),
    );
    ga_init(
        &raw mut (*su).su_ga,
        ::core::mem::size_of::<suggest_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut (*su).su_sga,
        ::core::mem::size_of::<suggest_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    if *badptr as ::core::ffi::c_int == NUL {
        return;
    }
    hash_init(&raw mut (*su).su_banned);
    (*su).su_badptr = badptr;
    if badlen != 0 as ::core::ffi::c_int {
        (*su).su_badlen = badlen;
    } else {
        let mut tmplen: size_t = spell_check(
            curwin.get(),
            (*su).su_badptr,
            &raw mut attr,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            false_0 != 0,
        );
        '_c2rust_label: {
            if tmplen <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"tmplen <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/spellsuggest.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    715 as ::core::ffi::c_uint,
                    b"void spell_find_suggest(char *, int, suginfo_T *, int, _Bool, _Bool, _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        (*su).su_badlen = tmplen as ::core::ffi::c_int;
    }
    (*su).su_maxcount = maxcount;
    (*su).su_maxscore = SCORE_MAXINIT as ::core::ffi::c_int;
    (*su).su_badlen = if (*su).su_badlen < MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        (*su).su_badlen
    } else {
        MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    };
    xmemcpyz(
        &raw mut (*su).su_badword as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        (*su).su_badptr as *const ::core::ffi::c_void,
        (*su).su_badlen as size_t,
    );
    spell_casefold(
        curwin.get(),
        (*su).su_badptr,
        (*su).su_badlen,
        &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
        MAXWLEN as ::core::ffi::c_int,
    );
    (*su).su_fbadword[(*su).su_badlen as usize] = NUL as ::core::ffi::c_char;
    (*su).su_badflags = badword_captype(
        (*su).su_badptr,
        (*su).su_badptr.offset((*su).su_badlen as isize),
    );
    if need_cap {
        (*su).su_badflags |= WF_ONECAP as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*curbuf.get()).b_s.b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*curbuf.get()).b_s.b_langp.ga_data as *mut langp_T).offset(i as isize);
        if !(*lp).lp_sallang.is_null() {
            (*su).su_sallang = (*lp).lp_sallang;
            break;
        } else {
            i += 1;
        }
    }
    if !(*su).su_sallang.is_null() {
        spell_soundfold(
            (*su).su_sallang,
            &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
            true_0 != 0,
            &raw mut (*su).su_sal_badword as *mut ::core::ffi::c_char,
        );
    }
    let mut c: ::core::ffi::c_int = utf_ptr2char((*su).su_badptr);
    if (if c >= 128 as ::core::ffi::c_int {
        mb_isupper(c) as ::core::ffi::c_int
    } else {
        (*spelltab.ptr()).st_isu[c as usize] as ::core::ffi::c_int
    }) == 0
        && attr as ::core::ffi::c_uint == HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        make_case_word(
            &raw mut (*su).su_badword as *mut ::core::ffi::c_char,
            &raw mut buf as *mut ::core::ffi::c_char,
            WF_ONECAP as ::core::ffi::c_int,
        );
        add_suggestion(
            su,
            &raw mut (*su).su_ga,
            &raw mut buf as *mut ::core::ffi::c_char,
            (*su).su_badlen,
            SCORE_ICASE as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            (*su).su_sallang,
            false_0 != 0,
        );
    }
    if banbadword {
        add_banned(su, &raw mut (*su).su_badword as *mut ::core::ffi::c_char);
    }
    let mut sps_copy: *mut ::core::ffi::c_char = xstrdup(p_sps.get());
    let mut p: *mut ::core::ffi::c_char = sps_copy;
    while *p as ::core::ffi::c_int != NUL {
        copy_option_part(
            &raw mut p,
            &raw mut buf as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if strncmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"expr:\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            if !expr_busy.get() {
                expr_busy.set(true_0 != 0);
                spell_suggest_expr(
                    su,
                    (&raw mut buf as *mut ::core::ffi::c_char)
                        .offset(5 as ::core::ffi::c_int as isize),
                );
                expr_busy.set(false_0 != 0);
            }
        } else if strncmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"file:\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            spell_suggest_file(
                su,
                (&raw mut buf as *mut ::core::ffi::c_char).offset(5 as ::core::ffi::c_int as isize),
            );
        } else if strncmp(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"timeout:\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            spell_suggest_timeout.set(atoi(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(8 as ::core::ffi::c_int as isize),
            ));
        } else if !did_intern {
            spell_suggest_intern(su, interactive);
            if sps_flags.get() & SPS_DOUBLE as ::core::ffi::c_int != 0 {
                do_combine = true_0 != 0;
            }
            did_intern = true_0 != 0;
        }
    }
    xfree(sps_copy as *mut ::core::ffi::c_void);
    if do_combine {
        score_combine(su);
    }
}
unsafe extern "C" fn spell_suggest_expr(
    mut su: *mut suginfo_T,
    mut expr: *mut ::core::ffi::c_char,
) {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let list: *mut list_T =
        eval_spell_expr(&raw mut (*su).su_badword as *mut ::core::ffi::c_char, expr);
    if !list.is_null() {
        let l_: *mut list_T = list;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if (*li).li_tv.v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut score: ::core::ffi::c_int =
                        get_spellword((*li).li_tv.vval.v_list, &raw mut p);
                    if score >= 0 as ::core::ffi::c_int && score <= (*su).su_maxscore {
                        add_suggestion(
                            su,
                            &raw mut (*su).su_ga,
                            p,
                            (*su).su_badlen,
                            score,
                            0 as ::core::ffi::c_int,
                            true,
                            (*su).su_sallang,
                            false,
                        );
                    }
                }
                li = (*li).li_next;
            }
        }
        tv_list_unref(list);
    }
    check_suggestions(su, &raw mut (*su).su_ga);
    cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
}
unsafe extern "C" fn spell_suggest_file(
    mut su: *mut suginfo_T,
    mut fname: *mut ::core::ffi::c_char,
) {
    let mut line: [::core::ffi::c_char; 508] = [0; 508];
    let mut len: ::core::ffi::c_int = 0;
    let mut cword: [::core::ffi::c_char; 254] = [0; 254];
    let mut fd: *mut FILE = os_fopen(fname, b"r\0".as_ptr() as *const ::core::ffi::c_char);
    if fd.is_null() {
        semsg(
            gettext(&raw const e_notopen as *const ::core::ffi::c_char),
            fname,
        );
        return;
    }
    while !vim_fgets(
        &raw mut line as *mut ::core::ffi::c_char,
        MAXWLEN as ::core::ffi::c_int * 2 as ::core::ffi::c_int,
        fd,
    ) && !got_int.get()
    {
        line_breakcheck();
        let mut p: *mut ::core::ffi::c_char = vim_strchr(
            &raw mut line as *mut ::core::ffi::c_char,
            '/' as ::core::ffi::c_int,
        );
        if p.is_null() {
            continue;
        }
        let c2rust_fresh25 = p;
        p = p.offset(1);
        *c2rust_fresh25 = NUL as ::core::ffi::c_char;
        if strcasecmp(
            &raw mut (*su).su_badword as *mut ::core::ffi::c_char,
            &raw mut line as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            len = 0 as ::core::ffi::c_int;
            while (*p.offset(len as isize) as uint8_t as ::core::ffi::c_int)
                >= ' ' as ::core::ffi::c_int
            {
                len += 1;
            }
            *p.offset(len as isize) = NUL as ::core::ffi::c_char;
            if captype(p, ::core::ptr::null::<::core::ffi::c_char>()) == 0 as ::core::ffi::c_int {
                make_case_word(
                    p,
                    &raw mut cword as *mut ::core::ffi::c_char,
                    (*su).su_badflags,
                );
                p = &raw mut cword as *mut ::core::ffi::c_char;
            }
            add_suggestion(
                su,
                &raw mut (*su).su_ga,
                p,
                (*su).su_badlen,
                SCORE_FILE as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                (*su).su_sallang,
                false_0 != 0,
            );
        }
    }
    fclose(fd);
    check_suggestions(su, &raw mut (*su).su_ga);
    cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
}
unsafe extern "C" fn spell_suggest_intern(mut su: *mut suginfo_T, mut interactive: bool) {
    suggest_load_files();
    suggest_try_special(su);
    suggest_try_change(su);
    if sps_flags.get() & SPS_DOUBLE as ::core::ffi::c_int != 0 {
        score_comp_sal(su);
    }
    if sps_flags.get() & SPS_FAST as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if sps_flags.get() & SPS_BEST as ::core::ffi::c_int != 0 {
            rescore_suggestions(su);
        }
        suggest_try_soundalike_prep();
        (*su).su_maxscore = SCORE_SFMAX1 as ::core::ffi::c_int;
        (*su).su_sfmaxscore = SCORE_MAXINIT as ::core::ffi::c_int * 3 as ::core::ffi::c_int;
        suggest_try_soundalike(su);
        if (*su).su_ga.ga_len
            < (if (*su).su_maxcount < 130 as ::core::ffi::c_int {
                150 as ::core::ffi::c_int
            } else {
                (*su).su_maxcount + 20 as ::core::ffi::c_int
            })
        {
            (*su).su_maxscore = SCORE_SFMAX2 as ::core::ffi::c_int;
            suggest_try_soundalike(su);
            if (*su).su_ga.ga_len
                < (if (*su).su_maxcount < 130 as ::core::ffi::c_int {
                    150 as ::core::ffi::c_int
                } else {
                    (*su).su_maxcount + 20 as ::core::ffi::c_int
                })
            {
                (*su).su_maxscore = SCORE_SFMAX3 as ::core::ffi::c_int;
                suggest_try_soundalike(su);
            }
        }
        (*su).su_maxscore = (*su).su_sfmaxscore;
        suggest_try_soundalike_finish();
    }
    os_breakcheck();
    if interactive as ::core::ffi::c_int != 0 && got_int.get() as ::core::ffi::c_int != 0 {
        vgetc();
        got_int.set(false_0 != 0);
    }
    if sps_flags.get() & SPS_DOUBLE as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && (*su).su_ga.ga_len != 0 as ::core::ffi::c_int
    {
        if sps_flags.get() & SPS_BEST as ::core::ffi::c_int != 0 {
            rescore_suggestions(su);
        }
        check_suggestions(su, &raw mut (*su).su_ga);
        cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
    }
}
unsafe extern "C" fn spell_find_cleanup(mut su: *mut suginfo_T) {
    let mut _gap: *mut garray_T = &raw mut (*su).su_ga;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut suggest_T = ((*_gap).ga_data as *mut suggest_T).offset(i as isize);
            xfree((*_item).st_word as *mut ::core::ffi::c_void);
            i += 1;
        }
    }
    ga_clear(_gap);
    let mut _gap_0: *mut garray_T = &raw mut (*su).su_sga;
    if !(*_gap_0).ga_data.is_null() {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*_gap_0).ga_len {
            let mut _item_0: *mut suggest_T =
                ((*_gap_0).ga_data as *mut suggest_T).offset(i_0 as isize);
            xfree((*_item_0).st_word as *mut ::core::ffi::c_void);
            i_0 += 1;
        }
    }
    ga_clear(_gap_0);
    hash_clear_all(&raw mut (*su).su_banned, 0 as ::core::ffi::c_uint);
}
unsafe extern "C" fn suggest_try_special(mut su: *mut suginfo_T) {
    let mut word: [::core::ffi::c_char; 254] = [0; 254];
    let mut p: *mut ::core::ffi::c_char =
        skiptowhite(&raw mut (*su).su_fbadword as *mut ::core::ffi::c_char);
    let mut len: size_t =
        p.offset_from(&raw mut (*su).su_fbadword as *mut ::core::ffi::c_char) as size_t;
    p = skipwhite(p);
    if strlen(p) == len
        && strncmp(
            &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
            p,
            len,
        ) == 0 as ::core::ffi::c_int
    {
        let mut c: ::core::ffi::c_char = (*su).su_fbadword[len as usize];
        (*su).su_fbadword[len as usize] = NUL as ::core::ffi::c_char;
        make_case_word(
            &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
            &raw mut word as *mut ::core::ffi::c_char,
            (*su).su_badflags,
        );
        (*su).su_fbadword[len as usize] = c;
        add_suggestion(
            su,
            &raw mut (*su).su_ga,
            &raw mut word as *mut ::core::ffi::c_char,
            (*su).su_badlen,
            (3 as ::core::ffi::c_int * SCORE_REP as ::core::ffi::c_int + 0 as ::core::ffi::c_int)
                / 4 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            (*su).su_sallang,
            false_0 != 0,
        );
    }
}
unsafe extern "C" fn suggest_try_change(mut su: *mut suginfo_T) {
    let mut fword: [::core::ffi::c_char; 254] = [0; 254];
    strcpy(
        &raw mut fword as *mut ::core::ffi::c_char,
        &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
    );
    let mut n: ::core::ffi::c_int =
        strlen(&raw mut fword as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = (*su).su_badptr.offset((*su).su_badlen as isize);
    spell_casefold(
        curwin.get(),
        p,
        strlen(p) as ::core::ffi::c_int,
        (&raw mut fword as *mut ::core::ffi::c_char).offset(n as isize),
        MAXWLEN as ::core::ffi::c_int - n,
    );
    n = strlen((*su).su_badptr) as ::core::ffi::c_int;
    if n < MAXWLEN as ::core::ffi::c_int {
        fword[n as usize] = NUL as ::core::ffi::c_char;
    }
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        if !(*(*lp).lp_slang).sl_fbyts.is_null() {
            suggest_trie_walk(
                su,
                lp,
                &raw mut fword as *mut ::core::ffi::c_char,
                false_0 != 0,
            );
        }
        lpi += 1;
    }
}
unsafe extern "C" fn suggest_trie_walk(
    mut su: *mut suginfo_T,
    mut lp: *mut langp_T,
    mut fword: *mut ::core::ffi::c_char,
    mut soundfold: bool,
) {
    let mut tword: [::core::ffi::c_char; 254] = [0; 254];
    let mut stack: [trystate_T; 254] = [trystate_T {
        ts_state: STATE_START,
        ts_score: 0,
        ts_arridx: 0,
        ts_curi: 0,
        ts_fidx: 0,
        ts_fidxtry: 0,
        ts_twordlen: 0,
        ts_prefixdepth: 0,
        ts_flags: 0,
        ts_tcharlen: 0,
        ts_tcharidx: 0,
        ts_isdiff: 0,
        ts_fcharstart: 0,
        ts_prewordlen: 0,
        ts_splitoff: 0,
        ts_splitfidx: 0,
        ts_complen: 0,
        ts_compsplit: 0,
        ts_save_badflags: 0,
        ts_delidx: 0,
    }; 254];
    let mut preword: [::core::ffi::c_char; 762] = [
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
    let mut compflags: [uint8_t; 254] = [0; 254];
    let mut byts: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut fbyts: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut pbyts: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut idxs: *mut idx_T = ::core::ptr::null_mut::<idx_T>();
    let mut fidxs: *mut idx_T = ::core::ptr::null_mut::<idx_T>();
    let mut pidxs: *mut idx_T = ::core::ptr::null_mut::<idx_T>();
    let mut c: ::core::ffi::c_int = 0;
    let mut c2: ::core::ffi::c_int = 0;
    let mut c3: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut arridx: idx_T = 0;
    let mut fl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tl: ::core::ffi::c_int = 0;
    let mut repextra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut slang: *mut slang_T = (*lp).lp_slang;
    let mut goodword_ends: bool = false;
    let mut breakcheckcount: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
    let mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sp: *mut trystate_T =
        (&raw mut stack as *mut trystate_T).offset(0 as ::core::ffi::c_int as isize);
    memset(
        sp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<trystate_T>(),
    );
    (*sp).ts_curi = 1 as int16_t;
    if soundfold {
        fbyts = (*slang).sl_sbyts;
        byts = fbyts;
        fidxs = (*slang).sl_sidxs;
        idxs = fidxs;
        pbyts = ::core::ptr::null_mut::<uint8_t>();
        pidxs = ::core::ptr::null_mut::<idx_T>();
        (*sp).ts_prefixdepth = PFD_NOPREFIX as ::core::ffi::c_int as uint8_t;
        (*sp).ts_state = STATE_START;
    } else {
        fbyts = (*slang).sl_fbyts;
        fidxs = (*slang).sl_fidxs;
        pbyts = (*slang).sl_pbyts;
        pidxs = (*slang).sl_pidxs;
        if !pbyts.is_null() {
            byts = pbyts;
            idxs = pidxs;
            (*sp).ts_prefixdepth = PFD_PREFIXTREE as ::core::ffi::c_int as uint8_t;
            (*sp).ts_state = STATE_NOPREFIX;
        } else {
            byts = fbyts;
            idxs = fidxs;
            (*sp).ts_prefixdepth = PFD_NOPREFIX as ::core::ffi::c_int as uint8_t;
            (*sp).ts_state = STATE_START;
        }
    }
    let mut time_limit: proftime_T = 0 as proftime_T;
    if spell_suggest_timeout.get() > 0 as ::core::ffi::c_int {
        time_limit = profile_setlimit(spell_suggest_timeout.get() as int64_t);
    }
    while depth >= 0 as ::core::ffi::c_int && !got_int.get() {
        sp = (&raw mut stack as *mut trystate_T).offset(depth as isize);
        let mut len: ::core::ffi::c_int = 0;
        let mut flags_0: ::core::ffi::c_int = 0;
        let mut fword_ends: bool = false;
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut compound_ok: bool = false;
        let mut newscore_0: ::core::ffi::c_int = 0;
        let mut gap: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
        let mut ftp_0: *mut fromto_T = ::core::ptr::null_mut::<fromto_T>();
        's_2231: {
            'c_37115: {
                'c_37029: {
                    'c_37039: {
                        'c_37069: {
                            match (*sp).ts_state as ::core::ffi::c_uint {
                                0 | 1 => {
                                    arridx = (*sp).ts_arridx;
                                    len = *byts.offset(arridx as isize) as ::core::ffi::c_int;
                                    arridx += (*sp).ts_curi as ::core::ffi::c_int;
                                    if (*sp).ts_prefixdepth as ::core::ffi::c_int
                                        == PFD_PREFIXTREE as ::core::ffi::c_int
                                    {
                                        n = 0 as ::core::ffi::c_int;
                                        while n < len
                                            && *byts
                                                .offset((arridx as ::core::ffi::c_int + n) as isize)
                                                as ::core::ffi::c_int
                                                == 0 as ::core::ffi::c_int
                                        {
                                            n += 1;
                                        }
                                        (*sp).ts_curi =
                                            ((*sp).ts_curi as ::core::ffi::c_int + n) as int16_t;
                                        n = (*sp).ts_state as ::core::ffi::c_int;
                                        (*sp).ts_state = STATE_ENDNUL;
                                        (*sp).ts_save_badflags = (*su).su_badflags as uint8_t;
                                        if depth
                                            < MAXWLEN as ::core::ffi::c_int
                                                - 1 as ::core::ffi::c_int
                                            && (*byts.offset(arridx as isize) as ::core::ffi::c_int
                                                == 0 as ::core::ffi::c_int
                                                || n == STATE_NOPREFIX as ::core::ffi::c_int)
                                        {
                                            n = nofold_len(
                                                fword,
                                                (*sp).ts_fidx as ::core::ffi::c_int,
                                                (*su).su_badptr,
                                            );
                                            let mut flags: ::core::ffi::c_int = badword_captype(
                                                (*su).su_badptr,
                                                (*su).su_badptr.offset(n as isize),
                                            );
                                            (*su).su_badflags = badword_captype(
                                                (*su).su_badptr.offset(n as isize),
                                                (*su).su_badptr.offset((*su).su_badlen as isize),
                                            );
                                            go_deeper(
                                                &raw mut stack as *mut trystate_T,
                                                depth,
                                                0 as ::core::ffi::c_int,
                                            );
                                            depth += 1;
                                            sp = (&raw mut stack as *mut trystate_T)
                                                .offset(depth as isize);
                                            (*sp).ts_prefixdepth =
                                                (depth - 1 as ::core::ffi::c_int) as uint8_t;
                                            byts = fbyts;
                                            idxs = fidxs;
                                            (*sp).ts_arridx = 0 as ::core::ffi::c_int as idx_T;
                                            tword[(*sp).ts_twordlen as usize] =
                                                NUL as ::core::ffi::c_char;
                                            make_case_word(
                                                (&raw mut tword as *mut ::core::ffi::c_char)
                                                    .offset(
                                                        (*sp).ts_splitoff as ::core::ffi::c_int
                                                            as isize,
                                                    ),
                                                (&raw mut preword as *mut ::core::ffi::c_char)
                                                    .offset(
                                                        (*sp).ts_prewordlen as ::core::ffi::c_int
                                                            as isize,
                                                    ),
                                                flags,
                                            );
                                            (*sp).ts_prewordlen = strlen(
                                                &raw mut preword as *mut ::core::ffi::c_char,
                                            )
                                                as uint8_t;
                                            (*sp).ts_splitoff = (*sp).ts_twordlen;
                                        }
                                        break 's_2231;
                                    } else if (*sp).ts_curi as ::core::ffi::c_int > len
                                        || *byts.offset(arridx as isize) as ::core::ffi::c_int
                                            != 0 as ::core::ffi::c_int
                                    {
                                        (*sp).ts_state = STATE_ENDNUL;
                                        (*sp).ts_save_badflags = (*su).su_badflags as uint8_t;
                                        break 's_2231;
                                    } else {
                                        (*sp).ts_curi += 1;
                                        flags_0 = *idxs.offset(arridx as isize);
                                        if flags_0 & WF_NOSUGGEST as ::core::ffi::c_int != 0 {
                                            break 's_2231;
                                        } else {
                                            fword_ends = *fword.offset((*sp).ts_fidx as isize)
                                                as ::core::ffi::c_int
                                                == NUL
                                                || (if soundfold as ::core::ffi::c_int != 0 {
                                                    ascii_iswhite(
                                                        *fword.offset((*sp).ts_fidx as isize)
                                                            as ::core::ffi::c_int,
                                                    )
                                                        as ::core::ffi::c_int
                                                } else {
                                                    !spell_iswordp(
                                                        fword.offset(
                                                            (*sp).ts_fidx as ::core::ffi::c_int
                                                                as isize,
                                                        ),
                                                        curwin.get(),
                                                    )
                                                        as ::core::ffi::c_int
                                                }) != 0;
                                            tword[(*sp).ts_twordlen as usize] =
                                                NUL as ::core::ffi::c_char;
                                            if (*sp).ts_prefixdepth as ::core::ffi::c_int
                                                <= PFD_NOTSPECIAL as ::core::ffi::c_int
                                                && (*sp).ts_flags as ::core::ffi::c_int
                                                    & TSF_PREFIXOK as ::core::ffi::c_int
                                                    == 0 as ::core::ffi::c_int
                                                && !pbyts.is_null()
                                            {
                                                n = stack[(*sp).ts_prefixdepth as usize].ts_arridx
                                                    as ::core::ffi::c_int;
                                                let c2rust_fresh5 = n;
                                                n = n + 1;
                                                len = *pbyts.offset(c2rust_fresh5 as isize)
                                                    as ::core::ffi::c_int;
                                                c = 0 as ::core::ffi::c_int;
                                                while c < len
                                                    && *pbyts.offset((n + c) as isize)
                                                        as ::core::ffi::c_int
                                                        == 0 as ::core::ffi::c_int
                                                {
                                                    c += 1;
                                                }
                                                if c > 0 as ::core::ffi::c_int {
                                                    c = valid_word_prefix(
                                                        c,
                                                        n,
                                                        flags_0,
                                                        (&raw mut tword
                                                            as *mut ::core::ffi::c_char)
                                                            .offset(
                                                                (*sp).ts_splitoff
                                                                    as ::core::ffi::c_int
                                                                    as isize,
                                                            ),
                                                        slang,
                                                        false_0 != 0,
                                                    );
                                                    if c == 0 as ::core::ffi::c_int {
                                                        break 's_2231;
                                                    } else {
                                                        if c & WF_RAREPFX as ::core::ffi::c_int != 0
                                                        {
                                                            flags_0 |=
                                                                WF_RARE as ::core::ffi::c_int;
                                                        }
                                                        (*sp).ts_flags = ((*sp).ts_flags
                                                            as ::core::ffi::c_int
                                                            | TSF_PREFIXOK as ::core::ffi::c_int)
                                                            as uint8_t;
                                                    }
                                                }
                                            }
                                            if (*sp).ts_complen as ::core::ffi::c_int
                                                == (*sp).ts_compsplit as ::core::ffi::c_int
                                                && fword_ends as ::core::ffi::c_int != 0
                                                && flags_0 & WF_NEEDCOMP as ::core::ffi::c_int != 0
                                            {
                                                goodword_ends = false_0 != 0;
                                            } else {
                                                goodword_ends = true_0 != 0;
                                            }
                                            p = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            compound_ok = true_0 != 0;
                                            if (*sp).ts_complen as ::core::ffi::c_int
                                                > (*sp).ts_compsplit as ::core::ffi::c_int
                                            {
                                                if (*slang).sl_nobreak {
                                                    if (*sp).ts_fidx as ::core::ffi::c_int
                                                        - (*sp).ts_splitfidx as ::core::ffi::c_int
                                                        == (*sp).ts_twordlen as ::core::ffi::c_int
                                                            - (*sp).ts_splitoff
                                                                as ::core::ffi::c_int
                                                        && strncmp(
                                                            fword.offset(
                                                                (*sp).ts_splitfidx
                                                                    as ::core::ffi::c_int
                                                                    as isize,
                                                            ),
                                                            (&raw mut tword
                                                                as *mut ::core::ffi::c_char)
                                                                .offset(
                                                                    (*sp).ts_splitoff
                                                                        as ::core::ffi::c_int
                                                                        as isize,
                                                                ),
                                                            ((*sp).ts_fidx as ::core::ffi::c_int
                                                                - (*sp).ts_splitfidx
                                                                    as ::core::ffi::c_int)
                                                                as size_t,
                                                        ) == 0 as ::core::ffi::c_int
                                                    {
                                                        preword[(*sp).ts_prewordlen as usize] =
                                                            NUL as ::core::ffi::c_char;
                                                        let mut newscore: ::core::ffi::c_int =
                                                            score_wordcount_adj(
                                                                slang,
                                                                (*sp).ts_score,
                                                                (&raw mut preword
                                                                    as *mut ::core::ffi::c_char)
                                                                    .offset(
                                                                        (*sp).ts_prewordlen
                                                                            as ::core::ffi::c_int
                                                                            as isize,
                                                                    ),
                                                                (*sp).ts_prewordlen
                                                                    as ::core::ffi::c_int
                                                                    > 0 as ::core::ffi::c_int,
                                                            );
                                                        if newscore <= (*su).su_maxscore {
                                                            add_suggestion(
                                                                su,
                                                                &raw mut (*su).su_ga,
                                                                &raw mut preword
                                                                    as *mut ::core::ffi::c_char,
                                                                (*sp).ts_splitfidx
                                                                    as ::core::ffi::c_int
                                                                    - repextra,
                                                                newscore,
                                                                0 as ::core::ffi::c_int,
                                                                false_0 != 0,
                                                                (*lp).lp_sallang,
                                                                false_0 != 0,
                                                            );
                                                        }
                                                        break 's_2231;
                                                    }
                                                } else if flags_0 as ::core::ffi::c_uint
                                                    >> 24 as ::core::ffi::c_int
                                                    == 0 as ::core::ffi::c_uint
                                                    || ((*sp).ts_twordlen as ::core::ffi::c_int
                                                        - (*sp).ts_splitoff as ::core::ffi::c_int)
                                                        < (*slang).sl_compminlen
                                                {
                                                    break 's_2231;
                                                } else if (*slang).sl_compminlen
                                                    > 0 as ::core::ffi::c_int
                                                    && mb_charlen(
                                                        (&raw mut tword
                                                            as *mut ::core::ffi::c_char)
                                                            .offset(
                                                                (*sp).ts_splitoff
                                                                    as ::core::ffi::c_int
                                                                    as isize,
                                                            ),
                                                    ) < (*slang).sl_compminlen
                                                {
                                                    break 's_2231;
                                                } else {
                                                    compflags[(*sp).ts_complen as usize] = (flags_0
                                                        as ::core::ffi::c_uint
                                                        >> 24 as ::core::ffi::c_int)
                                                        as uint8_t;
                                                    compflags[((*sp).ts_complen
                                                        as ::core::ffi::c_int
                                                        + 1 as ::core::ffi::c_int)
                                                        as usize] = NUL as uint8_t;
                                                    xmemcpyz(
                                                        (&raw mut preword
                                                            as *mut ::core::ffi::c_char)
                                                            .offset(
                                                                (*sp).ts_prewordlen
                                                                    as ::core::ffi::c_int
                                                                    as isize,
                                                            )
                                                            as *mut ::core::ffi::c_void,
                                                        (&raw mut tword as *mut ::core::ffi::c_char)
                                                            .offset(
                                                                (*sp).ts_splitoff
                                                                    as ::core::ffi::c_int
                                                                    as isize,
                                                            )
                                                            as *const ::core::ffi::c_void,
                                                        ((*sp).ts_twordlen as ::core::ffi::c_int
                                                            - (*sp).ts_splitoff
                                                                as ::core::ffi::c_int)
                                                            as size_t,
                                                    );
                                                    if match_checkcompoundpattern(
                                                        &raw mut preword
                                                            as *mut ::core::ffi::c_char,
                                                        (*sp).ts_prewordlen as ::core::ffi::c_int,
                                                        &raw mut (*slang).sl_comppat,
                                                    ) {
                                                        compound_ok = false_0 != 0;
                                                    }
                                                    if compound_ok {
                                                        p = &raw mut preword
                                                            as *mut ::core::ffi::c_char;
                                                        while *skiptowhite(p) as ::core::ffi::c_int
                                                            != NUL
                                                        {
                                                            p = skipwhite(skiptowhite(p));
                                                        }
                                                        if fword_ends as ::core::ffi::c_int != 0
                                                            && !can_compound(
                                                                slang,
                                                                p,
                                                                (&raw mut compflags
                                                                    as *mut uint8_t)
                                                                    .offset(
                                                                        (*sp).ts_compsplit
                                                                            as ::core::ffi::c_int
                                                                            as isize,
                                                                    ),
                                                            )
                                                        {
                                                            compound_ok = false_0 != 0;
                                                        }
                                                    }
                                                    p = (&raw mut preword
                                                        as *mut ::core::ffi::c_char)
                                                        .offset(
                                                            (*sp).ts_prewordlen
                                                                as ::core::ffi::c_int
                                                                as isize,
                                                        );
                                                    p = p.offset(
                                                        -((utf_head_off(
                                                            &raw mut preword
                                                                as *mut ::core::ffi::c_char,
                                                            p.offset(
                                                                -(1 as ::core::ffi::c_int as isize),
                                                            ),
                                                        ) + 1 as ::core::ffi::c_int)
                                                            as isize),
                                                    );
                                                }
                                            }
                                            if soundfold {
                                                strcpy(
                                                    (&raw mut preword as *mut ::core::ffi::c_char)
                                                        .offset(
                                                            (*sp).ts_prewordlen
                                                                as ::core::ffi::c_int
                                                                as isize,
                                                        ),
                                                    (&raw mut tword as *mut ::core::ffi::c_char)
                                                        .offset(
                                                            (*sp).ts_splitoff as ::core::ffi::c_int
                                                                as isize,
                                                        ),
                                                );
                                            } else if flags_0 & WF_KEEPCAP as ::core::ffi::c_int
                                                != 0
                                            {
                                                find_keepcap_word(
                                                    slang,
                                                    (&raw mut tword as *mut ::core::ffi::c_char)
                                                        .offset(
                                                            (*sp).ts_splitoff as ::core::ffi::c_int
                                                                as isize,
                                                        ),
                                                    (&raw mut preword as *mut ::core::ffi::c_char)
                                                        .offset(
                                                            (*sp).ts_prewordlen
                                                                as ::core::ffi::c_int
                                                                as isize,
                                                        ),
                                                );
                                            } else {
                                                c = (*su).su_badflags;
                                                if c & WF_ALLCAP as ::core::ffi::c_int != 0
                                                    && (*su).su_badlen
                                                        == utfc_ptr2len((*su).su_badptr)
                                                {
                                                    c = WF_ONECAP as ::core::ffi::c_int;
                                                }
                                                c |= flags_0;
                                                if !p.is_null()
                                                    && spell_iswordp_nmw(p, curwin.get())
                                                        as ::core::ffi::c_int
                                                        != 0
                                                {
                                                    c &= !(WF_ONECAP as ::core::ffi::c_int);
                                                }
                                                make_case_word(
                                                    (&raw mut tword as *mut ::core::ffi::c_char)
                                                        .offset(
                                                            (*sp).ts_splitoff as ::core::ffi::c_int
                                                                as isize,
                                                        ),
                                                    (&raw mut preword as *mut ::core::ffi::c_char)
                                                        .offset(
                                                            (*sp).ts_prewordlen
                                                                as ::core::ffi::c_int
                                                                as isize,
                                                        ),
                                                    c,
                                                );
                                            }
                                            if !soundfold {
                                                if flags_0 & WF_BANNED as ::core::ffi::c_int != 0 {
                                                    add_banned(
                                                        su,
                                                        (&raw mut preword
                                                            as *mut ::core::ffi::c_char)
                                                            .offset(
                                                                (*sp).ts_prewordlen
                                                                    as ::core::ffi::c_int
                                                                    as isize,
                                                            ),
                                                    );
                                                    break 's_2231;
                                                } else if (*sp).ts_complen as ::core::ffi::c_int
                                                    == (*sp).ts_compsplit as ::core::ffi::c_int
                                                    && !((*hash_find(
                                                        &raw mut (*su).su_banned,
                                                        (&raw mut preword
                                                            as *mut ::core::ffi::c_char)
                                                            .offset(
                                                                (*sp).ts_prewordlen
                                                                    as ::core::ffi::c_int
                                                                    as isize,
                                                            ),
                                                    ))
                                                    .hi_key
                                                    .is_null()
                                                        || (*hash_find(
                                                            &raw mut (*su).su_banned,
                                                            (&raw mut preword
                                                                as *mut ::core::ffi::c_char)
                                                                .offset(
                                                                    (*sp).ts_prewordlen
                                                                        as ::core::ffi::c_int
                                                                        as isize,
                                                                ),
                                                        ))
                                                        .hi_key
                                                            == &raw const hash_removed
                                                                as *mut ::core::ffi::c_char)
                                                    || !((*hash_find(
                                                        &raw mut (*su).su_banned,
                                                        &raw mut preword
                                                            as *mut ::core::ffi::c_char,
                                                    ))
                                                    .hi_key
                                                    .is_null()
                                                        || (*hash_find(
                                                            &raw mut (*su).su_banned,
                                                            &raw mut preword
                                                                as *mut ::core::ffi::c_char,
                                                        ))
                                                        .hi_key
                                                            == &raw const hash_removed
                                                                as *mut ::core::ffi::c_char)
                                                {
                                                    if (*slang).sl_compprog.is_null() {
                                                        break 's_2231;
                                                    } else {
                                                        goodword_ends = false_0 != 0;
                                                    }
                                                }
                                            }
                                            newscore_0 = 0 as ::core::ffi::c_int;
                                            if !soundfold {
                                                if flags_0 & WF_REGION as ::core::ffi::c_int != 0
                                                    && flags_0 as ::core::ffi::c_uint
                                                        >> 16 as ::core::ffi::c_int
                                                        & (*lp).lp_region as ::core::ffi::c_uint
                                                        == 0 as ::core::ffi::c_uint
                                                {
                                                    newscore_0 +=
                                                        SCORE_REGION as ::core::ffi::c_int;
                                                }
                                                if flags_0 & WF_RARE as ::core::ffi::c_int != 0 {
                                                    newscore_0 += SCORE_RARE as ::core::ffi::c_int;
                                                }
                                                if !spell_valid_case(
                                                    (*su).su_badflags,
                                                    captype(
                                                        (&raw mut preword
                                                            as *mut ::core::ffi::c_char)
                                                            .offset(
                                                                (*sp).ts_prewordlen
                                                                    as ::core::ffi::c_int
                                                                    as isize,
                                                            ),
                                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                                    ),
                                                ) {
                                                    newscore_0 += SCORE_ICASE as ::core::ffi::c_int;
                                                }
                                            }
                                            if fword_ends as ::core::ffi::c_int != 0
                                                && goodword_ends as ::core::ffi::c_int != 0
                                                && (*sp).ts_fidx as ::core::ffi::c_int
                                                    >= (*sp).ts_fidxtry as ::core::ffi::c_int
                                                && compound_ok as ::core::ffi::c_int != 0
                                            {
                                                if soundfold {
                                                    add_sound_suggest(
                                                        su,
                                                        &raw mut preword
                                                            as *mut ::core::ffi::c_char,
                                                        (*sp).ts_score,
                                                        lp,
                                                    );
                                                } else if (*sp).ts_fidx as ::core::ffi::c_int
                                                    > 0 as ::core::ffi::c_int
                                                {
                                                    p = fword.offset(
                                                        (*sp).ts_fidx as ::core::ffi::c_int
                                                            as isize,
                                                    );
                                                    p = p.offset(
                                                        -((utf_head_off(
                                                            fword,
                                                            p.offset(
                                                                -(1 as ::core::ffi::c_int as isize),
                                                            ),
                                                        ) + 1 as ::core::ffi::c_int)
                                                            as isize),
                                                    );
                                                    if !spell_iswordp(p, curwin.get())
                                                        && *(&raw mut preword
                                                            as *mut ::core::ffi::c_char)
                                                            as ::core::ffi::c_int
                                                            != NUL
                                                    {
                                                        p = (&raw mut preword
                                                            as *mut ::core::ffi::c_char)
                                                            .offset(strlen(
                                                                &raw mut preword
                                                                    as *mut ::core::ffi::c_char,
                                                            )
                                                                as isize);
                                                        p = p.offset(
                                                            -((utf_head_off(
                                                                &raw mut preword
                                                                    as *mut ::core::ffi::c_char,
                                                                p.offset(
                                                                    -(1 as ::core::ffi::c_int
                                                                        as isize),
                                                                ),
                                                            ) + 1 as ::core::ffi::c_int)
                                                                as isize),
                                                        );
                                                        if spell_iswordp(p, curwin.get()) {
                                                            newscore_0 +=
                                                                SCORE_NONWORD as ::core::ffi::c_int;
                                                        }
                                                    }
                                                    let mut score: ::core::ffi::c_int =
                                                        score_wordcount_adj(
                                                            slang,
                                                            (*sp).ts_score + newscore_0,
                                                            (&raw mut preword
                                                                as *mut ::core::ffi::c_char)
                                                                .offset(
                                                                    (*sp).ts_prewordlen
                                                                        as ::core::ffi::c_int
                                                                        as isize,
                                                                ),
                                                            (*sp).ts_prewordlen
                                                                as ::core::ffi::c_int
                                                                > 0 as ::core::ffi::c_int,
                                                        );
                                                    if score <= (*su).su_maxscore {
                                                        add_suggestion(
                                                            su,
                                                            &raw mut (*su).su_ga,
                                                            &raw mut preword
                                                                as *mut ::core::ffi::c_char,
                                                            (*sp).ts_fidx as ::core::ffi::c_int
                                                                - repextra,
                                                            score,
                                                            0 as ::core::ffi::c_int,
                                                            false_0 != 0,
                                                            (*lp).lp_sallang,
                                                            false_0 != 0,
                                                        );
                                                        if (*su).su_badflags & WF_MIXCAP != 0 {
                                                            c = captype(
                                                                &raw mut preword
                                                                    as *mut ::core::ffi::c_char,
                                                                ::core::ptr::null::<
                                                                    ::core::ffi::c_char,
                                                                >(
                                                                ),
                                                            );
                                                            if c == 0 as ::core::ffi::c_int
                                                                || c == WF_ALLCAP
                                                                    as ::core::ffi::c_int
                                                            {
                                                                make_case_word(
                                                                    (&raw mut tword as *mut ::core::ffi::c_char)
                                                                        .offset((*sp).ts_splitoff as ::core::ffi::c_int as isize),
                                                                    (&raw mut preword as *mut ::core::ffi::c_char)
                                                                        .offset((*sp).ts_prewordlen as ::core::ffi::c_int as isize),
                                                                    if c == 0 as ::core::ffi::c_int {
                                                                        WF_ALLCAP as ::core::ffi::c_int
                                                                    } else {
                                                                        0 as ::core::ffi::c_int
                                                                    },
                                                                );
                                                                add_suggestion(
                                                                    su,
                                                                    &raw mut (*su).su_ga,
                                                                    &raw mut preword
                                                                        as *mut ::core::ffi::c_char,
                                                                    (*sp).ts_fidx
                                                                        as ::core::ffi::c_int
                                                                        - repextra,
                                                                    score
                                                                        + SCORE_ICASE
                                                                            as ::core::ffi::c_int,
                                                                    0 as ::core::ffi::c_int,
                                                                    false_0 != 0,
                                                                    (*lp).lp_sallang,
                                                                    false_0 != 0,
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if ((*sp).ts_fidx as ::core::ffi::c_int
                                                >= (*sp).ts_fidxtry as ::core::ffi::c_int
                                                || fword_ends as ::core::ffi::c_int != 0)
                                                && (*sp).ts_tcharlen as ::core::ffi::c_int
                                                    == 0 as ::core::ffi::c_int
                                            {
                                                let mut try_compound: bool = false;
                                                let mut try_split: ::core::ffi::c_int = 0;
                                                try_split = ((*sp).ts_fidx as ::core::ffi::c_int
                                                    - repextra
                                                    < (*su).su_badlen
                                                    && !soundfold)
                                                    as ::core::ffi::c_int;
                                                try_compound = false_0 != 0;
                                                if !soundfold
                                                    && !(*slang).sl_nocompoundsugs
                                                    && !(*slang).sl_compprog.is_null()
                                                    && flags_0 as ::core::ffi::c_uint
                                                        >> 24 as ::core::ffi::c_int
                                                        != 0 as ::core::ffi::c_uint
                                                    && (*sp).ts_twordlen as ::core::ffi::c_int
                                                        - (*sp).ts_splitoff as ::core::ffi::c_int
                                                        >= (*slang).sl_compminlen
                                                    && ((*slang).sl_compminlen
                                                        == 0 as ::core::ffi::c_int
                                                        || mb_charlen(
                                                            (&raw mut tword
                                                                as *mut ::core::ffi::c_char)
                                                                .offset(
                                                                    (*sp).ts_splitoff
                                                                        as ::core::ffi::c_int
                                                                        as isize,
                                                                ),
                                                        ) >= (*slang).sl_compminlen)
                                                    && ((*slang).sl_compsylmax
                                                        < MAXWLEN as ::core::ffi::c_int
                                                        || ((*sp).ts_complen as ::core::ffi::c_int
                                                            + 1 as ::core::ffi::c_int
                                                            - (*sp).ts_compsplit
                                                                as ::core::ffi::c_int)
                                                            < (*slang).sl_compmax)
                                                    && can_be_compound(
                                                        sp,
                                                        slang,
                                                        &raw mut compflags as *mut uint8_t,
                                                        (flags_0 as ::core::ffi::c_uint
                                                            >> 24 as ::core::ffi::c_int)
                                                            as ::core::ffi::c_int,
                                                    )
                                                        as ::core::ffi::c_int
                                                        != 0
                                                {
                                                    try_compound = true_0 != 0;
                                                    compflags[(*sp).ts_complen as usize] = (flags_0
                                                        as ::core::ffi::c_uint
                                                        >> 24 as ::core::ffi::c_int)
                                                        as uint8_t;
                                                    compflags[((*sp).ts_complen
                                                        as ::core::ffi::c_int
                                                        + 1 as ::core::ffi::c_int)
                                                        as usize] = NUL as uint8_t;
                                                }
                                                if (*slang).sl_nobreak as ::core::ffi::c_int != 0
                                                    && !(*slang).sl_nocompoundsugs
                                                {
                                                    try_compound = true_0 != 0;
                                                } else if !fword_ends
                                                    && try_compound as ::core::ffi::c_int != 0
                                                    && (*sp).ts_flags as ::core::ffi::c_int
                                                        & TSF_DIDSPLIT as ::core::ffi::c_int
                                                        == 0 as ::core::ffi::c_int
                                                {
                                                    try_compound = false_0 != 0;
                                                    (*sp).ts_flags = ((*sp).ts_flags
                                                        as ::core::ffi::c_int
                                                        | TSF_DIDSPLIT as ::core::ffi::c_int)
                                                        as uint8_t;
                                                    (*sp).ts_curi -= 1;
                                                    compflags[(*sp).ts_complen as usize] =
                                                        NUL as uint8_t;
                                                } else {
                                                    (*sp).ts_flags = ((*sp).ts_flags
                                                        as ::core::ffi::c_int
                                                        & !(TSF_DIDSPLIT as ::core::ffi::c_int)
                                                            as uint8_t
                                                            as ::core::ffi::c_int)
                                                        as uint8_t;
                                                }
                                                if try_split != 0
                                                    || try_compound as ::core::ffi::c_int != 0
                                                {
                                                    if !try_compound
                                                        && (!fword_ends || !goodword_ends)
                                                    {
                                                        if (*sp).ts_complen as ::core::ffi::c_int
                                                            == (*sp).ts_compsplit
                                                                as ::core::ffi::c_int
                                                            && flags_0
                                                                & WF_NEEDCOMP as ::core::ffi::c_int
                                                                != 0
                                                        {
                                                            break 's_2231;
                                                        } else {
                                                            p = &raw mut preword
                                                                as *mut ::core::ffi::c_char;
                                                            while *skiptowhite(p)
                                                                as ::core::ffi::c_int
                                                                != NUL
                                                            {
                                                                p = skipwhite(skiptowhite(p));
                                                            }
                                                            if (*sp).ts_complen as ::core::ffi::c_int
                                                                > (*sp).ts_compsplit as ::core::ffi::c_int
                                                                && !can_compound(
                                                                    slang,
                                                                    p,
                                                                    (&raw mut compflags as *mut uint8_t)
                                                                        .offset((*sp).ts_compsplit as ::core::ffi::c_int as isize),
                                                                )
                                                            {
                                                                break 's_2231;
                                                            } else {
                                                                if (*slang).sl_nosplitsugs {
                                                                    newscore_0 += SCORE_SPLIT_NO as ::core::ffi::c_int;
                                                                } else {
                                                                    newscore_0 += SCORE_SPLIT as ::core::ffi::c_int;
                                                                }
                                                                newscore_0 = score_wordcount_adj(
                                                                    slang,
                                                                    newscore_0,
                                                                    (&raw mut preword as *mut ::core::ffi::c_char)
                                                                        .offset((*sp).ts_prewordlen as ::core::ffi::c_int as isize),
                                                                    true_0 != 0,
                                                                );
                                                            }
                                                        }
                                                    }
                                                    if depth
                                                        < MAXWLEN as ::core::ffi::c_int
                                                            - 1 as ::core::ffi::c_int
                                                        && stack[depth as usize].ts_score
                                                            + newscore_0
                                                            < (*su).su_maxscore
                                                    {
                                                        go_deeper(
                                                            &raw mut stack as *mut trystate_T,
                                                            depth,
                                                            newscore_0,
                                                        );
                                                        (*sp).ts_save_badflags =
                                                            (*su).su_badflags as uint8_t;
                                                        (*sp).ts_state = STATE_SPLITUNDO;
                                                        depth += 1;
                                                        sp = (&raw mut stack as *mut trystate_T)
                                                            .offset(depth as isize);
                                                        if !try_compound && !fword_ends {
                                                            strcat(
                                                                &raw mut preword
                                                                    as *mut ::core::ffi::c_char,
                                                                b" \0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                        }
                                                        (*sp).ts_prewordlen = strlen(
                                                            &raw mut preword
                                                                as *mut ::core::ffi::c_char,
                                                        )
                                                            as uint8_t;
                                                        (*sp).ts_splitoff = (*sp).ts_twordlen;
                                                        (*sp).ts_splitfidx = (*sp).ts_fidx;
                                                        if (!try_compound
                                                            && !spell_iswordp_nmw(
                                                                fword.offset(
                                                                    (*sp).ts_fidx
                                                                        as ::core::ffi::c_int
                                                                        as isize,
                                                                ),
                                                                curwin.get(),
                                                            )
                                                            || fword_ends as ::core::ffi::c_int
                                                                != 0)
                                                            && *fword.offset((*sp).ts_fidx as isize)
                                                                as ::core::ffi::c_int
                                                                != NUL
                                                            && goodword_ends as ::core::ffi::c_int
                                                                != 0
                                                        {
                                                            let mut l: ::core::ffi::c_int = 0;
                                                            l = utfc_ptr2len(fword.offset(
                                                                (*sp).ts_fidx as ::core::ffi::c_int
                                                                    as isize,
                                                            ));
                                                            if fword_ends {
                                                                memmove(
                                                                    (&raw mut preword as *mut ::core::ffi::c_char)
                                                                        .offset((*sp).ts_prewordlen as ::core::ffi::c_int as isize)
                                                                        as *mut ::core::ffi::c_void,
                                                                    fword.offset((*sp).ts_fidx as ::core::ffi::c_int as isize)
                                                                        as *const ::core::ffi::c_void,
                                                                    l as size_t,
                                                                );
                                                                (*sp).ts_prewordlen = ((*sp)
                                                                    .ts_prewordlen
                                                                    as ::core::ffi::c_int
                                                                    + l)
                                                                    as uint8_t;
                                                                preword[(*sp).ts_prewordlen
                                                                    as usize] =
                                                                    NUL as ::core::ffi::c_char;
                                                            } else {
                                                                (*sp).ts_score -= SCORE_SPLIT
                                                                    as ::core::ffi::c_int
                                                                    - SCORE_SUBST
                                                                        as ::core::ffi::c_int;
                                                            }
                                                            (*sp).ts_fidx = ((*sp).ts_fidx
                                                                as ::core::ffi::c_int
                                                                + l)
                                                                as uint8_t;
                                                        }
                                                        if try_compound {
                                                            (*sp).ts_complen =
                                                                (*sp).ts_complen.wrapping_add(1);
                                                        } else {
                                                            (*sp).ts_compsplit = (*sp).ts_complen;
                                                        }
                                                        (*sp).ts_prefixdepth = PFD_NOPREFIX
                                                            as ::core::ffi::c_int
                                                            as uint8_t;
                                                        n = nofold_len(
                                                            fword,
                                                            (*sp).ts_fidx as ::core::ffi::c_int,
                                                            (*su).su_badptr,
                                                        );
                                                        (*su).su_badflags = badword_captype(
                                                            (*su).su_badptr.offset(n as isize),
                                                            (*su)
                                                                .su_badptr
                                                                .offset((*su).su_badlen as isize),
                                                        );
                                                        (*sp).ts_arridx =
                                                            0 as ::core::ffi::c_int as idx_T;
                                                        if !pbyts.is_null() {
                                                            byts = pbyts;
                                                            idxs = pidxs;
                                                            (*sp).ts_prefixdepth = PFD_PREFIXTREE
                                                                as ::core::ffi::c_int
                                                                as uint8_t;
                                                            (*sp).ts_state = STATE_NOPREFIX;
                                                        }
                                                    }
                                                    break 's_2231;
                                                } else {
                                                    break 's_2231;
                                                }
                                            } else {
                                                break 's_2231;
                                            }
                                        }
                                    }
                                }
                                2 => {
                                    (*su).su_badflags =
                                        (*sp).ts_save_badflags as ::core::ffi::c_int;
                                    (*sp).ts_state = STATE_START;
                                    byts = fbyts;
                                    idxs = fidxs;
                                    break 's_2231;
                                }
                                3 => {
                                    (*su).su_badflags =
                                        (*sp).ts_save_badflags as ::core::ffi::c_int;
                                    if *fword.offset((*sp).ts_fidx as isize) as ::core::ffi::c_int
                                        == NUL
                                        && (*sp).ts_tcharlen as ::core::ffi::c_int
                                            == 0 as ::core::ffi::c_int
                                    {
                                        (*sp).ts_state = STATE_DEL;
                                        break 's_2231;
                                    } else {
                                        (*sp).ts_state = STATE_PLAIN;
                                        break 'c_37029;
                                    }
                                }
                                4 => {
                                    break 'c_37029;
                                }
                                5 => {
                                    if (*sp).ts_tcharlen as ::core::ffi::c_int
                                        > 0 as ::core::ffi::c_int
                                    {
                                        (*sp).ts_state = STATE_FINAL;
                                        break 's_2231;
                                    } else {
                                        (*sp).ts_state = STATE_INS_PREP;
                                        (*sp).ts_curi = 1 as int16_t;
                                        if soundfold as ::core::ffi::c_int != 0
                                            && (*sp).ts_fidx as ::core::ffi::c_int
                                                == 0 as ::core::ffi::c_int
                                            && *fword.offset((*sp).ts_fidx as isize)
                                                as ::core::ffi::c_int
                                                == '*' as ::core::ffi::c_int
                                        {
                                            newscore_0 = 2 as ::core::ffi::c_int
                                                * SCORE_DEL as ::core::ffi::c_int
                                                / 3 as ::core::ffi::c_int;
                                        } else {
                                            newscore_0 = SCORE_DEL as ::core::ffi::c_int;
                                        }
                                        if *fword.offset((*sp).ts_fidx as isize)
                                            as ::core::ffi::c_int
                                            != NUL
                                            && (depth
                                                < MAXWLEN as ::core::ffi::c_int
                                                    - 1 as ::core::ffi::c_int
                                                && stack[depth as usize].ts_score + newscore_0
                                                    < (*su).su_maxscore)
                                        {
                                            go_deeper(
                                                &raw mut stack as *mut trystate_T,
                                                depth,
                                                newscore_0,
                                            );
                                            depth += 1;
                                            stack[depth as usize].ts_flags = (stack[depth as usize]
                                                .ts_flags
                                                as ::core::ffi::c_int
                                                | TSF_DIDDEL as ::core::ffi::c_int)
                                                as uint8_t;
                                            stack[depth as usize].ts_delidx = (*sp).ts_fidx;
                                            c = utf_ptr2char(fword.offset(
                                                (*sp).ts_fidx as ::core::ffi::c_int as isize,
                                            ));
                                            stack[depth as usize].ts_fidx = (stack[depth as usize]
                                                .ts_fidx
                                                as ::core::ffi::c_int
                                                + utfc_ptr2len(fword.offset(
                                                    (*sp).ts_fidx as ::core::ffi::c_int as isize,
                                                )))
                                                as uint8_t;
                                            if utf_iscomposing_legacy(c) {
                                                stack[depth as usize].ts_score -= SCORE_DEL
                                                    as ::core::ffi::c_int
                                                    - SCORE_DELCOMP as ::core::ffi::c_int;
                                            } else if c
                                                == utf_ptr2char(fword.offset(
                                                    stack[depth as usize].ts_fidx
                                                        as ::core::ffi::c_int
                                                        as isize,
                                                ))
                                            {
                                                stack[depth as usize].ts_score -= SCORE_DEL
                                                    as ::core::ffi::c_int
                                                    - SCORE_DELDUP as ::core::ffi::c_int;
                                            }
                                            break 's_2231;
                                        } else {
                                            break 'c_37039;
                                        }
                                    }
                                }
                                6 => {
                                    break 'c_37039;
                                }
                                7 => {
                                    n = (*sp).ts_arridx as ::core::ffi::c_int;
                                    if (*sp).ts_curi as ::core::ffi::c_int
                                        > *byts.offset(n as isize) as ::core::ffi::c_int
                                    {
                                        (*sp).ts_state = STATE_SWAP;
                                        break 's_2231;
                                    } else {
                                        let c2rust_fresh8 = (*sp).ts_curi;
                                        (*sp).ts_curi = (*sp).ts_curi + 1;
                                        n += c2rust_fresh8 as ::core::ffi::c_int;
                                        if byts == (*slang).sl_fbyts && n >= (*slang).sl_fbyts_len {
                                            got_int.set(true_0 != 0);
                                            break 's_2231;
                                        } else {
                                            c = *byts.offset(n as isize) as ::core::ffi::c_int;
                                            if soundfold as ::core::ffi::c_int != 0
                                                && (*sp).ts_twordlen as ::core::ffi::c_int
                                                    == 0 as ::core::ffi::c_int
                                                && c == '*' as ::core::ffi::c_int
                                            {
                                                newscore_0 = 2 as ::core::ffi::c_int
                                                    * SCORE_INS as ::core::ffi::c_int
                                                    / 3 as ::core::ffi::c_int;
                                            } else {
                                                newscore_0 = SCORE_INS as ::core::ffi::c_int;
                                            }
                                            if c != *fword.offset((*sp).ts_fidx as isize) as uint8_t
                                                as ::core::ffi::c_int
                                                && (depth
                                                    < MAXWLEN as ::core::ffi::c_int
                                                        - 1 as ::core::ffi::c_int
                                                    && stack[depth as usize].ts_score + newscore_0
                                                        < (*su).su_maxscore)
                                            {
                                                go_deeper(
                                                    &raw mut stack as *mut trystate_T,
                                                    depth,
                                                    newscore_0,
                                                );
                                                depth += 1;
                                                sp = (&raw mut stack as *mut trystate_T)
                                                    .offset(depth as isize);
                                                let c2rust_fresh9 = (*sp).ts_twordlen;
                                                (*sp).ts_twordlen =
                                                    (*sp).ts_twordlen.wrapping_add(1);
                                                tword[c2rust_fresh9 as usize] =
                                                    c as ::core::ffi::c_char;
                                                (*sp).ts_arridx = *idxs.offset(n as isize);
                                                fl = (*utf8len_tab.ptr())[c as usize]
                                                    as ::core::ffi::c_int;
                                                if fl > 1 as ::core::ffi::c_int {
                                                    (*sp).ts_tcharlen = fl as uint8_t;
                                                    (*sp).ts_tcharidx = 1 as uint8_t;
                                                    (*sp).ts_isdiff = DIFF_INSERT
                                                        as ::core::ffi::c_int
                                                        as uint8_t;
                                                }
                                                if fl == 1 as ::core::ffi::c_int {
                                                    if (*sp).ts_twordlen as ::core::ffi::c_int
                                                        >= 2 as ::core::ffi::c_int
                                                        && tword[((*sp).ts_twordlen
                                                            as ::core::ffi::c_int
                                                            - 2 as ::core::ffi::c_int)
                                                            as usize]
                                                            as uint8_t
                                                            as ::core::ffi::c_int
                                                            == c
                                                    {
                                                        (*sp).ts_score -= SCORE_INS
                                                            as ::core::ffi::c_int
                                                            - SCORE_INSDUP as ::core::ffi::c_int;
                                                    }
                                                }
                                            }
                                            break 's_2231;
                                        }
                                    }
                                }
                                8 => {
                                    p = fword.offset((*sp).ts_fidx as ::core::ffi::c_int as isize);
                                    c = *p as uint8_t as ::core::ffi::c_int;
                                    if c == NUL {
                                        (*sp).ts_state = STATE_FINAL;
                                        break 's_2231;
                                    } else if !soundfold && !spell_iswordp(p, curwin.get()) {
                                        (*sp).ts_state = STATE_REP_INI;
                                        break 's_2231;
                                    } else {
                                        n = utf_ptr2len(p);
                                        c = utf_ptr2char(p);
                                        if *p.offset(n as isize) as ::core::ffi::c_int == NUL {
                                            c2 = NUL;
                                        } else if !soundfold
                                            && !spell_iswordp(p.offset(n as isize), curwin.get())
                                        {
                                            c2 = c;
                                        } else {
                                            c2 = utf_ptr2char(p.offset(n as isize));
                                        }
                                        if c2 == NUL {
                                            (*sp).ts_state = STATE_REP_INI;
                                            break 's_2231;
                                        } else if c == c2 {
                                            (*sp).ts_state = STATE_SWAP3;
                                            break 's_2231;
                                        } else {
                                            if depth
                                                < MAXWLEN as ::core::ffi::c_int
                                                    - 1 as ::core::ffi::c_int
                                                && (stack[depth as usize].ts_score
                                                    + SCORE_SWAP as ::core::ffi::c_int)
                                                    < (*su).su_maxscore
                                            {
                                                go_deeper(
                                                    &raw mut stack as *mut trystate_T,
                                                    depth,
                                                    SCORE_SWAP as ::core::ffi::c_int,
                                                );
                                                (*sp).ts_state = STATE_UNSWAP;
                                                depth += 1;
                                                fl = utf_char2len(c2);
                                                memmove(
                                                    p as *mut ::core::ffi::c_void,
                                                    p.offset(n as isize)
                                                        as *const ::core::ffi::c_void,
                                                    fl as size_t,
                                                );
                                                utf_char2bytes(c, p.offset(fl as isize));
                                                stack[depth as usize].ts_fidxtry =
                                                    ((*sp).ts_fidx as ::core::ffi::c_int + n + fl)
                                                        as uint8_t;
                                            } else {
                                                (*sp).ts_state = STATE_REP_INI;
                                            }
                                            break 's_2231;
                                        }
                                    }
                                }
                                9 => {
                                    p = fword.offset((*sp).ts_fidx as ::core::ffi::c_int as isize);
                                    n = utfc_ptr2len(p);
                                    c = utf_ptr2char(p.offset(n as isize));
                                    memmove(
                                        p.offset(utfc_ptr2len(p.offset(n as isize)) as isize)
                                            as *mut ::core::ffi::c_void,
                                        p as *const ::core::ffi::c_void,
                                        n as size_t,
                                    );
                                    utf_char2bytes(c, p);
                                    break 'c_37069;
                                }
                                10 => {
                                    break 'c_37069;
                                }
                                11 => {
                                    p = fword.offset((*sp).ts_fidx as ::core::ffi::c_int as isize);
                                    n = utfc_ptr2len(p);
                                    c2 = utf_ptr2char(p.offset(n as isize));
                                    fl = utfc_ptr2len(p.offset(n as isize));
                                    c = utf_ptr2char(p.offset(n as isize).offset(fl as isize));
                                    tl = utfc_ptr2len(p.offset(n as isize).offset(fl as isize));
                                    memmove(
                                        p.offset(fl as isize).offset(tl as isize)
                                            as *mut ::core::ffi::c_void,
                                        p as *const ::core::ffi::c_void,
                                        n as size_t,
                                    );
                                    utf_char2bytes(c, p);
                                    utf_char2bytes(c2, p.offset(tl as isize));
                                    p = p.offset(tl as isize);
                                    if !soundfold && !spell_iswordp(p, curwin.get()) {
                                        (*sp).ts_state = STATE_REP_INI;
                                        break 's_2231;
                                    } else {
                                        if depth
                                            < MAXWLEN as ::core::ffi::c_int
                                                - 1 as ::core::ffi::c_int
                                            && (stack[depth as usize].ts_score
                                                + SCORE_SWAP3 as ::core::ffi::c_int)
                                                < (*su).su_maxscore
                                        {
                                            go_deeper(
                                                &raw mut stack as *mut trystate_T,
                                                depth,
                                                SCORE_SWAP3 as ::core::ffi::c_int,
                                            );
                                            (*sp).ts_state = STATE_UNROT3L;
                                            depth += 1;
                                            p = fword.offset(
                                                (*sp).ts_fidx as ::core::ffi::c_int as isize,
                                            );
                                            n = utf_ptr2len(p);
                                            c = utf_ptr2char(p);
                                            fl = utf_ptr2len(p.offset(n as isize));
                                            fl += utf_ptr2len(
                                                p.offset(n as isize).offset(fl as isize),
                                            );
                                            memmove(
                                                p as *mut ::core::ffi::c_void,
                                                p.offset(n as isize) as *const ::core::ffi::c_void,
                                                fl as size_t,
                                            );
                                            utf_char2bytes(c, p.offset(fl as isize));
                                            stack[depth as usize].ts_fidxtry =
                                                ((*sp).ts_fidx as ::core::ffi::c_int + n + fl)
                                                    as uint8_t;
                                        } else {
                                            (*sp).ts_state = STATE_REP_INI;
                                        }
                                        break 's_2231;
                                    }
                                }
                                12 => {
                                    p = fword.offset((*sp).ts_fidx as ::core::ffi::c_int as isize);
                                    n = utfc_ptr2len(p);
                                    n += utfc_ptr2len(p.offset(n as isize));
                                    c = utf_ptr2char(p.offset(n as isize));
                                    tl = utfc_ptr2len(p.offset(n as isize));
                                    memmove(
                                        p.offset(tl as isize) as *mut ::core::ffi::c_void,
                                        p as *const ::core::ffi::c_void,
                                        n as size_t,
                                    );
                                    utf_char2bytes(c, p);
                                    if depth
                                        < MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                        && (stack[depth as usize].ts_score
                                            + SCORE_SWAP3 as ::core::ffi::c_int)
                                            < (*su).su_maxscore
                                    {
                                        go_deeper(
                                            &raw mut stack as *mut trystate_T,
                                            depth,
                                            SCORE_SWAP3 as ::core::ffi::c_int,
                                        );
                                        (*sp).ts_state = STATE_UNROT3R;
                                        depth += 1;
                                        p = fword
                                            .offset((*sp).ts_fidx as ::core::ffi::c_int as isize);
                                        n = utf_ptr2len(p);
                                        n += utf_ptr2len(p.offset(n as isize));
                                        c = utf_ptr2char(p.offset(n as isize));
                                        tl = utf_ptr2len(p.offset(n as isize));
                                        memmove(
                                            p.offset(tl as isize) as *mut ::core::ffi::c_void,
                                            p as *const ::core::ffi::c_void,
                                            n as size_t,
                                        );
                                        utf_char2bytes(c, p);
                                        stack[depth as usize].ts_fidxtry =
                                            ((*sp).ts_fidx as ::core::ffi::c_int + n + tl)
                                                as uint8_t;
                                    } else {
                                        (*sp).ts_state = STATE_REP_INI;
                                    }
                                    break 's_2231;
                                }
                                13 => {
                                    p = fword.offset((*sp).ts_fidx as ::core::ffi::c_int as isize);
                                    c = utf_ptr2char(p);
                                    tl = utfc_ptr2len(p);
                                    n = utfc_ptr2len(p.offset(tl as isize));
                                    n += utfc_ptr2len(p.offset(tl as isize).offset(n as isize));
                                    memmove(
                                        p as *mut ::core::ffi::c_void,
                                        p.offset(tl as isize) as *const ::core::ffi::c_void,
                                        n as size_t,
                                    );
                                    utf_char2bytes(c, p.offset(n as isize));
                                }
                                14 => {}
                                15 => {
                                    break 'c_37115;
                                }
                                16 => {
                                    if soundfold {
                                        gap = &raw mut (*slang).sl_repsal;
                                    } else {
                                        gap = &raw mut (*(*lp).lp_replang).sl_rep;
                                    }
                                    ftp_0 = ((*gap).ga_data as *mut fromto_T)
                                        .offset((*sp).ts_curi as ::core::ffi::c_int as isize)
                                        .offset(-(1 as ::core::ffi::c_int as isize));
                                    fl = strlen((*ftp_0).ft_from) as ::core::ffi::c_int;
                                    tl = strlen((*ftp_0).ft_to) as ::core::ffi::c_int;
                                    p = fword.offset((*sp).ts_fidx as ::core::ffi::c_int as isize);
                                    if fl != tl {
                                        memmove(
                                            p.offset(fl as isize) as *mut ::core::ffi::c_void,
                                            p.offset(tl as isize) as *const ::core::ffi::c_void,
                                            strlen(p.offset(tl as isize)).wrapping_add(1 as size_t),
                                        );
                                        repextra -= tl - fl;
                                    }
                                    memmove(
                                        p as *mut ::core::ffi::c_void,
                                        (*ftp_0).ft_from as *const ::core::ffi::c_void,
                                        fl as size_t,
                                    );
                                    (*sp).ts_state = STATE_REP;
                                    break 's_2231;
                                }
                                _ => {
                                    depth -= 1;
                                    if depth >= 0 as ::core::ffi::c_int
                                        && stack[depth as usize].ts_prefixdepth
                                            as ::core::ffi::c_int
                                            == PFD_PREFIXTREE as ::core::ffi::c_int
                                    {
                                        byts = pbyts;
                                        idxs = pidxs;
                                    }
                                    breakcheckcount -= 1;
                                    if breakcheckcount == 0 as ::core::ffi::c_int {
                                        os_breakcheck();
                                        breakcheckcount = 1000 as ::core::ffi::c_int;
                                        if spell_suggest_timeout.get() > 0 as ::core::ffi::c_int
                                            && profile_passed_limit(time_limit)
                                                as ::core::ffi::c_int
                                                != 0
                                        {
                                            got_int.set(true_0 != 0);
                                        }
                                    }
                                    break 's_2231;
                                }
                            }
                            if (*lp).lp_replang.is_null() && !soundfold
                                || (*sp).ts_score + SCORE_REP as ::core::ffi::c_int
                                    >= (*su).su_maxscore
                                || ((*sp).ts_fidx as ::core::ffi::c_int)
                                    < (*sp).ts_fidxtry as ::core::ffi::c_int
                            {
                                (*sp).ts_state = STATE_FINAL;
                                break 's_2231;
                            } else {
                                if soundfold {
                                    (*sp).ts_curi = (*slang).sl_repsal_first
                                        [*fword.offset((*sp).ts_fidx as isize) as uint8_t as usize];
                                } else {
                                    (*sp).ts_curi = (*(*lp).lp_replang).sl_rep_first
                                        [*fword.offset((*sp).ts_fidx as isize) as uint8_t as usize];
                                }
                                if ((*sp).ts_curi as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
                                    (*sp).ts_state = STATE_FINAL;
                                    break 's_2231;
                                } else {
                                    (*sp).ts_state = STATE_REP;
                                    break 'c_37115;
                                }
                            }
                        }
                        p = fword.offset((*sp).ts_fidx as ::core::ffi::c_int as isize);
                        n = utf_ptr2len(p);
                        c = utf_ptr2char(p);
                        fl = utf_ptr2len(p.offset(n as isize));
                        c2 = utf_ptr2char(p.offset(n as isize));
                        if !soundfold
                            && !spell_iswordp(
                                p.offset(n as isize).offset(fl as isize),
                                curwin.get(),
                            )
                        {
                            c3 = c;
                        } else {
                            c3 = utf_ptr2char(p.offset(n as isize).offset(fl as isize));
                        }
                        if c == c3 || c3 == NUL {
                            (*sp).ts_state = STATE_REP_INI;
                            break 's_2231;
                        } else {
                            if depth < MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                                && (stack[depth as usize].ts_score
                                    + SCORE_SWAP3 as ::core::ffi::c_int)
                                    < (*su).su_maxscore
                            {
                                go_deeper(
                                    &raw mut stack as *mut trystate_T,
                                    depth,
                                    SCORE_SWAP3 as ::core::ffi::c_int,
                                );
                                (*sp).ts_state = STATE_UNSWAP3;
                                depth += 1;
                                tl = utf_char2len(c3);
                                memmove(
                                    p as *mut ::core::ffi::c_void,
                                    p.offset(n as isize).offset(fl as isize)
                                        as *const ::core::ffi::c_void,
                                    tl as size_t,
                                );
                                utf_char2bytes(c2, p.offset(tl as isize));
                                utf_char2bytes(c, p.offset(fl as isize).offset(tl as isize));
                                stack[depth as usize].ts_fidxtry =
                                    ((*sp).ts_fidx as ::core::ffi::c_int + n + fl + tl) as uint8_t;
                            } else {
                                (*sp).ts_state = STATE_REP_INI;
                            }
                            break 's_2231;
                        }
                    }
                    if (*sp).ts_flags as ::core::ffi::c_int & TSF_DIDDEL as ::core::ffi::c_int != 0
                    {
                        (*sp).ts_state = STATE_SWAP;
                        break 's_2231;
                    } else {
                        n = (*sp).ts_arridx as ::core::ffi::c_int;
                        loop {
                            if (*sp).ts_curi as ::core::ffi::c_int
                                > *byts.offset(n as isize) as ::core::ffi::c_int
                            {
                                (*sp).ts_state = STATE_SWAP;
                                break;
                            } else if *byts
                                .offset((n + (*sp).ts_curi as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int
                                != NUL
                            {
                                (*sp).ts_state = STATE_INS;
                                break;
                            } else {
                                (*sp).ts_curi += 1;
                            }
                        }
                        break 's_2231;
                    }
                }
                arridx = (*sp).ts_arridx;
                if (*sp).ts_curi as ::core::ffi::c_int
                    > *byts.offset(arridx as isize) as ::core::ffi::c_int
                {
                    (*sp).ts_state = (if (*sp).ts_fidx as ::core::ffi::c_int
                        >= (*sp).ts_fidxtry as ::core::ffi::c_int
                    {
                        STATE_DEL as ::core::ffi::c_int
                    } else {
                        STATE_FINAL as ::core::ffi::c_int
                    }) as state_T;
                } else {
                    let c2rust_fresh6 = (*sp).ts_curi;
                    (*sp).ts_curi = (*sp).ts_curi + 1;
                    arridx += c2rust_fresh6 as ::core::ffi::c_int;
                    c = *byts.offset(arridx as isize) as ::core::ffi::c_int;
                    if c == *fword.offset((*sp).ts_fidx as isize) as uint8_t as ::core::ffi::c_int
                        || (*sp).ts_tcharlen as ::core::ffi::c_int > 0 as ::core::ffi::c_int
                            && (*sp).ts_isdiff as ::core::ffi::c_int
                                != DIFF_NONE as ::core::ffi::c_int
                    {
                        newscore_0 = 0 as ::core::ffi::c_int;
                    } else {
                        newscore_0 = SCORE_SUBST as ::core::ffi::c_int;
                    }
                    if (newscore_0 == 0 as ::core::ffi::c_int
                        || (*sp).ts_fidx as ::core::ffi::c_int
                            >= (*sp).ts_fidxtry as ::core::ffi::c_int
                            && ((*sp).ts_flags as ::core::ffi::c_int
                                & TSF_DIDDEL as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                                || c != *fword.offset((*sp).ts_delidx as isize) as uint8_t
                                    as ::core::ffi::c_int))
                        && (depth < MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                            && stack[depth as usize].ts_score + newscore_0 < (*su).su_maxscore)
                    {
                        go_deeper(&raw mut stack as *mut trystate_T, depth, newscore_0);
                        depth += 1;
                        sp = (&raw mut stack as *mut trystate_T).offset(depth as isize);
                        if *fword.offset((*sp).ts_fidx as isize) as ::core::ffi::c_int != NUL {
                            (*sp).ts_fidx = (*sp).ts_fidx.wrapping_add(1);
                        }
                        let c2rust_fresh7 = (*sp).ts_twordlen;
                        (*sp).ts_twordlen = (*sp).ts_twordlen.wrapping_add(1);
                        tword[c2rust_fresh7 as usize] = c as ::core::ffi::c_char;
                        (*sp).ts_arridx = *idxs.offset(arridx as isize);
                        if newscore_0 == SCORE_SUBST as ::core::ffi::c_int {
                            (*sp).ts_isdiff = DIFF_YES as ::core::ffi::c_int as uint8_t;
                        }
                        if (*sp).ts_tcharlen as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                            (*sp).ts_tcharidx = 0 as uint8_t;
                            (*sp).ts_tcharlen = (*utf8len_tab.ptr())[c as usize];
                            (*sp).ts_fcharstart = ((*sp).ts_fidx as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int)
                                as uint8_t;
                            (*sp).ts_isdiff = (if newscore_0 != 0 as ::core::ffi::c_int {
                                DIFF_YES as ::core::ffi::c_int
                            } else {
                                DIFF_NONE as ::core::ffi::c_int
                            }) as uint8_t;
                        } else if (*sp).ts_isdiff as ::core::ffi::c_int
                            == DIFF_INSERT as ::core::ffi::c_int
                            && (*sp).ts_fidx as ::core::ffi::c_int > 0 as ::core::ffi::c_int
                        {
                            (*sp).ts_fidx = (*sp).ts_fidx.wrapping_sub(1);
                        }
                        (*sp).ts_tcharidx = (*sp).ts_tcharidx.wrapping_add(1);
                        if (*sp).ts_tcharidx as ::core::ffi::c_int
                            == (*sp).ts_tcharlen as ::core::ffi::c_int
                        {
                            if (*sp).ts_isdiff as ::core::ffi::c_int
                                == DIFF_YES as ::core::ffi::c_int
                            {
                                (*sp).ts_fidx = ((*sp).ts_fcharstart as ::core::ffi::c_int
                                    + utfc_ptr2len(fword.offset(
                                        (*sp).ts_fcharstart as ::core::ffi::c_int as isize,
                                    ))) as uint8_t;
                                if utf_iscomposing_legacy(utf_ptr2char(
                                    (&raw mut tword as *mut ::core::ffi::c_char)
                                        .offset((*sp).ts_twordlen as ::core::ffi::c_int as isize)
                                        .offset(
                                            -((*sp).ts_tcharlen as ::core::ffi::c_int as isize),
                                        ),
                                )) as ::core::ffi::c_int
                                    != 0
                                    && utf_iscomposing_legacy(utf_ptr2char(fword.offset(
                                        (*sp).ts_fcharstart as ::core::ffi::c_int as isize,
                                    ))) as ::core::ffi::c_int
                                        != 0
                                {
                                    (*sp).ts_score -= SCORE_SUBST as ::core::ffi::c_int
                                        - SCORE_SUBCOMP as ::core::ffi::c_int;
                                } else if !soundfold
                                    && (*slang).sl_has_map as ::core::ffi::c_int != 0
                                    && similar_chars(
                                        slang,
                                        utf_ptr2char(
                                            (&raw mut tword as *mut ::core::ffi::c_char)
                                                .offset(
                                                    (*sp).ts_twordlen as ::core::ffi::c_int
                                                        as isize,
                                                )
                                                .offset(
                                                    -((*sp).ts_tcharlen as ::core::ffi::c_int
                                                        as isize),
                                                ),
                                        ),
                                        utf_ptr2char(fword.offset(
                                            (*sp).ts_fcharstart as ::core::ffi::c_int as isize,
                                        )),
                                    ) as ::core::ffi::c_int
                                        != 0
                                {
                                    (*sp).ts_score -= SCORE_SUBST as ::core::ffi::c_int
                                        - SCORE_SIMILAR as ::core::ffi::c_int;
                                }
                            } else if (*sp).ts_isdiff as ::core::ffi::c_int
                                == DIFF_INSERT as ::core::ffi::c_int
                                && (*sp).ts_twordlen as ::core::ffi::c_int
                                    > (*sp).ts_tcharlen as ::core::ffi::c_int
                            {
                                p = (&raw mut tword as *mut ::core::ffi::c_char)
                                    .offset((*sp).ts_twordlen as ::core::ffi::c_int as isize)
                                    .offset(-((*sp).ts_tcharlen as ::core::ffi::c_int as isize));
                                c = utf_ptr2char(p);
                                if utf_iscomposing_legacy(c) {
                                    (*sp).ts_score -= SCORE_INS as ::core::ffi::c_int
                                        - SCORE_INSCOMP as ::core::ffi::c_int;
                                } else {
                                    p = p.offset(
                                        -((utf_head_off(
                                            &raw mut tword as *mut ::core::ffi::c_char,
                                            p.offset(-(1 as ::core::ffi::c_int as isize)),
                                        ) + 1 as ::core::ffi::c_int)
                                            as isize),
                                    );
                                    if c == utf_ptr2char(p) {
                                        (*sp).ts_score -= SCORE_INS as ::core::ffi::c_int
                                            - SCORE_INSDUP as ::core::ffi::c_int;
                                    }
                                }
                            }
                            (*sp).ts_tcharlen = 0 as uint8_t;
                        }
                    }
                }
                break 's_2231;
            }
            p = fword.offset((*sp).ts_fidx as ::core::ffi::c_int as isize);
            gap = if soundfold as ::core::ffi::c_int != 0 {
                &raw mut (*slang).sl_repsal
            } else {
                &raw mut (*(*lp).lp_replang).sl_rep
            };
            while ((*sp).ts_curi as ::core::ffi::c_int) < (*gap).ga_len {
                let c2rust_fresh10 = (*sp).ts_curi;
                (*sp).ts_curi = (*sp).ts_curi + 1;
                let mut ftp: *mut fromto_T = ((*gap).ga_data as *mut fromto_T)
                    .offset(c2rust_fresh10 as ::core::ffi::c_int as isize);
                if *(*ftp).ft_from as ::core::ffi::c_int != *p as ::core::ffi::c_int {
                    (*sp).ts_curi = (*gap).ga_len as int16_t;
                    break;
                } else {
                    if !(strncmp((*ftp).ft_from, p, strlen((*ftp).ft_from))
                        == 0 as ::core::ffi::c_int
                        && (depth < MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                            && (stack[depth as usize].ts_score + SCORE_REP as ::core::ffi::c_int)
                                < (*su).su_maxscore))
                    {
                        continue;
                    }
                    go_deeper(
                        &raw mut stack as *mut trystate_T,
                        depth,
                        SCORE_REP as ::core::ffi::c_int,
                    );
                    (*sp).ts_state = STATE_REP_UNDO;
                    depth += 1;
                    fl = strlen((*ftp).ft_from) as ::core::ffi::c_int;
                    tl = strlen((*ftp).ft_to) as ::core::ffi::c_int;
                    if fl != tl {
                        memmove(
                            p.offset(tl as isize) as *mut ::core::ffi::c_void,
                            p.offset(fl as isize) as *const ::core::ffi::c_void,
                            strlen(p.offset(fl as isize)).wrapping_add(1 as size_t),
                        );
                        repextra += tl - fl;
                    }
                    memmove(
                        p as *mut ::core::ffi::c_void,
                        (*ftp).ft_to as *const ::core::ffi::c_void,
                        tl as size_t,
                    );
                    stack[depth as usize].ts_fidxtry =
                        ((*sp).ts_fidx as ::core::ffi::c_int + tl) as uint8_t;
                    stack[depth as usize].ts_tcharlen = 0 as uint8_t;
                    break;
                }
            }
            if (*sp).ts_curi as ::core::ffi::c_int >= (*gap).ga_len
                && (*sp).ts_state as ::core::ffi::c_uint
                    == STATE_REP as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*sp).ts_state = STATE_FINAL;
            }
        }
    }
}
unsafe extern "C" fn go_deeper(
    mut stack: *mut trystate_T,
    mut depth: ::core::ffi::c_int,
    mut score_add: ::core::ffi::c_int,
) {
    *stack.offset((depth + 1 as ::core::ffi::c_int) as isize) = *stack.offset(depth as isize);
    (*stack.offset((depth + 1 as ::core::ffi::c_int) as isize)).ts_state = STATE_START;
    (*stack.offset((depth + 1 as ::core::ffi::c_int) as isize)).ts_score =
        (*stack.offset(depth as isize)).ts_score + score_add;
    (*stack.offset((depth + 1 as ::core::ffi::c_int) as isize)).ts_curi = 1 as int16_t;
    (*stack.offset((depth + 1 as ::core::ffi::c_int) as isize)).ts_flags = 0 as uint8_t;
}
unsafe extern "C" fn find_keepcap_word(
    mut slang: *mut slang_T,
    mut fword: *mut ::core::ffi::c_char,
    mut kword: *mut ::core::ffi::c_char,
) {
    let mut uword: [::core::ffi::c_char; 254] = [0; 254];
    let mut tryidx: idx_T = 0;
    let mut arridx: [idx_T; 254] = [0; 254];
    let mut round: [::core::ffi::c_int; 254] = [0; 254];
    let mut fwordidx: [::core::ffi::c_int; 254] = [0; 254];
    let mut uwordidx: [::core::ffi::c_int; 254] = [0; 254];
    let mut kwordlen: [::core::ffi::c_int; 254] = [0; 254];
    let mut l: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut byts: *mut uint8_t = (*slang).sl_kbyts;
    let mut idxs: *mut idx_T = (*slang).sl_kidxs;
    if byts.is_null() {
        *kword = NUL as ::core::ffi::c_char;
        return;
    }
    allcap_copy(fword, &raw mut uword as *mut ::core::ffi::c_char);
    let mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    arridx[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int as idx_T;
    round[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    fwordidx[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    uwordidx[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    kwordlen[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int;
    while depth >= 0 as ::core::ffi::c_int {
        if *fword.offset(fwordidx[depth as usize] as isize) as ::core::ffi::c_int == NUL {
            if *byts.offset(
                (arridx[depth as usize] as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                *kword.offset(kwordlen[depth as usize] as isize) = NUL as ::core::ffi::c_char;
                return;
            }
            depth -= 1;
        } else {
            round[depth as usize] += 1;
            if round[depth as usize] > 2 as ::core::ffi::c_int {
                depth -= 1;
            } else {
                let mut flen: ::core::ffi::c_int =
                    utf_ptr2len(fword.offset(fwordidx[depth as usize] as isize));
                let mut ulen: ::core::ffi::c_int = utf_ptr2len(
                    (&raw mut uword as *mut ::core::ffi::c_char)
                        .offset(uwordidx[depth as usize] as isize),
                );
                if round[depth as usize] == 1 as ::core::ffi::c_int {
                    p = fword.offset(fwordidx[depth as usize] as isize);
                    l = flen;
                } else {
                    p = (&raw mut uword as *mut ::core::ffi::c_char)
                        .offset(uwordidx[depth as usize] as isize);
                    l = ulen;
                }
                tryidx = arridx[depth as usize];
                while l > 0 as ::core::ffi::c_int {
                    let c2rust_fresh15 = tryidx;
                    tryidx = tryidx + 1;
                    let mut len: ::core::ffi::c_int =
                        *byts.offset(c2rust_fresh15 as isize) as ::core::ffi::c_int;
                    let c2rust_fresh16 = p;
                    p = p.offset(1);
                    let mut c: ::core::ffi::c_int =
                        *c2rust_fresh16 as uint8_t as ::core::ffi::c_int;
                    let mut lo: idx_T = tryidx;
                    let mut hi: idx_T = tryidx + len as idx_T - 1 as idx_T;
                    while lo < hi {
                        let mut m: idx_T = (lo + hi) / 2 as idx_T;
                        if *byts.offset(m as isize) as ::core::ffi::c_int > c {
                            hi = (m as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as idx_T;
                        } else if (*byts.offset(m as isize) as ::core::ffi::c_int) < c {
                            lo = (m as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as idx_T;
                        } else {
                            hi = m;
                            lo = hi;
                            break;
                        }
                    }
                    if hi < lo || *byts.offset(lo as isize) as ::core::ffi::c_int != c {
                        break;
                    }
                    tryidx = *idxs.offset(lo as isize);
                    l -= 1;
                }
                if l == 0 as ::core::ffi::c_int {
                    if round[depth as usize] == 1 as ::core::ffi::c_int {
                        strncpy(
                            kword.offset(kwordlen[depth as usize] as isize),
                            fword.offset(fwordidx[depth as usize] as isize),
                            flen as size_t,
                        );
                        kwordlen[(depth + 1 as ::core::ffi::c_int) as usize] =
                            kwordlen[depth as usize] + flen;
                    } else {
                        strncpy(
                            kword.offset(kwordlen[depth as usize] as isize),
                            (&raw mut uword as *mut ::core::ffi::c_char)
                                .offset(uwordidx[depth as usize] as isize),
                            ulen as size_t,
                        );
                        kwordlen[(depth + 1 as ::core::ffi::c_int) as usize] =
                            kwordlen[depth as usize] + ulen;
                    }
                    fwordidx[(depth + 1 as ::core::ffi::c_int) as usize] =
                        fwordidx[depth as usize] + flen;
                    uwordidx[(depth + 1 as ::core::ffi::c_int) as usize] =
                        uwordidx[depth as usize] + ulen;
                    depth += 1;
                    arridx[depth as usize] = tryidx;
                    round[depth as usize] = 0 as ::core::ffi::c_int;
                }
            }
        }
    }
    *kword = NUL as ::core::ffi::c_char;
}
unsafe extern "C" fn score_comp_sal(mut su: *mut suginfo_T) {
    let mut badsound: [::core::ffi::c_char; 254] = [0; 254];
    ga_grow(&raw mut (*su).su_sga, (*su).su_ga.ga_len);
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        if !((*(*lp).lp_slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int) {
            spell_soundfold(
                (*lp).lp_slang,
                &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
                true_0 != 0,
                &raw mut badsound as *mut ::core::ffi::c_char,
            );
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*su).su_ga.ga_len {
                let mut stp: *mut suggest_T =
                    ((*su).su_ga.ga_data as *mut suggest_T).offset(i as isize);
                let mut score: ::core::ffi::c_int = stp_sal_score(
                    stp,
                    su,
                    (*lp).lp_slang,
                    &raw mut badsound as *mut ::core::ffi::c_char,
                );
                if score < SCORE_MAXMAX as ::core::ffi::c_int {
                    let mut sstp: *mut suggest_T = ((*su).su_sga.ga_data as *mut suggest_T)
                        .offset((*su).su_sga.ga_len as isize);
                    (*sstp).st_word = xstrdup((*stp).st_word);
                    (*sstp).st_wordlen = (*stp).st_wordlen;
                    (*sstp).st_score = score;
                    (*sstp).st_altscore = 0 as ::core::ffi::c_int;
                    (*sstp).st_orglen = (*stp).st_orglen;
                    (*su).su_sga.ga_len += 1;
                }
                i += 1;
            }
            break;
        } else {
            lpi += 1;
        }
    }
}
unsafe extern "C" fn score_combine(mut su: *mut suginfo_T) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut badsound: [::core::ffi::c_char; 254] = [0; 254];
    let mut slang: *mut slang_T = ::core::ptr::null_mut::<slang_T>();
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        if !((*(*lp).lp_slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int) {
            slang = (*lp).lp_slang;
            spell_soundfold(
                slang,
                &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
                true_0 != 0,
                &raw mut badsound as *mut ::core::ffi::c_char,
            );
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*su).su_ga.ga_len {
                let mut stp: *mut suggest_T =
                    ((*su).su_ga.ga_data as *mut suggest_T).offset(i as isize);
                (*stp).st_altscore = stp_sal_score(
                    stp,
                    su,
                    slang,
                    &raw mut badsound as *mut ::core::ffi::c_char,
                );
                if (*stp).st_altscore == SCORE_MAXMAX as ::core::ffi::c_int {
                    (*stp).st_score = ((*stp).st_score * 3 as ::core::ffi::c_int
                        + SCORE_INS as ::core::ffi::c_int * 3 as ::core::ffi::c_int)
                        / 4 as ::core::ffi::c_int;
                } else {
                    (*stp).st_score = ((*stp).st_score * 3 as ::core::ffi::c_int
                        + (*stp).st_altscore)
                        / 4 as ::core::ffi::c_int;
                }
                (*stp).st_salscore = false_0 != 0;
                i += 1;
            }
            break;
        } else {
            lpi += 1;
        }
    }
    if slang.is_null() {
        cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
        return;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*su).su_sga.ga_len {
        let mut stp_0: *mut suggest_T =
            ((*su).su_sga.ga_data as *mut suggest_T).offset(i_0 as isize);
        (*stp_0).st_altscore = spell_edit_score(
            slang,
            &raw mut (*su).su_badword as *mut ::core::ffi::c_char,
            (*stp_0).st_word,
        );
        if (*stp_0).st_score == SCORE_MAXMAX as ::core::ffi::c_int {
            (*stp_0).st_score = (SCORE_INS as ::core::ffi::c_int
                * 3 as ::core::ffi::c_int
                * 7 as ::core::ffi::c_int
                + (*stp_0).st_altscore)
                / 8 as ::core::ffi::c_int;
        } else {
            (*stp_0).st_score = ((*stp_0).st_score * 7 as ::core::ffi::c_int
                + (*stp_0).st_altscore)
                / 8 as ::core::ffi::c_int;
        }
        (*stp_0).st_salscore = true_0 != 0;
        i_0 += 1;
    }
    check_suggestions(su, &raw mut (*su).su_ga);
    cleanup_suggestions(&raw mut (*su).su_ga, (*su).su_maxscore, (*su).su_maxcount);
    check_suggestions(su, &raw mut (*su).su_sga);
    cleanup_suggestions(&raw mut (*su).su_sga, (*su).su_maxscore, (*su).su_maxcount);
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<suginfo_T>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ga_grow(&raw mut ga, (*su).su_ga.ga_len + (*su).su_sga.ga_len);
    let mut stp_1: *mut suggest_T =
        (ga.ga_data as *mut suggest_T).offset(0 as ::core::ffi::c_int as isize);
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < (*su).su_ga.ga_len || i_1 < (*su).su_sga.ga_len {
        let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while round <= 2 as ::core::ffi::c_int {
            let mut gap: *mut garray_T = if round == 1 as ::core::ffi::c_int {
                &raw mut (*su).su_ga
            } else {
                &raw mut (*su).su_sga
            };
            if i_1 < (*gap).ga_len {
                p = (*((*gap).ga_data as *mut suggest_T).offset(i_1 as isize)).st_word;
                let mut j: ::core::ffi::c_int = 0;
                j = 0 as ::core::ffi::c_int;
                while j < ga.ga_len {
                    if strcmp((*stp_1.offset(j as isize)).st_word, p) == 0 as ::core::ffi::c_int {
                        break;
                    }
                    j += 1;
                }
                if j == ga.ga_len {
                    let c2rust_fresh0 = ga.ga_len;
                    ga.ga_len = ga.ga_len + 1;
                    *stp_1.offset(c2rust_fresh0 as isize) =
                        *((*gap).ga_data as *mut suggest_T).offset(i_1 as isize);
                } else {
                    xfree(p as *mut ::core::ffi::c_void);
                }
            }
            round += 1;
        }
        i_1 += 1;
    }
    ga_clear(&raw mut (*su).su_ga);
    ga_clear(&raw mut (*su).su_sga);
    if ga.ga_len > (*su).su_maxcount {
        let mut i_2: ::core::ffi::c_int = (*su).su_maxcount;
        while i_2 < ga.ga_len {
            xfree((*stp_1.offset(i_2 as isize)).st_word as *mut ::core::ffi::c_void);
            i_2 += 1;
        }
        ga.ga_len = (*su).su_maxcount;
    }
    (*su).su_ga = ga;
}
unsafe extern "C" fn stp_sal_score(
    mut stp: *mut suggest_T,
    mut su: *mut suginfo_T,
    mut slang: *mut slang_T,
    mut badsound: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut pbad: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pgood: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut badsound2: [::core::ffi::c_char; 254] = [0; 254];
    let mut fword: [::core::ffi::c_char; 254] = [0; 254];
    let mut goodsound: [::core::ffi::c_char; 254] = [0; 254];
    let mut goodword: [::core::ffi::c_char; 254] = [0; 254];
    let mut lendiff: ::core::ffi::c_int = (*su).su_badlen - (*stp).st_orglen;
    if lendiff >= 0 as ::core::ffi::c_int {
        pbad = badsound;
    } else {
        spell_casefold(
            curwin.get(),
            (*su).su_badptr,
            (*stp).st_orglen,
            &raw mut fword as *mut ::core::ffi::c_char,
            MAXWLEN as ::core::ffi::c_int,
        );
        if ascii_iswhite(*(*su).su_badptr.offset((*su).su_badlen as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
            && *skiptowhite((*stp).st_word) as ::core::ffi::c_int == NUL
        {
            let mut p: *mut ::core::ffi::c_char = &raw mut fword as *mut ::core::ffi::c_char;
            loop {
                p = skiptowhite(p);
                if *p as ::core::ffi::c_int == NUL {
                    break;
                }
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            }
        }
        spell_soundfold(
            slang,
            &raw mut fword as *mut ::core::ffi::c_char,
            true_0 != 0,
            &raw mut badsound2 as *mut ::core::ffi::c_char,
        );
        pbad = &raw mut badsound2 as *mut ::core::ffi::c_char;
    }
    if lendiff > 0 as ::core::ffi::c_int
        && (*stp).st_wordlen + lendiff < MAXWLEN as ::core::ffi::c_int
    {
        strcpy(
            &raw mut goodword as *mut ::core::ffi::c_char,
            (*stp).st_word,
        );
        xmemcpyz(
            (&raw mut goodword as *mut ::core::ffi::c_char).offset((*stp).st_wordlen as isize)
                as *mut ::core::ffi::c_void,
            (*su)
                .su_badptr
                .offset((*su).su_badlen as isize)
                .offset(-(lendiff as isize)) as *const ::core::ffi::c_void,
            lendiff as size_t,
        );
        pgood = &raw mut goodword as *mut ::core::ffi::c_char;
    } else {
        pgood = (*stp).st_word;
    }
    spell_soundfold(
        slang,
        pgood,
        false_0 != 0,
        &raw mut goodsound as *mut ::core::ffi::c_char,
    );
    return soundalike_score(&raw mut goodsound as *mut ::core::ffi::c_char, pbad);
}
static dumsft: GlobalCell<sftword_T> = GlobalCell::new(sftword_T {
    sft_score: 0,
    sft_word: [],
});
unsafe extern "C" fn suggest_try_soundalike_prep() {
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        let mut slang: *mut slang_T = (*lp).lp_slang;
        if !((*slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int) && !(*slang).sl_sbyts.is_null() {
            hash_init(&raw mut (*slang).sl_sounddone);
        }
        lpi += 1;
    }
}
unsafe extern "C" fn suggest_try_soundalike(mut su: *mut suginfo_T) {
    let mut salword: [::core::ffi::c_char; 254] = [0; 254];
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        let mut slang: *mut slang_T = (*lp).lp_slang;
        if !((*slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int) && !(*slang).sl_sbyts.is_null() {
            spell_soundfold(
                slang,
                &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
                true_0 != 0,
                &raw mut salword as *mut ::core::ffi::c_char,
            );
            suggest_trie_walk(
                su,
                lp,
                &raw mut salword as *mut ::core::ffi::c_char,
                true_0 != 0,
            );
        }
        lpi += 1;
    }
}
unsafe extern "C" fn suggest_try_soundalike_finish() {
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        let mut slang: *mut slang_T = (*lp).lp_slang;
        if !((*slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int) && !(*slang).sl_sbyts.is_null() {
            let mut todo: ::core::ffi::c_int = (*slang).sl_sounddone.ht_used as ::core::ffi::c_int;
            let mut hi: *mut hashitem_T = (*slang).sl_sounddone.ht_array;
            while todo > 0 as ::core::ffi::c_int {
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    xfree(
                        (*hi).hi_key.offset(
                            -((&raw mut (*dumsft.ptr()).sft_word as *mut uint8_t)
                                .offset_from(dumsft.ptr() as *mut uint8_t)
                                as isize),
                        ) as *mut sftword_T as *mut ::core::ffi::c_void,
                    );
                    todo -= 1;
                }
                hi = hi.offset(1);
            }
            hash_clear(&raw mut (*slang).sl_sounddone);
            hash_init(&raw mut (*slang).sl_sounddone);
        }
        lpi += 1;
    }
}
unsafe extern "C" fn add_sound_suggest(
    mut su: *mut suginfo_T,
    mut goodword: *mut ::core::ffi::c_char,
    mut score: ::core::ffi::c_int,
    mut lp: *mut langp_T,
) {
    let mut slang: *mut slang_T = (*lp).lp_slang;
    let mut theword: [::core::ffi::c_char; 254] = [0; 254];
    let mut i: ::core::ffi::c_int = 0;
    let mut wlen: ::core::ffi::c_int = 0;
    let mut byts: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut wc: ::core::ffi::c_int = 0;
    let mut goodscore: ::core::ffi::c_int = 0;
    let mut sft: *mut sftword_T = ::core::ptr::null_mut::<sftword_T>();
    let mut hash: hash_T = hash_hash(goodword);
    let goodword_len: size_t = strlen(goodword);
    let mut hi: *mut hashitem_T =
        hash_lookup(&raw mut (*slang).sl_sounddone, goodword, goodword_len, hash);
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
    {
        sft = xmalloc(
            (2 as size_t)
                .wrapping_add(goodword_len)
                .wrapping_add(1 as size_t),
        ) as *mut sftword_T;
        (*sft).sft_score = score as int16_t;
        memcpy(
            &raw mut (*sft).sft_word as *mut uint8_t as *mut ::core::ffi::c_void,
            goodword as *const ::core::ffi::c_void,
            goodword_len.wrapping_add(1 as size_t),
        );
        hash_add_item(
            &raw mut (*slang).sl_sounddone,
            hi,
            &raw mut (*sft).sft_word as *mut uint8_t as *mut ::core::ffi::c_char,
            hash,
        );
    } else {
        sft = (*hi).hi_key.offset(
            -((&raw mut (*dumsft.ptr()).sft_word as *mut uint8_t)
                .offset_from(dumsft.ptr() as *mut uint8_t) as isize),
        ) as *mut sftword_T;
        if score >= (*sft).sft_score as ::core::ffi::c_int {
            return;
        }
        (*sft).sft_score = score as int16_t;
    }
    let mut sfwordnr: ::core::ffi::c_int = soundfold_find(slang, goodword);
    if sfwordnr < 0 as ::core::ffi::c_int {
        internal_error(b"add_sound_suggest()\0".as_ptr() as *const ::core::ffi::c_char);
        return;
    }
    let mut nrline: *mut ::core::ffi::c_char =
        ml_get_buf((*slang).sl_sugbuf, sfwordnr as linenr_T + 1 as linenr_T);
    let mut orgnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while *nrline as ::core::ffi::c_int != NUL {
        orgnr += bytes2offset(&raw mut nrline);
        byts = (*slang).sl_fbyts;
        let mut idxs: *mut idx_T = (*slang).sl_fidxs;
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut wordcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        wlen = 0 as ::core::ffi::c_int;
        '_badword: while wlen < MAXWLEN as ::core::ffi::c_int - 3 as ::core::ffi::c_int {
            i = 1 as ::core::ffi::c_int;
            if wordcount == orgnr
                && *byts.offset((n + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int == NUL
            {
                break;
            }
            if *byts.offset((n + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int == NUL {
                wordcount += 1;
            }
            while *byts.offset((n + i) as isize) as ::core::ffi::c_int == NUL {
                if i > *byts.offset(n as isize) as ::core::ffi::c_int {
                    strcpy(
                        (&raw mut theword as *mut ::core::ffi::c_char).offset(wlen as isize),
                        b"BAD\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    );
                    wlen += 3 as ::core::ffi::c_int;
                    break '_badword;
                } else {
                    i += 1;
                }
            }
            while i < *byts.offset(n as isize) as ::core::ffi::c_int {
                wc = *idxs.offset(*idxs.offset((n + i) as isize) as isize) as ::core::ffi::c_int;
                if wordcount + wc > orgnr {
                    break;
                }
                wordcount += wc;
                i += 1;
            }
            theword[wlen as usize] = *byts.offset((n + i) as isize) as ::core::ffi::c_char;
            n = *idxs.offset((n + i) as isize) as ::core::ffi::c_int;
            wlen += 1;
        }
        theword[wlen as usize] = NUL as ::core::ffi::c_char;
        while i <= *byts.offset(n as isize) as ::core::ffi::c_int
            && *byts.offset((n + i) as isize) as ::core::ffi::c_int == NUL
        {
            let mut cword: [::core::ffi::c_char; 254] = [0; 254];
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut flags: ::core::ffi::c_int = *idxs.offset((n + i) as isize);
            if flags & WF_NOSUGGEST as ::core::ffi::c_int == 0 {
                if flags & WF_KEEPCAP as ::core::ffi::c_int != 0 {
                    find_keepcap_word(
                        slang,
                        &raw mut theword as *mut ::core::ffi::c_char,
                        &raw mut cword as *mut ::core::ffi::c_char,
                    );
                    p = &raw mut cword as *mut ::core::ffi::c_char;
                } else {
                    flags |= (*su).su_badflags;
                    if flags & WF_CAPMASK as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                        make_case_word(
                            &raw mut theword as *mut ::core::ffi::c_char,
                            &raw mut cword as *mut ::core::ffi::c_char,
                            flags,
                        );
                        p = &raw mut cword as *mut ::core::ffi::c_char;
                    } else {
                        p = &raw mut theword as *mut ::core::ffi::c_char;
                    }
                }
                if sps_flags.get() & SPS_DOUBLE as ::core::ffi::c_int != 0 {
                    if score <= (*su).su_maxscore {
                        add_suggestion(
                            su,
                            &raw mut (*su).su_sga,
                            p,
                            (*su).su_badlen,
                            score,
                            0 as ::core::ffi::c_int,
                            false_0 != 0,
                            slang,
                            false_0 != 0,
                        );
                    }
                } else {
                    if flags & WF_REGION as ::core::ffi::c_int != 0
                        && flags as ::core::ffi::c_uint >> 16 as ::core::ffi::c_int
                            & (*lp).lp_region as ::core::ffi::c_uint
                            == 0 as ::core::ffi::c_uint
                    {
                        goodscore = SCORE_REGION as ::core::ffi::c_int;
                    } else {
                        goodscore = 0 as ::core::ffi::c_int;
                    }
                    let mut gc: ::core::ffi::c_int = utf_ptr2char(p);
                    if if gc >= 128 as ::core::ffi::c_int {
                        mb_isupper(gc) as ::core::ffi::c_int
                    } else {
                        (*spelltab.ptr()).st_isu[gc as usize] as ::core::ffi::c_int
                    } != 0
                    {
                        let mut bc: ::core::ffi::c_int =
                            utf_ptr2char(&raw mut (*su).su_badword as *mut ::core::ffi::c_char);
                        if (if bc >= 128 as ::core::ffi::c_int {
                            mb_isupper(bc) as ::core::ffi::c_int
                        } else {
                            (*spelltab.ptr()).st_isu[bc as usize] as ::core::ffi::c_int
                        }) == 0
                            && (if bc >= 128 as ::core::ffi::c_int {
                                utf_fold(bc)
                            } else {
                                (*spelltab.ptr()).st_fold[bc as usize] as ::core::ffi::c_int
                            }) != (if gc >= 128 as ::core::ffi::c_int {
                                utf_fold(gc)
                            } else {
                                (*spelltab.ptr()).st_fold[gc as usize] as ::core::ffi::c_int
                            })
                        {
                            goodscore +=
                                SCORE_ICASE as ::core::ffi::c_int / 2 as ::core::ffi::c_int;
                        }
                    }
                    let mut limit: ::core::ffi::c_int =
                        (4 as ::core::ffi::c_int * ((*su).su_sfmaxscore - goodscore) - score)
                            / 3 as ::core::ffi::c_int;
                    if limit > SCORE_LIMITMAX as ::core::ffi::c_int {
                        goodscore += spell_edit_score(
                            slang,
                            &raw mut (*su).su_badword as *mut ::core::ffi::c_char,
                            p,
                        );
                    } else {
                        goodscore += spell_edit_score_limit(
                            slang,
                            &raw mut (*su).su_badword as *mut ::core::ffi::c_char,
                            p,
                            limit,
                        );
                    }
                    if goodscore < SCORE_MAXMAX as ::core::ffi::c_int {
                        goodscore = score_wordcount_adj(slang, goodscore, p, false_0 != 0);
                        goodscore =
                            (3 as ::core::ffi::c_int * goodscore + score) / 4 as ::core::ffi::c_int;
                        if goodscore <= (*su).su_sfmaxscore {
                            add_suggestion(
                                su,
                                &raw mut (*su).su_ga,
                                p,
                                (*su).su_badlen,
                                goodscore,
                                score,
                                true_0 != 0,
                                slang,
                                true_0 != 0,
                            );
                        }
                    }
                }
            }
            i += 1;
        }
    }
}
unsafe extern "C" fn soundfold_find(
    mut slang: *mut slang_T,
    mut word: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut arridx: idx_T = 0 as idx_T;
    let mut wlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ptr: *mut uint8_t = word as *mut uint8_t;
    let mut wordnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut byts: *mut uint8_t = (*slang).sl_sbyts;
    let mut idxs: *mut idx_T = (*slang).sl_sidxs;
    loop {
        let c2rust_fresh17 = arridx;
        arridx = arridx + 1;
        let mut len: ::core::ffi::c_int =
            *byts.offset(c2rust_fresh17 as isize) as ::core::ffi::c_int;
        let mut c: ::core::ffi::c_int = *ptr.offset(wlen as isize) as ::core::ffi::c_int;
        if *byts.offset(arridx as isize) as ::core::ffi::c_int == NUL {
            if c == NUL {
                break;
            }
            while len > 0 as ::core::ffi::c_int
                && *byts.offset(arridx as isize) as ::core::ffi::c_int == NUL
            {
                arridx += 1;
                len -= 1;
            }
            if len == 0 as ::core::ffi::c_int {
                return -1 as ::core::ffi::c_int;
            }
            wordnr += 1;
        }
        if c == NUL {
            return -1 as ::core::ffi::c_int;
        }
        if c == TAB {
            c = ' ' as ::core::ffi::c_int;
        }
        while (*byts.offset(arridx as isize) as ::core::ffi::c_int) < c {
            wordnr += *idxs.offset(*idxs.offset(arridx as isize) as isize) as ::core::ffi::c_int;
            arridx += 1;
            len -= 1;
            if len == 0 as ::core::ffi::c_int {
                return -1 as ::core::ffi::c_int;
            }
        }
        if *byts.offset(arridx as isize) as ::core::ffi::c_int != c {
            return -1 as ::core::ffi::c_int;
        }
        arridx = *idxs.offset(arridx as isize);
        wlen += 1;
        if c == ' ' as ::core::ffi::c_int {
            while *ptr.offset(wlen as isize) as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                || *ptr.offset(wlen as isize) as ::core::ffi::c_int == TAB
            {
                wlen += 1;
            }
        }
    }
    return wordnr;
}
unsafe extern "C" fn similar_chars(
    mut slang: *mut slang_T,
    mut c1: ::core::ffi::c_int,
    mut c2: ::core::ffi::c_int,
) -> bool {
    let mut m1: ::core::ffi::c_int = 0;
    let mut m2: ::core::ffi::c_int = 0;
    let mut buf: [::core::ffi::c_char; 7] = [0; 7];
    if c1 >= 256 as ::core::ffi::c_int {
        buf[utf_char2bytes(c1, &raw mut buf as *mut ::core::ffi::c_char) as usize] =
            0 as ::core::ffi::c_char;
        let mut hi: *mut hashitem_T = hash_find(
            &raw mut (*slang).sl_map_hash,
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if (*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            m1 = 0 as ::core::ffi::c_int;
        } else {
            m1 = utf_ptr2char(
                (*hi)
                    .hi_key
                    .offset(strlen((*hi).hi_key) as isize)
                    .offset(1 as ::core::ffi::c_int as isize),
            );
        }
    } else {
        m1 = (*slang).sl_map_array[c1 as usize];
    }
    if m1 == 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    if c2 >= 256 as ::core::ffi::c_int {
        buf[utf_char2bytes(c2, &raw mut buf as *mut ::core::ffi::c_char) as usize] =
            0 as ::core::ffi::c_char;
        let mut hi_0: *mut hashitem_T = hash_find(
            &raw mut (*slang).sl_map_hash,
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if (*hi_0).hi_key.is_null()
            || (*hi_0).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            m2 = 0 as ::core::ffi::c_int;
        } else {
            m2 = utf_ptr2char(
                (*hi_0)
                    .hi_key
                    .offset(strlen((*hi_0).hi_key) as isize)
                    .offset(1 as ::core::ffi::c_int as isize),
            );
        }
    } else {
        m2 = (*slang).sl_map_array[c2 as usize];
    }
    return m1 == m2;
}
unsafe extern "C" fn add_suggestion(
    mut su: *mut suginfo_T,
    mut gap: *mut garray_T,
    mut goodword: *const ::core::ffi::c_char,
    mut badlenarg: ::core::ffi::c_int,
    mut score: ::core::ffi::c_int,
    mut altscore: ::core::ffi::c_int,
    mut had_bonus: bool,
    mut slang: *mut slang_T,
    mut maxsf: bool,
) {
    let mut goodlen: ::core::ffi::c_int = 0;
    let mut badlen: ::core::ffi::c_int = 0;
    let mut new_sug: suggest_T = suggest_T {
        st_word: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        st_wordlen: 0,
        st_orglen: 0,
        st_score: 0,
        st_altscore: 0,
        st_salscore: false,
        st_had_bonus: false,
        st_slang: ::core::ptr::null_mut::<slang_T>(),
    };
    let mut pgood: *const ::core::ffi::c_char = goodword.offset(strlen(goodword) as isize);
    let mut pbad: *mut ::core::ffi::c_char = (*su).su_badptr.offset(badlenarg as isize);
    loop {
        goodlen = pgood.offset_from(goodword) as ::core::ffi::c_int;
        badlen = pbad.offset_from((*su).su_badptr) as ::core::ffi::c_int;
        if goodlen <= 0 as ::core::ffi::c_int || badlen <= 0 as ::core::ffi::c_int {
            break;
        }
        pgood = pgood.offset(
            -((utf_head_off(
                goodword as *mut ::core::ffi::c_char,
                (pgood as *mut ::core::ffi::c_char).offset(-(1 as ::core::ffi::c_int as isize)),
            ) + 1 as ::core::ffi::c_int) as isize),
        );
        pbad = pbad.offset(
            -((utf_head_off(
                (*su).su_badptr,
                pbad.offset(-(1 as ::core::ffi::c_int as isize)),
            ) + 1 as ::core::ffi::c_int) as isize),
        );
        if utf_ptr2char(pgood) != utf_ptr2char(pbad) {
            break;
        }
    }
    if badlen == 0 as ::core::ffi::c_int && goodlen == 0 as ::core::ffi::c_int {
        return;
    }
    let mut i: ::core::ffi::c_int = 0;
    if (*gap).ga_len <= 0 as ::core::ffi::c_int {
        i = -1 as ::core::ffi::c_int;
    } else {
        let mut stp: *mut suggest_T =
            ((*gap).ga_data as *mut suggest_T).offset(0 as ::core::ffi::c_int as isize);
        i = (*gap).ga_len;
        loop {
            i -= 1;
            if i < 0 as ::core::ffi::c_int {
                break;
            }
            if (*stp).st_wordlen == goodlen
                && (*stp).st_orglen == badlen
                && strncmp((*stp).st_word, goodword, goodlen as size_t) == 0 as ::core::ffi::c_int
            {
                if (*stp).st_slang.is_null() {
                    (*stp).st_slang = slang;
                }
                new_sug.st_score = score;
                new_sug.st_altscore = altscore;
                new_sug.st_had_bonus = had_bonus;
                if (*stp).st_had_bonus as ::core::ffi::c_int != had_bonus as ::core::ffi::c_int {
                    if had_bonus {
                        rescore_one(su, stp);
                    } else {
                        new_sug.st_word = (*stp).st_word;
                        new_sug.st_wordlen = (*stp).st_wordlen;
                        new_sug.st_slang = (*stp).st_slang;
                        new_sug.st_orglen = badlen;
                        rescore_one(su, &raw mut new_sug);
                    }
                }
                if (*stp).st_score > new_sug.st_score {
                    (*stp).st_score = new_sug.st_score;
                    (*stp).st_altscore = new_sug.st_altscore;
                    (*stp).st_had_bonus = new_sug.st_had_bonus;
                }
                break;
            } else {
                stp = stp.offset(1);
            }
        }
    }
    if i < 0 as ::core::ffi::c_int {
        let mut stp_0: *mut suggest_T =
            ga_append_via_ptr(gap, ::core::mem::size_of::<suggest_T>()) as *mut suggest_T;
        (*stp_0).st_word = xmemdupz(goodword as *const ::core::ffi::c_void, goodlen as size_t)
            as *mut ::core::ffi::c_char;
        (*stp_0).st_wordlen = goodlen;
        (*stp_0).st_score = score;
        (*stp_0).st_altscore = altscore;
        (*stp_0).st_had_bonus = had_bonus;
        (*stp_0).st_orglen = badlen;
        (*stp_0).st_slang = slang;
        if (*gap).ga_len
            > (if (*su).su_maxcount < 130 as ::core::ffi::c_int {
                150 as ::core::ffi::c_int
            } else {
                (*su).su_maxcount + 20 as ::core::ffi::c_int
            }) + 50 as ::core::ffi::c_int
        {
            if maxsf {
                (*su).su_sfmaxscore = cleanup_suggestions(
                    gap,
                    (*su).su_sfmaxscore,
                    if (*su).su_maxcount < 130 as ::core::ffi::c_int {
                        150 as ::core::ffi::c_int
                    } else {
                        (*su).su_maxcount + 20 as ::core::ffi::c_int
                    },
                );
            } else {
                (*su).su_maxscore = cleanup_suggestions(
                    gap,
                    (*su).su_maxscore,
                    if (*su).su_maxcount < 130 as ::core::ffi::c_int {
                        150 as ::core::ffi::c_int
                    } else {
                        (*su).su_maxcount + 20 as ::core::ffi::c_int
                    },
                );
            }
        }
    }
}
unsafe extern "C" fn check_suggestions(mut su: *mut suginfo_T, mut gap: *mut garray_T) {
    let mut longword: [::core::ffi::c_char; 255] = [0; 255];
    if (*gap).ga_len == 0 as ::core::ffi::c_int {
        return;
    }
    let mut stp: *mut suggest_T =
        ((*gap).ga_data as *mut suggest_T).offset(0 as ::core::ffi::c_int as isize);
    let mut i: ::core::ffi::c_int = (*gap).ga_len - 1 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        xstrlcpy(
            &raw mut longword as *mut ::core::ffi::c_char,
            (*stp.offset(i as isize)).st_word,
            (MAXWLEN as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
        );
        let mut len: ::core::ffi::c_int = (*stp.offset(i as isize)).st_wordlen;
        xstrlcpy(
            (&raw mut longword as *mut ::core::ffi::c_char).offset(len as isize),
            (*su)
                .su_badptr
                .offset((*stp.offset(i as isize)).st_orglen as isize),
            ((MAXWLEN as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t)
                .wrapping_sub(len as size_t),
        );
        let mut attr: hlf_T = HLF_COUNT;
        spell_check(
            curwin.get(),
            &raw mut longword as *mut ::core::ffi::c_char,
            &raw mut attr,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            false_0 != 0,
        );
        if attr as ::core::ffi::c_uint != HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint {
            xfree((*stp.offset(i as isize)).st_word as *mut ::core::ffi::c_void);
            (*gap).ga_len -= 1;
            if i < (*gap).ga_len {
                memmove(
                    stp.offset(i as isize) as *mut ::core::ffi::c_void,
                    stp.offset(i as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<suggest_T>().wrapping_mul(((*gap).ga_len - i) as size_t),
                );
            }
        }
        i -= 1;
    }
}
unsafe extern "C" fn add_banned(mut su: *mut suginfo_T, mut word: *mut ::core::ffi::c_char) {
    let mut hash: hash_T = hash_hash(word);
    let word_len: size_t = strlen(word);
    let mut hi: *mut hashitem_T = hash_lookup(&raw mut (*su).su_banned, word, word_len, hash);
    if !((*hi).hi_key.is_null()
        || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
    {
        return;
    }
    let mut s: *mut ::core::ffi::c_char =
        xmemdupz(word as *const ::core::ffi::c_void, word_len) as *mut ::core::ffi::c_char;
    hash_add_item(&raw mut (*su).su_banned, hi, s, hash);
}
unsafe extern "C" fn rescore_suggestions(mut su: *mut suginfo_T) {
    if !(*su).su_sallang.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*su).su_ga.ga_len {
            rescore_one(
                su,
                ((*su).su_ga.ga_data as *mut suggest_T).offset(i as isize),
            );
            i += 1;
        }
    }
}
unsafe extern "C" fn rescore_one(mut su: *mut suginfo_T, mut stp: *mut suggest_T) {
    let mut slang: *mut slang_T = (*stp).st_slang;
    let mut sal_badword: [::core::ffi::c_char; 254] = [0; 254];
    if !slang.is_null()
        && !((*slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int)
        && !(*stp).st_had_bonus
    {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if slang == (*su).su_sallang {
            p = &raw mut (*su).su_sal_badword as *mut ::core::ffi::c_char;
        } else {
            spell_soundfold(
                slang,
                &raw mut (*su).su_fbadword as *mut ::core::ffi::c_char,
                true_0 != 0,
                &raw mut sal_badword as *mut ::core::ffi::c_char,
            );
            p = &raw mut sal_badword as *mut ::core::ffi::c_char;
        }
        (*stp).st_altscore = stp_sal_score(stp, su, slang, p);
        if (*stp).st_altscore == SCORE_MAXMAX as ::core::ffi::c_int {
            (*stp).st_altscore = SCORE_INS as ::core::ffi::c_int * 3 as ::core::ffi::c_int;
        }
        (*stp).st_score = (3 as ::core::ffi::c_int * (*stp).st_score + (*stp).st_altscore)
            / 4 as ::core::ffi::c_int;
        (*stp).st_had_bonus = true_0 != 0;
    }
}
unsafe extern "C" fn sug_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut p1: *mut suggest_T = s1 as *mut suggest_T;
    let mut p2: *mut suggest_T = s2 as *mut suggest_T;
    let mut n: ::core::ffi::c_int = if (*p1).st_score == (*p2).st_score {
        0 as ::core::ffi::c_int
    } else if (*p1).st_score > (*p2).st_score {
        1 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
    if n == 0 as ::core::ffi::c_int {
        n = if (*p1).st_altscore == (*p2).st_altscore {
            0 as ::core::ffi::c_int
        } else if (*p1).st_altscore > (*p2).st_altscore {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
        if n == 0 as ::core::ffi::c_int {
            n = strcasecmp((*p1).st_word, (*p2).st_word);
        }
    }
    return n;
}
unsafe extern "C" fn cleanup_suggestions(
    mut gap: *mut garray_T,
    mut maxscore: ::core::ffi::c_int,
    mut keep: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*gap).ga_len <= 0 as ::core::ffi::c_int {
        return maxscore;
    }
    qsort(
        (*gap).ga_data,
        (*gap).ga_len as size_t,
        ::core::mem::size_of::<suggest_T>(),
        Some(
            sug_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    if (*gap).ga_len > keep {
        let stp: *mut suggest_T =
            ((*gap).ga_data as *mut suggest_T).offset(0 as ::core::ffi::c_int as isize);
        let mut i: ::core::ffi::c_int = keep;
        while i < (*gap).ga_len {
            xfree((*stp.offset(i as isize)).st_word as *mut ::core::ffi::c_void);
            i += 1;
        }
        (*gap).ga_len = keep;
        if keep >= 1 as ::core::ffi::c_int {
            return (*stp.offset((keep - 1 as ::core::ffi::c_int) as isize)).st_score;
        }
    }
    return maxscore;
}
unsafe extern "C" fn soundalike_score(
    mut goodstart: *mut ::core::ffi::c_char,
    mut badstart: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut goodsound: *mut ::core::ffi::c_char = goodstart;
    let mut badsound: *mut ::core::ffi::c_char = badstart;
    let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*badsound as ::core::ffi::c_int == '*' as ::core::ffi::c_int
        || *goodsound as ::core::ffi::c_int == '*' as ::core::ffi::c_int)
        && *badsound as ::core::ffi::c_int != *goodsound as ::core::ffi::c_int
    {
        if *badsound.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            && *goodsound.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || *goodsound.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                && *badsound.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            return SCORE_DEL as ::core::ffi::c_int;
        }
        if *badsound.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || *goodsound.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            return SCORE_MAXMAX as ::core::ffi::c_int;
        }
        if !(*badsound.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == *goodsound.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            || *badsound.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *goodsound.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *badsound.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *goodsound.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        {
            score =
                2 as ::core::ffi::c_int * SCORE_DEL as ::core::ffi::c_int / 3 as ::core::ffi::c_int;
            if *badsound as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                badsound = badsound.offset(1);
            } else {
                goodsound = goodsound.offset(1);
            }
        }
    }
    let mut goodlen: ::core::ffi::c_int = strlen(goodsound) as ::core::ffi::c_int;
    let mut badlen: ::core::ffi::c_int = strlen(badsound) as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = goodlen - badlen;
    if n < -2 as ::core::ffi::c_int || n > 2 as ::core::ffi::c_int {
        return SCORE_MAXMAX as ::core::ffi::c_int;
    }
    let mut pl: *mut ::core::ffi::c_char = if n > 0 as ::core::ffi::c_int {
        goodsound
    } else {
        badsound
    };
    let mut ps: *mut ::core::ffi::c_char = if n > 0 as ::core::ffi::c_int {
        badsound
    } else {
        goodsound
    };
    while *pl as ::core::ffi::c_int == *ps as ::core::ffi::c_int && *pl as ::core::ffi::c_int != NUL
    {
        pl = pl.offset(1);
        ps = ps.offset(1);
    }
    let mut pl2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ps2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match n {
        -2 | 2 => {
            pl = pl.offset(1);
            while *pl as ::core::ffi::c_int == *ps as ::core::ffi::c_int {
                pl = pl.offset(1);
                ps = ps.offset(1);
            }
            if strcmp(pl.offset(1 as ::core::ffi::c_int as isize), ps) == 0 as ::core::ffi::c_int {
                return score + SCORE_DEL as ::core::ffi::c_int * 2 as ::core::ffi::c_int;
            }
        }
        -1 | 1 => {
            pl2 = pl.offset(1 as ::core::ffi::c_int as isize);
            ps2 = ps;
            while *pl2 as ::core::ffi::c_int == *ps2 as ::core::ffi::c_int {
                if *pl2 as ::core::ffi::c_int == NUL {
                    return score + SCORE_DEL as ::core::ffi::c_int;
                }
                pl2 = pl2.offset(1);
                ps2 = ps2.offset(1);
            }
            if *pl2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == *ps2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                && *pl2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *ps2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                && strcmp(
                    pl2.offset(2 as ::core::ffi::c_int as isize),
                    ps2.offset(2 as ::core::ffi::c_int as isize),
                ) == 0 as ::core::ffi::c_int
            {
                return score + SCORE_DEL as ::core::ffi::c_int + SCORE_SWAP as ::core::ffi::c_int;
            }
            if strcmp(
                pl2.offset(1 as ::core::ffi::c_int as isize),
                ps2.offset(1 as ::core::ffi::c_int as isize),
            ) == 0 as ::core::ffi::c_int
            {
                return score + SCORE_DEL as ::core::ffi::c_int + SCORE_SUBST as ::core::ffi::c_int;
            }
            if *pl.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == *ps.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                && *pl.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *ps.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            {
                pl2 = pl.offset(2 as ::core::ffi::c_int as isize);
                ps2 = ps.offset(2 as ::core::ffi::c_int as isize);
                while *pl2 as ::core::ffi::c_int == *ps2 as ::core::ffi::c_int {
                    pl2 = pl2.offset(1);
                    ps2 = ps2.offset(1);
                }
                if strcmp(pl2.offset(1 as ::core::ffi::c_int as isize), ps2)
                    == 0 as ::core::ffi::c_int
                {
                    return score
                        + SCORE_SWAP as ::core::ffi::c_int
                        + SCORE_DEL as ::core::ffi::c_int;
                }
            }
            pl2 = pl.offset(1 as ::core::ffi::c_int as isize);
            ps2 = ps.offset(1 as ::core::ffi::c_int as isize);
            while *pl2 as ::core::ffi::c_int == *ps2 as ::core::ffi::c_int {
                pl2 = pl2.offset(1);
                ps2 = ps2.offset(1);
            }
            if strcmp(pl2.offset(1 as ::core::ffi::c_int as isize), ps2) == 0 as ::core::ffi::c_int
            {
                return score + SCORE_SUBST as ::core::ffi::c_int + SCORE_DEL as ::core::ffi::c_int;
            }
        }
        0 => {
            if *pl as ::core::ffi::c_int == NUL {
                return score;
            }
            if *pl.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == *ps.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                && *pl.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *ps.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            {
                pl2 = pl.offset(2 as ::core::ffi::c_int as isize);
                ps2 = ps.offset(2 as ::core::ffi::c_int as isize);
                while *pl2 as ::core::ffi::c_int == *ps2 as ::core::ffi::c_int {
                    if *pl2 as ::core::ffi::c_int == NUL {
                        return score + SCORE_SWAP as ::core::ffi::c_int;
                    }
                    pl2 = pl2.offset(1);
                    ps2 = ps2.offset(1);
                }
                if *pl2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *ps2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    && *pl2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == *ps2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    && strcmp(
                        pl2.offset(2 as ::core::ffi::c_int as isize),
                        ps2.offset(2 as ::core::ffi::c_int as isize),
                    ) == 0 as ::core::ffi::c_int
                {
                    return score
                        + SCORE_SWAP as ::core::ffi::c_int
                        + SCORE_SWAP as ::core::ffi::c_int;
                }
                if strcmp(
                    pl2.offset(1 as ::core::ffi::c_int as isize),
                    ps2.offset(1 as ::core::ffi::c_int as isize),
                ) == 0 as ::core::ffi::c_int
                {
                    return score
                        + SCORE_SWAP as ::core::ffi::c_int
                        + SCORE_SUBST as ::core::ffi::c_int;
                }
            }
            pl2 = pl.offset(1 as ::core::ffi::c_int as isize);
            ps2 = ps.offset(1 as ::core::ffi::c_int as isize);
            while *pl2 as ::core::ffi::c_int == *ps2 as ::core::ffi::c_int {
                if *pl2 as ::core::ffi::c_int == NUL {
                    return score + SCORE_SUBST as ::core::ffi::c_int;
                }
                pl2 = pl2.offset(1);
                ps2 = ps2.offset(1);
            }
            if *pl2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == *ps2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                && *pl2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == *ps2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                && strcmp(
                    pl2.offset(2 as ::core::ffi::c_int as isize),
                    ps2.offset(2 as ::core::ffi::c_int as isize),
                ) == 0 as ::core::ffi::c_int
            {
                return score
                    + SCORE_SUBST as ::core::ffi::c_int
                    + SCORE_SWAP as ::core::ffi::c_int;
            }
            if strcmp(
                pl2.offset(1 as ::core::ffi::c_int as isize),
                ps2.offset(1 as ::core::ffi::c_int as isize),
            ) == 0 as ::core::ffi::c_int
            {
                return score
                    + SCORE_SUBST as ::core::ffi::c_int
                    + SCORE_SUBST as ::core::ffi::c_int;
            }
            pl2 = pl;
            ps2 = ps.offset(1 as ::core::ffi::c_int as isize);
            while *pl2 as ::core::ffi::c_int == *ps2 as ::core::ffi::c_int {
                pl2 = pl2.offset(1);
                ps2 = ps2.offset(1);
            }
            if strcmp(pl2.offset(1 as ::core::ffi::c_int as isize), ps2) == 0 as ::core::ffi::c_int
            {
                return score + SCORE_INS as ::core::ffi::c_int + SCORE_DEL as ::core::ffi::c_int;
            }
            pl2 = pl.offset(1 as ::core::ffi::c_int as isize);
            ps2 = ps;
            while *pl2 as ::core::ffi::c_int == *ps2 as ::core::ffi::c_int {
                pl2 = pl2.offset(1);
                ps2 = ps2.offset(1);
            }
            if strcmp(pl2, ps2.offset(1 as ::core::ffi::c_int as isize)) == 0 as ::core::ffi::c_int
            {
                return score + SCORE_INS as ::core::ffi::c_int + SCORE_DEL as ::core::ffi::c_int;
            }
        }
        _ => {}
    }
    return SCORE_MAXMAX as ::core::ffi::c_int;
}
unsafe extern "C" fn spell_edit_score(
    mut slang: *mut slang_T,
    mut badword: *const ::core::ffi::c_char,
    mut goodword: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut wbadword: [::core::ffi::c_int; 254] = [0; 254];
    let mut wgoodword: [::core::ffi::c_int; 254] = [0; 254];
    let mut badlen: ::core::ffi::c_int = 0;
    let mut goodlen: ::core::ffi::c_int = 0;
    badlen = 0 as ::core::ffi::c_int;
    let mut p: *const ::core::ffi::c_char = badword;
    while *p as ::core::ffi::c_int != NUL {
        let c2rust_fresh1 = badlen;
        badlen = badlen + 1;
        wbadword[c2rust_fresh1 as usize] = mb_cptr2char_adv(&raw mut p);
    }
    let c2rust_fresh2 = badlen;
    badlen = badlen + 1;
    wbadword[c2rust_fresh2 as usize] = 0 as ::core::ffi::c_int;
    goodlen = 0 as ::core::ffi::c_int;
    let mut p_0: *const ::core::ffi::c_char = goodword;
    while *p_0 as ::core::ffi::c_int != NUL {
        let c2rust_fresh3 = goodlen;
        goodlen = goodlen + 1;
        wgoodword[c2rust_fresh3 as usize] = mb_cptr2char_adv(&raw mut p_0);
    }
    let c2rust_fresh4 = goodlen;
    goodlen = goodlen + 1;
    wgoodword[c2rust_fresh4 as usize] = 0 as ::core::ffi::c_int;
    let mut cnt: *mut ::core::ffi::c_int = xmalloc(
        ::core::mem::size_of::<::core::ffi::c_int>()
            .wrapping_mul((badlen as size_t).wrapping_add(1 as size_t))
            .wrapping_mul((goodlen as size_t).wrapping_add(1 as size_t)),
    ) as *mut ::core::ffi::c_int;
    *cnt.offset(
        (0 as ::core::ffi::c_int + 0 as ::core::ffi::c_int * (badlen + 1 as ::core::ffi::c_int))
            as isize,
    ) = 0 as ::core::ffi::c_int;
    let mut j: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while j <= goodlen {
        *cnt.offset((0 as ::core::ffi::c_int + j * (badlen + 1 as ::core::ffi::c_int)) as isize) =
            *cnt.offset(
                (0 as ::core::ffi::c_int
                    + (j - 1 as ::core::ffi::c_int) * (badlen + 1 as ::core::ffi::c_int))
                    as isize,
            ) + SCORE_INS as ::core::ffi::c_int;
        j += 1;
    }
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i <= badlen {
        *cnt.offset((i + 0 as ::core::ffi::c_int * (badlen + 1 as ::core::ffi::c_int)) as isize) =
            *cnt.offset(
                (i - 1 as ::core::ffi::c_int
                    + 0 as ::core::ffi::c_int * (badlen + 1 as ::core::ffi::c_int))
                    as isize,
            ) + SCORE_DEL as ::core::ffi::c_int;
        let mut j_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while j_0 <= goodlen {
            let mut bc: ::core::ffi::c_int = wbadword[(i - 1 as ::core::ffi::c_int) as usize];
            let mut gc: ::core::ffi::c_int = wgoodword[(j_0 - 1 as ::core::ffi::c_int) as usize];
            if bc == gc {
                *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize) = *cnt.offset(
                    (i - 1 as ::core::ffi::c_int
                        + (j_0 - 1 as ::core::ffi::c_int) * (badlen + 1 as ::core::ffi::c_int))
                        as isize,
                );
            } else {
                if (if bc >= 128 as ::core::ffi::c_int {
                    utf_fold(bc)
                } else {
                    (*spelltab.ptr()).st_fold[bc as usize] as ::core::ffi::c_int
                }) == (if gc >= 128 as ::core::ffi::c_int {
                    utf_fold(gc)
                } else {
                    (*spelltab.ptr()).st_fold[gc as usize] as ::core::ffi::c_int
                }) {
                    *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize) =
                        SCORE_ICASE as ::core::ffi::c_int
                            + *cnt.offset(
                                (i - 1 as ::core::ffi::c_int
                                    + (j_0 - 1 as ::core::ffi::c_int)
                                        * (badlen + 1 as ::core::ffi::c_int))
                                    as isize,
                            );
                } else if !slang.is_null()
                    && (*slang).sl_has_map as ::core::ffi::c_int != 0
                    && similar_chars(slang, gc, bc) as ::core::ffi::c_int != 0
                {
                    *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize) =
                        SCORE_SIMILAR as ::core::ffi::c_int
                            + *cnt.offset(
                                (i - 1 as ::core::ffi::c_int
                                    + (j_0 - 1 as ::core::ffi::c_int)
                                        * (badlen + 1 as ::core::ffi::c_int))
                                    as isize,
                            );
                } else {
                    *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize) =
                        SCORE_SUBST as ::core::ffi::c_int
                            + *cnt.offset(
                                (i - 1 as ::core::ffi::c_int
                                    + (j_0 - 1 as ::core::ffi::c_int)
                                        * (badlen + 1 as ::core::ffi::c_int))
                                    as isize,
                            );
                }
                if i > 1 as ::core::ffi::c_int && j_0 > 1 as ::core::ffi::c_int {
                    let mut pbc: ::core::ffi::c_int =
                        wbadword[(i - 2 as ::core::ffi::c_int) as usize];
                    let mut pgc: ::core::ffi::c_int =
                        wgoodword[(j_0 - 2 as ::core::ffi::c_int) as usize];
                    if bc == pgc && pbc == gc {
                        let mut t: ::core::ffi::c_int = SCORE_SWAP as ::core::ffi::c_int
                            + *cnt.offset(
                                (i - 2 as ::core::ffi::c_int
                                    + (j_0 - 2 as ::core::ffi::c_int)
                                        * (badlen + 1 as ::core::ffi::c_int))
                                    as isize,
                            );
                        *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize) =
                            if *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize)
                                < t
                            {
                                *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize)
                            } else {
                                t
                            };
                    }
                }
                let mut t_0: ::core::ffi::c_int = SCORE_DEL as ::core::ffi::c_int
                    + *cnt.offset(
                        (i - 1 as ::core::ffi::c_int + j_0 * (badlen + 1 as ::core::ffi::c_int))
                            as isize,
                    );
                *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize) =
                    if *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize) < t_0 {
                        *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize)
                    } else {
                        t_0
                    };
                t_0 = SCORE_INS as ::core::ffi::c_int
                    + *cnt.offset(
                        (i + (j_0 - 1 as ::core::ffi::c_int) * (badlen + 1 as ::core::ffi::c_int))
                            as isize,
                    );
                *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize) =
                    if *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize) < t_0 {
                        *cnt.offset((i + j_0 * (badlen + 1 as ::core::ffi::c_int)) as isize)
                    } else {
                        t_0
                    };
            }
            j_0 += 1;
        }
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = *cnt.offset(
        (badlen - 1 as ::core::ffi::c_int
            + (goodlen - 1 as ::core::ffi::c_int) * (badlen + 1 as ::core::ffi::c_int))
            as isize,
    );
    xfree(cnt as *mut ::core::ffi::c_void);
    return i_0;
}
unsafe extern "C" fn spell_edit_score_limit(
    mut slang: *mut slang_T,
    mut badword: *mut ::core::ffi::c_char,
    mut goodword: *mut ::core::ffi::c_char,
    mut limit: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return spell_edit_score_limit_w(slang, badword, goodword, limit);
}
unsafe extern "C" fn spell_edit_score_limit_w(
    mut slang: *mut slang_T,
    mut badword: *const ::core::ffi::c_char,
    mut goodword: *const ::core::ffi::c_char,
    mut limit: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut stack: [limitscore_T; 10] = [limitscore_T {
        badi: 0,
        goodi: 0,
        score: 0,
    }; 10];
    let mut bc: ::core::ffi::c_int = 0;
    let mut gc: ::core::ffi::c_int = 0;
    let mut score_off: ::core::ffi::c_int = 0;
    let mut wbadword: [::core::ffi::c_int; 254] = [0; 254];
    let mut wgoodword: [::core::ffi::c_int; 254] = [0; 254];
    let mut bi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *const ::core::ffi::c_char = badword;
    while *p as ::core::ffi::c_int != NUL {
        let c2rust_fresh11 = bi;
        bi = bi + 1;
        wbadword[c2rust_fresh11 as usize] = mb_cptr2char_adv(&raw mut p);
    }
    let c2rust_fresh12 = bi;
    bi = bi + 1;
    wbadword[c2rust_fresh12 as usize] = 0 as ::core::ffi::c_int;
    let mut gi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p_0: *const ::core::ffi::c_char = goodword;
    while *p_0 as ::core::ffi::c_int != NUL {
        let c2rust_fresh13 = gi;
        gi = gi + 1;
        wgoodword[c2rust_fresh13 as usize] = mb_cptr2char_adv(&raw mut p_0);
    }
    let c2rust_fresh14 = gi;
    gi = gi + 1;
    wgoodword[c2rust_fresh14 as usize] = 0 as ::core::ffi::c_int;
    let mut stackidx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    bi = 0 as ::core::ffi::c_int;
    gi = 0 as ::core::ffi::c_int;
    let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut minscore: ::core::ffi::c_int = limit + 1 as ::core::ffi::c_int;
    's_350: loop {
        bc = wbadword[bi as usize];
        gc = wgoodword[gi as usize];
        '_pop: {
            if bc != gc {
                if gc == NUL {
                    loop {
                        score += SCORE_DEL as ::core::ffi::c_int;
                        if score >= minscore {
                            break '_pop;
                        } else {
                            bi += 1;
                            if wbadword[bi as usize] == NUL {
                                break;
                            }
                        }
                    }
                    minscore = score;
                } else if bc == NUL {
                    loop {
                        score += SCORE_INS as ::core::ffi::c_int;
                        if score >= minscore {
                            break '_pop;
                        } else {
                            gi += 1;
                            if wgoodword[gi as usize] == NUL {
                                break;
                            }
                        }
                    }
                    minscore = score;
                } else {
                    let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while round <= 1 as ::core::ffi::c_int {
                        score_off = score
                            + (if round == 0 as ::core::ffi::c_int {
                                SCORE_DEL as ::core::ffi::c_int
                            } else {
                                SCORE_INS as ::core::ffi::c_int
                            });
                        if score_off < minscore {
                            if score_off + SCORE_SIMILAR as ::core::ffi::c_int >= minscore {
                                let mut bi2: ::core::ffi::c_int =
                                    bi + 1 as ::core::ffi::c_int - round;
                                let mut gi2: ::core::ffi::c_int = gi + round;
                                while wgoodword[gi2 as usize] == wbadword[bi2 as usize] {
                                    if wgoodword[gi2 as usize] == NUL {
                                        minscore = score_off;
                                        break;
                                    } else {
                                        bi2 += 1;
                                        gi2 += 1;
                                    }
                                }
                            } else {
                                stack[stackidx as usize].badi =
                                    bi + 1 as ::core::ffi::c_int - round;
                                stack[stackidx as usize].goodi = gi + round;
                                stack[stackidx as usize].score = score_off;
                                stackidx += 1;
                            }
                        }
                        round += 1;
                    }
                    if (score + SCORE_SWAP as ::core::ffi::c_int) < minscore {
                        if gc == wbadword[(bi + 1 as ::core::ffi::c_int) as usize]
                            && bc == wgoodword[(gi + 1 as ::core::ffi::c_int) as usize]
                        {
                            gi += 2 as ::core::ffi::c_int;
                            bi += 2 as ::core::ffi::c_int;
                            score += SCORE_SWAP as ::core::ffi::c_int;
                            continue 's_350;
                        }
                    }
                    if (if bc >= 128 as ::core::ffi::c_int {
                        utf_fold(bc)
                    } else {
                        (*spelltab.ptr()).st_fold[bc as usize] as ::core::ffi::c_int
                    }) == (if gc >= 128 as ::core::ffi::c_int {
                        utf_fold(gc)
                    } else {
                        (*spelltab.ptr()).st_fold[gc as usize] as ::core::ffi::c_int
                    }) {
                        score += SCORE_ICASE as ::core::ffi::c_int;
                    } else if !slang.is_null()
                        && (*slang).sl_has_map as ::core::ffi::c_int != 0
                        && similar_chars(slang, gc, bc) as ::core::ffi::c_int != 0
                    {
                        score += SCORE_SIMILAR as ::core::ffi::c_int;
                    } else {
                        score += SCORE_SUBST as ::core::ffi::c_int;
                    }
                    if score < minscore {
                        gi += 1;
                        bi += 1;
                        continue 's_350;
                    }
                }
            } else if bc == NUL {
                if score < minscore {
                    minscore = score;
                }
            } else {
                bi += 1;
                gi += 1;
                continue 's_350;
            }
        }
        if stackidx == 0 as ::core::ffi::c_int {
            break;
        }
        stackidx -= 1;
        gi = stack[stackidx as usize].goodi;
        bi = stack[stackidx as usize].badi;
        score = stack[stackidx as usize].score;
    }
    if minscore > limit {
        return SCORE_MAXMAX as ::core::ffi::c_int;
    }
    return minscore;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
