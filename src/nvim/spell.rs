use crate::src::nvim::autocmd::apply_autocmds;
use crate::src::nvim::buffer::{buf_is_empty, bufref_valid, set_bufref};
use crate::src::nvim::change::inserted_bytes;
use crate::src::nvim::charset::{
    getwhitecols, skipbin, skipdigits, skiphex, skipwhite, vim_is_fname_char,
};
use crate::src::nvim::cursor::{get_cursor_line_len, get_cursor_line_ptr};
use crate::src::nvim::decoration::{
    decor_redraw_col_impl, decor_redraw_line, decor_redraw_reset, decor_state_free,
};
use crate::src::nvim::decoration_provider::decor_providers_invoke_spell;
use crate::src::nvim::drawscreen::redraw_later;
use crate::src::nvim::ex_cmds::do_sub_msg;
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::insexpand::{
    ins_compl_add_infercase, ins_compl_check_keys, ins_compl_interrupted,
};
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{
    bot_top_msg, curbuf, curtab, curwin, decor_state, e_invarg, e_no_spell, firstbuf, firstwin,
    got_int, p_enc, p_ic, p_ws, starting, sub_nlines, sub_nsubs, top_bot_msg, IObuff,
};
use crate::src::nvim::mbyte::{
    mb_charlen_len, mb_cptr2char_adv, mb_get_class, mb_islower, mb_isupper, mb_ptr2char_adv,
    mb_strnicmp, mb_toupper, utf_char2bytes, utf_class, utf_fold, utf_head_off, utf_ptr2char,
    utfc_ptr2len,
};
use crate::src::nvim::memline::{
    ml_append, ml_close, ml_delete, ml_get_buf, ml_get_buf_len, ml_open, ml_open_file, ml_replace,
};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemcpyz, xmemdupz, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    emsg, give_warning, msg_end, msg_ext_set_kind, msg_putchar, msg_puts, msg_start, semsg, smsg,
};
use crate::src::nvim::option::{
    copy_option_part, get_option_value, optval_free, set_option_value_give_err, shortmess,
    valid_name,
};
use crate::src::nvim::os::fs::os_remove;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, gettext, memcpy, memmove, memset, snprintf, strcasecmp, strcat, strcmp, strcpy,
    strlen, strncmp, strstr,
};
use crate::src::nvim::path::{path_fnamecmp, path_full_compare, path_tail};
use crate::src::nvim::runtime::do_in_runtimepath;
use crate::src::nvim::search::do_search;
use crate::src::nvim::spellfile::spell_load_file;
use crate::src::nvim::spellsuggest::spell_suggest_list;
use crate::src::nvim::strings::{concat_str, vim_snprintf, vim_strchr, xstrnsave};
use crate::src::nvim::syntax::{syn_get_id, syntax_present};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CMD_index, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_4, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorPriorityInternal, DecorRange,
    DecorRangeKind, DecorRangeSlot, DecorRange_data as C2Rust_Unnamed_17,
    DecorRange_data_ui as C2Rust_Unnamed_18, DecorSignHighlight, DecorState,
    DecorState_ranges_i as C2Rust_Unnamed_19, DecorState_slots as C2Rust_Unnamed_20, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_1, Direction, DoInRuntimepathCB, ExtmarkUndoObject,
    FileComparison, FileID, FloatAnchor, FloatRelative, GridView, Intersection, LineGetter, LuaRef,
    MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_13, MotionType,
    OptIndex, OptInt, OptVal, OptValData, OptValType, ScopeDictDictItem, ScopeType, ScreenGrid,
    Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_11, String_0, Terminal, Timestamp, TriState,
    VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, __time_t, alist_T, auto_event, bhdr_T, blob_T, blobvar_S,
    blocknr_T, buf_T, bufref_T, bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_14, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    eslist_T, eslist_elem, event_T, exarg, exarg_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_2, file_buffer_b_wininfo as C2Rust_Unnamed_10,
    file_buffer_update_callbacks as C2Rust_Unnamed,
    file_buffer_update_channels as C2Rust_Unnamed_0, file_comparison, float_T, fmark_T, fmarkv_T,
    frame_S, frame_T, fromto_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_5, funccall_T,
    garray_T, handle_T, hash_T, hashitem_T, hashtab_T, hlf_T, idx_T, infoptr_T, int16_t, int32_t,
    int64_t, intptr_t, langp_T, lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S,
    listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem,
    matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s, mtnode_s, oparg_T, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmatch_T, regmmatch_T, regprog, regprog_T, salfirst_T, salitem_T, sattr_T,
    schar_T, scid_T, sctx_T, searchit_arg_T, size_t, slang_S, slang_T, smt_T, spelltab_T,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_3, syn_time_T, synblock_T, synstate_T,
    tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_7,
    u_header_uh_alt_prev as C2Rust_Unnamed_6, u_header_uh_next as C2Rust_Unnamed_9,
    u_header_uh_prev as C2Rust_Unnamed_8, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, wordcount_T, xfmark_T, QUEUE,
};
use crate::src::nvim::undo::u_save_cursor;
use crate::src::nvim::window::win_valid_any_tab;
extern "C" {
    fn ga_clear(gap: *mut garray_T);
    fn ga_clear_strings(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_append_via_ptr(gap: *mut garray_T, item_size: size_t) -> *mut ::core::ffi::c_void;
    fn hash_init(ht: *mut hashtab_T);
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
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_prog(
        prog: *mut *mut regprog_T,
        ignore_case: bool,
        line: *const ::core::ffi::c_char,
        col: colnr_T,
    ) -> bool;
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
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
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const kOptWritedelay: OptIndex = 373;
pub const kOptWritebackup: OptIndex = 372;
pub const kOptWriteany: OptIndex = 371;
pub const kOptWrite: OptIndex = 370;
pub const kOptWrapscan: OptIndex = 369;
pub const kOptWrapmargin: OptIndex = 368;
pub const kOptWrap: OptIndex = 367;
pub const kOptWinwidth: OptIndex = 366;
pub const kOptWinminwidth: OptIndex = 365;
pub const kOptWinminheight: OptIndex = 364;
pub const kOptWinhighlight: OptIndex = 363;
pub const kOptWinheight: OptIndex = 362;
pub const kOptWinfixwidth: OptIndex = 361;
pub const kOptWinfixheight: OptIndex = 360;
pub const kOptWinfixbuf: OptIndex = 359;
pub const kOptWindow: OptIndex = 358;
pub const kOptWinborder: OptIndex = 357;
pub const kOptWinblend: OptIndex = 356;
pub const kOptWinbar: OptIndex = 355;
pub const kOptWinaltkeys: OptIndex = 354;
pub const kOptWildoptions: OptIndex = 353;
pub const kOptWildmode: OptIndex = 352;
pub const kOptWildmenu: OptIndex = 351;
pub const kOptWildignorecase: OptIndex = 350;
pub const kOptWildignore: OptIndex = 349;
pub const kOptWildcharm: OptIndex = 348;
pub const kOptWildchar: OptIndex = 347;
pub const kOptWhichwrap: OptIndex = 346;
pub const kOptWarn: OptIndex = 345;
pub const kOptVisualbell: OptIndex = 344;
pub const kOptVirtualedit: OptIndex = 343;
pub const kOptViewoptions: OptIndex = 342;
pub const kOptViewdir: OptIndex = 341;
pub const kOptVerbosefile: OptIndex = 340;
pub const kOptVerbose: OptIndex = 339;
pub const kOptVartabstop: OptIndex = 338;
pub const kOptVarsofttabstop: OptIndex = 337;
pub const kOptUpdatetime: OptIndex = 336;
pub const kOptUpdatecount: OptIndex = 335;
pub const kOptUndoreload: OptIndex = 334;
pub const kOptUndolevels: OptIndex = 333;
pub const kOptUndofile: OptIndex = 332;
pub const kOptUndodir: OptIndex = 331;
pub const kOptTtyfast: OptIndex = 330;
pub const kOptTtimeoutlen: OptIndex = 329;
pub const kOptTtimeout: OptIndex = 328;
pub const kOptTitlestring: OptIndex = 327;
pub const kOptTitleold: OptIndex = 326;
pub const kOptTitlelen: OptIndex = 325;
pub const kOptTitle: OptIndex = 324;
pub const kOptTimeoutlen: OptIndex = 323;
pub const kOptTimeout: OptIndex = 322;
pub const kOptTildeop: OptIndex = 321;
pub const kOptThesaurusfunc: OptIndex = 320;
pub const kOptThesaurus: OptIndex = 319;
pub const kOptTextwidth: OptIndex = 318;
pub const kOptTerse: OptIndex = 317;
pub const kOptTermsync: OptIndex = 316;
pub const kOptTermpastefilter: OptIndex = 315;
pub const kOptTermguicolors: OptIndex = 314;
pub const kOptTermencoding: OptIndex = 313;
pub const kOptTermbidi: OptIndex = 312;
pub const kOptTagstack: OptIndex = 311;
pub const kOptTags: OptIndex = 310;
pub const kOptTagrelative: OptIndex = 309;
pub const kOptTaglength: OptIndex = 308;
pub const kOptTagfunc: OptIndex = 307;
pub const kOptTagcase: OptIndex = 306;
pub const kOptTagbsearch: OptIndex = 305;
pub const kOptTabstop: OptIndex = 304;
pub const kOptTabpagemax: OptIndex = 303;
pub const kOptTabline: OptIndex = 302;
pub const kOptTabclose: OptIndex = 301;
pub const kOptSyntax: OptIndex = 300;
pub const kOptSynmaxcol: OptIndex = 299;
pub const kOptSwitchbuf: OptIndex = 298;
pub const kOptSwapfile: OptIndex = 297;
pub const kOptSuffixesadd: OptIndex = 296;
pub const kOptSuffixes: OptIndex = 295;
pub const kOptStatusline: OptIndex = 294;
pub const kOptStatuscolumn: OptIndex = 293;
pub const kOptStartofline: OptIndex = 292;
pub const kOptSplitright: OptIndex = 291;
pub const kOptSplitkeep: OptIndex = 290;
pub const kOptSplitbelow: OptIndex = 289;
pub const kOptSpellsuggest: OptIndex = 288;
pub const kOptSpelloptions: OptIndex = 287;
pub const kOptSpelllang: OptIndex = 286;
pub const kOptSpellfile: OptIndex = 285;
pub const kOptSpellcapcheck: OptIndex = 284;
pub const kOptSpell: OptIndex = 283;
pub const kOptSofttabstop: OptIndex = 282;
pub const kOptSmoothscroll: OptIndex = 281;
pub const kOptSmarttab: OptIndex = 280;
pub const kOptSmartindent: OptIndex = 279;
pub const kOptSmartcase: OptIndex = 278;
pub const kOptSigncolumn: OptIndex = 277;
pub const kOptSidescrolloff: OptIndex = 276;
pub const kOptSidescroll: OptIndex = 275;
pub const kOptShowtabline: OptIndex = 274;
pub const kOptShowmode: OptIndex = 273;
pub const kOptShowmatch: OptIndex = 272;
pub const kOptShowfulltag: OptIndex = 271;
pub const kOptShowcmdloc: OptIndex = 270;
pub const kOptShowcmd: OptIndex = 269;
pub const kOptShowbreak: OptIndex = 268;
pub const kOptShortmess: OptIndex = 267;
pub const kOptShiftwidth: OptIndex = 266;
pub const kOptShiftround: OptIndex = 265;
pub const kOptShellxquote: OptIndex = 264;
pub const kOptShellxescape: OptIndex = 263;
pub const kOptShelltemp: OptIndex = 262;
pub const kOptShellslash: OptIndex = 261;
pub const kOptShellredir: OptIndex = 260;
pub const kOptShellquote: OptIndex = 259;
pub const kOptShellpipe: OptIndex = 258;
pub const kOptShellcmdflag: OptIndex = 257;
pub const kOptShell: OptIndex = 256;
pub const kOptShadafile: OptIndex = 255;
pub const kOptShada: OptIndex = 254;
pub const kOptSessionoptions: OptIndex = 253;
pub const kOptSelectmode: OptIndex = 252;
pub const kOptSelection: OptIndex = 251;
pub const kOptSecure: OptIndex = 250;
pub const kOptSections: OptIndex = 249;
pub const kOptScrollopt: OptIndex = 248;
pub const kOptScrolloff: OptIndex = 247;
pub const kOptScrolljump: OptIndex = 246;
pub const kOptScrollbind: OptIndex = 245;
pub const kOptScrollback: OptIndex = 244;
pub const kOptScroll: OptIndex = 243;
pub const kOptRuntimepath: OptIndex = 242;
pub const kOptRulerformat: OptIndex = 241;
pub const kOptRuler: OptIndex = 240;
pub const kOptRightleftcmd: OptIndex = 239;
pub const kOptRightleft: OptIndex = 238;
pub const kOptRevins: OptIndex = 237;
pub const kOptReport: OptIndex = 236;
pub const kOptRemap: OptIndex = 235;
pub const kOptRelativenumber: OptIndex = 234;
pub const kOptRegexpengine: OptIndex = 233;
pub const kOptRedrawtime: OptIndex = 232;
pub const kOptRedrawdebug: OptIndex = 231;
pub const kOptReadonly: OptIndex = 230;
pub const kOptQuoteescape: OptIndex = 229;
pub const kOptQuickfixtextfunc: OptIndex = 228;
pub const kOptPyxversion: OptIndex = 227;
pub const kOptPumwidth: OptIndex = 226;
pub const kOptPummaxwidth: OptIndex = 225;
pub const kOptPumheight: OptIndex = 224;
pub const kOptPumborder: OptIndex = 223;
pub const kOptPumblend: OptIndex = 222;
pub const kOptPrompt: OptIndex = 221;
pub const kOptPreviewwindow: OptIndex = 220;
pub const kOptPreviewheight: OptIndex = 219;
pub const kOptPreserveindent: OptIndex = 218;
pub const kOptPath: OptIndex = 217;
pub const kOptPatchmode: OptIndex = 216;
pub const kOptPatchexpr: OptIndex = 215;
pub const kOptPastetoggle: OptIndex = 214;
pub const kOptPaste: OptIndex = 213;
pub const kOptParagraphs: OptIndex = 212;
pub const kOptPackpath: OptIndex = 211;
pub const kOptOperatorfunc: OptIndex = 210;
pub const kOptOpendevice: OptIndex = 209;
pub const kOptOmnifunc: OptIndex = 208;
pub const kOptNumberwidth: OptIndex = 207;
pub const kOptNumber: OptIndex = 206;
pub const kOptNrformats: OptIndex = 205;
pub const kOptMousetime: OptIndex = 204;
pub const kOptMouseshape: OptIndex = 203;
pub const kOptMousescroll: OptIndex = 202;
pub const kOptMousemoveevent: OptIndex = 201;
pub const kOptMousemodel: OptIndex = 200;
pub const kOptMousehide: OptIndex = 199;
pub const kOptMousefocus: OptIndex = 198;
pub const kOptMouse: OptIndex = 197;
pub const kOptMore: OptIndex = 196;
pub const kOptModified: OptIndex = 195;
pub const kOptModifiable: OptIndex = 194;
pub const kOptModelines: OptIndex = 193;
pub const kOptModelineexpr: OptIndex = 192;
pub const kOptModeline: OptIndex = 191;
pub const kOptMkspellmem: OptIndex = 190;
pub const kOptMessagesopt: OptIndex = 189;
pub const kOptMenuitems: OptIndex = 188;
pub const kOptMaxsearchcount: OptIndex = 187;
pub const kOptMaxmempattern: OptIndex = 186;
pub const kOptMaxmapdepth: OptIndex = 185;
pub const kOptMaxfuncdepth: OptIndex = 184;
pub const kOptMaxcombine: OptIndex = 183;
pub const kOptMatchtime: OptIndex = 182;
pub const kOptMatchpairs: OptIndex = 181;
pub const kOptMakeprg: OptIndex = 180;
pub const kOptMakeencoding: OptIndex = 179;
pub const kOptMakeef: OptIndex = 178;
pub const kOptMagic: OptIndex = 177;
pub const kOptLoadplugins: OptIndex = 176;
pub const kOptListchars: OptIndex = 175;
pub const kOptList: OptIndex = 174;
pub const kOptLispwords: OptIndex = 173;
pub const kOptLispoptions: OptIndex = 172;
pub const kOptLisp: OptIndex = 171;
pub const kOptLinespace: OptIndex = 170;
pub const kOptLines: OptIndex = 169;
pub const kOptLinebreak: OptIndex = 168;
pub const kOptLhistory: OptIndex = 167;
pub const kOptLazyredraw: OptIndex = 166;
pub const kOptLaststatus: OptIndex = 165;
pub const kOptLangremap: OptIndex = 164;
pub const kOptLangnoremap: OptIndex = 163;
pub const kOptLangmenu: OptIndex = 162;
pub const kOptLangmap: OptIndex = 161;
pub const kOptKeywordprg: OptIndex = 160;
pub const kOptKeymodel: OptIndex = 159;
pub const kOptKeymap: OptIndex = 158;
pub const kOptJumpoptions: OptIndex = 157;
pub const kOptJoinspaces: OptIndex = 156;
pub const kOptIsprint: OptIndex = 155;
pub const kOptIskeyword: OptIndex = 154;
pub const kOptIsident: OptIndex = 153;
pub const kOptIsfname: OptIndex = 152;
pub const kOptInsertmode: OptIndex = 151;
pub const kOptInfercase: OptIndex = 150;
pub const kOptIndentkeys: OptIndex = 149;
pub const kOptIndentexpr: OptIndex = 148;
pub const kOptIncsearch: OptIndex = 147;
pub const kOptIncludeexpr: OptIndex = 146;
pub const kOptInclude: OptIndex = 145;
pub const kOptInccommand: OptIndex = 144;
pub const kOptImsearch: OptIndex = 143;
pub const kOptIminsert: OptIndex = 142;
pub const kOptImdisable: OptIndex = 141;
pub const kOptImcmdline: OptIndex = 140;
pub const kOptIgnorecase: OptIndex = 139;
pub const kOptIconstring: OptIndex = 138;
pub const kOptIcon: OptIndex = 137;
pub const kOptHlsearch: OptIndex = 136;
pub const kOptHkmapp: OptIndex = 135;
pub const kOptHkmap: OptIndex = 134;
pub const kOptHistory: OptIndex = 133;
pub const kOptHighlight: OptIndex = 132;
pub const kOptHidden: OptIndex = 131;
pub const kOptHelplang: OptIndex = 130;
pub const kOptHelpheight: OptIndex = 129;
pub const kOptHelpfile: OptIndex = 128;
pub const kOptGuitabtooltip: OptIndex = 127;
pub const kOptGuitablabel: OptIndex = 126;
pub const kOptGuioptions: OptIndex = 125;
pub const kOptGuifontwide: OptIndex = 124;
pub const kOptGuifont: OptIndex = 123;
pub const kOptGuicursor: OptIndex = 122;
pub const kOptGrepprg: OptIndex = 121;
pub const kOptGrepformat: OptIndex = 120;
pub const kOptGdefault: OptIndex = 119;
pub const kOptFsync: OptIndex = 118;
pub const kOptFormatprg: OptIndex = 117;
pub const kOptFormatoptions: OptIndex = 116;
pub const kOptFormatlistpat: OptIndex = 115;
pub const kOptFormatexpr: OptIndex = 114;
pub const kOptFoldtext: OptIndex = 113;
pub const kOptFoldopen: OptIndex = 112;
pub const kOptFoldnestmax: OptIndex = 111;
pub const kOptFoldminlines: OptIndex = 110;
pub const kOptFoldmethod: OptIndex = 109;
pub const kOptFoldmarker: OptIndex = 108;
pub const kOptFoldlevelstart: OptIndex = 107;
pub const kOptFoldlevel: OptIndex = 106;
pub const kOptFoldignore: OptIndex = 105;
pub const kOptFoldexpr: OptIndex = 104;
pub const kOptFoldenable: OptIndex = 103;
pub const kOptFoldcolumn: OptIndex = 102;
pub const kOptFoldclose: OptIndex = 101;
pub const kOptFixendofline: OptIndex = 100;
pub const kOptFindfunc: OptIndex = 99;
pub const kOptFillchars: OptIndex = 98;
pub const kOptFiletype: OptIndex = 97;
pub const kOptFileignorecase: OptIndex = 96;
pub const kOptFileformats: OptIndex = 95;
pub const kOptFileformat: OptIndex = 94;
pub const kOptFileencodings: OptIndex = 93;
pub const kOptFileencoding: OptIndex = 92;
pub const kOptExrc: OptIndex = 91;
pub const kOptExpandtab: OptIndex = 90;
pub const kOptEventignorewin: OptIndex = 89;
pub const kOptEventignore: OptIndex = 88;
pub const kOptErrorformat: OptIndex = 87;
pub const kOptErrorfile: OptIndex = 86;
pub const kOptErrorbells: OptIndex = 85;
pub const kOptEqualprg: OptIndex = 84;
pub const kOptEqualalways: OptIndex = 83;
pub const kOptEndofline: OptIndex = 82;
pub const kOptEndoffile: OptIndex = 81;
pub const kOptEncoding: OptIndex = 80;
pub const kOptEmoji: OptIndex = 79;
pub const kOptEdcompatible: OptIndex = 78;
pub const kOptEadirection: OptIndex = 77;
pub const kOptDisplay: OptIndex = 76;
pub const kOptDirectory: OptIndex = 75;
pub const kOptDigraph: OptIndex = 74;
pub const kOptDiffopt: OptIndex = 73;
pub const kOptDiffexpr: OptIndex = 72;
pub const kOptDiffanchors: OptIndex = 71;
pub const kOptDiff: OptIndex = 70;
pub const kOptDictionary: OptIndex = 69;
pub const kOptDelcombine: OptIndex = 68;
pub const kOptDefine: OptIndex = 67;
pub const kOptDebug: OptIndex = 66;
pub const kOptCursorlineopt: OptIndex = 65;
pub const kOptCursorline: OptIndex = 64;
pub const kOptCursorcolumn: OptIndex = 63;
pub const kOptCursorbind: OptIndex = 62;
pub const kOptCpoptions: OptIndex = 61;
pub const kOptCopyindent: OptIndex = 60;
pub const kOptConfirm: OptIndex = 59;
pub const kOptConceallevel: OptIndex = 58;
pub const kOptConcealcursor: OptIndex = 57;
pub const kOptCompletetimeout: OptIndex = 56;
pub const kOptCompleteslash: OptIndex = 55;
pub const kOptCompleteopt: OptIndex = 54;
pub const kOptCompleteitemalign: OptIndex = 53;
pub const kOptCompletefunc: OptIndex = 52;
pub const kOptComplete: OptIndex = 51;
pub const kOptCompatible: OptIndex = 50;
pub const kOptCommentstring: OptIndex = 49;
pub const kOptComments: OptIndex = 48;
pub const kOptColumns: OptIndex = 47;
pub const kOptColorcolumn: OptIndex = 46;
pub const kOptCmdwinheight: OptIndex = 45;
pub const kOptCmdheight: OptIndex = 44;
pub const kOptClipboard: OptIndex = 43;
pub const kOptCinwords: OptIndex = 42;
pub const kOptCinscopedecls: OptIndex = 41;
pub const kOptCinoptions: OptIndex = 40;
pub const kOptCinkeys: OptIndex = 39;
pub const kOptCindent: OptIndex = 38;
pub const kOptChistory: OptIndex = 37;
pub const kOptCharconvert: OptIndex = 36;
pub const kOptChannel: OptIndex = 35;
pub const kOptCedit: OptIndex = 34;
pub const kOptCdpath: OptIndex = 33;
pub const kOptCdhome: OptIndex = 32;
pub const kOptCasemap: OptIndex = 31;
pub const kOptBusy: OptIndex = 30;
pub const kOptBuftype: OptIndex = 29;
pub const kOptBuflisted: OptIndex = 28;
pub const kOptBufhidden: OptIndex = 27;
pub const kOptBrowsedir: OptIndex = 26;
pub const kOptBreakindentopt: OptIndex = 25;
pub const kOptBreakindent: OptIndex = 24;
pub const kOptBreakat: OptIndex = 23;
pub const kOptBomb: OptIndex = 22;
pub const kOptBinary: OptIndex = 21;
pub const kOptBelloff: OptIndex = 20;
pub const kOptBackupskip: OptIndex = 19;
pub const kOptBackupext: OptIndex = 18;
pub const kOptBackupdir: OptIndex = 17;
pub const kOptBackupcopy: OptIndex = 16;
pub const kOptBackup: OptIndex = 15;
pub const kOptBackspace: OptIndex = 14;
pub const kOptBackground: OptIndex = 13;
pub const kOptAutowriteall: OptIndex = 12;
pub const kOptAutowrite: OptIndex = 11;
pub const kOptAutoread: OptIndex = 10;
pub const kOptAutoindent: OptIndex = 9;
pub const kOptAutocompletetimeout: OptIndex = 8;
pub const kOptAutocompletedelay: OptIndex = 7;
pub const kOptAutocomplete: OptIndex = 6;
pub const kOptAutochdir: OptIndex = 5;
pub const kOptArabicshape: OptIndex = 4;
pub const kOptArabic: OptIndex = 3;
pub const kOptAmbiwidth: OptIndex = 2;
pub const kOptAllowrevins: OptIndex = 1;
pub const kOptAleph: OptIndex = 0;
pub const kOptInvalid: OptIndex = -1;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
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
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const kOptSpoFlagNoplainbuffer: C2Rust_Unnamed_15 = 2;
pub const kOptSpoFlagCamel: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_16 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_16 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_16 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_16 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_16 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_16 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_16 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_16 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_16 = 79;
pub const SHM_OVER: C2Rust_Unnamed_16 = 111;
pub const SHM_TRUNCALL: C2Rust_Unnamed_16 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_16 = 116;
pub const SHM_WRITE: C2Rust_Unnamed_16 = 87;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_16 = 97;
pub const SHM_WRI: C2Rust_Unnamed_16 = 119;
pub const SHM_LINES: C2Rust_Unnamed_16 = 108;
pub const SHM_MOD: C2Rust_Unnamed_16 = 109;
pub const SHM_RO: C2Rust_Unnamed_16 = 114;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_21 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_21 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_21 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_21 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_21 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_21 = 20;
pub const UPD_VALID: C2Rust_Unnamed_21 = 10;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const MB_MAXCHAR: C2Rust_Unnamed_22 = 6;
pub const MB_MAXBYTES: C2Rust_Unnamed_22 = 21;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_23 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_23 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_23 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_23 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_23 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_23 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_23 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_23 = 1;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const kEqualFiles: file_comparison = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const DIP_DIRFILE: C2Rust_Unnamed_24 = 512;
pub const DIP_AFTER: C2Rust_Unnamed_24 = 128;
pub const DIP_NOAFTER: C2Rust_Unnamed_24 = 64;
pub const DIP_NORTP: C2Rust_Unnamed_24 = 32;
pub const DIP_OPT: C2Rust_Unnamed_24 = 16;
pub const DIP_START: C2Rust_Unnamed_24 = 8;
pub const DIP_ERR: C2Rust_Unnamed_24 = 4;
pub const DIP_DIR: C2Rust_Unnamed_24 = 2;
pub const DIP_ALL: C2Rust_Unnamed_24 = 1;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const SEARCH_COL: C2Rust_Unnamed_25 = 4096;
pub const SEARCH_PEEK: C2Rust_Unnamed_25 = 2048;
pub const SEARCH_KEEP: C2Rust_Unnamed_25 = 1024;
pub const SEARCH_MARK: C2Rust_Unnamed_25 = 512;
pub const SEARCH_START: C2Rust_Unnamed_25 = 256;
pub const SEARCH_NOOF: C2Rust_Unnamed_25 = 128;
pub const SEARCH_END: C2Rust_Unnamed_25 = 64;
pub const SEARCH_HIS: C2Rust_Unnamed_25 = 32;
pub const SEARCH_OPT: C2Rust_Unnamed_25 = 16;
pub const SEARCH_NFMSG: C2Rust_Unnamed_25 = 8;
pub const SEARCH_MSG: C2Rust_Unnamed_25 = 12;
pub const SEARCH_ECHO: C2Rust_Unnamed_25 = 2;
pub const SEARCH_REV: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const MAXWLEN: C2Rust_Unnamed_26 = 254;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const WF_CAPMASK: C2Rust_Unnamed_27 = 198;
pub const WF_KEEPCAP: C2Rust_Unnamed_27 = 128;
pub const WF_FIXCAP: C2Rust_Unnamed_27 = 64;
pub const WF_AFX: C2Rust_Unnamed_27 = 32;
pub const WF_BANNED: C2Rust_Unnamed_27 = 16;
pub const WF_RARE: C2Rust_Unnamed_27 = 8;
pub const WF_ALLCAP: C2Rust_Unnamed_27 = 4;
pub const WF_ONECAP: C2Rust_Unnamed_27 = 2;
pub const WF_REGION: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const WF_NOCOMPAFT: C2Rust_Unnamed_28 = 8192;
pub const WF_NOCOMPBEF: C2Rust_Unnamed_28 = 4096;
pub const WF_COMPROOT: C2Rust_Unnamed_28 = 2048;
pub const WF_NOSUGGEST: C2Rust_Unnamed_28 = 1024;
pub const WF_NEEDCOMP: C2Rust_Unnamed_28 = 512;
pub const WF_HAS_AFF: C2Rust_Unnamed_28 = 256;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const WF_PFX_COMPFORBID: C2Rust_Unnamed_29 = 268435456;
pub const WF_PFX_COMPPERMIT: C2Rust_Unnamed_29 = 134217728;
pub const WF_PFX_UP: C2Rust_Unnamed_29 = 67108864;
pub const WF_PFX_NC: C2Rust_Unnamed_29 = 33554432;
pub const WF_RAREPFX: C2Rust_Unnamed_29 = 16777216;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_int;
pub const SP_OTHERERROR: C2Rust_Unnamed_30 = -3;
pub const SP_FORMERROR: C2Rust_Unnamed_30 = -2;
pub const SP_TRUNCERROR: C2Rust_Unnamed_30 = -1;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const REGION_ALL: C2Rust_Unnamed_31 = 255;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const MAXWORDCOUNT: C2Rust_Unnamed_32 = 65535;
pub const SMT_RARE: smt_T = 2;
pub const SMT_BAD: smt_T = 1;
pub const SMT_ALL: smt_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct matchinf_T {
    pub mi_lp: *mut langp_T,
    pub mi_word: *mut ::core::ffi::c_char,
    pub mi_end: *mut ::core::ffi::c_char,
    pub mi_fend: *mut ::core::ffi::c_char,
    pub mi_cend: *mut ::core::ffi::c_char,
    pub mi_fword: [::core::ffi::c_char; 255],
    pub mi_fwordlen: ::core::ffi::c_int,
    pub mi_prefarridx: ::core::ffi::c_int,
    pub mi_prefcnt: ::core::ffi::c_int,
    pub mi_prefixlen: ::core::ffi::c_int,
    pub mi_cprefixlen: ::core::ffi::c_int,
    pub mi_compoff: ::core::ffi::c_int,
    pub mi_compflags: [uint8_t; 254],
    pub mi_complen: ::core::ffi::c_int,
    pub mi_compextra: ::core::ffi::c_int,
    pub mi_result: ::core::ffi::c_int,
    pub mi_capflags: ::core::ffi::c_int,
    pub mi_win: *mut win_T,
    pub mi_result2: ::core::ffi::c_int,
    pub mi_end2: *mut ::core::ffi::c_char,
}
pub const SP_RARE: C2Rust_Unnamed_33 = 0;
pub const SP_OK: C2Rust_Unnamed_33 = 1;
pub const SP_BANNED: C2Rust_Unnamed_33 = -1;
pub const SP_BAD: C2Rust_Unnamed_33 = 3;
pub const FIND_COMPOUND: C2Rust_Unnamed_34 = 3;
pub const SP_LOCAL: C2Rust_Unnamed_33 = 2;
pub const FIND_KEEPCOMPOUND: C2Rust_Unnamed_34 = 4;
pub const FIND_KEEPWORD: C2Rust_Unnamed_34 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syl_item_T {
    pub sy_chars: [::core::ffi::c_char; 30],
    pub sy_len: ::core::ffi::c_int,
}
pub const FIND_PREFIX: C2Rust_Unnamed_34 = 2;
pub const FIND_FOLDWORD: C2Rust_Unnamed_34 = 0;
pub const CHAR_OTHER: C2Rust_Unnamed_35 = 0;
pub const CHAR_UPPER: C2Rust_Unnamed_35 = 1;
pub const CHAR_DIGIT: C2Rust_Unnamed_35 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spelload_T {
    pub sl_lang: [::core::ffi::c_char; 255],
    pub sl_slang: *mut slang_T,
    pub sl_nobreak: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_33 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
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
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn clearpos(mut a: *mut pos_T) {
    (*a).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*a).col = 0 as ::core::ffi::c_int as colnr_T;
    (*a).coladd = 0 as ::core::ffi::c_int as colnr_T;
}
#[inline(always)]
unsafe extern "C" fn decor_redraw_col(
    mut wp: *mut win_T,
    mut col: ::core::ffi::c_int,
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut state: *mut DecorState,
    mut max_col_last: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if col <= (*state).col_last {
        return (*state).current;
    }
    return decor_redraw_col_impl(wp, col, win_col, hidden, state, max_col_last);
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
#[no_mangle]
pub static first_lang: GlobalCell<*mut slang_T> =
    GlobalCell::new(::core::ptr::null_mut::<slang_T>());
#[no_mangle]
pub static int_wordlist: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub const SY_MAXLEN: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
#[no_mangle]
pub static spelltab: GlobalCell<spelltab_T> = GlobalCell::new(spelltab_T {
    st_isw: [false; 256],
    st_isu: [false; 256],
    st_fold: [0; 256],
    st_upper: [0; 256],
});
#[no_mangle]
pub static did_set_spelltab: GlobalCell<bool> = GlobalCell::new(false);
#[no_mangle]
pub static e_format: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
    b"E759: Format error in spell file\0".as_ptr() as *const ::core::ffi::c_char
        as *mut ::core::ffi::c_char,
);
#[no_mangle]
pub static repl_from: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[no_mangle]
pub static repl_to: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
#[no_mangle]
pub unsafe extern "C" fn spell_check(
    mut wp: *mut win_T,
    mut ptr: *mut ::core::ffi::c_char,
    mut attrp: *mut hlf_T,
    mut capcol: *mut ::core::ffi::c_int,
    mut docount: bool,
) -> size_t {
    if *ptr as uint8_t as ::core::ffi::c_int <= ' ' as ::core::ffi::c_int {
        return 1 as size_t;
    }
    if (*(*wp).w_s).b_langp.ga_len <= 0 as ::core::ffi::c_int {
        return 1 as size_t;
    }
    let mut nrlen: size_t = 0 as size_t;
    let mut wrongcaplen: size_t = 0 as size_t;
    let mut count_word: bool = docount;
    let mut use_camel_case: bool = (*(*wp).w_s).b_p_spo_flags
        & kOptSpoFlagCamel as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint;
    let mut is_camel_case: bool = false_0 != 0;
    let mut mi: matchinf_T = matchinf_T {
        mi_lp: ::core::ptr::null_mut::<langp_T>(),
        mi_word: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        mi_end: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        mi_fend: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        mi_cend: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        mi_fword: [0; 255],
        mi_fwordlen: 0,
        mi_prefarridx: 0,
        mi_prefcnt: 0,
        mi_prefixlen: 0,
        mi_cprefixlen: 0,
        mi_compoff: 0,
        mi_compflags: [0; 254],
        mi_complen: 0,
        mi_compextra: 0,
        mi_result: 0,
        mi_capflags: 0,
        mi_win: ::core::ptr::null_mut::<win_T>(),
        mi_result2: 0,
        mi_end2: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    memset(
        &raw mut mi as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<matchinf_T>(),
    );
    if *ptr as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
        && *ptr as ::core::ffi::c_int <= '9' as ::core::ffi::c_int
    {
        if *ptr as ::core::ffi::c_int == '0' as ::core::ffi::c_int
            && (*ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'b' as ::core::ffi::c_int
                || *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'B' as ::core::ffi::c_int)
        {
            mi.mi_end =
                skipbin(ptr.offset(2 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char;
        } else if *ptr as ::core::ffi::c_int == '0' as ::core::ffi::c_int
            && (*ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'x' as ::core::ffi::c_int
                || *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'X' as ::core::ffi::c_int)
        {
            mi.mi_end = skiphex(ptr.offset(2 as ::core::ffi::c_int as isize));
        } else {
            mi.mi_end = skipdigits(ptr);
        }
        nrlen = mi.mi_end.offset_from(ptr) as size_t;
    }
    mi.mi_word = ptr;
    mi.mi_fend = ptr;
    if spell_iswordp(mi.mi_fend, wp) {
        if use_camel_case {
            mi.mi_fend = advance_camelcase_word(ptr, wp, &raw mut is_camel_case);
        } else {
            loop {
                mi.mi_fend = mi.mi_fend.offset(utfc_ptr2len(mi.mi_fend) as isize);
                if !(*mi.mi_fend as ::core::ffi::c_int != NUL
                    && spell_iswordp(mi.mi_fend, wp) as ::core::ffi::c_int != 0)
                {
                    break;
                }
            }
        }
        if !capcol.is_null()
            && *capcol == 0 as ::core::ffi::c_int
            && !(*(*wp).w_s).b_cap_prog.is_null()
        {
            let mut c: ::core::ffi::c_int = utf_ptr2char(ptr);
            if if c >= 128 as ::core::ffi::c_int {
                mb_isupper(c) as ::core::ffi::c_int
            } else {
                (*spelltab.ptr()).st_isu[c as usize] as ::core::ffi::c_int
            } == 0
            {
                wrongcaplen = mi.mi_fend.offset_from(ptr) as size_t;
            }
        }
    }
    if !capcol.is_null() {
        *capcol = -1 as ::core::ffi::c_int;
    }
    mi.mi_end = mi.mi_fend;
    mi.mi_capflags = 0 as ::core::ffi::c_int;
    mi.mi_cend = ::core::ptr::null_mut::<::core::ffi::c_char>();
    mi.mi_win = wp;
    if *mi.mi_fend as ::core::ffi::c_int != NUL {
        mi.mi_fend = mi.mi_fend.offset(utfc_ptr2len(mi.mi_fend) as isize);
    }
    spell_casefold(
        wp,
        ptr,
        mi.mi_fend.offset_from(ptr) as ::core::ffi::c_int,
        &raw mut mi.mi_fword as *mut ::core::ffi::c_char,
        MAXWLEN as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
    );
    mi.mi_fwordlen = strlen(&raw mut mi.mi_fword as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    if is_camel_case as ::core::ffi::c_int != 0 && mi.mi_fwordlen > 0 as ::core::ffi::c_int {
        mi.mi_fword[(mi.mi_fwordlen - 1 as ::core::ffi::c_int) as usize] =
            ' ' as ::core::ffi::c_char;
    }
    mi.mi_result = SP_BAD as ::core::ffi::c_int;
    mi.mi_result2 = SP_BAD as ::core::ffi::c_int;
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*wp).w_s).b_langp.ga_len {
        mi.mi_lp = ((*(*wp).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        if !(*(*mi.mi_lp).lp_slang).sl_fidxs.is_null() {
            find_word(&raw mut mi, FIND_FOLDWORD as ::core::ffi::c_int);
            find_word(&raw mut mi, FIND_KEEPWORD as ::core::ffi::c_int);
            find_prefix(&raw mut mi, FIND_FOLDWORD as ::core::ffi::c_int);
            if (*(*mi.mi_lp).lp_slang).sl_nobreak as ::core::ffi::c_int != 0
                && mi.mi_result == SP_BAD as ::core::ffi::c_int
                && mi.mi_result2 != SP_BAD as ::core::ffi::c_int
            {
                mi.mi_result = mi.mi_result2;
                mi.mi_end = mi.mi_end2;
            }
            if count_word as ::core::ffi::c_int != 0 && mi.mi_result == SP_OK as ::core::ffi::c_int
            {
                count_common_word(
                    (*mi.mi_lp).lp_slang,
                    ptr,
                    mi.mi_end.offset_from(ptr) as ::core::ffi::c_int,
                    1 as uint8_t,
                );
                count_word = false_0 != 0;
            }
        }
        lpi += 1;
    }
    if mi.mi_result != SP_OK as ::core::ffi::c_int {
        if nrlen > 0 as size_t {
            if mi.mi_result == SP_BAD as ::core::ffi::c_int
                || mi.mi_result == SP_BANNED as ::core::ffi::c_int
            {
                return nrlen;
            }
        } else if !spell_iswordp_nmw(ptr, wp) {
            if !capcol.is_null() && !(*(*wp).w_s).b_cap_prog.is_null() {
                let mut regmatch: regmatch_T = regmatch_T {
                    regprog: ::core::ptr::null_mut::<regprog_T>(),
                    startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                    endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                    rm_matchcol: 0,
                    rm_ic: false,
                };
                regmatch.regprog = (*(*wp).w_s).b_cap_prog;
                regmatch.rm_ic = false_0 != 0;
                let mut r: bool = vim_regexec(&raw mut regmatch, ptr, 0 as colnr_T);
                (*(*wp).w_s).b_cap_prog = regmatch.regprog;
                if r {
                    *capcol = regmatch.endp[0 as ::core::ffi::c_int as usize].offset_from(ptr)
                        as ::core::ffi::c_int;
                }
            }
            return utfc_ptr2len(ptr) as size_t;
        } else if mi.mi_end == ptr {
            mi.mi_end = mi.mi_end.offset(utfc_ptr2len(mi.mi_end) as isize);
        } else if mi.mi_result == SP_BAD as ::core::ffi::c_int
            && (*(*((*(*wp).w_s).b_langp.ga_data as *mut langp_T)
                .offset(0 as ::core::ffi::c_int as isize))
            .lp_slang)
                .sl_nobreak as ::core::ffi::c_int
                != 0
        {
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut save_result: ::core::ffi::c_int = mi.mi_result;
            mi.mi_lp = ((*(*wp).w_s).b_langp.ga_data as *mut langp_T)
                .offset(0 as ::core::ffi::c_int as isize);
            if !(*(*mi.mi_lp).lp_slang).sl_fidxs.is_null() {
                p = mi.mi_word;
                let mut fp: *mut ::core::ffi::c_char =
                    &raw mut mi.mi_fword as *mut ::core::ffi::c_char;
                loop {
                    p = p.offset(utfc_ptr2len(p) as isize);
                    fp = fp.offset(utfc_ptr2len(fp) as isize);
                    if p >= mi.mi_end {
                        break;
                    }
                    mi.mi_compoff = fp.offset_from(&raw mut mi.mi_fword as *mut ::core::ffi::c_char)
                        as ::core::ffi::c_int;
                    find_word(&raw mut mi, FIND_COMPOUND as ::core::ffi::c_int);
                    if mi.mi_result == SP_BAD as ::core::ffi::c_int {
                        continue;
                    }
                    mi.mi_end = p;
                    break;
                }
                mi.mi_result = save_result;
            }
        }
        if mi.mi_result == SP_BAD as ::core::ffi::c_int
            || mi.mi_result == SP_BANNED as ::core::ffi::c_int
        {
            *attrp = HLF_SPB;
        } else if mi.mi_result == SP_RARE as ::core::ffi::c_int {
            *attrp = HLF_SPR;
        } else {
            *attrp = HLF_SPL;
        }
    }
    if wrongcaplen > 0 as size_t
        && (mi.mi_result == SP_OK as ::core::ffi::c_int
            || mi.mi_result == SP_RARE as ::core::ffi::c_int)
    {
        *attrp = HLF_SPC;
        return wrongcaplen;
    }
    return mi.mi_end.offset_from(ptr) as size_t;
}
unsafe extern "C" fn get_char_type(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if ascii_isdigit(c) {
        return CHAR_DIGIT as ::core::ffi::c_int;
    }
    if if c >= 128 as ::core::ffi::c_int {
        mb_isupper(c) as ::core::ffi::c_int
    } else {
        (*spelltab.ptr()).st_isu[c as usize] as ::core::ffi::c_int
    } != 0
    {
        return CHAR_UPPER as ::core::ffi::c_int;
    }
    return CHAR_OTHER as ::core::ffi::c_int;
}
unsafe extern "C" fn advance_camelcase_word(
    mut str: *mut ::core::ffi::c_char,
    mut wp: *mut win_T,
    mut is_camel_case: *mut bool,
) -> *mut ::core::ffi::c_char {
    let mut end: *mut ::core::ffi::c_char = str;
    *is_camel_case = false_0 != 0;
    if *str as ::core::ffi::c_int == NUL {
        return str;
    }
    let mut c: ::core::ffi::c_int = utf_ptr2char(end);
    end = end.offset(utfc_ptr2len(end) as isize);
    let mut last_last_type: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut last_type: ::core::ffi::c_int = get_char_type(c);
    while *end as ::core::ffi::c_int != NUL && spell_iswordp(end, wp) as ::core::ffi::c_int != 0 {
        c = utf_ptr2char(end);
        let mut this_type: ::core::ffi::c_int = get_char_type(c);
        if last_last_type == CHAR_UPPER as ::core::ffi::c_int
            && last_type == CHAR_UPPER as ::core::ffi::c_int
            && this_type == CHAR_OTHER as ::core::ffi::c_int
        {
            *is_camel_case = true_0 != 0;
            end = end.offset(
                -((utf_head_off(str, end.offset(-(1 as ::core::ffi::c_int as isize)))
                    + 1 as ::core::ffi::c_int) as isize),
            );
            break;
        } else if this_type == CHAR_UPPER as ::core::ffi::c_int
            && last_type == CHAR_OTHER as ::core::ffi::c_int
            || this_type != last_type
                && (this_type == CHAR_DIGIT as ::core::ffi::c_int
                    || last_type == CHAR_DIGIT as ::core::ffi::c_int)
        {
            *is_camel_case = true_0 != 0;
            break;
        } else {
            last_last_type = last_type;
            last_type = this_type;
            end = end.offset(utfc_ptr2len(end) as isize);
        }
    }
    return end;
}
unsafe extern "C" fn find_word(mut mip: *mut matchinf_T, mut mode: ::core::ffi::c_int) {
    let mut wlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut flen: ::core::ffi::c_int = 0;
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slang: *mut slang_T = (*(*mip).mi_lp).lp_slang;
    let mut byts: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut idxs: *mut idx_T = ::core::ptr::null_mut::<idx_T>();
    if mode == FIND_KEEPWORD as ::core::ffi::c_int
        || mode == FIND_KEEPCOMPOUND as ::core::ffi::c_int
    {
        ptr = (*mip).mi_word;
        flen = 9999 as ::core::ffi::c_int;
        byts = (*slang).sl_kbyts;
        idxs = (*slang).sl_kidxs;
        if mode == FIND_KEEPCOMPOUND as ::core::ffi::c_int {
            wlen += (*mip).mi_compoff;
        }
    } else {
        ptr = &raw mut (*mip).mi_fword as *mut ::core::ffi::c_char;
        flen = (*mip).mi_fwordlen;
        byts = (*slang).sl_fbyts;
        idxs = (*slang).sl_fidxs;
        if mode == FIND_PREFIX as ::core::ffi::c_int {
            wlen = (*mip).mi_prefixlen;
            flen -= (*mip).mi_prefixlen;
        } else if mode == FIND_COMPOUND as ::core::ffi::c_int {
            wlen = (*mip).mi_compoff;
            flen -= (*mip).mi_compoff;
        }
    }
    if byts.is_null() {
        return;
    }
    let mut arridx: idx_T = 0 as idx_T;
    let mut endlen: [::core::ffi::c_int; 254] = [0; 254];
    let mut endidx: [idx_T; 254] = [0; 254];
    let mut endidxcnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        if flen <= 0 as ::core::ffi::c_int && *(*mip).mi_fend as ::core::ffi::c_int != NUL {
            flen = fold_more(mip);
        }
        let c2rust_fresh0 = arridx;
        arridx = arridx + 1;
        let mut len: ::core::ffi::c_int =
            *byts.offset(c2rust_fresh0 as isize) as ::core::ffi::c_int;
        if *byts.offset(arridx as isize) as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            if endidxcnt == MAXWLEN as ::core::ffi::c_int {
                emsg(gettext(e_format.get()));
                return;
            }
            endlen[endidxcnt as usize] = wlen;
            let c2rust_fresh1 = arridx;
            arridx = arridx + 1;
            let c2rust_fresh2 = endidxcnt;
            endidxcnt = endidxcnt + 1;
            endidx[c2rust_fresh2 as usize] = c2rust_fresh1;
            len -= 1;
            while len > 0 as ::core::ffi::c_int
                && *byts.offset(arridx as isize) as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                arridx += 1;
                len -= 1;
            }
            if len == 0 as ::core::ffi::c_int {
                break;
            }
        }
        if *ptr.offset(wlen as isize) as ::core::ffi::c_int == NUL {
            break;
        }
        let mut c: ::core::ffi::c_int = *ptr.offset(wlen as isize) as uint8_t as ::core::ffi::c_int;
        if c == TAB {
            c = ' ' as ::core::ffi::c_int;
        }
        let mut lo: idx_T = arridx;
        let mut hi: idx_T = arridx + len as idx_T - 1 as idx_T;
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
        arridx = *idxs.offset(lo as isize);
        wlen += 1;
        flen -= 1;
        if c == ' ' as ::core::ffi::c_int {
            loop {
                if flen <= 0 as ::core::ffi::c_int && *(*mip).mi_fend as ::core::ffi::c_int != NUL {
                    flen = fold_more(mip);
                }
                if *ptr.offset(wlen as isize) as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
                    && *ptr.offset(wlen as isize) as ::core::ffi::c_int != TAB
                {
                    break;
                }
                wlen += 1;
                flen -= 1;
            }
        }
    }
    while endidxcnt > 0 as ::core::ffi::c_int {
        endidxcnt -= 1;
        arridx = endidx[endidxcnt as usize];
        wlen = endlen[endidxcnt as usize];
        if utf_head_off(ptr, ptr.offset(wlen as isize)) <= 0 as ::core::ffi::c_int {
            let mut word_ends: bool = false;
            if spell_iswordp(ptr.offset(wlen as isize), (*mip).mi_win) {
                if (*slang).sl_compprog.is_null() && !(*slang).sl_nobreak {
                    continue;
                } else {
                    word_ends = false_0 != 0;
                }
            } else {
                word_ends = true_0 != 0;
            }
            let mut prefix_found: bool = false_0 != 0;
            if mode != FIND_KEEPWORD as ::core::ffi::c_int {
                let mut p: *mut ::core::ffi::c_char = (*mip).mi_word;
                if strncmp(ptr, p, wlen as size_t) != 0 as ::core::ffi::c_int {
                    let mut s: *mut ::core::ffi::c_char = ptr;
                    while s < ptr.offset(wlen as isize) {
                        p = p.offset(utfc_ptr2len(p) as isize);
                        s = s.offset(utfc_ptr2len(s) as isize);
                    }
                    wlen = p.offset_from((*mip).mi_word) as ::core::ffi::c_int;
                }
            }
            let mut len_0: ::core::ffi::c_int = *byts
                .offset((arridx as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int;
            's_1014: while len_0 > 0 as ::core::ffi::c_int
                && *byts.offset(arridx as isize) as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                let mut flags: uint32_t = *idxs.offset(arridx as isize) as uint32_t;
                's_404: {
                    if mode == FIND_FOLDWORD as ::core::ffi::c_int {
                        if (*mip).mi_cend != (*mip).mi_word.offset(wlen as isize) {
                            (*mip).mi_cend = (*mip).mi_word.offset(wlen as isize);
                            (*mip).mi_capflags = captype((*mip).mi_word, (*mip).mi_cend);
                        }
                        if (*mip).mi_capflags == WF_KEEPCAP as ::core::ffi::c_int
                            || !spell_valid_case((*mip).mi_capflags, flags as ::core::ffi::c_int)
                        {
                            break 's_404;
                        }
                    } else if mode == FIND_PREFIX as ::core::ffi::c_int && !prefix_found {
                        let mut c_0: ::core::ffi::c_int = valid_word_prefix(
                            (*mip).mi_prefcnt,
                            (*mip).mi_prefarridx,
                            flags as ::core::ffi::c_int,
                            (*mip).mi_word.offset((*mip).mi_cprefixlen as isize),
                            slang,
                            false_0 != 0,
                        );
                        if c_0 == 0 as ::core::ffi::c_int {
                            break 's_404;
                        } else {
                            if c_0 & WF_RAREPFX as ::core::ffi::c_int != 0 {
                                flags |= WF_RARE as ::core::ffi::c_int as uint32_t;
                            }
                            prefix_found = true_0 != 0;
                        }
                    }
                    if (*slang).sl_nobreak {
                        if (mode == FIND_COMPOUND as ::core::ffi::c_int
                            || mode == FIND_KEEPCOMPOUND as ::core::ffi::c_int)
                            && flags & WF_BANNED as ::core::ffi::c_int as uint32_t == 0 as uint32_t
                        {
                            (*mip).mi_result = SP_OK as ::core::ffi::c_int;
                            break 's_1014;
                        }
                    } else if mode == FIND_COMPOUND as ::core::ffi::c_int
                        || mode == FIND_KEEPCOMPOUND as ::core::ffi::c_int
                        || !word_ends
                    {
                        if flags as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int
                            == 0 as ::core::ffi::c_uint
                            || wlen - (*mip).mi_compoff < (*slang).sl_compminlen
                        {
                            break 's_404;
                        } else if (*slang).sl_compminlen > 0 as ::core::ffi::c_int
                            && mb_charlen_len(
                                (*mip).mi_word.offset((*mip).mi_compoff as isize),
                                wlen - (*mip).mi_compoff,
                            ) < (*slang).sl_compminlen
                        {
                            break 's_404;
                        } else if !word_ends
                            && (*mip).mi_complen + (*mip).mi_compextra + 2 as ::core::ffi::c_int
                                > (*slang).sl_compmax
                            && (*slang).sl_compsylmax == MAXWLEN as ::core::ffi::c_int
                        {
                            break 's_404;
                        } else if (*mip).mi_complen > 0 as ::core::ffi::c_int
                            && flags & WF_NOCOMPBEF as ::core::ffi::c_int as uint32_t != 0
                        {
                            break 's_404;
                        } else if !word_ends
                            && flags & WF_NOCOMPAFT as ::core::ffi::c_int as uint32_t != 0
                        {
                            break 's_404;
                        } else if !byte_in_str(
                            if (*mip).mi_complen == 0 as ::core::ffi::c_int {
                                (*slang).sl_compstartflags
                            } else {
                                (*slang).sl_compallflags
                            },
                            (flags as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int)
                                as ::core::ffi::c_int,
                        ) {
                            break 's_404;
                        } else if match_checkcompoundpattern(
                            ptr,
                            wlen,
                            &raw mut (*slang).sl_comppat,
                        ) {
                            break 's_404;
                        } else {
                            if mode == FIND_COMPOUND as ::core::ffi::c_int {
                                let mut capflags: ::core::ffi::c_int = 0;
                                let mut p_0: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                if strncmp(ptr, (*mip).mi_word, (*mip).mi_compoff as size_t)
                                    != 0 as ::core::ffi::c_int
                                {
                                    p_0 = (*mip).mi_word;
                                    let mut s_0: *mut ::core::ffi::c_char = ptr;
                                    while s_0 < ptr.offset((*mip).mi_compoff as isize) {
                                        p_0 = p_0.offset(utfc_ptr2len(p_0) as isize);
                                        s_0 = s_0.offset(utfc_ptr2len(s_0) as isize);
                                    }
                                } else {
                                    p_0 = (*mip).mi_word.offset((*mip).mi_compoff as isize);
                                }
                                capflags = captype(p_0, (*mip).mi_word.offset(wlen as isize));
                                if capflags == WF_KEEPCAP as ::core::ffi::c_int
                                    || capflags == WF_ALLCAP as ::core::ffi::c_int
                                        && flags & WF_FIXCAP as ::core::ffi::c_int as uint32_t
                                            != 0 as uint32_t
                                {
                                    break 's_404;
                                } else if capflags != WF_ALLCAP as ::core::ffi::c_int {
                                    p_0 = p_0.offset(
                                        -((utf_head_off(
                                            (*mip).mi_word,
                                            p_0.offset(-(1 as ::core::ffi::c_int as isize)),
                                        ) + 1 as ::core::ffi::c_int)
                                            as isize),
                                    );
                                    if if spell_iswordp_nmw(p_0, (*mip).mi_win)
                                        as ::core::ffi::c_int
                                        != 0
                                    {
                                        (capflags == WF_ONECAP as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                    } else {
                                        (flags & WF_ONECAP as ::core::ffi::c_int as uint32_t
                                            != 0 as uint32_t
                                            && capflags != WF_ONECAP as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                    } != 0
                                    {
                                        break 's_404;
                                    }
                                }
                            }
                            (*mip).mi_compflags[(*mip).mi_complen as usize] =
                                (flags as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int)
                                    as uint8_t;
                            (*mip).mi_compflags
                                [((*mip).mi_complen + 1 as ::core::ffi::c_int) as usize] =
                                NUL as uint8_t;
                            if word_ends {
                                let mut fword: [::core::ffi::c_char; 254] = [
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
                                ];
                                if (*slang).sl_compsylmax < MAXWLEN as ::core::ffi::c_int {
                                    if ptr == (*mip).mi_word {
                                        spell_casefold(
                                            (*mip).mi_win,
                                            ptr,
                                            wlen,
                                            &raw mut fword as *mut ::core::ffi::c_char,
                                            MAXWLEN as ::core::ffi::c_int,
                                        );
                                    } else {
                                        xmemcpyz(
                                            &raw mut fword as *mut ::core::ffi::c_char
                                                as *mut ::core::ffi::c_void,
                                            ptr as *const ::core::ffi::c_void,
                                            endlen[endidxcnt as usize] as size_t,
                                        );
                                    }
                                }
                                if !can_compound(
                                    slang,
                                    &raw mut fword as *mut ::core::ffi::c_char,
                                    &raw mut (*mip).mi_compflags as *mut uint8_t,
                                ) {
                                    break 's_404;
                                }
                            } else if !(*slang).sl_comprules.is_null()
                                && !match_compoundrule(
                                    slang,
                                    &raw mut (*mip).mi_compflags as *mut uint8_t,
                                )
                            {
                                break 's_404;
                            }
                        }
                    } else if flags & WF_NEEDCOMP as ::core::ffi::c_int as uint32_t != 0 {
                        break 's_404;
                    }
                    let mut nobreak_result: ::core::ffi::c_int = SP_OK as ::core::ffi::c_int;
                    if !word_ends {
                        let mut save_result: ::core::ffi::c_int = (*mip).mi_result;
                        let mut save_end: *mut ::core::ffi::c_char = (*mip).mi_end;
                        let mut save_lp: *mut langp_T = (*mip).mi_lp;
                        if (*slang).sl_nobreak {
                            (*mip).mi_result = SP_BAD as ::core::ffi::c_int;
                        }
                        (*mip).mi_compoff = endlen[endidxcnt as usize];
                        if mode == FIND_KEEPWORD as ::core::ffi::c_int {
                            let mut p_1: *mut ::core::ffi::c_char =
                                &raw mut (*mip).mi_fword as *mut ::core::ffi::c_char;
                            if strncmp(ptr, p_1, wlen as size_t) != 0 as ::core::ffi::c_int {
                                let mut s_1: *mut ::core::ffi::c_char = ptr;
                                while s_1 < ptr.offset(wlen as isize) {
                                    p_1 = p_1.offset(utfc_ptr2len(p_1) as isize);
                                    s_1 = s_1.offset(utfc_ptr2len(s_1) as isize);
                                }
                                (*mip).mi_compoff = p_1.offset_from(
                                    &raw mut (*mip).mi_fword as *mut ::core::ffi::c_char,
                                )
                                    as ::core::ffi::c_int;
                            }
                        }
                        (*mip).mi_complen += 1;
                        if flags & WF_COMPROOT as ::core::ffi::c_int as uint32_t != 0 {
                            (*mip).mi_compextra += 1;
                        }
                        let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        's_821: while lpi < (*(*(*mip).mi_win).w_s).b_langp.ga_len {
                            's_772: {
                                if (*slang).sl_nobreak {
                                    (*mip).mi_lp = ((*(*(*mip).mi_win).w_s).b_langp.ga_data
                                        as *mut langp_T)
                                        .offset(lpi as isize);
                                    if (*(*(*mip).mi_lp).lp_slang).sl_fidxs.is_null()
                                        || !(*(*(*mip).mi_lp).lp_slang).sl_nobreak
                                    {
                                        break 's_772;
                                    }
                                }
                                find_word(mip, FIND_COMPOUND as ::core::ffi::c_int);
                                if !(*slang).sl_nobreak
                                    || (*mip).mi_result == SP_BAD as ::core::ffi::c_int
                                {
                                    (*mip).mi_compoff = wlen;
                                    find_word(mip, FIND_KEEPCOMPOUND as ::core::ffi::c_int);
                                }
                                if !(*slang).sl_nobreak {
                                    break 's_821;
                                }
                            }
                            lpi += 1;
                        }
                        (*mip).mi_complen -= 1;
                        if flags & WF_COMPROOT as ::core::ffi::c_int as uint32_t != 0 {
                            (*mip).mi_compextra -= 1;
                        }
                        (*mip).mi_lp = save_lp;
                        if (*slang).sl_nobreak {
                            nobreak_result = (*mip).mi_result;
                            (*mip).mi_result = save_result;
                            (*mip).mi_end = save_end;
                        } else if (*mip).mi_result == SP_OK as ::core::ffi::c_int {
                            break 's_1014;
                        } else {
                            break 's_404;
                        }
                    }
                    let mut res: ::core::ffi::c_int = SP_BAD as ::core::ffi::c_int;
                    if flags & WF_BANNED as ::core::ffi::c_int as uint32_t != 0 {
                        res = SP_BANNED as ::core::ffi::c_int;
                    } else if flags & WF_REGION as ::core::ffi::c_int as uint32_t != 0 {
                        if (*(*mip).mi_lp).lp_region as uint32_t & flags >> 16 as ::core::ffi::c_int
                            != 0 as uint32_t
                        {
                            res = SP_OK as ::core::ffi::c_int;
                        } else {
                            res = SP_LOCAL as ::core::ffi::c_int;
                        }
                    } else if flags & WF_RARE as ::core::ffi::c_int as uint32_t != 0 {
                        res = SP_RARE as ::core::ffi::c_int;
                    } else {
                        res = SP_OK as ::core::ffi::c_int;
                    }
                    if nobreak_result == SP_BAD as ::core::ffi::c_int {
                        if (*mip).mi_result2 > res {
                            (*mip).mi_result2 = res;
                            (*mip).mi_end2 = (*mip).mi_word.offset(wlen as isize);
                        } else if (*mip).mi_result2 == res
                            && (*mip).mi_end2 < (*mip).mi_word.offset(wlen as isize)
                        {
                            (*mip).mi_end2 = (*mip).mi_word.offset(wlen as isize);
                        }
                    } else if (*mip).mi_result > res {
                        (*mip).mi_result = res;
                        (*mip).mi_end = (*mip).mi_word.offset(wlen as isize);
                    } else if (*mip).mi_result == res
                        && (*mip).mi_end < (*mip).mi_word.offset(wlen as isize)
                    {
                        (*mip).mi_end = (*mip).mi_word.offset(wlen as isize);
                    }
                    if (*mip).mi_result == SP_OK as ::core::ffi::c_int {
                        break 's_1014;
                    }
                }
                len_0 -= 1;
                arridx += 1;
            }
            if (*mip).mi_result == SP_OK as ::core::ffi::c_int {
                break;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn match_checkcompoundpattern(
    mut ptr: *mut ::core::ffi::c_char,
    mut wlen: ::core::ffi::c_int,
    mut gap: *mut garray_T,
) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i + 1 as ::core::ffi::c_int) < (*gap).ga_len {
        let mut p: *mut ::core::ffi::c_char = *((*gap).ga_data as *mut *mut ::core::ffi::c_char)
            .offset((i + 1 as ::core::ffi::c_int) as isize);
        if strncmp(ptr.offset(wlen as isize), p, strlen(p)) == 0 as ::core::ffi::c_int {
            p = *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
            let mut len: ::core::ffi::c_int = strlen(p) as ::core::ffi::c_int;
            if len <= wlen
                && strncmp(
                    ptr.offset(wlen as isize).offset(-(len as isize)),
                    p,
                    len as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                return true_0 != 0;
            }
        }
        i += 2 as ::core::ffi::c_int;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn can_compound(
    mut slang: *mut slang_T,
    mut word: *const ::core::ffi::c_char,
    mut flags: *const uint8_t,
) -> bool {
    let mut uflags: [::core::ffi::c_char; 508] = [
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
    ];
    if (*slang).sl_compprog.is_null() {
        return false_0 != 0;
    }
    let mut p: *mut ::core::ffi::c_char = &raw mut uflags as *mut ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while *flags.offset(i as isize) as ::core::ffi::c_int != NUL {
        p = p.offset(utf_char2bytes(*flags.offset(i as isize) as ::core::ffi::c_int, p) as isize);
        i += 1;
    }
    *p = NUL as ::core::ffi::c_char;
    p = &raw mut uflags as *mut ::core::ffi::c_char;
    if !vim_regexec_prog(&raw mut (*slang).sl_compprog, false_0 != 0, p, 0 as colnr_T) {
        return false_0 != 0;
    }
    if (*slang).sl_compsylmax < MAXWLEN as ::core::ffi::c_int
        && count_syllables(slang, word) > (*slang).sl_compsylmax
    {
        return (strlen(flags as *mut ::core::ffi::c_char) as ::core::ffi::c_int)
            < (*slang).sl_compmax;
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn match_compoundrule(
    mut slang: *mut slang_T,
    mut compflags: *const uint8_t,
) -> bool {
    let mut p: *mut ::core::ffi::c_char = (*slang).sl_comprules as *mut ::core::ffi::c_char;
    while *p as ::core::ffi::c_int != NUL {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            let mut c: ::core::ffi::c_int = *compflags.offset(i as isize) as ::core::ffi::c_int;
            if c == NUL {
                return true_0 != 0;
            }
            if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == NUL
            {
                break;
            } else {
                if *p as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
                    let mut match_0: bool = false_0 != 0;
                    p = p.offset(1);
                    while *p as ::core::ffi::c_int != ']' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int != NUL
                    {
                        let c2rust_fresh3 = p;
                        p = p.offset(1);
                        if *c2rust_fresh3 as uint8_t as ::core::ffi::c_int == c {
                            match_0 = true_0 != 0;
                        }
                    }
                    if !match_0 {
                        break;
                    }
                } else if *p as uint8_t as ::core::ffi::c_int != c {
                    break;
                }
                p = p.offset(1);
                i += 1;
            }
        }
        p = vim_strchr(p, '/' as ::core::ffi::c_int);
        if p.is_null() {
            break;
        }
        p = p.offset(1);
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn valid_word_prefix(
    mut totprefcnt: ::core::ffi::c_int,
    mut arridx: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut word: *mut ::core::ffi::c_char,
    mut slang: *mut slang_T,
    mut cond_req: bool,
) -> ::core::ffi::c_int {
    let mut prefid: ::core::ffi::c_int =
        (flags as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int) as ::core::ffi::c_int;
    let mut prefcnt: ::core::ffi::c_int = totprefcnt - 1 as ::core::ffi::c_int;
    while prefcnt >= 0 as ::core::ffi::c_int {
        let mut pidx: ::core::ffi::c_int =
            *(*slang).sl_pidxs.offset((arridx + prefcnt) as isize) as ::core::ffi::c_int;
        's_6: {
            if prefid == pidx & 0xff as ::core::ffi::c_int {
                if !(flags & WF_HAS_AFF as ::core::ffi::c_int != 0
                    && pidx & WF_PFX_NC as ::core::ffi::c_int != 0)
                {
                    let mut rp: *mut *mut regprog_T = (*slang).sl_prefprog.offset(
                        (pidx as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                            & 0xffff as ::core::ffi::c_uint) as isize,
                    );
                    if !(*rp).is_null() {
                        if !vim_regexec_prog(rp, false_0 != 0, word, 0 as colnr_T) {
                            break 's_6;
                        }
                    } else if cond_req {
                        break 's_6;
                    }
                    return pidx;
                }
            }
        }
        prefcnt -= 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn find_prefix(mut mip: *mut matchinf_T, mut mode: ::core::ffi::c_int) {
    let mut arridx: idx_T = 0 as idx_T;
    let mut wlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut slang: *mut slang_T = (*(*mip).mi_lp).lp_slang;
    let mut byts: *mut uint8_t = (*slang).sl_pbyts;
    if byts.is_null() {
        return;
    }
    let mut ptr: *mut ::core::ffi::c_char = &raw mut (*mip).mi_fword as *mut ::core::ffi::c_char;
    let mut flen: ::core::ffi::c_int = (*mip).mi_fwordlen;
    if mode == FIND_COMPOUND as ::core::ffi::c_int {
        ptr = ptr.offset((*mip).mi_compoff as isize);
        flen -= (*mip).mi_compoff;
    }
    let mut idxs: *mut idx_T = (*slang).sl_pidxs;
    loop {
        if flen == 0 as ::core::ffi::c_int && *(*mip).mi_fend as ::core::ffi::c_int != NUL {
            flen = fold_more(mip);
        }
        let c2rust_fresh4 = arridx;
        arridx = arridx + 1;
        let mut len: ::core::ffi::c_int =
            *byts.offset(c2rust_fresh4 as isize) as ::core::ffi::c_int;
        if *byts.offset(arridx as isize) as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*mip).mi_prefarridx = arridx as ::core::ffi::c_int;
            (*mip).mi_prefcnt = len;
            while len > 0 as ::core::ffi::c_int
                && *byts.offset(arridx as isize) as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                arridx += 1;
                len -= 1;
            }
            (*mip).mi_prefcnt -= len;
            (*mip).mi_prefixlen = wlen;
            if mode == FIND_COMPOUND as ::core::ffi::c_int {
                (*mip).mi_prefixlen += (*mip).mi_compoff;
            }
            (*mip).mi_cprefixlen = nofold_len(
                &raw mut (*mip).mi_fword as *mut ::core::ffi::c_char,
                (*mip).mi_prefixlen,
                (*mip).mi_word,
            );
            find_word(mip, FIND_PREFIX as ::core::ffi::c_int);
            if len == 0 as ::core::ffi::c_int {
                break;
            }
        }
        if *ptr.offset(wlen as isize) as ::core::ffi::c_int == NUL {
            break;
        }
        let mut c: ::core::ffi::c_int = *ptr.offset(wlen as isize) as uint8_t as ::core::ffi::c_int;
        let mut lo: idx_T = arridx;
        let mut hi: idx_T = arridx + len as idx_T - 1 as idx_T;
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
        arridx = *idxs.offset(lo as isize);
        wlen += 1;
        flen -= 1;
    }
}
unsafe extern "C" fn fold_more(mut mip: *mut matchinf_T) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = (*mip).mi_fend;
    loop {
        (*mip).mi_fend = (*mip).mi_fend.offset(utfc_ptr2len((*mip).mi_fend) as isize);
        if !(*(*mip).mi_fend as ::core::ffi::c_int != NUL
            && spell_iswordp((*mip).mi_fend, (*mip).mi_win) as ::core::ffi::c_int != 0)
        {
            break;
        }
    }
    if *(*mip).mi_fend as ::core::ffi::c_int != NUL {
        (*mip).mi_fend = (*mip).mi_fend.offset(utfc_ptr2len((*mip).mi_fend) as isize);
    }
    spell_casefold(
        (*mip).mi_win,
        p,
        (*mip).mi_fend.offset_from(p) as ::core::ffi::c_int,
        (&raw mut (*mip).mi_fword as *mut ::core::ffi::c_char).offset((*mip).mi_fwordlen as isize),
        MAXWLEN as ::core::ffi::c_int - (*mip).mi_fwordlen,
    );
    let mut flen: ::core::ffi::c_int = strlen(
        (&raw mut (*mip).mi_fword as *mut ::core::ffi::c_char).offset((*mip).mi_fwordlen as isize),
    ) as ::core::ffi::c_int;
    (*mip).mi_fwordlen += flen;
    return flen;
}
#[no_mangle]
pub unsafe extern "C" fn spell_valid_case(
    mut wordflags: ::core::ffi::c_int,
    mut treeflags: ::core::ffi::c_int,
) -> bool {
    return wordflags == WF_ALLCAP as ::core::ffi::c_int
        && treeflags & WF_FIXCAP as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || treeflags & (WF_ALLCAP as ::core::ffi::c_int | WF_KEEPCAP as ::core::ffi::c_int)
            == 0 as ::core::ffi::c_int
            && (treeflags & WF_ONECAP as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                || wordflags & WF_ONECAP as ::core::ffi::c_int != 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn spell_check_window(mut wp: *mut win_T) -> bool {
    return (*wp).w_onebuf_opt.wo_spell != 0
        && *(*(*wp).w_s).b_p_spl as ::core::ffi::c_int != NUL
        && (*(*wp).w_s).b_langp.ga_len > 0 as ::core::ffi::c_int
        && !(*((*(*wp).w_s).b_langp.ga_data as *mut *mut ::core::ffi::c_char)).is_null();
}
#[no_mangle]
pub unsafe extern "C" fn no_spell_checking(mut wp: *mut win_T) -> bool {
    if (*wp).w_onebuf_opt.wo_spell == 0
        || *(*(*wp).w_s).b_p_spl as ::core::ffi::c_int == NUL
        || (*(*wp).w_s).b_langp.ga_len <= 0 as ::core::ffi::c_int
    {
        emsg(gettext(&raw const e_no_spell as *const ::core::ffi::c_char));
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn decor_spell_nav_col(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut decor_lnum: *mut linenr_T,
    mut col: ::core::ffi::c_int,
) -> TriState {
    if *decor_lnum != lnum {
        decor_redraw_reset(wp, decor_state.ptr());
        decor_providers_invoke_spell(
            wp,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            col,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            -1 as ::core::ffi::c_int,
        );
        decor_redraw_line(
            wp,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            decor_state.ptr(),
        );
        *decor_lnum = lnum;
    }
    decor_redraw_col(
        wp,
        col,
        0 as ::core::ffi::c_int,
        false_0 != 0,
        decor_state.ptr(),
        MAXCOL as ::core::ffi::c_int,
    );
    return (*decor_state.ptr()).spell;
}
#[inline]
unsafe extern "C" fn can_syn_spell(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: ::core::ffi::c_int,
) -> bool {
    let mut can_spell: bool = false;
    syn_get_id(
        wp,
        lnum,
        col as colnr_T,
        false_0,
        &raw mut can_spell,
        false_0,
    );
    return can_spell;
}
#[no_mangle]
pub unsafe extern "C" fn spell_move_to(
    mut wp: *mut win_T,
    mut dir: ::core::ffi::c_int,
    mut behaviour: smt_T,
    mut curline: bool,
    mut attrp: *mut hlf_T,
) -> size_t {
    if no_spell_checking(wp) {
        return 0 as size_t;
    }
    let mut found_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut found_len: size_t = 0 as size_t;
    let mut attr: hlf_T = HLF_COUNT;
    let mut has_syntax: bool = syntax_present(wp);
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buflen: size_t = 0 as size_t;
    let mut skip: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut capcol: colnr_T = -1 as colnr_T;
    let mut found_one: bool = false_0 != 0;
    let mut wrapped: bool = false_0 != 0;
    let mut ret: size_t = 0 as size_t;
    let mut lnum: linenr_T = (*wp).w_cursor.lnum;
    clearpos(&raw mut found_pos);
    let mut saved_decor_start: DecorState = decor_state.get();
    let mut decor_lnum: linenr_T = -1 as linenr_T;
    decor_state.set(DecorState {
        itr: [MarkTreeIter {
            pos: MTPos {
                row: 0 as int32_t,
                col: 0,
            },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_13 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }],
        slots: C2Rust_Unnamed_20 {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<DecorRangeSlot>(),
        },
        ranges_i: C2Rust_Unnamed_19 {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<::core::ffi::c_int>(),
        },
        current_end: 0,
        future_begin: 0,
        free_slot_i: 0,
        new_range_ordering: 0,
        win: ::core::ptr::null_mut::<win_T>(),
        top_row: 0,
        row: 0,
        col_last: 0,
        current: 0,
        eol_col: 0,
        conceal: 0,
        conceal_char: 0,
        conceal_attr: 0,
        spell: kFalse,
        running_decor_provider: false,
        itr_valid: false,
    });
    '_theend: while !got_int.get() {
        let mut line: *mut ::core::ffi::c_char = ml_get_buf((*wp).w_buffer, lnum);
        let mut len: size_t = ml_get_buf_len((*wp).w_buffer, lnum) as size_t;
        if buflen
            < len
                .wrapping_add(MAXWLEN as ::core::ffi::c_int as size_t)
                .wrapping_add(2 as size_t)
        {
            xfree(buf as *mut ::core::ffi::c_void);
            buflen = len
                .wrapping_add(MAXWLEN as ::core::ffi::c_int as size_t)
                .wrapping_add(2 as size_t);
            buf = xmalloc(buflen) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label: {
            if !buf.is_null()
                && buflen
                    >= len
                        .wrapping_add(MAXWLEN as ::core::ffi::c_int as size_t)
                        .wrapping_add(2 as size_t)
            {
            } else {
                __assert_fail(
                    b"buf && buflen >= len + MAXWLEN + 2\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/spell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1340 as ::core::ffi::c_uint,
                    b"size_t spell_move_to(win_T *, int, smt_T, _Bool, hlf_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if lnum == 1 as linenr_T {
            capcol = 0 as ::core::ffi::c_int as colnr_T;
        }
        if capcol == 0 as ::core::ffi::c_int {
            capcol = getwhitecols(line) as colnr_T;
        } else if curline as ::core::ffi::c_int != 0 && wp == curwin.get() {
            let mut col: colnr_T = getwhitecols(line) as colnr_T;
            if check_need_cap(curwin.get(), lnum, col) {
                capcol = col;
            }
            line = ml_get_buf((*wp).w_buffer, lnum);
        }
        let mut empty_line: bool = *skipwhite(line) as ::core::ffi::c_int == NUL;
        strcpy(buf, line);
        if lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
            spell_cat_line(
                buf.offset(strlen(buf) as isize),
                ml_get_buf((*wp).w_buffer, lnum + 1 as linenr_T),
                MAXWLEN as ::core::ffi::c_int,
            );
        }
        let mut p: *mut ::core::ffi::c_char = buf.offset(skip as isize);
        let mut endp: *mut ::core::ffi::c_char = buf.offset(len as isize);
        while p < endp {
            if dir == BACKWARD as ::core::ffi::c_int
                && lnum == (*wp).w_cursor.lnum
                && !wrapped
                && p.offset_from(buf) as colnr_T >= (*wp).w_cursor.col
            {
                break;
            }
            attr = HLF_COUNT;
            len = spell_check(wp, p, &raw mut attr, &raw mut capcol, false_0 != 0);
            if attr as ::core::ffi::c_uint != HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if behaviour as ::core::ffi::c_uint
                    == SMT_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                    || behaviour as ::core::ffi::c_uint
                        == SMT_BAD as ::core::ffi::c_int as ::core::ffi::c_uint
                        && attr as ::core::ffi::c_uint
                            == HLF_SPB as ::core::ffi::c_int as ::core::ffi::c_uint
                    || behaviour as ::core::ffi::c_uint
                        == SMT_RARE as ::core::ffi::c_int as ::core::ffi::c_uint
                        && attr as ::core::ffi::c_uint
                            == HLF_SPR as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if dir == BACKWARD as ::core::ffi::c_int
                        || lnum != (*wp).w_cursor.lnum
                        || wrapped as ::core::ffi::c_int != 0
                        || (if curline as ::core::ffi::c_int != 0 {
                            p.offset_from(buf) + len as isize
                        } else {
                            p.offset_from(buf)
                        }) as colnr_T
                            > (*wp).w_cursor.col
                    {
                        let mut col_0: colnr_T = p.offset_from(buf) as colnr_T;
                        let mut no_plain_buffer: bool = (*(*wp).w_s).b_p_spo_flags
                            & kOptSpoFlagNoplainbuffer as ::core::ffi::c_int as ::core::ffi::c_uint
                            != 0 as ::core::ffi::c_uint;
                        let mut can_spell: bool = !no_plain_buffer;
                        match decor_spell_nav_col(
                            wp,
                            lnum,
                            &raw mut decor_lnum,
                            col_0 as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                        {
                            1 => {
                                can_spell = true_0 != 0;
                            }
                            0 => {
                                can_spell = false_0 != 0;
                            }
                            -1 => {
                                if has_syntax {
                                    can_spell =
                                        can_syn_spell(wp, lnum, col_0 as ::core::ffi::c_int);
                                }
                            }
                            _ => {}
                        }
                        if !can_spell {
                            attr = HLF_COUNT;
                        }
                        if can_spell {
                            found_one = true_0 != 0;
                            found_pos = pos_T {
                                lnum: lnum,
                                col: col_0,
                                coladd: 0 as colnr_T,
                            };
                            if dir == FORWARD as ::core::ffi::c_int {
                                (*wp).w_cursor = found_pos;
                                if !attrp.is_null() {
                                    *attrp = attr;
                                }
                                ret = len;
                                break '_theend;
                            } else {
                                if curline {
                                    '_c2rust_label_0: {
                                        if len <= 2147483647 as ::core::ffi::c_int as size_t {
                                        } else {
                                            __assert_fail(
                                                b"len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/spell.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                1438 as ::core::ffi::c_uint,
                                                b"size_t spell_move_to(win_T *, int, smt_T, _Bool, hlf_T *)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    found_pos.col += len as ::core::ffi::c_int;
                                }
                                found_len = len;
                            }
                        }
                    } else {
                        found_one = true_0 != 0;
                    }
                }
            }
            p = p.offset(len as isize);
            '_c2rust_label_1: {
                if len <= 2147483647 as ::core::ffi::c_int as size_t {
                } else {
                    __assert_fail(
                        b"len <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/spell.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1451 as ::core::ffi::c_uint,
                        b"size_t spell_move_to(win_T *, int, smt_T, _Bool, hlf_T *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            capcol -= len as ::core::ffi::c_int;
        }
        if dir == BACKWARD as ::core::ffi::c_int && found_pos.lnum != 0 as linenr_T {
            (*wp).w_cursor = found_pos;
            ret = found_len;
            break;
        } else {
            if curline {
                break;
            }
            if lnum == (*wp).w_cursor.lnum && wrapped as ::core::ffi::c_int != 0 {
                break;
            }
            if dir == BACKWARD as ::core::ffi::c_int {
                if lnum > 1 as linenr_T {
                    lnum -= 1;
                } else {
                    if p_ws.get() == 0 {
                        break;
                    }
                    lnum = (*(*wp).w_buffer).b_ml.ml_line_count;
                    wrapped = true_0 != 0;
                    if !shortmess(SHM_SEARCH as ::core::ffi::c_int) {
                        give_warning(
                            gettext(&raw const top_bot_msg as *const ::core::ffi::c_char),
                            true_0 != 0,
                            false_0 != 0,
                        );
                    }
                }
                capcol = -1 as ::core::ffi::c_int as colnr_T;
            } else {
                if lnum < (*(*wp).w_buffer).b_ml.ml_line_count {
                    lnum += 1;
                } else {
                    if p_ws.get() == 0 {
                        break;
                    }
                    lnum = 1 as ::core::ffi::c_int as linenr_T;
                    wrapped = true_0 != 0;
                    if !shortmess(SHM_SEARCH as ::core::ffi::c_int) {
                        give_warning(
                            gettext(&raw const bot_top_msg as *const ::core::ffi::c_char),
                            true_0 != 0,
                            false_0 != 0,
                        );
                    }
                }
                if lnum == (*wp).w_cursor.lnum && !found_one {
                    break;
                }
                if attr as ::core::ffi::c_uint
                    == HLF_COUNT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    skip = p.offset_from(endp) as ::core::ffi::c_int;
                } else {
                    skip = 0 as ::core::ffi::c_int;
                }
                capcol -= 1;
                if empty_line {
                    capcol = 0 as ::core::ffi::c_int as colnr_T;
                }
            }
            line_breakcheck();
        }
    }
    decor_state_free(decor_state.ptr());
    decor_state.set(saved_decor_start);
    xfree(buf as *mut ::core::ffi::c_void);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn spell_cat_line(
    mut buf: *mut ::core::ffi::c_char,
    mut line: *mut ::core::ffi::c_char,
    mut maxlen: ::core::ffi::c_int,
) {
    let mut p: *mut ::core::ffi::c_char = skipwhite(line);
    while !vim_strchr(
        b"*#/\"\t\0".as_ptr() as *const ::core::ffi::c_char,
        *p as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
    {
        p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
    }
    if *p as ::core::ffi::c_int == NUL {
        return;
    }
    let mut n: ::core::ffi::c_int =
        p.offset_from(line) as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if n < maxlen - 1 as ::core::ffi::c_int {
        memset(
            buf as *mut ::core::ffi::c_void,
            ' ' as ::core::ffi::c_int,
            n as size_t,
        );
        xstrlcpy(buf.offset(n as isize), p, (maxlen - n) as size_t);
    }
}
unsafe extern "C" fn spell_load_lang(mut lang: *mut ::core::ffi::c_char) {
    let mut fname_enc: [::core::ffi::c_char; 85] = [0; 85];
    let mut r: ::core::ffi::c_int = 0;
    let mut sl: spelload_T = spelload_T {
        sl_lang: [0; 255],
        sl_slang: ::core::ptr::null_mut::<slang_T>(),
        sl_nobreak: 0,
    };
    strcpy(&raw mut sl.sl_lang as *mut ::core::ffi::c_char, lang);
    sl.sl_slang = ::core::ptr::null_mut::<slang_T>();
    sl.sl_nobreak = false_0;
    (*curbuf.get()).b_locked += 1;
    let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while round <= 2 as ::core::ffi::c_int {
        vim_snprintf(
            &raw mut fname_enc as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 85]>().wrapping_sub(5 as size_t),
            b"spell/%s.%s.spl\0".as_ptr() as *const ::core::ffi::c_char,
            lang,
            spell_enc(),
        );
        r = do_in_runtimepath(
            &raw mut fname_enc as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            Some(
                spell_load_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            &raw mut sl as *mut ::core::ffi::c_void,
        );
        if !(r == FAIL
            && *(&raw mut sl.sl_lang as *mut ::core::ffi::c_char) as ::core::ffi::c_int != NUL)
        {
            break;
        }
        vim_snprintf(
            &raw mut fname_enc as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 85]>().wrapping_sub(5 as size_t),
            b"spell/%s.ascii.spl\0".as_ptr() as *const ::core::ffi::c_char,
            lang,
        );
        r = do_in_runtimepath(
            &raw mut fname_enc as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            Some(
                spell_load_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            &raw mut sl as *mut ::core::ffi::c_void,
        );
        if !(r == FAIL
            && *(&raw mut sl.sl_lang as *mut ::core::ffi::c_char) as ::core::ffi::c_int != NUL
            && round == 1 as ::core::ffi::c_int
            && apply_autocmds(
                EVENT_SPELLFILEMISSING,
                lang,
                (*curbuf.get()).b_fname,
                false_0 != 0,
                curbuf.get(),
            ) as ::core::ffi::c_int
                != 0)
        {
            break;
        }
        round += 1;
    }
    if r == FAIL {
        if starting.get() != 0 {
            let mut autocmd_buf: [::core::ffi::c_char; 512] = [
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
            ];
            snprintf(
                &raw mut autocmd_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 512]>(),
                b"autocmd VimEnter * call v:lua.require'nvim.spellfile'.get('%s')|set spell\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                lang,
            );
            do_cmdline_cmd(&raw mut autocmd_buf as *mut ::core::ffi::c_char);
        } else {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(
                    b"Warning: Cannot find word list \"%s.%s.spl\" or \"%s.ascii.spl\"\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                lang,
                spell_enc(),
                lang,
            );
        }
    } else if !sl.sl_slang.is_null() {
        strcpy(
            (&raw mut fname_enc as *mut ::core::ffi::c_char)
                .offset(strlen(&raw mut fname_enc as *mut ::core::ffi::c_char) as isize)
                .offset(-(3 as ::core::ffi::c_int as isize)),
            b"add.spl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        do_in_runtimepath(
            &raw mut fname_enc as *mut ::core::ffi::c_char,
            DIP_ALL as ::core::ffi::c_int,
            Some(
                spell_load_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut *mut ::core::ffi::c_char,
                        bool,
                        *mut ::core::ffi::c_void,
                    ) -> bool,
            ),
            &raw mut sl as *mut ::core::ffi::c_void,
        );
    }
    (*curbuf.get()).b_locked -= 1;
}
#[no_mangle]
pub unsafe extern "C" fn spell_enc() -> *mut ::core::ffi::c_char {
    if strlen(p_enc.get()) < 60 as size_t
        && strcmp(
            p_enc.get(),
            b"iso-8859-15\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_int
    {
        return p_enc.get();
    }
    return b"latin1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn int_wordlist_spl(mut fname: *mut ::core::ffi::c_char) {
    vim_snprintf(
        fname,
        MAXPATHL as size_t,
        SPL_FNAME_TMPL.as_ptr(),
        int_wordlist.get(),
        spell_enc(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn slang_alloc(mut lang: *mut ::core::ffi::c_char) -> *mut slang_T {
    let mut lp: *mut slang_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<slang_T>()) as *mut slang_T;
    if !lang.is_null() {
        (*lp).sl_name = xstrdup(lang);
    }
    ga_init(
        &raw mut (*lp).sl_rep,
        ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut (*lp).sl_repsal,
        ::core::mem::size_of::<fromto_T>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    (*lp).sl_compmax = MAXWLEN as ::core::ffi::c_int;
    (*lp).sl_compsylmax = MAXWLEN as ::core::ffi::c_int;
    hash_init(&raw mut (*lp).sl_wordcount);
    return lp;
}
#[no_mangle]
pub unsafe extern "C" fn slang_free(mut lp: *mut slang_T) {
    xfree((*lp).sl_name as *mut ::core::ffi::c_void);
    xfree((*lp).sl_fname as *mut ::core::ffi::c_void);
    slang_clear(lp);
    xfree(lp as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn free_salitem(mut smp: *mut salitem_T) {
    xfree((*smp).sm_lead as *mut ::core::ffi::c_void);
    xfree((*smp).sm_to as *mut ::core::ffi::c_void);
    xfree((*smp).sm_lead_w as *mut ::core::ffi::c_void);
    xfree((*smp).sm_oneof_w as *mut ::core::ffi::c_void);
    xfree((*smp).sm_to_w as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn free_fromto(mut ftp: *mut fromto_T) {
    xfree((*ftp).ft_from as *mut ::core::ffi::c_void);
    xfree((*ftp).ft_to as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn slang_clear(mut lp: *mut slang_T) {
    let mut gap: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_fbyts as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_kbyts as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_pbyts as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL;
    let _ = *ptr__1;
    let mut ptr__2: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_fidxs as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__2);
    *ptr__2 = NULL;
    let _ = *ptr__2;
    let mut ptr__3: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_kidxs as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__3);
    *ptr__3 = NULL;
    let _ = *ptr__3;
    let mut ptr__4: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_pidxs as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__4);
    *ptr__4 = NULL;
    let _ = *ptr__4;
    let mut _gap: *mut garray_T = &raw mut (*lp).sl_rep;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut fromto_T = ((*_gap).ga_data as *mut fromto_T).offset(i as isize);
            free_fromto(_item);
            i += 1;
        }
    }
    ga_clear(_gap);
    let mut _gap_0: *mut garray_T = &raw mut (*lp).sl_repsal;
    if !(*_gap_0).ga_data.is_null() {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*_gap_0).ga_len {
            let mut _item_0: *mut fromto_T =
                ((*_gap_0).ga_data as *mut fromto_T).offset(i_0 as isize);
            free_fromto(_item_0);
            i_0 += 1;
        }
    }
    ga_clear(_gap_0);
    gap = &raw mut (*lp).sl_sal;
    if (*lp).sl_sofo {
        let mut _gap_1: *mut garray_T = gap;
        if !(*_gap_1).ga_data.is_null() {
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < (*_gap_1).ga_len {
                let mut _item_1: *mut *mut ::core::ffi::c_void =
                    ((*_gap_1).ga_data as *mut *mut ::core::ffi::c_void).offset(i_1 as isize);
                xfree(*_item_1);
                i_1 += 1;
            }
        }
        ga_clear(_gap_1);
    } else {
        let mut _gap_2: *mut garray_T = gap;
        if !(*_gap_2).ga_data.is_null() {
            let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_2 < (*_gap_2).ga_len {
                let mut _item_2: *mut salitem_T =
                    ((*_gap_2).ga_data as *mut salitem_T).offset(i_2 as isize);
                free_salitem(_item_2);
                i_2 += 1;
            }
        }
        ga_clear(_gap_2);
    }
    let mut i_3: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_3 < (*lp).sl_prefixcnt {
        vim_regfree(*(*lp).sl_prefprog.offset(i_3 as isize));
        i_3 += 1;
    }
    (*lp).sl_prefixcnt = 0 as ::core::ffi::c_int;
    let mut ptr__5: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_prefprog as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__5);
    *ptr__5 = NULL;
    let _ = *ptr__5;
    let mut ptr__6: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_info as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__6);
    *ptr__6 = NULL;
    let _ = *ptr__6;
    let mut ptr__7: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_midword as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__7);
    *ptr__7 = NULL;
    let _ = *ptr__7;
    vim_regfree((*lp).sl_compprog);
    (*lp).sl_compprog = ::core::ptr::null_mut::<regprog_T>();
    let mut ptr__8: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_comprules as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__8);
    *ptr__8 = NULL;
    let _ = *ptr__8;
    let mut ptr__9: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_compstartflags as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__9);
    *ptr__9 = NULL;
    let _ = *ptr__9;
    let mut ptr__10: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_compallflags as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__10);
    *ptr__10 = NULL;
    let _ = *ptr__10;
    let mut ptr__11: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_syllable as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__11);
    *ptr__11 = NULL;
    let _ = *ptr__11;
    ga_clear(&raw mut (*lp).sl_syl_items);
    ga_clear_strings(&raw mut (*lp).sl_comppat);
    hash_clear_all(
        &raw mut (*lp).sl_wordcount,
        WC_KEY_OFF as ::core::ffi::c_uint,
    );
    hash_init(&raw mut (*lp).sl_wordcount);
    hash_clear_all(&raw mut (*lp).sl_map_hash, 0 as ::core::ffi::c_uint);
    slang_clear_sug(lp);
    (*lp).sl_compmax = MAXWLEN as ::core::ffi::c_int;
    (*lp).sl_compminlen = 0 as ::core::ffi::c_int;
    (*lp).sl_compsylmax = MAXWLEN as ::core::ffi::c_int;
    (*lp).sl_regions[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn slang_clear_sug(mut lp: *mut slang_T) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_sbyts as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*lp).sl_sidxs as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
    close_spellbuf((*lp).sl_sugbuf);
    (*lp).sl_sugbuf = ::core::ptr::null_mut::<buf_T>();
    (*lp).sl_sugloaded = false_0 != 0;
    (*lp).sl_sugtime = 0 as time_t;
}
unsafe extern "C" fn spell_load_cb(
    mut num_fnames: ::core::ffi::c_int,
    mut fnames: *mut *mut ::core::ffi::c_char,
    mut all: bool,
    mut cookie: *mut ::core::ffi::c_void,
) -> bool {
    let mut slp: *mut spelload_T = cookie as *mut spelload_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < num_fnames {
        let mut slang: *mut slang_T = spell_load_file(
            *fnames.offset(i as isize),
            &raw mut (*slp).sl_lang as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<slang_T>(),
            false_0 != 0,
        );
        if !slang.is_null() {
            if (*slp).sl_nobreak != 0 && (*slang).sl_add as ::core::ffi::c_int != 0 {
                (*slang).sl_nobreak = true_0 != 0;
            } else if (*slang).sl_nobreak {
                (*slp).sl_nobreak = true_0;
            }
            (*slp).sl_slang = slang;
            if !all {
                break;
            }
        }
        i += 1;
    }
    return num_fnames > 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn count_common_word(
    mut lp: *mut slang_T,
    mut word: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut count: uint8_t,
) {
    let mut buf: [::core::ffi::c_char; 254] = [0; 254];
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if len == -1 as ::core::ffi::c_int {
        p = word;
    } else if len >= MAXWLEN as ::core::ffi::c_int {
        return;
    } else {
        xmemcpyz(
            &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            word as *const ::core::ffi::c_void,
            len as size_t,
        );
        p = &raw mut buf as *mut ::core::ffi::c_char;
    }
    let mut hash: hash_T = hash_hash(p);
    let p_len: size_t = strlen(p);
    let mut hi: *mut hashitem_T = hash_lookup(&raw mut (*lp).sl_wordcount, p, p_len, hash);
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
    {
        let mut wc: *mut wordcount_T =
            xmalloc((2 as size_t).wrapping_add(p_len).wrapping_add(1 as size_t))
                as *mut wordcount_T;
        memcpy(
            &raw mut (*wc).wc_word as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            p_len.wrapping_add(1 as size_t),
        );
        (*wc).wc_count = count as uint16_t;
        hash_add_item(
            &raw mut (*lp).sl_wordcount,
            hi,
            &raw mut (*wc).wc_word as *mut ::core::ffi::c_char,
            hash,
        );
    } else {
        let mut wc_0: *mut wordcount_T =
            (*hi).hi_key.offset(-(WC_KEY_OFF as isize)) as *mut wordcount_T;
        (*wc_0).wc_count =
            ((*wc_0).wc_count as ::core::ffi::c_int + count as ::core::ffi::c_int) as uint16_t;
        if ((*wc_0).wc_count as ::core::ffi::c_int) < count as ::core::ffi::c_int {
            (*wc_0).wc_count = MAXWORDCOUNT as ::core::ffi::c_int as uint16_t;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn byte_in_str(mut str: *mut uint8_t, mut n: ::core::ffi::c_int) -> bool {
    let mut p: *mut uint8_t = str;
    while *p as ::core::ffi::c_int != NUL {
        if *p as ::core::ffi::c_int == n {
            return true_0 != 0;
        }
        p = p.offset(1);
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn init_syl_tab(mut slang: *mut slang_T) -> ::core::ffi::c_int {
    ga_init(
        &raw mut (*slang).sl_syl_items,
        ::core::mem::size_of::<syl_item_T>() as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
    );
    let mut p: *mut ::core::ffi::c_char =
        vim_strchr((*slang).sl_syllable, '/' as ::core::ffi::c_int);
    while !p.is_null() {
        let c2rust_fresh5 = p;
        p = p.offset(1);
        *c2rust_fresh5 = NUL as ::core::ffi::c_char;
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
        let mut s: *mut ::core::ffi::c_char = p;
        p = vim_strchr(p, '/' as ::core::ffi::c_int);
        let mut l: ::core::ffi::c_int = 0;
        if p.is_null() {
            l = strlen(s) as ::core::ffi::c_int;
        } else {
            l = p.offset_from(s) as ::core::ffi::c_int;
        }
        if l >= SY_MAXLEN {
            return SP_FORMERROR as ::core::ffi::c_int;
        }
        let mut syl: *mut syl_item_T = ga_append_via_ptr(
            &raw mut (*slang).sl_syl_items,
            ::core::mem::size_of::<syl_item_T>(),
        ) as *mut syl_item_T;
        xmemcpyz(
            &raw mut (*syl).sy_chars as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            s as *const ::core::ffi::c_void,
            l as size_t,
        );
        (*syl).sy_len = l;
    }
    return OK;
}
unsafe extern "C" fn count_syllables(
    mut slang: *mut slang_T,
    mut word: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*slang).sl_syllable.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut skip: bool = false_0 != 0;
    let mut len: ::core::ffi::c_int = 0;
    let mut p: *const ::core::ffi::c_char = word;
    while *p as ::core::ffi::c_int != NUL {
        if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
            len = 1 as ::core::ffi::c_int;
            cnt = 0 as ::core::ffi::c_int;
        } else {
            len = 0 as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*slang).sl_syl_items.ga_len {
                let mut syl: *mut syl_item_T =
                    ((*slang).sl_syl_items.ga_data as *mut syl_item_T).offset(i as isize);
                if (*syl).sy_len > len
                    && strncmp(
                        p,
                        &raw mut (*syl).sy_chars as *mut ::core::ffi::c_char,
                        (*syl).sy_len as size_t,
                    ) == 0 as ::core::ffi::c_int
                {
                    len = (*syl).sy_len;
                }
                i += 1;
            }
            if len != 0 as ::core::ffi::c_int {
                cnt += 1;
                skip = false_0 != 0;
            } else {
                let mut c: ::core::ffi::c_int = utf_ptr2char(p);
                len = utfc_ptr2len(p);
                if vim_strchr((*slang).sl_syllable, c).is_null() {
                    skip = false_0 != 0;
                } else if !skip {
                    cnt += 1;
                    skip = true_0 != 0;
                }
            }
        }
        p = p.offset(len as isize);
    }
    return cnt;
}
#[no_mangle]
pub unsafe extern "C" fn parse_spelllang(mut wp: *mut win_T) -> *mut ::core::ffi::c_char {
    let mut spf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut region_cp: [::core::ffi::c_char; 3] = [0; 3];
    let mut lang: [::core::ffi::c_char; 255] = [0; 255];
    let mut spf_name: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut use_region: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dont_use_region: bool = false_0 != 0;
    let mut nobreak: bool = false_0 != 0;
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut ret_msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut bufref, (*wp).w_buffer);
    if recursive.get() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    recursive.set(true_0 != 0);
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<langp_T>() as ::core::ffi::c_int,
        2 as ::core::ffi::c_int,
    );
    clear_midword(wp);
    let mut spl_copy: *mut ::core::ffi::c_char = xstrdup((*(*wp).w_s).b_p_spl);
    (*(*wp).w_s).b_cjk = 0 as ::core::ffi::c_int;
    let mut splp: *mut ::core::ffi::c_char = spl_copy;
    '_theend: {
        while *splp as ::core::ffi::c_int != NUL {
            let mut len: ::core::ffi::c_int = copy_option_part(
                &raw mut splp,
                &raw mut lang as *mut ::core::ffi::c_char,
                MAXWLEN as ::core::ffi::c_int as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            let mut region: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            if !valid_spelllang(&raw mut lang as *mut ::core::ffi::c_char) {
                continue;
            }
            if strcmp(
                &raw mut lang as *mut ::core::ffi::c_char,
                b"cjk\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*(*wp).w_s).b_cjk = 1 as ::core::ffi::c_int;
            } else {
                let mut slang: *mut slang_T = ::core::ptr::null_mut::<slang_T>();
                let mut filename: bool = false;
                if len > 4 as ::core::ffi::c_int
                    && path_fnamecmp(
                        (&raw mut lang as *mut ::core::ffi::c_char)
                            .offset(len as isize)
                            .offset(-(4 as ::core::ffi::c_int as isize)),
                        b".spl\0".as_ptr() as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                {
                    filename = true_0 != 0;
                    let mut p: *mut ::core::ffi::c_char = vim_strchr(
                        path_tail(&raw mut lang as *mut ::core::ffi::c_char),
                        '_' as ::core::ffi::c_int,
                    );
                    if !p.is_null()
                        && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'A' as ::core::ffi::c_uint
                            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'Z' as ::core::ffi::c_uint
                            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                >= 'a' as ::core::ffi::c_uint
                                && *p.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_uint
                                    <= 'z' as ::core::ffi::c_uint)
                        && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'A' as ::core::ffi::c_uint
                            && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'Z' as ::core::ffi::c_uint
                            || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                >= 'a' as ::core::ffi::c_uint
                                && *p.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_uint
                                    <= 'z' as ::core::ffi::c_uint)
                        && !(*p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            >= 'A' as ::core::ffi::c_uint
                            && *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                <= 'Z' as ::core::ffi::c_uint
                            || *p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                                >= 'a' as ::core::ffi::c_uint
                                && *p.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_uint
                                    <= 'z' as ::core::ffi::c_uint)
                    {
                        xstrlcpy(
                            &raw mut region_cp as *mut ::core::ffi::c_char,
                            p.offset(1 as ::core::ffi::c_int as isize),
                            3 as size_t,
                        );
                        memmove(
                            p as *mut ::core::ffi::c_void,
                            p.offset(3 as ::core::ffi::c_int as isize)
                                as *const ::core::ffi::c_void,
                            (len as isize
                                - p.offset_from(&raw mut lang as *mut ::core::ffi::c_char)
                                - 2 as isize) as size_t,
                        );
                        region = &raw mut region_cp as *mut ::core::ffi::c_char;
                    } else {
                        dont_use_region = true_0 != 0;
                    }
                    slang = first_lang.get();
                    while !slang.is_null() {
                        if path_full_compare(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            (*slang).sl_fname,
                            false_0 != 0,
                            true_0 != 0,
                        ) as ::core::ffi::c_uint
                            == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            break;
                        }
                        slang = (*slang).sl_next;
                    }
                } else {
                    filename = false_0 != 0;
                    if len > 3 as ::core::ffi::c_int
                        && lang[(len - 3 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
                            == '_' as ::core::ffi::c_int
                    {
                        region = (&raw mut lang as *mut ::core::ffi::c_char)
                            .offset(len as isize)
                            .offset(-(2 as ::core::ffi::c_int as isize));
                        lang[(len - 3 as ::core::ffi::c_int) as usize] = NUL as ::core::ffi::c_char;
                    } else {
                        dont_use_region = true_0 != 0;
                    }
                    slang = first_lang.get();
                    while !slang.is_null() {
                        if strcasecmp(&raw mut lang as *mut ::core::ffi::c_char, (*slang).sl_name)
                            == 0 as ::core::ffi::c_int
                        {
                            break;
                        }
                        slang = (*slang).sl_next;
                    }
                }
                if !region.is_null() {
                    if !use_region.is_null()
                        && strcmp(region, use_region) != 0 as ::core::ffi::c_int
                    {
                        dont_use_region = true_0 != 0;
                    }
                    use_region = region;
                }
                if slang.is_null() {
                    if filename {
                        spell_load_file(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            &raw mut lang as *mut ::core::ffi::c_char,
                            ::core::ptr::null_mut::<slang_T>(),
                            false_0 != 0,
                        );
                    } else {
                        spell_load_lang(&raw mut lang as *mut ::core::ffi::c_char);
                        if !bufref_valid(&raw mut bufref) || !win_valid_any_tab(wp) {
                            ret_msg = b"E797: SpellFileMissing autocommand deleted buffer\0"
                                .as_ptr()
                                as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                            break '_theend;
                        }
                    }
                }
                slang = first_lang.get();
                while !slang.is_null() {
                    if if filename as ::core::ffi::c_int != 0 {
                        (path_full_compare(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            (*slang).sl_fname,
                            false_0 != 0,
                            true_0 != 0,
                        ) as ::core::ffi::c_uint
                            == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                    } else {
                        (strcasecmp(&raw mut lang as *mut ::core::ffi::c_char, (*slang).sl_name)
                            == 0 as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                    } != 0
                    {
                        let mut region_mask: ::core::ffi::c_int = REGION_ALL as ::core::ffi::c_int;
                        if !filename && !region.is_null() {
                            let mut c: ::core::ffi::c_int = find_region(
                                &raw mut (*slang).sl_regions as *mut ::core::ffi::c_char,
                                region,
                            );
                            if c == REGION_ALL as ::core::ffi::c_int {
                                if (*slang).sl_add {
                                    if *(&raw mut (*slang).sl_regions as *mut ::core::ffi::c_char)
                                        as ::core::ffi::c_int
                                        != NUL
                                    {
                                        region_mask = 0 as ::core::ffi::c_int;
                                    }
                                } else {
                                    smsg(
                                        0 as ::core::ffi::c_int,
                                        gettext(b"Warning: region %s not supported\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        region,
                                    );
                                }
                            } else {
                                region_mask = (1 as ::core::ffi::c_int) << c;
                            }
                        }
                        if region_mask != 0 as ::core::ffi::c_int {
                            let mut p_: *mut langp_T =
                                ga_append_via_ptr(&raw mut ga, ::core::mem::size_of::<langp_T>())
                                    as *mut langp_T;
                            (*p_).lp_slang = slang;
                            (*p_).lp_region = region_mask;
                            use_midword(slang, wp);
                            if (*slang).sl_nobreak {
                                nobreak = true_0 != 0;
                            }
                        }
                    }
                    slang = (*slang).sl_next;
                }
            }
        }
        spf = (*(*curwin.get()).w_s).b_p_spf;
        let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while round == 0 as ::core::ffi::c_int || *spf as ::core::ffi::c_int != NUL {
            's_377: {
                if round == 0 as ::core::ffi::c_int {
                    if (*int_wordlist.ptr()).is_null() {
                        break 's_377;
                    } else {
                        int_wordlist_spl(&raw mut spf_name as *mut ::core::ffi::c_char);
                    }
                } else {
                    let mut len_0: ::core::ffi::c_int = copy_option_part(
                        &raw mut spf,
                        &raw mut spf_name as *mut ::core::ffi::c_char,
                        (MAXPATHL - 4 as ::core::ffi::c_int) as size_t,
                        b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    ) as ::core::ffi::c_int;
                    strcpy(
                        (&raw mut spf_name as *mut ::core::ffi::c_char).offset(len_0 as isize),
                        b".spl\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    );
                    let mut c_0: ::core::ffi::c_int = 0;
                    c_0 = 0 as ::core::ffi::c_int;
                    while c_0 < ga.ga_len {
                        let mut p_0: *mut ::core::ffi::c_char =
                            (*(*(ga.ga_data as *mut langp_T).offset(c_0 as isize)).lp_slang)
                                .sl_fname;
                        if !p_0.is_null()
                            && path_full_compare(
                                &raw mut spf_name as *mut ::core::ffi::c_char,
                                p_0,
                                false_0 != 0,
                                true_0 != 0,
                            ) as ::core::ffi::c_uint
                                == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            break;
                        }
                        c_0 += 1;
                    }
                    if c_0 < ga.ga_len {
                        break 's_377;
                    }
                }
                let mut slang_0: *mut slang_T = ::core::ptr::null_mut::<slang_T>();
                slang_0 = first_lang.get();
                while !slang_0.is_null() {
                    if path_full_compare(
                        &raw mut spf_name as *mut ::core::ffi::c_char,
                        (*slang_0).sl_fname,
                        false_0 != 0,
                        true_0 != 0,
                    ) as ::core::ffi::c_uint
                        == kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        break;
                    }
                    slang_0 = (*slang_0).sl_next;
                }
                if slang_0.is_null() {
                    if round == 0 as ::core::ffi::c_int {
                        strcpy(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            b"internal wordlist\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                    } else {
                        xstrlcpy(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            path_tail(&raw mut spf_name as *mut ::core::ffi::c_char),
                            (MAXWLEN as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
                        );
                        let mut p_1: *mut ::core::ffi::c_char = vim_strchr(
                            &raw mut lang as *mut ::core::ffi::c_char,
                            '.' as ::core::ffi::c_int,
                        );
                        if !p_1.is_null() {
                            *p_1 = NUL as ::core::ffi::c_char;
                        }
                    }
                    slang_0 = spell_load_file(
                        &raw mut spf_name as *mut ::core::ffi::c_char,
                        &raw mut lang as *mut ::core::ffi::c_char,
                        ::core::ptr::null_mut::<slang_T>(),
                        true_0 != 0,
                    );
                    if !slang_0.is_null() && nobreak as ::core::ffi::c_int != 0 {
                        (*slang_0).sl_nobreak = true_0 != 0;
                    }
                }
                if !slang_0.is_null() {
                    let mut region_mask_0: ::core::ffi::c_int = REGION_ALL as ::core::ffi::c_int;
                    if !use_region.is_null() && !dont_use_region {
                        let mut c_1: ::core::ffi::c_int = find_region(
                            &raw mut (*slang_0).sl_regions as *mut ::core::ffi::c_char,
                            use_region,
                        );
                        if c_1 != REGION_ALL as ::core::ffi::c_int {
                            region_mask_0 = (1 as ::core::ffi::c_int) << c_1;
                        } else if *(&raw mut (*slang_0).sl_regions as *mut ::core::ffi::c_char)
                            as ::core::ffi::c_int
                            != NUL
                        {
                            region_mask_0 = 0 as ::core::ffi::c_int;
                        }
                    }
                    if region_mask_0 != 0 as ::core::ffi::c_int {
                        let mut p__0: *mut langp_T =
                            ga_append_via_ptr(&raw mut ga, ::core::mem::size_of::<langp_T>())
                                as *mut langp_T;
                        (*p__0).lp_slang = slang_0;
                        (*p__0).lp_sallang = ::core::ptr::null_mut::<slang_T>();
                        (*p__0).lp_replang = ::core::ptr::null_mut::<slang_T>();
                        (*p__0).lp_region = region_mask_0;
                        use_midword(slang_0, wp);
                    }
                }
            }
            round += 1;
        }
        ga_clear(&raw mut (*(*wp).w_s).b_langp);
        (*(*wp).w_s).b_langp = ga;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < ga.ga_len {
            let mut lp: *mut langp_T = (ga.ga_data as *mut langp_T).offset(i as isize);
            if !((*(*lp).lp_slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int) {
                (*lp).lp_sallang = (*lp).lp_slang;
            } else {
                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while j < ga.ga_len {
                    let mut lp2: *mut langp_T = (ga.ga_data as *mut langp_T).offset(j as isize);
                    if !((*(*lp2).lp_slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int)
                        && strncmp(
                            (*(*lp).lp_slang).sl_name,
                            (*(*lp2).lp_slang).sl_name,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        (*lp).lp_sallang = (*lp2).lp_slang;
                        break;
                    } else {
                        j += 1;
                    }
                }
            }
            if !((*(*lp).lp_slang).sl_rep.ga_len <= 0 as ::core::ffi::c_int) {
                (*lp).lp_replang = (*lp).lp_slang;
            } else {
                let mut j_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while j_0 < ga.ga_len {
                    let mut lp2_0: *mut langp_T = (ga.ga_data as *mut langp_T).offset(j_0 as isize);
                    if !((*(*lp2_0).lp_slang).sl_rep.ga_len <= 0 as ::core::ffi::c_int)
                        && strncmp(
                            (*(*lp).lp_slang).sl_name,
                            (*(*lp2_0).lp_slang).sl_name,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        (*lp).lp_replang = (*lp2_0).lp_slang;
                        break;
                    } else {
                        j_0 += 1;
                    }
                }
            }
            i += 1;
        }
        redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
    }
    xfree(spl_copy as *mut ::core::ffi::c_void);
    recursive.set(false_0 != 0);
    return ret_msg;
}
unsafe extern "C" fn clear_midword(mut wp: *mut win_T) {
    memset(
        &raw mut (*(*wp).w_s).b_spell_ismw as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[bool; 256]>(),
    );
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*(*wp).w_s).b_spell_ismw_mb as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
}
unsafe extern "C" fn use_midword(mut lp: *mut slang_T, mut wp: *mut win_T) {
    if (*lp).sl_midword.is_null() {
        return;
    }
    let mut p: *mut ::core::ffi::c_char = (*lp).sl_midword;
    while *p as ::core::ffi::c_int != NUL {
        let c: ::core::ffi::c_int = utf_ptr2char(p);
        let l: ::core::ffi::c_int = utfc_ptr2len(p);
        if c < 256 as ::core::ffi::c_int && l <= 2 as ::core::ffi::c_int {
            (*(*wp).w_s).b_spell_ismw[c as usize] = true_0 != 0;
        } else if (*(*wp).w_s).b_spell_ismw_mb.is_null() {
            (*(*wp).w_s).b_spell_ismw_mb =
                xmemdupz(p as *const ::core::ffi::c_void, l as size_t) as *mut ::core::ffi::c_char;
        } else {
            let n: ::core::ffi::c_int = strlen((*(*wp).w_s).b_spell_ismw_mb) as ::core::ffi::c_int;
            let mut bp: *mut ::core::ffi::c_char = xstrnsave(
                (*(*wp).w_s).b_spell_ismw_mb,
                (n as size_t).wrapping_add(l as size_t),
            );
            xfree((*(*wp).w_s).b_spell_ismw_mb as *mut ::core::ffi::c_void);
            (*(*wp).w_s).b_spell_ismw_mb = bp;
            xmemcpyz(
                bp.offset(n as isize) as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                l as size_t,
            );
        }
        p = p.offset(l as isize);
    }
}
unsafe extern "C" fn find_region(
    mut rp: *const ::core::ffi::c_char,
    mut region: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    loop {
        if *rp.offset(i as isize) as ::core::ffi::c_int == NUL {
            return REGION_ALL as ::core::ffi::c_int;
        }
        if *rp.offset(i as isize) as ::core::ffi::c_int
            == *region.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            && *rp.offset((i + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == *region.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            break;
        }
        i += 2 as ::core::ffi::c_int;
    }
    return i / 2 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn captype(
    mut word: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    p = word;
    while !spell_iswordp_nmw(p, curwin.get()) {
        if if end.is_null() {
            (*p as ::core::ffi::c_int == NUL) as ::core::ffi::c_int
        } else {
            (p >= end) as ::core::ffi::c_int
        } != 0
        {
            return 0 as ::core::ffi::c_int;
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    let mut c: ::core::ffi::c_int = mb_ptr2char_adv(&raw mut p);
    let mut allcap: bool = false;
    allcap = if c >= 128 as ::core::ffi::c_int {
        mb_isupper(c) as ::core::ffi::c_int
    } else {
        (*spelltab.ptr()).st_isu[c as usize] as ::core::ffi::c_int
    } != 0;
    let mut firstcap: bool = allcap;
    let mut past_second: bool = false_0 != 0;
    while if end.is_null() {
        (*p as ::core::ffi::c_int != NUL) as ::core::ffi::c_int
    } else {
        (p < end) as ::core::ffi::c_int
    } != 0
    {
        if spell_iswordp_nmw(p, curwin.get()) {
            c = utf_ptr2char(p);
            if if c >= 128 as ::core::ffi::c_int {
                mb_isupper(c) as ::core::ffi::c_int
            } else {
                (*spelltab.ptr()).st_isu[c as usize] as ::core::ffi::c_int
            } == 0
            {
                if past_second as ::core::ffi::c_int != 0 && allcap as ::core::ffi::c_int != 0 {
                    return WF_KEEPCAP as ::core::ffi::c_int;
                }
                allcap = false_0 != 0;
            } else if !allcap {
                return WF_KEEPCAP as ::core::ffi::c_int;
            }
            past_second = true_0 != 0;
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    if allcap {
        return WF_ALLCAP as ::core::ffi::c_int;
    }
    if firstcap {
        return WF_ONECAP as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn spell_delete_wordlist() {
    if (*int_wordlist.ptr()).is_null() {
        return;
    }
    let mut fname: [::core::ffi::c_char; 4096] = [
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
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
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
    os_remove(int_wordlist.get());
    int_wordlist_spl(&raw mut fname as *mut ::core::ffi::c_char);
    os_remove(&raw mut fname as *mut ::core::ffi::c_char);
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        int_wordlist.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
}
#[no_mangle]
pub unsafe extern "C" fn spell_free_all() {
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        ga_clear(&raw mut (*buf).b_s.b_langp);
        buf = (*buf).b_next;
    }
    while !(*first_lang.ptr()).is_null() {
        let mut slang: *mut slang_T = first_lang.get();
        first_lang.set((*slang).sl_next);
        slang_free(slang);
    }
    spell_delete_wordlist();
    let mut ptr_: *mut *mut ::core::ffi::c_void = repl_to.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        repl_from.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
}
#[no_mangle]
pub unsafe extern "C" fn spell_reload() {
    init_spell_chartab();
    spell_free_all();
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if *(*(*wp).w_s).b_p_spl as ::core::ffi::c_int != NUL {
            if (*wp).w_onebuf_opt.wo_spell != 0 {
                parse_spelllang(wp);
                break;
            }
        }
        wp = (*wp).w_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn open_spellbuf() -> *mut buf_T {
    let mut buf: *mut buf_T = xcalloc(1 as size_t, ::core::mem::size_of::<buf_T>()) as *mut buf_T;
    (*buf).b_spell = true_0 != 0;
    (*buf).b_p_swf = true_0;
    if ml_open(buf) == FAIL {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"open_spellbuf\0".as_ptr() as *const ::core::ffi::c_char,
            2387 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error opening a new memline\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    ml_open_file(buf);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn close_spellbuf(mut buf: *mut buf_T) {
    if buf.is_null() {
        return;
    }
    ml_close(buf, true_0);
    xfree(buf as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn clear_spell_chartab(mut sp: *mut spelltab_T) {
    memset(
        &raw mut (*sp).st_isw as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[bool; 256]>(),
    );
    memset(
        &raw mut (*sp).st_isu as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[bool; 256]>(),
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        (*sp).st_fold[i as usize] = i as uint8_t;
        (*sp).st_upper[i as usize] = i as uint8_t;
        i += 1;
    }
    let mut i_0: ::core::ffi::c_int = '0' as ::core::ffi::c_int;
    while i_0 <= '9' as ::core::ffi::c_int {
        (*sp).st_isw[i_0 as usize] = true_0 != 0;
        i_0 += 1;
    }
    let mut i_1: ::core::ffi::c_int = 'A' as ::core::ffi::c_int;
    while i_1 <= 'Z' as ::core::ffi::c_int {
        (*sp).st_isw[i_1 as usize] = true_0 != 0;
        (*sp).st_isu[i_1 as usize] = true_0 != 0;
        (*sp).st_fold[i_1 as usize] = (i_1 + 0x20 as ::core::ffi::c_int) as uint8_t;
        i_1 += 1;
    }
    let mut i_2: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
    while i_2 <= 'z' as ::core::ffi::c_int {
        (*sp).st_isw[i_2 as usize] = true_0 != 0;
        (*sp).st_upper[i_2 as usize] = (i_2 - 0x20 as ::core::ffi::c_int) as uint8_t;
        i_2 += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn init_spell_chartab() {
    did_set_spelltab.set(false_0 != 0);
    clear_spell_chartab(spelltab.ptr());
    let mut i: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        let mut f: ::core::ffi::c_int = utf_fold(i);
        let mut u: ::core::ffi::c_int = mb_toupper(i);
        (*spelltab.ptr()).st_isu[i as usize] = mb_isupper(i);
        (*spelltab.ptr()).st_isw[i as usize] =
            (*spelltab.ptr()).st_isu[i as usize] as ::core::ffi::c_int != 0
                || mb_islower(i) as ::core::ffi::c_int != 0;
        (*spelltab.ptr()).st_fold[i as usize] = (if f < 256 as ::core::ffi::c_int {
            f as uint8_t as ::core::ffi::c_int
        } else {
            i as uint8_t as ::core::ffi::c_int
        }) as uint8_t;
        (*spelltab.ptr()).st_upper[i as usize] = (if u < 256 as ::core::ffi::c_int {
            u as uint8_t as ::core::ffi::c_int
        } else {
            i as uint8_t as ::core::ffi::c_int
        }) as uint8_t;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn spell_iswordp(
    mut p: *const ::core::ffi::c_char,
    mut wp: *const win_T,
) -> bool {
    let l: ::core::ffi::c_int = utfc_ptr2len(p);
    let mut s: *const ::core::ffi::c_char = p;
    if l == 1 as ::core::ffi::c_int {
        if (*(*wp).w_s).b_spell_ismw[*p as uint8_t as usize] {
            s = p.offset(1 as ::core::ffi::c_int as isize);
        }
    } else {
        let mut c: ::core::ffi::c_int = utf_ptr2char(p);
        if if c < 256 as ::core::ffi::c_int {
            (*(*wp).w_s).b_spell_ismw[c as usize] as ::core::ffi::c_int
        } else {
            (!(*(*wp).w_s).b_spell_ismw_mb.is_null()
                && !vim_strchr((*(*wp).w_s).b_spell_ismw_mb, c).is_null())
                as ::core::ffi::c_int
        } != 0
        {
            s = p.offset(l as isize);
        }
    }
    let mut c_0: ::core::ffi::c_int = utf_ptr2char(s);
    if c_0 > 255 as ::core::ffi::c_int {
        return spell_mb_isword_class(mb_get_class(s), wp);
    }
    return (*spelltab.ptr()).st_isw[c_0 as usize];
}
#[no_mangle]
pub unsafe extern "C" fn spell_iswordp_nmw(
    mut p: *const ::core::ffi::c_char,
    mut wp: *mut win_T,
) -> bool {
    let mut c: ::core::ffi::c_int = utf_ptr2char(p);
    if c > 255 as ::core::ffi::c_int {
        return spell_mb_isword_class(mb_get_class(p), wp);
    }
    return (*spelltab.ptr()).st_isw[c as usize];
}
unsafe extern "C" fn spell_mb_isword_class(
    mut cl: ::core::ffi::c_int,
    mut wp: *const win_T,
) -> bool {
    if (*(*wp).w_s).b_cjk != 0 {
        return cl == 2 as ::core::ffi::c_int || cl == 0x2800 as ::core::ffi::c_int;
    }
    return cl >= 2 as ::core::ffi::c_int
        && cl != 0x2070 as ::core::ffi::c_int
        && cl != 0x2080 as ::core::ffi::c_int
        && cl != 3 as ::core::ffi::c_int;
}
unsafe extern "C" fn spell_iswordp_w(
    mut p: *const ::core::ffi::c_int,
    mut wp: *const win_T,
) -> bool {
    let mut s: *const ::core::ffi::c_int = ::core::ptr::null::<::core::ffi::c_int>();
    if if *p < 256 as ::core::ffi::c_int {
        (*(*wp).w_s).b_spell_ismw[*p as usize] as ::core::ffi::c_int
    } else {
        (!(*(*wp).w_s).b_spell_ismw_mb.is_null()
            && !vim_strchr((*(*wp).w_s).b_spell_ismw_mb, *p).is_null())
            as ::core::ffi::c_int
    } != 0
    {
        s = p.offset(1 as ::core::ffi::c_int as isize);
    } else {
        s = p;
    }
    if *s > 255 as ::core::ffi::c_int {
        return spell_mb_isword_class(utf_class(*s), wp);
    }
    return (*spelltab.ptr()).st_isw[*s as usize];
}
#[no_mangle]
pub unsafe extern "C" fn spell_casefold(
    mut wp: *const win_T,
    mut str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if len >= buflen {
        *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        return FAIL;
    }
    let mut outi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *const ::core::ffi::c_char = str;
    while p < str.offset(len as isize) {
        if outi + MB_MAXBYTES as ::core::ffi::c_int > buflen {
            *buf.offset(outi as isize) = NUL as ::core::ffi::c_char;
            return FAIL;
        }
        let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut p);
        if c == 0x3a3 as ::core::ffi::c_int || c == 0x3c2 as ::core::ffi::c_int {
            if p == str.offset(len as isize) || !spell_iswordp(p, wp) {
                c = 0x3c2 as ::core::ffi::c_int;
            } else {
                c = 0x3c3 as ::core::ffi::c_int;
            }
        } else {
            c = if c >= 128 as ::core::ffi::c_int {
                utf_fold(c)
            } else {
                (*spelltab.ptr()).st_fold[c as usize] as ::core::ffi::c_int
            };
        }
        outi += utf_char2bytes(c, buf.offset(outi as isize));
    }
    *buf.offset(outi as isize) = NUL as ::core::ffi::c_char;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn check_need_cap(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
) -> bool {
    if (*(*wp).w_s).b_cap_prog.is_null() {
        return false_0 != 0;
    }
    let mut need_cap: bool = false_0 != 0;
    let mut line: *mut ::core::ffi::c_char = if col != 0 {
        ml_get_buf((*wp).w_buffer, lnum)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    let mut line_copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut endcol: colnr_T = 0 as colnr_T;
    if col == 0 as ::core::ffi::c_int || getwhitecols(line) >= col as intptr_t {
        if lnum == 1 as linenr_T {
            need_cap = true_0 != 0;
        } else {
            line = ml_get_buf((*wp).w_buffer, lnum - 1 as linenr_T);
            if *skipwhite(line) as ::core::ffi::c_int == NUL {
                need_cap = true_0 != 0;
            } else {
                line_copy = concat_str(line, b" \0".as_ptr() as *const ::core::ffi::c_char);
                line = line_copy;
                endcol = strlen(line) as colnr_T;
            }
        }
    } else {
        endcol = col;
    }
    if endcol > 0 as ::core::ffi::c_int {
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: (*(*wp).w_s).b_cap_prog,
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false_0 != 0,
        };
        let mut p: *mut ::core::ffi::c_char = line.offset(endcol as isize);
        loop {
            p = p.offset(
                -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                    + 1 as ::core::ffi::c_int) as isize),
            );
            if p == line || spell_iswordp_nmw(p, wp) as ::core::ffi::c_int != 0 {
                break;
            }
            if !(vim_regexec(&raw mut regmatch, p, 0 as colnr_T) as ::core::ffi::c_int != 0
                && regmatch.endp[0 as ::core::ffi::c_int as usize] == line.offset(endcol as isize))
            {
                continue;
            }
            need_cap = true_0 != 0;
            break;
        }
        (*(*wp).w_s).b_cap_prog = regmatch.regprog;
    }
    xfree(line_copy as *mut ::core::ffi::c_void);
    return need_cap;
}
#[no_mangle]
pub unsafe extern "C" fn ex_spellrepall(mut _eap: *mut exarg_T) {
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut save_ws: bool = p_ws.get() != 0;
    let mut prev_lnum: linenr_T = 0 as linenr_T;
    if (*repl_from.ptr()).is_null() || (*repl_to.ptr()).is_null() {
        emsg(gettext(
            b"E752: No previous spell replacement\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    let repl_from_len: size_t = strlen(repl_from.get());
    let repl_to_len: size_t = strlen(repl_to.get());
    let addlen: int64_t = repl_to_len as int64_t - repl_from_len as int64_t;
    let frompatsize: size_t = repl_from_len.wrapping_add(7 as size_t);
    let mut frompat: *mut ::core::ffi::c_char = xmalloc(frompatsize) as *mut ::core::ffi::c_char;
    let mut frompatlen: size_t = snprintf(
        frompat,
        frompatsize,
        b"\\V\\<%s\\>\0".as_ptr() as *const ::core::ffi::c_char,
        repl_from.get(),
    ) as size_t;
    p_ws.set(false_0);
    sub_nsubs.set(0 as ::core::ffi::c_int);
    sub_nlines.set(0 as ::core::ffi::c_int as linenr_T);
    (*curwin.get()).w_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
    while !got_int.get() {
        if do_search(
            ::core::ptr::null_mut::<oparg_T>(),
            '/' as ::core::ffi::c_int,
            '/' as ::core::ffi::c_int,
            frompat,
            frompatlen,
            1 as ::core::ffi::c_int,
            SEARCH_KEEP as ::core::ffi::c_int,
            ::core::ptr::null_mut::<searchit_arg_T>(),
        ) == 0 as ::core::ffi::c_int
            || u_save_cursor() == FAIL
        {
            break;
        }
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        if addlen <= 0 as int64_t
            || strncmp(
                line.offset((*curwin.get()).w_cursor.col as isize),
                repl_to.get(),
                repl_to_len,
            ) != 0 as ::core::ffi::c_int
        {
            let mut p: *mut ::core::ffi::c_char = xmalloc(
                ((get_cursor_line_len() as int64_t + addlen) as size_t).wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            memmove(
                p as *mut ::core::ffi::c_void,
                line as *const ::core::ffi::c_void,
                (*curwin.get()).w_cursor.col as size_t,
            );
            strcpy(
                p.offset((*curwin.get()).w_cursor.col as isize),
                repl_to.get(),
            );
            strcat(
                p,
                line.offset((*curwin.get()).w_cursor.col as isize)
                    .offset(repl_from_len as isize),
            );
            ml_replace((*curwin.get()).w_cursor.lnum, p, false_0 != 0);
            inserted_bytes(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.col,
                repl_from_len as ::core::ffi::c_int,
                repl_to_len as ::core::ffi::c_int,
            );
            if (*curwin.get()).w_cursor.lnum != prev_lnum {
                (*sub_nlines.ptr()) += 1;
                prev_lnum = (*curwin.get()).w_cursor.lnum;
            }
            (*sub_nsubs.ptr()) += 1;
        }
        (*curwin.get()).w_cursor.col += repl_to_len as colnr_T;
    }
    p_ws.set(save_ws as ::core::ffi::c_int);
    (*curwin.get()).w_cursor = pos;
    xfree(frompat as *mut ::core::ffi::c_void);
    if sub_nsubs.get() == 0 as ::core::ffi::c_int {
        semsg(
            gettext(b"E753: Not found: %s\0".as_ptr() as *const ::core::ffi::c_char),
            repl_from.get(),
        );
    } else {
        do_sub_msg(false_0 != 0);
    };
}
#[no_mangle]
pub unsafe extern "C" fn onecap_copy(
    mut word: *const ::core::ffi::c_char,
    mut wcopy: *mut ::core::ffi::c_char,
    mut upper: bool,
) {
    let mut p: *const ::core::ffi::c_char = word;
    let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut p);
    if upper {
        c = if c >= 128 as ::core::ffi::c_int {
            mb_toupper(c)
        } else {
            (*spelltab.ptr()).st_upper[c as usize] as ::core::ffi::c_int
        };
    } else {
        c = if c >= 128 as ::core::ffi::c_int {
            utf_fold(c)
        } else {
            (*spelltab.ptr()).st_fold[c as usize] as ::core::ffi::c_int
        };
    }
    let mut l: ::core::ffi::c_int = utf_char2bytes(c, wcopy);
    xstrlcpy(
        wcopy.offset(l as isize),
        p,
        (MAXWLEN as ::core::ffi::c_int - l) as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn allcap_copy(
    mut word: *const ::core::ffi::c_char,
    mut wcopy: *mut ::core::ffi::c_char,
) {
    let mut d: *mut ::core::ffi::c_char = wcopy;
    let mut s: *const ::core::ffi::c_char = word;
    while *s as ::core::ffi::c_int != NUL {
        let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
        if c == 0xdf as ::core::ffi::c_int {
            c = 'S' as ::core::ffi::c_int;
            if d.offset_from(wcopy)
                >= (MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize
            {
                break;
            }
            let c2rust_fresh6 = d;
            d = d.offset(1);
            *c2rust_fresh6 = c as ::core::ffi::c_char;
        } else {
            c = if c >= 128 as ::core::ffi::c_int {
                mb_toupper(c)
            } else {
                (*spelltab.ptr()).st_upper[c as usize] as ::core::ffi::c_int
            };
        }
        if d.offset_from(wcopy)
            >= (MAXWLEN as ::core::ffi::c_int - MB_MAXBYTES as ::core::ffi::c_int) as isize
        {
            break;
        }
        d = d.offset(utf_char2bytes(c, d) as isize);
    }
    *d = NUL as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn nofold_len(
    mut fword: *mut ::core::ffi::c_char,
    mut flen: ::core::ffi::c_int,
    mut word: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    p = fword;
    while p < fword.offset(flen as isize) {
        i += 1;
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    p = word;
    while i > 0 as ::core::ffi::c_int {
        i -= 1;
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return p.offset_from(word) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn make_case_word(
    mut fword: *mut ::core::ffi::c_char,
    mut cword: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    if flags & WF_ALLCAP as ::core::ffi::c_int != 0 {
        allcap_copy(fword, cword);
    } else if flags & WF_ONECAP as ::core::ffi::c_int != 0 {
        onecap_copy(fword, cword, true_0 != 0);
    } else {
        strcpy(cword, fword);
    };
}
#[no_mangle]
pub unsafe extern "C" fn eval_soundfold(
    word: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if (*curwin.get()).w_onebuf_opt.wo_spell != 0
        && *(*(*curwin.get()).w_s).b_p_spl as ::core::ffi::c_int != NUL
    {
        let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
            let lp: *mut langp_T =
                ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
            if !((*(*lp).lp_slang).sl_sal.ga_len <= 0 as ::core::ffi::c_int) {
                let mut sound: [::core::ffi::c_char; 254] = [0; 254];
                spell_soundfold(
                    (*lp).lp_slang,
                    word as *mut ::core::ffi::c_char,
                    false_0 != 0,
                    &raw mut sound as *mut ::core::ffi::c_char,
                );
                return xstrdup(&raw mut sound as *mut ::core::ffi::c_char);
            }
            lpi += 1;
        }
    }
    return xstrdup(word);
}
#[no_mangle]
pub unsafe extern "C" fn spell_soundfold(
    mut slang: *mut slang_T,
    mut inword: *mut ::core::ffi::c_char,
    mut folded: bool,
    mut res: *mut ::core::ffi::c_char,
) {
    if (*slang).sl_sofo {
        spell_soundfold_sofo(slang, inword, res);
    } else {
        let mut fword: [::core::ffi::c_char; 254] = [0; 254];
        let mut word: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if folded {
            word = inword;
        } else {
            spell_casefold(
                curwin.get(),
                inword,
                strlen(inword) as ::core::ffi::c_int,
                &raw mut fword as *mut ::core::ffi::c_char,
                MAXWLEN as ::core::ffi::c_int,
            );
            word = &raw mut fword as *mut ::core::ffi::c_char;
        }
        spell_soundfold_wsal(slang, word, res);
    };
}
unsafe extern "C" fn spell_soundfold_sofo(
    mut slang: *mut slang_T,
    mut inword: *const ::core::ffi::c_char,
    mut res: *mut ::core::ffi::c_char,
) {
    let mut ri: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prevc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut s: *const ::core::ffi::c_char = inword;
    while *s as ::core::ffi::c_int != NUL {
        let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
        if utf_class(c) == 0 as ::core::ffi::c_int {
            c = ' ' as ::core::ffi::c_int;
        } else if c < 256 as ::core::ffi::c_int {
            c = (*slang).sl_sal_first[c as usize] as ::core::ffi::c_int;
        } else {
            let mut ip: *mut ::core::ffi::c_int = *((*slang).sl_sal.ga_data
                as *mut *mut ::core::ffi::c_int)
                .offset((c & 0xff as ::core::ffi::c_int) as isize);
            if ip.is_null() {
                c = NUL;
            } else {
                loop {
                    if *ip == 0 as ::core::ffi::c_int {
                        c = NUL;
                        break;
                    } else if *ip == c {
                        c = *ip.offset(1 as ::core::ffi::c_int as isize);
                        break;
                    } else {
                        ip = ip.offset(2 as ::core::ffi::c_int as isize);
                    }
                }
            }
        }
        if !(c != NUL && c != prevc) {
            continue;
        }
        ri += utf_char2bytes(c, res.offset(ri as isize));
        if ri + MB_MAXBYTES as ::core::ffi::c_int > MAXWLEN as ::core::ffi::c_int {
            break;
        }
        prevc = c;
    }
    *res.offset(ri as isize) = NUL as ::core::ffi::c_char;
}
unsafe extern "C" fn spell_soundfold_wsal(
    mut slang: *mut slang_T,
    mut inword: *const ::core::ffi::c_char,
    mut res: *mut ::core::ffi::c_char,
) {
    let mut word: [::core::ffi::c_int; 254] = [0 as ::core::ffi::c_int; 254];
    let mut did_white: bool = false_0 != 0;
    let mut wordlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut s: *const ::core::ffi::c_char = inword;
    while *s as ::core::ffi::c_int != NUL {
        let mut t: *const ::core::ffi::c_char = s;
        let mut c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
        if (*slang).sl_rem_accents {
            if utf_class(c) == 0 as ::core::ffi::c_int {
                if did_white {
                    continue;
                }
                c = ' ' as ::core::ffi::c_int;
                did_white = true_0 != 0;
            } else {
                did_white = false_0 != 0;
                if !spell_iswordp_nmw(t, curwin.get()) {
                    continue;
                }
            }
        }
        let c2rust_fresh7 = wordlen;
        wordlen = wordlen + 1;
        word[c2rust_fresh7 as usize] = c;
    }
    word[wordlen as usize] = NUL;
    let mut smp: *mut salitem_T = (*slang).sl_sal.ga_data as *mut salitem_T;
    let mut wres: [::core::ffi::c_int; 254] = [0 as ::core::ffi::c_int; 254];
    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p0: ::core::ffi::c_int = -333 as ::core::ffi::c_int;
    let mut c_0: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut reslen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut z: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        c_0 = word[i as usize];
        if c_0 == NUL {
            break;
        }
        let mut n: ::core::ffi::c_int = (*slang).sl_sal_first
            [(c_0 & 0xff as ::core::ffi::c_int) as usize]
            as ::core::ffi::c_int;
        let mut z0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if n >= 0 as ::core::ffi::c_int {
            let mut ws: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
            's_643: loop {
                ws = (*smp.offset(n as isize)).sm_lead_w;
                if !(*ws.offset(0 as ::core::ffi::c_int as isize) & 0xff as ::core::ffi::c_int
                    == c_0 & 0xff as ::core::ffi::c_int
                    && *ws.offset(0 as ::core::ffi::c_int as isize) != NUL)
                {
                    break;
                }
                's_104: {
                    if c_0 == *ws.offset(0 as ::core::ffi::c_int as isize) {
                        k = (*smp.offset(n as isize)).sm_leadlen;
                        if k > 1 as ::core::ffi::c_int {
                            if word[(i + 1 as ::core::ffi::c_int) as usize]
                                != *ws.offset(1 as ::core::ffi::c_int as isize)
                            {
                                break 's_104;
                            } else if k > 2 as ::core::ffi::c_int {
                                let mut j: ::core::ffi::c_int = 0;
                                j = 2 as ::core::ffi::c_int;
                                while j < k {
                                    if word[(i + j) as usize] != *ws.offset(j as isize) {
                                        break;
                                    }
                                    j += 1;
                                }
                                if j < k {
                                    break 's_104;
                                }
                            }
                        }
                        let mut pf: *mut ::core::ffi::c_int =
                            ::core::ptr::null_mut::<::core::ffi::c_int>();
                        pf = (*smp.offset(n as isize)).sm_oneof_w;
                        if !pf.is_null() {
                            while *pf != NUL && *pf != word[(i + k) as usize] {
                                pf = pf.offset(1);
                            }
                            if *pf == NUL {
                                break 's_104;
                            } else {
                                k += 1;
                            }
                        }
                        let mut s_0: *mut ::core::ffi::c_char = (*smp.offset(n as isize)).sm_rules;
                        let mut pri: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
                        p0 = *s_0 as uint8_t as ::core::ffi::c_int;
                        let mut k0: ::core::ffi::c_int = k;
                        while *s_0 as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                            && k > 1 as ::core::ffi::c_int
                        {
                            k -= 1;
                            s_0 = s_0.offset(1);
                        }
                        if *s_0 as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
                            s_0 = s_0.offset(1);
                        }
                        if ascii_isdigit(*s_0 as ::core::ffi::c_int) {
                            pri = *s_0 as uint8_t as ::core::ffi::c_int - '0' as ::core::ffi::c_int;
                            s_0 = s_0.offset(1);
                        }
                        if *s_0 as ::core::ffi::c_int == '^' as ::core::ffi::c_int
                            && *s_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '^' as ::core::ffi::c_int
                        {
                            s_0 = s_0.offset(1);
                        }
                        if *s_0 as ::core::ffi::c_int == NUL
                            || *s_0 as ::core::ffi::c_int == '^' as ::core::ffi::c_int
                                && (i == 0 as ::core::ffi::c_int
                                    || !(word[(i - 1 as ::core::ffi::c_int) as usize]
                                        == ' ' as ::core::ffi::c_int
                                        || spell_iswordp_w(
                                            (&raw mut word as *mut ::core::ffi::c_int)
                                                .offset(i as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize)),
                                            curwin.get(),
                                        )
                                            as ::core::ffi::c_int
                                            != 0))
                                && (*s_0.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    != '$' as ::core::ffi::c_int
                                    || !spell_iswordp_w(
                                        (&raw mut word as *mut ::core::ffi::c_int)
                                            .offset(i as isize)
                                            .offset(k0 as isize),
                                        curwin.get(),
                                    ))
                            || *s_0 as ::core::ffi::c_int == '$' as ::core::ffi::c_int
                                && i > 0 as ::core::ffi::c_int
                                && spell_iswordp_w(
                                    (&raw mut word as *mut ::core::ffi::c_int)
                                        .offset(i as isize)
                                        .offset(-(1 as ::core::ffi::c_int as isize)),
                                    curwin.get(),
                                ) as ::core::ffi::c_int
                                    != 0
                                && !spell_iswordp_w(
                                    (&raw mut word as *mut ::core::ffi::c_int)
                                        .offset(i as isize)
                                        .offset(k0 as isize),
                                    curwin.get(),
                                )
                        {
                            let mut c0: ::core::ffi::c_int =
                                word[(i + k - 1 as ::core::ffi::c_int) as usize];
                            let mut n0: ::core::ffi::c_int = (*slang).sl_sal_first
                                [(c0 & 0xff as ::core::ffi::c_int) as usize]
                                as ::core::ffi::c_int;
                            if (*slang).sl_followup as ::core::ffi::c_int != 0
                                && k > 1 as ::core::ffi::c_int
                                && n0 >= 0 as ::core::ffi::c_int
                                && p0 != '-' as ::core::ffi::c_int
                                && word[(i + k) as usize] != NUL
                            {
                                's_446: loop {
                                    ws = (*smp.offset(n0 as isize)).sm_lead_w;
                                    if *ws.offset(0 as ::core::ffi::c_int as isize)
                                        & 0xff as ::core::ffi::c_int
                                        != c0 & 0xff as ::core::ffi::c_int
                                    {
                                        break;
                                    }
                                    's_277: {
                                        if c0 == *ws.offset(0 as ::core::ffi::c_int as isize) {
                                            k0 = (*smp.offset(n0 as isize)).sm_leadlen;
                                            if k0 > 1 as ::core::ffi::c_int {
                                                if word[(i + k) as usize]
                                                    != *ws.offset(1 as ::core::ffi::c_int as isize)
                                                {
                                                    break 's_277;
                                                } else if k0 > 2 as ::core::ffi::c_int {
                                                    pf = (&raw mut word as *mut ::core::ffi::c_int)
                                                        .offset(i as isize)
                                                        .offset(k as isize)
                                                        .offset(1 as ::core::ffi::c_int as isize);
                                                    let mut j_0: ::core::ffi::c_int = 0;
                                                    j_0 = 2 as ::core::ffi::c_int;
                                                    while j_0 < k0 {
                                                        let c2rust_fresh8 = pf;
                                                        pf = pf.offset(1);
                                                        if *c2rust_fresh8
                                                            != *ws.offset(j_0 as isize)
                                                        {
                                                            break;
                                                        }
                                                        j_0 += 1;
                                                    }
                                                    if j_0 < k0 {
                                                        break 's_277;
                                                    }
                                                }
                                            }
                                            k0 += k - 1 as ::core::ffi::c_int;
                                            pf = (*smp.offset(n0 as isize)).sm_oneof_w;
                                            if !pf.is_null() {
                                                while *pf != NUL && *pf != word[(i + k0) as usize] {
                                                    pf = pf.offset(1);
                                                }
                                                if *pf == NUL {
                                                    break 's_277;
                                                } else {
                                                    k0 += 1;
                                                }
                                            }
                                            p0 = 5 as ::core::ffi::c_int;
                                            s_0 = (*smp.offset(n0 as isize)).sm_rules;
                                            while *s_0 as ::core::ffi::c_int
                                                == '-' as ::core::ffi::c_int
                                            {
                                                s_0 = s_0.offset(1);
                                            }
                                            if *s_0 as ::core::ffi::c_int
                                                == '<' as ::core::ffi::c_int
                                            {
                                                s_0 = s_0.offset(1);
                                            }
                                            if ascii_isdigit(*s_0 as ::core::ffi::c_int) {
                                                p0 = *s_0 as uint8_t as ::core::ffi::c_int
                                                    - '0' as ::core::ffi::c_int;
                                                s_0 = s_0.offset(1);
                                            }
                                            if *s_0 as ::core::ffi::c_int == NUL
                                                || *s_0 as ::core::ffi::c_int
                                                    == '$' as ::core::ffi::c_int
                                                    && !spell_iswordp_w(
                                                        (&raw mut word as *mut ::core::ffi::c_int)
                                                            .offset(i as isize)
                                                            .offset(k0 as isize),
                                                        curwin.get(),
                                                    )
                                            {
                                                if k0 != k {
                                                    if p0 >= pri {
                                                        break 's_446;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    n0 += 1;
                                }
                                if p0 >= pri
                                    && *(*smp.offset(n0 as isize))
                                        .sm_lead_w
                                        .offset(0 as ::core::ffi::c_int as isize)
                                        & 0xff as ::core::ffi::c_int
                                        == c0 & 0xff as ::core::ffi::c_int
                                {
                                    break 's_104;
                                }
                            }
                            ws = (*smp.offset(n as isize)).sm_to_w;
                            s_0 = (*smp.offset(n as isize)).sm_rules;
                            p0 = if !vim_strchr(s_0, '<' as ::core::ffi::c_int).is_null() {
                                1 as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            };
                            if p0 == 1 as ::core::ffi::c_int && z == 0 as ::core::ffi::c_int {
                                if reslen > 0 as ::core::ffi::c_int
                                    && !ws.is_null()
                                    && *ws != NUL
                                    && (wres[(reslen - 1 as ::core::ffi::c_int) as usize] == c_0
                                        || wres[(reslen - 1 as ::core::ffi::c_int) as usize] == *ws)
                                {
                                    reslen -= 1;
                                }
                                z0 = 1 as ::core::ffi::c_int;
                                z = 1 as ::core::ffi::c_int;
                                k0 = 0 as ::core::ffi::c_int;
                                if !ws.is_null() {
                                    while *ws != NUL && word[(i + k0) as usize] != NUL {
                                        word[(i + k0) as usize] = *ws;
                                        k0 += 1;
                                        ws = ws.offset(1);
                                    }
                                }
                                if k > k0 {
                                    memmove(
                                        (&raw mut word as *mut ::core::ffi::c_int)
                                            .offset(i as isize)
                                            .offset(k0 as isize)
                                            as *mut ::core::ffi::c_void,
                                        (&raw mut word as *mut ::core::ffi::c_int)
                                            .offset(i as isize)
                                            .offset(k as isize)
                                            as *const ::core::ffi::c_void,
                                        ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(
                                            (wordlen - (i + k) + 1 as ::core::ffi::c_int) as size_t,
                                        ),
                                    );
                                }
                                c_0 = word[i as usize];
                            } else {
                                i += k - 1 as ::core::ffi::c_int;
                                z = 0 as ::core::ffi::c_int;
                                if !ws.is_null() {
                                    while *ws != NUL
                                        && *ws.offset(1 as ::core::ffi::c_int as isize) != NUL
                                        && reslen < MAXWLEN as ::core::ffi::c_int
                                    {
                                        if reslen == 0 as ::core::ffi::c_int
                                            || wres[(reslen - 1 as ::core::ffi::c_int) as usize]
                                                != *ws
                                        {
                                            let c2rust_fresh9 = reslen;
                                            reslen = reslen + 1;
                                            wres[c2rust_fresh9 as usize] = *ws;
                                        }
                                        ws = ws.offset(1);
                                    }
                                }
                                if ws.is_null() {
                                    c_0 = NUL;
                                } else {
                                    c_0 = *ws;
                                }
                                if !strstr(s_0, b"^^\0".as_ptr() as *const ::core::ffi::c_char)
                                    .is_null()
                                {
                                    if c_0 != NUL && reslen < MAXWLEN as ::core::ffi::c_int {
                                        let c2rust_fresh10 = reslen;
                                        reslen = reslen + 1;
                                        wres[c2rust_fresh10 as usize] = c_0;
                                    }
                                    memmove(
                                        &raw mut word as *mut ::core::ffi::c_int
                                            as *mut ::core::ffi::c_void,
                                        (&raw mut word as *mut ::core::ffi::c_int)
                                            .offset(i as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(
                                            (wordlen - (i + 1 as ::core::ffi::c_int)
                                                + 1 as ::core::ffi::c_int)
                                                as size_t,
                                        ),
                                    );
                                    i = 0 as ::core::ffi::c_int;
                                    z0 = 1 as ::core::ffi::c_int;
                                }
                            }
                            break 's_643;
                        }
                    }
                }
                n += 1;
            }
        } else if ascii_iswhite(c_0) {
            c_0 = ' ' as ::core::ffi::c_int;
            k = 1 as ::core::ffi::c_int;
        }
        if z0 == 0 as ::core::ffi::c_int {
            if k != 0
                && p0 == 0
                && reslen < MAXWLEN as ::core::ffi::c_int
                && c_0 != NUL
                && (!(*slang).sl_collapse
                    || reslen == 0 as ::core::ffi::c_int
                    || wres[(reslen - 1 as ::core::ffi::c_int) as usize] != c_0)
            {
                let c2rust_fresh11 = reslen;
                reslen = reslen + 1;
                wres[c2rust_fresh11 as usize] = c_0;
            }
            i += 1;
            z = 0 as ::core::ffi::c_int;
            k = 0 as ::core::ffi::c_int;
        }
    }
    let mut l: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while n_0 < reslen {
        l += utf_char2bytes(wres[n_0 as usize], res.offset(l as isize));
        if l + MB_MAXBYTES as ::core::ffi::c_int > MAXWLEN as ::core::ffi::c_int {
            break;
        }
        n_0 += 1;
    }
    *res.offset(l as isize) = NUL as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn ex_spellinfo(mut _eap: *mut exarg_T) {
    if no_spell_checking(curwin.get()) {
        return;
    }
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    msg_start();
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len && !got_int.get() {
        let lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        msg_puts(b"file: \0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts((*(*lp).lp_slang).sl_fname);
        let p: *const ::core::ffi::c_char = (*(*lp).lp_slang).sl_info;
        if lpi < (*(*curwin.get()).w_s).b_langp.ga_len || !p.is_null() {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        if !p.is_null() {
            msg_puts(p);
            if lpi < (*(*curwin.get()).w_s).b_langp.ga_len - 1 as ::core::ffi::c_int {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
        }
        lpi += 1;
    }
    msg_end();
}
pub const DUMPFLAG_KEEPCASE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DUMPFLAG_COUNT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DUMPFLAG_ICASE: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const DUMPFLAG_ONECAP: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const DUMPFLAG_ALLCAP: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn ex_spelldump(mut eap: *mut exarg_T) {
    if no_spell_checking(curwin.get()) {
        return;
    }
    let mut spl: OptVal = get_option_value(kOptSpelllang, OPT_LOCAL as ::core::ffi::c_int);
    do_cmdline_cmd(b"new\0".as_ptr() as *const ::core::ffi::c_char);
    set_option_value_give_err(
        kOptSpell,
        OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData { boolean: kTrue },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    set_option_value_give_err(kOptSpelllang, spl, OPT_LOCAL as ::core::ffi::c_int);
    optval_free(spl);
    if !buf_is_empty(curbuf.get()) {
        return;
    }
    spell_dump_compl(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<Direction>(),
        if (*eap).forceit != 0 {
            DUMPFLAG_COUNT
        } else {
            0 as ::core::ffi::c_int
        },
    );
    if (*curbuf.get()).b_ml.ml_line_count > 1 as linenr_T {
        ml_delete((*curbuf.get()).b_ml.ml_line_count);
    }
    redraw_later(curwin.get(), UPD_NOT_VALID as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn spell_dump_compl(
    mut pat: *mut ::core::ffi::c_char,
    mut ic: ::core::ffi::c_int,
    mut dir: *mut Direction,
    mut dumpflags_arg: ::core::ffi::c_int,
) {
    let mut arridx: [idx_T; 254] = [0; 254];
    let mut curi: [::core::ffi::c_int; 254] = [0; 254];
    let mut word: [::core::ffi::c_char; 254] = [0; 254];
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut region_names: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut do_region: bool = true_0 != 0;
    let mut dumpflags: ::core::ffi::c_int = dumpflags_arg;
    if !pat.is_null() {
        if ic != 0 {
            dumpflags |= DUMPFLAG_ICASE;
        } else {
            let mut n: ::core::ffi::c_int =
                captype(pat, ::core::ptr::null::<::core::ffi::c_char>());
            if n == WF_ONECAP as ::core::ffi::c_int {
                dumpflags |= DUMPFLAG_ONECAP;
            } else if n == WF_ALLCAP as ::core::ffi::c_int
                && strlen(pat) as ::core::ffi::c_int > utfc_ptr2len(pat)
            {
                dumpflags |= DUMPFLAG_ALLCAP;
            }
        }
    }
    let mut lpi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi as isize);
        let mut p: *mut ::core::ffi::c_char =
            &raw mut (*(*lp).lp_slang).sl_regions as *mut ::core::ffi::c_char;
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if region_names.is_null() {
                region_names = p;
            } else if strcmp(region_names, p) != 0 as ::core::ffi::c_int {
                do_region = false_0 != 0;
                break;
            }
        }
        lpi += 1;
    }
    if do_region as ::core::ffi::c_int != 0 && !region_names.is_null() && pat.is_null() {
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"/regions=%s\0".as_ptr() as *const ::core::ffi::c_char,
            region_names,
        );
        let c2rust_fresh12 = lnum;
        lnum = lnum + 1;
        ml_append(
            c2rust_fresh12,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            0 as colnr_T,
            false_0 != 0,
        );
    } else {
        do_region = false_0 != 0;
    }
    let mut lpi_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lpi_0 < (*(*curwin.get()).w_s).b_langp.ga_len {
        let mut lp_0: *mut langp_T =
            ((*(*curwin.get()).w_s).b_langp.ga_data as *mut langp_T).offset(lpi_0 as isize);
        let mut slang: *mut slang_T = (*lp_0).lp_slang;
        if !(*slang).sl_fbyts.is_null() {
            if pat.is_null() {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"# file: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    (*slang).sl_fname,
                );
                let c2rust_fresh13 = lnum;
                lnum = lnum + 1;
                ml_append(
                    c2rust_fresh13,
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    0 as colnr_T,
                    false_0 != 0,
                );
            }
            let mut patlen: ::core::ffi::c_int = 0;
            if !pat.is_null() && (*slang).sl_pbyts.is_null() {
                patlen = strlen(pat) as ::core::ffi::c_int;
            } else {
                patlen = -1 as ::core::ffi::c_int;
            }
            let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while round <= 2 as ::core::ffi::c_int {
                let mut byts: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
                let mut idxs: *mut idx_T = ::core::ptr::null_mut::<idx_T>();
                if round == 1 as ::core::ffi::c_int {
                    dumpflags &= !DUMPFLAG_KEEPCASE;
                    byts = (*slang).sl_fbyts;
                    idxs = (*slang).sl_fidxs;
                } else {
                    dumpflags |= DUMPFLAG_KEEPCASE;
                    byts = (*slang).sl_kbyts;
                    idxs = (*slang).sl_kidxs;
                }
                if !byts.is_null() {
                    let mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    arridx[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int as idx_T;
                    curi[0 as ::core::ffi::c_int as usize] = 1 as ::core::ffi::c_int;
                    while depth >= 0 as ::core::ffi::c_int
                        && !got_int.get()
                        && (pat.is_null() || !ins_compl_interrupted())
                    {
                        if curi[depth as usize]
                            > *byts.offset(arridx[depth as usize] as isize) as ::core::ffi::c_int
                        {
                            depth -= 1;
                            line_breakcheck();
                            ins_compl_check_keys(50 as ::core::ffi::c_int, false_0 != 0);
                        } else {
                            let mut n_0: ::core::ffi::c_int =
                                arridx[depth as usize] as ::core::ffi::c_int + curi[depth as usize];
                            curi[depth as usize] += 1;
                            let mut c: ::core::ffi::c_int =
                                *byts.offset(n_0 as isize) as ::core::ffi::c_int;
                            if c == 0 as ::core::ffi::c_int
                                || depth >= MAXWLEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                            {
                                let mut flags: ::core::ffi::c_int = *idxs.offset(n_0 as isize);
                                if (round == 2 as ::core::ffi::c_int
                                    || flags & WF_KEEPCAP as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int)
                                    && flags & WF_NEEDCOMP as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    && (do_region as ::core::ffi::c_int != 0
                                        || flags & WF_REGION as ::core::ffi::c_int
                                            == 0 as ::core::ffi::c_int
                                        || flags as ::core::ffi::c_uint >> 16 as ::core::ffi::c_int
                                            & (*lp_0).lp_region as ::core::ffi::c_uint
                                            != 0 as ::core::ffi::c_uint)
                                {
                                    word[depth as usize] = NUL as ::core::ffi::c_char;
                                    if !do_region {
                                        flags &= !(WF_REGION as ::core::ffi::c_int);
                                    }
                                    c = (flags as ::core::ffi::c_uint >> 24 as ::core::ffi::c_int)
                                        as ::core::ffi::c_int;
                                    if c == 0 as ::core::ffi::c_int
                                        || curi[depth as usize] == 2 as ::core::ffi::c_int
                                    {
                                        dump_word(
                                            slang,
                                            &raw mut word as *mut ::core::ffi::c_char,
                                            pat,
                                            dir,
                                            dumpflags,
                                            flags,
                                            lnum,
                                        );
                                        if pat.is_null() {
                                            lnum += 1;
                                        }
                                    }
                                    if c != 0 as ::core::ffi::c_int {
                                        lnum = dump_prefixes(
                                            slang,
                                            &raw mut word as *mut ::core::ffi::c_char,
                                            pat,
                                            dir,
                                            dumpflags,
                                            flags,
                                            lnum,
                                        );
                                    }
                                }
                            } else {
                                let c2rust_fresh14 = depth;
                                depth = depth + 1;
                                word[c2rust_fresh14 as usize] = c as ::core::ffi::c_char;
                                arridx[depth as usize] = *idxs.offset(n_0 as isize);
                                curi[depth as usize] = 1 as ::core::ffi::c_int;
                                '_c2rust_label: {
                                    if depth >= 0 as ::core::ffi::c_int {
                                    } else {
                                        __assert_fail(
                                            b"depth >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                            b"src/nvim/spell.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            3396 as ::core::ffi::c_uint,
                                            b"void spell_dump_compl(char *, int, Direction *, int)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                                if depth <= patlen
                                    && mb_strnicmp(
                                        &raw mut word as *mut ::core::ffi::c_char,
                                        pat,
                                        depth as size_t,
                                    ) != 0 as ::core::ffi::c_int
                                {
                                    depth -= 1;
                                }
                            }
                        }
                    }
                }
                round += 1;
            }
        }
        lpi_0 += 1;
    }
}
unsafe extern "C" fn dump_word(
    mut slang: *mut slang_T,
    mut word: *mut ::core::ffi::c_char,
    mut pat: *mut ::core::ffi::c_char,
    mut dir: *mut Direction,
    mut dumpflags: ::core::ffi::c_int,
    mut wordflags: ::core::ffi::c_int,
    mut lnum: linenr_T,
) {
    let mut keepcap: bool = false_0 != 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cword: [::core::ffi::c_char; 254] = [0; 254];
    let mut badword: [::core::ffi::c_char; 264] = [0; 264];
    let mut flags: ::core::ffi::c_int = wordflags;
    if dumpflags & DUMPFLAG_ONECAP != 0 {
        flags |= WF_ONECAP as ::core::ffi::c_int;
    }
    if dumpflags & DUMPFLAG_ALLCAP != 0 {
        flags |= WF_ALLCAP as ::core::ffi::c_int;
    }
    if dumpflags & DUMPFLAG_KEEPCASE == 0 as ::core::ffi::c_int
        && flags & WF_CAPMASK as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        make_case_word(word, &raw mut cword as *mut ::core::ffi::c_char, flags);
        p = &raw mut cword as *mut ::core::ffi::c_char;
    } else {
        p = word;
        if dumpflags & DUMPFLAG_KEEPCASE != 0
            && (captype(word, ::core::ptr::null::<::core::ffi::c_char>())
                & WF_KEEPCAP as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
                || flags & WF_FIXCAP as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
        {
            keepcap = true_0 != 0;
        }
    }
    let mut tw: *mut ::core::ffi::c_char = p;
    if pat.is_null() {
        if flags
            & (WF_BANNED as ::core::ffi::c_int
                | WF_RARE as ::core::ffi::c_int
                | WF_REGION as ::core::ffi::c_int)
            != 0
            || keepcap as ::core::ffi::c_int != 0
        {
            strcpy(&raw mut badword as *mut ::core::ffi::c_char, p);
            strcat(
                &raw mut badword as *mut ::core::ffi::c_char,
                b"/\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if keepcap {
                strcat(
                    &raw mut badword as *mut ::core::ffi::c_char,
                    b"=\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            if flags & WF_BANNED as ::core::ffi::c_int != 0 {
                strcat(
                    &raw mut badword as *mut ::core::ffi::c_char,
                    b"!\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else if flags & WF_RARE as ::core::ffi::c_int != 0 {
                strcat(
                    &raw mut badword as *mut ::core::ffi::c_char,
                    b"?\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            if flags & WF_REGION as ::core::ffi::c_int != 0 {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < 7 as ::core::ffi::c_int {
                    if flags & (0x10000 as ::core::ffi::c_int) << i != 0 {
                        let badword_len: size_t =
                            strlen(&raw mut badword as *mut ::core::ffi::c_char);
                        snprintf(
                            (&raw mut badword as *mut ::core::ffi::c_char)
                                .offset(badword_len as isize),
                            ::core::mem::size_of::<[::core::ffi::c_char; 264]>()
                                .wrapping_sub(badword_len),
                            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                            i + 1 as ::core::ffi::c_int,
                        );
                    }
                    i += 1;
                }
            }
            p = &raw mut badword as *mut ::core::ffi::c_char;
        }
        if dumpflags & DUMPFLAG_COUNT != 0 {
            let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
            hi = hash_find(&raw mut (*slang).sl_wordcount, tw);
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"%s\t%d\0".as_ptr() as *const ::core::ffi::c_char,
                    tw,
                    (*((*hi).hi_key.offset(-(WC_KEY_OFF as isize)) as *mut wordcount_T)).wc_count
                        as ::core::ffi::c_int,
                );
                p = IObuff.ptr() as *mut ::core::ffi::c_char;
            }
        }
        ml_append(lnum, p, 0 as colnr_T, false_0 != 0);
    } else if (if dumpflags & DUMPFLAG_ICASE != 0 {
        (mb_strnicmp(p, pat, strlen(pat)) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
    } else {
        (strncmp(p, pat, strlen(pat)) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
    }) != 0
        && ins_compl_add_infercase(
            p,
            strlen(p) as ::core::ffi::c_int,
            p_ic.get() != 0,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            *dir,
            false_0 != 0,
            0 as ::core::ffi::c_int,
        ) == OK
    {
        *dir = FORWARD;
    }
}
unsafe extern "C" fn dump_prefixes(
    mut slang: *mut slang_T,
    mut word: *mut ::core::ffi::c_char,
    mut pat: *mut ::core::ffi::c_char,
    mut dir: *mut Direction,
    mut dumpflags: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut startlnum: linenr_T,
) -> linenr_T {
    let mut arridx: [idx_T; 254] = [0; 254];
    let mut curi: [::core::ffi::c_int; 254] = [0; 254];
    let mut prefix: [::core::ffi::c_char; 254] = [0; 254];
    let mut word_up: [::core::ffi::c_char; 254] = [0; 254];
    let mut has_word_up: bool = false_0 != 0;
    let mut lnum: linenr_T = startlnum;
    let mut c: ::core::ffi::c_int = utf_ptr2char(word);
    if (if c >= 128 as ::core::ffi::c_int {
        mb_toupper(c)
    } else {
        (*spelltab.ptr()).st_upper[c as usize] as ::core::ffi::c_int
    }) != c
    {
        onecap_copy(
            word,
            &raw mut word_up as *mut ::core::ffi::c_char,
            true_0 != 0,
        );
        has_word_up = true_0 != 0;
    }
    let mut byts: *mut uint8_t = (*slang).sl_pbyts;
    let mut idxs: *mut idx_T = (*slang).sl_pidxs;
    if !byts.is_null() {
        let mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        arridx[0 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_int as idx_T;
        curi[0 as ::core::ffi::c_int as usize] = 1 as ::core::ffi::c_int;
        while depth >= 0 as ::core::ffi::c_int && !got_int.get() {
            let mut n: ::core::ffi::c_int = arridx[depth as usize] as ::core::ffi::c_int;
            let mut len: ::core::ffi::c_int = *byts.offset(n as isize) as ::core::ffi::c_int;
            if curi[depth as usize] > len {
                depth -= 1;
                line_breakcheck();
            } else {
                n += curi[depth as usize];
                curi[depth as usize] += 1;
                c = *byts.offset(n as isize) as ::core::ffi::c_int;
                if c == 0 as ::core::ffi::c_int {
                    let mut i: ::core::ffi::c_int = 0;
                    i = 1 as ::core::ffi::c_int;
                    while i < len {
                        if *byts.offset((n + i) as isize) as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                        {
                            break;
                        }
                        i += 1;
                    }
                    curi[depth as usize] += i - 1 as ::core::ffi::c_int;
                    c = valid_word_prefix(i, n, flags, word, slang, false_0 != 0);
                    if c != 0 as ::core::ffi::c_int {
                        xstrlcpy(
                            (&raw mut prefix as *mut ::core::ffi::c_char).offset(depth as isize),
                            word,
                            (MAXWLEN as ::core::ffi::c_int - depth) as size_t,
                        );
                        dump_word(
                            slang,
                            &raw mut prefix as *mut ::core::ffi::c_char,
                            pat,
                            dir,
                            dumpflags,
                            if c & WF_RAREPFX as ::core::ffi::c_int != 0 {
                                flags | WF_RARE as ::core::ffi::c_int
                            } else {
                                flags
                            },
                            lnum,
                        );
                        if lnum != 0 as linenr_T {
                            lnum += 1;
                        }
                    }
                    if has_word_up {
                        c = valid_word_prefix(
                            i,
                            n,
                            flags,
                            &raw mut word_up as *mut ::core::ffi::c_char,
                            slang,
                            true_0 != 0,
                        );
                        if c != 0 as ::core::ffi::c_int {
                            xstrlcpy(
                                (&raw mut prefix as *mut ::core::ffi::c_char)
                                    .offset(depth as isize),
                                &raw mut word_up as *mut ::core::ffi::c_char,
                                (MAXWLEN as ::core::ffi::c_int - depth) as size_t,
                            );
                            dump_word(
                                slang,
                                &raw mut prefix as *mut ::core::ffi::c_char,
                                pat,
                                dir,
                                dumpflags,
                                if c & WF_RAREPFX as ::core::ffi::c_int != 0 {
                                    flags | WF_RARE as ::core::ffi::c_int
                                } else {
                                    flags
                                },
                                lnum,
                            );
                            if lnum != 0 as linenr_T {
                                lnum += 1;
                            }
                        }
                    }
                } else {
                    let c2rust_fresh15 = depth;
                    depth = depth + 1;
                    prefix[c2rust_fresh15 as usize] = c as ::core::ffi::c_char;
                    arridx[depth as usize] = *idxs.offset(n as isize);
                    curi[depth as usize] = 1 as ::core::ffi::c_int;
                }
            }
        }
    }
    return lnum;
}
#[no_mangle]
pub unsafe extern "C" fn spell_to_word_end(
    mut start: *mut ::core::ffi::c_char,
    mut win: *mut win_T,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = start;
    while *p as ::core::ffi::c_int != NUL && spell_iswordp(p, win) as ::core::ffi::c_int != 0 {
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn spell_word_start(mut startcol: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if no_spell_checking(curwin.get()) {
        return startcol;
    }
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    p = line.offset(startcol as isize);
    while p > line {
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
        if spell_iswordp_nmw(p, curwin.get()) {
            break;
        }
    }
    let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while p > line {
        col = p.offset_from(line) as ::core::ffi::c_int;
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
        if !spell_iswordp(p, curwin.get()) {
            break;
        }
        col = 0 as ::core::ffi::c_int;
    }
    return col;
}
static spell_expand_need_cap: GlobalCell<bool> = GlobalCell::new(false);
#[no_mangle]
pub unsafe extern "C" fn spell_expand_check_cap(mut col: colnr_T) {
    spell_expand_need_cap.set(check_need_cap(
        curwin.get(),
        (*curwin.get()).w_cursor.lnum,
        col,
    ));
}
#[no_mangle]
pub unsafe extern "C" fn expand_spelling(
    mut _lnum: linenr_T,
    mut pat: *mut ::core::ffi::c_char,
    mut matchp: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    spell_suggest_list(
        &raw mut ga,
        pat,
        100 as ::core::ffi::c_int,
        spell_expand_need_cap.get(),
        true_0 != 0,
    );
    *matchp = ga.ga_data as *mut *mut ::core::ffi::c_char;
    return ga.ga_len;
}
#[no_mangle]
pub unsafe extern "C" fn valid_spelllang(mut val: *const ::core::ffi::c_char) -> bool {
    return valid_name(val, b".-_,@\0".as_ptr() as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn valid_spellfile(mut val: *const ::core::ffi::c_char) -> bool {
    let mut spf_name: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut spf: *mut ::core::ffi::c_char = val as *mut ::core::ffi::c_char;
    while *spf as ::core::ffi::c_int != NUL {
        let mut l: size_t = copy_option_part(
            &raw mut spf,
            &raw mut spf_name as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        if l >= (MAXPATHL - 4 as ::core::ffi::c_int) as size_t
            || l < 4 as size_t
            || strcmp(
                (&raw mut spf_name as *mut ::core::ffi::c_char)
                    .offset(l as isize)
                    .offset(-(4 as ::core::ffi::c_int as isize)),
                b".add\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0 as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        let mut s: *mut ::core::ffi::c_char = &raw mut spf_name as *mut ::core::ffi::c_char;
        while *s as ::core::ffi::c_int != NUL {
            if !vim_is_fname_char(*s as uint8_t as ::core::ffi::c_int) {
                return false_0 != 0;
            }
            s = s.offset(1);
        }
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn did_set_spell_option() -> *const ::core::ffi::c_char {
    let mut errmsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).w_buffer == curbuf.get() && (*wp).w_onebuf_opt.wo_spell != 0 {
            errmsg = parse_spelllang(wp);
            break;
        } else {
            wp = (*wp).w_next;
        }
    }
    return errmsg;
}
#[no_mangle]
pub unsafe extern "C" fn compile_cap_prog(
    mut synblock: *mut synblock_T,
) -> *const ::core::ffi::c_char {
    let mut rp: *mut regprog_T = (*synblock).b_cap_prog;
    if (*synblock).b_p_spc.is_null() || *(*synblock).b_p_spc as ::core::ffi::c_int == NUL {
        (*synblock).b_cap_prog = ::core::ptr::null_mut::<regprog_T>();
    } else {
        let mut re: *mut ::core::ffi::c_char = concat_str(
            b"^\0".as_ptr() as *const ::core::ffi::c_char,
            (*synblock).b_p_spc,
        );
        (*synblock).b_cap_prog = vim_regcomp(re, RE_MAGIC);
        xfree(re as *mut ::core::ffi::c_void);
        if (*synblock).b_cap_prog.is_null() {
            (*synblock).b_cap_prog = rp;
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
    }
    vim_regfree(rp);
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub const SPL_FNAME_TMPL: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"%s.%s.spl\0") };
pub const WC_KEY_OFF: ::core::ffi::c_ulong = 2 as ::core::ffi::c_ulong;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
