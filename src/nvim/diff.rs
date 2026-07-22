use crate::src::nvim::autocmd::{
    apply_autocmds, aucmd_prepbuf, aucmd_restbuf, augroup_exists, block_autocmds, unblock_autocmds,
};
use crate::src::nvim::buffer::{
    bt_prompt, buf_is_empty, buf_valid, buflist_findnr, buflist_findpat, bufref_valid, set_bufref,
};
use crate::src::nvim::bufwrite::buf_write;
use crate::src::nvim::change::{change_warning, changed_lines};
use crate::src::nvim::charset::{getdigits_int, getdigits_int32, skipwhite};
use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::decoration::decor_conceal_line;
use crate::src::nvim::drawscreen::redraw_later;
use crate::src::nvim::eval::typval::{tv_get_lnum, tv_get_number};
use crate::src::nvim::eval::vars::{eval_diff, eval_patch};
use crate::src::nvim::ex_cmds::{append_redir, ex_file};
use crate::src::nvim::ex_docmd::{do_cmdline_cmd, do_exedit, get_address};
use crate::src::nvim::extmark::extmark_adjust;
use crate::src::nvim::fileio::{
    buf_check_timestamp, shorten_fnames, vim_fgets, vim_gettempdir, vim_tempname,
};
use crate::src::nvim::fold::{
    foldUpdate, foldUpdateAll, foldmethodIsDiff, foldmethodIsManual, hasFolding, newFoldLevel,
};
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat_len, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::linematch::linematch_nbuffers;
use crate::src::nvim::main::{
    cmdmod, curbuf, curtab, curwin, diff_context, diff_foldcolumn, diff_need_scrollbind,
    e_cannot_have_more_than_nr_diff_anchors, e_diff_anchors_with_hidden_windows,
    e_failed_to_find_all_diff_anchors, e_invrange, e_prev_dir, e_problem_creating_internal_diff,
    first_tabpage, firstwin, need_diff_redraw, p_dex, p_dia, p_dip, p_pex, p_sbo, p_srr, KeyTyped,
};
use crate::src::nvim::mark::{mark_adjust, setpcmark};
use crate::src::nvim::mbyte::{
    mb_get_class_tab, mb_stricmp, utf_char2bytes, utf_char2len, utf_fold, utf_head_off,
    utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memline::{ml_append, ml_delete, ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{memchrsub, xcalloc, xfree, xmalloc, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::normal::check_scrollbind;
use crate::src::nvim::option::{set_option_direct_for, set_option_value_give_err};
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::os::env::{os_env_exists, os_unsetenv};
use crate::src::nvim::os::fs::{
    os_chdir, os_dirname, os_fileinfo, os_fileinfo_size, os_fopen, os_remove,
};
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, atol, fclose, fwrite, gettext, memcpy, memmove, memset, qsort,
    snprintf, strcat, strcmp, strcpy, strlen, strncmp, tolower,
};
use crate::src::nvim::os::shell::call_shell;
use crate::src::nvim::path::FullName_save;
use crate::src::nvim::r#move::{
    changed_line_abv_curs, changed_line_abv_curs_win, changed_window_setting, check_topfill,
    invalidate_botline_win, validate_cursor,
};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, vim_strsave_shellescape, xstrnsave};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CMD_index, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, EvalFuncData, ExtmarkMove, ExtmarkOp, ExtmarkSavePos,
    ExtmarkSplice, ExtmarkUndoObject, FileID, FileInfo, FloatAnchor, FloatRelative, GridView,
    Intersection, LineGetter, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t,
    Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    MsgpackRpcRequestHandler, OptIndex, OptInt, OptScope, OptVal, OptValData, OptValType,
    ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_13, String_0,
    Terminal, Timestamp, TriState, UndoObjectType, VarLockStatus, VarType, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, _IO_codecvt,
    _IO_lock_t, _IO_marker, _IO_wide_data, __compar_fn_t, __off64_t, __off_t, __time_t, aco_save_T,
    alist_T, auto_event, bcount_t, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufref_T,
    bufstate_T, chunksize_T, cmd_addr_T, cmdidx_T, cmdmod_T, colnr_T, cstack_T,
    cstack_T_cs_pend as C2Rust_Unnamed_17, dict_T, dictvar_S, diff_T, diffblock_S, diffline_S,
    diffline_T, diffline_change_S, diffline_change_T, disptick_T, eslist_T, eslist_elem, event_T,
    exarg, exarg_T, extmark_undo_vec_t, fcs_chars_T, file_buffer,
    file_buffer_b_signcols as C2Rust_Unnamed_3, file_buffer_b_wininfo as C2Rust_Unnamed_12,
    file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, find_func_t, float_T, fmark_T, fmarkv_T,
    frame_S, frame_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T,
    handle_T, hash_T, hashitem_T, hashtab_T, hlf_T, infoptr_T, int16_t, int32_t, int64_t,
    lcs_chars_T, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    llpos_T, lpos_T, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mmbuffer_t, mmfile_t, mtnode_inner_s, mtnode_s, partial_S, partial_T, pos_T,
    pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue, reg_extmatch_T,
    regmatch_T, regmmatch_T, regprog, regprog_T, s_mmbuffer, s_mmfile, s_xdemitcb, s_xdemitconf,
    s_xpparam, sattr_T, schar_T, scid_T, sctx_T, size_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, tabpage_S,
    tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_7, uv_stat_t, uv_timespec_t, varnumber_T,
    virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T, wline_T, xdemitcb_t,
    xdemitconf_t, xdl_emit_hunk_consume_func_t, xfmark_T, xpparam_t, FILE, QUEUE, _IO_FILE,
};
use crate::src::nvim::ui::vim_beep;
use crate::src::nvim::undo::{u_save, u_sync};
use crate::src::nvim::window::{
    frames_locked, scroll_to_fraction, set_fraction, win_split, win_valid,
};
use crate::src::xdiff::xdiffi::xdl_diff;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_16 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_16 = 3;
pub const BACKWARD: C2Rust_Unnamed_16 = -1;
pub const FORWARD: C2Rust_Unnamed_16 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_16 = 0;
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
pub const kOptScopeBuf: OptScope = 2;
pub const kOptScopeWin: OptScope = 1;
pub const kOptScopeGlobal: OptScope = 0;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
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
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kOptBoFlagWildmode: C2Rust_Unnamed_19 = 524288;
pub const kOptBoFlagTerm: C2Rust_Unnamed_19 = 262144;
pub const kOptBoFlagSpell: C2Rust_Unnamed_19 = 131072;
pub const kOptBoFlagShell: C2Rust_Unnamed_19 = 65536;
pub const kOptBoFlagRegister: C2Rust_Unnamed_19 = 32768;
pub const kOptBoFlagOperator: C2Rust_Unnamed_19 = 16384;
pub const kOptBoFlagShowmatch: C2Rust_Unnamed_19 = 8192;
pub const kOptBoFlagMess: C2Rust_Unnamed_19 = 4096;
pub const kOptBoFlagLang: C2Rust_Unnamed_19 = 2048;
pub const kOptBoFlagInsertmode: C2Rust_Unnamed_19 = 1024;
pub const kOptBoFlagHangul: C2Rust_Unnamed_19 = 512;
pub const kOptBoFlagEx: C2Rust_Unnamed_19 = 256;
pub const kOptBoFlagEsc: C2Rust_Unnamed_19 = 128;
pub const kOptBoFlagError: C2Rust_Unnamed_19 = 64;
pub const kOptBoFlagCtrlg: C2Rust_Unnamed_19 = 32;
pub const kOptBoFlagCopy: C2Rust_Unnamed_19 = 16;
pub const kOptBoFlagComplete: C2Rust_Unnamed_19 = 8;
pub const kOptBoFlagCursor: C2Rust_Unnamed_19 = 4;
pub const kOptBoFlagBackspace: C2Rust_Unnamed_19 = 2;
pub const kOptBoFlagAll: C2Rust_Unnamed_19 = 1;
pub const UPD_VALID: C2Rust_Unnamed_20 = 10;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffio_T {
    pub dio_orig: diffin_T,
    pub dio_new: diffin_T,
    pub dio_diff: diffout_T,
    pub dio_internal: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffout_T {
    pub dout_fname: *mut ::core::ffi::c_char,
    pub dout_ga: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffin_T {
    pub din_fname: *mut ::core::ffi::c_char,
    pub din_mmfile: mmfile_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffhunk_T {
    pub lnum_orig: linenr_T,
    pub count_orig: ::core::ffi::c_int,
    pub lnum_new: linenr_T,
    pub count_new: ::core::ffi::c_int,
}
pub type diffstyle_T = ::core::ffi::c_uint;
pub const DIFF_NONE: diffstyle_T = 2;
pub const DIFF_UNIFIED: diffstyle_T = 1;
pub const DIFF_ED: diffstyle_T = 0;
pub const kShellOptDoOut: C2Rust_Unnamed_22 = 4;
pub const kShellOptSilent: C2Rust_Unnamed_22 = 8;
pub const kShellOptFilter: C2Rust_Unnamed_22 = 1;
pub const MAX_DIFF_ANCHORS: C2Rust_Unnamed_24 = 20;
pub const UPD_SOME_VALID: C2Rust_Unnamed_20 = 35;
pub const UPD_NOT_VALID: C2Rust_Unnamed_20 = 40;
pub const OPT_LOCAL: C2Rust_Unnamed_21 = 2;
pub const WSP_VERT: C2Rust_Unnamed_23 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct linemap_entry_T {
    pub byte_start: colnr_T,
    pub num_bytes: colnr_T,
    pub lineoff: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_20 = 50;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_20 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_20 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_20 = 20;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_21 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_21 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_21 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_21 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_21 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_21 = 4;
pub const OPT_GLOBAL: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const kShellOptHideMess: C2Rust_Unnamed_22 = 64;
pub const kShellOptWrite: C2Rust_Unnamed_22 = 32;
pub const kShellOptRead: C2Rust_Unnamed_22 = 16;
pub const kShellOptExpand: C2Rust_Unnamed_22 = 2;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_23 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_23 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_23 = 256;
pub const WSP_ABOVE: C2Rust_Unnamed_23 = 128;
pub const WSP_BELOW: C2Rust_Unnamed_23 = 64;
pub const WSP_HELP: C2Rust_Unnamed_23 = 32;
pub const WSP_BOT: C2Rust_Unnamed_23 = 16;
pub const WSP_TOP: C2Rust_Unnamed_23 = 8;
pub const WSP_HOR: C2Rust_Unnamed_23 = 4;
pub const WSP_ROOM: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const DB_COUNT: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
static diff_busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static diff_need_update: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const DIFF_FILLER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const DIFF_IBLANK: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const DIFF_ICASE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const DIFF_IWHITE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const DIFF_IWHITEALL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const DIFF_IWHITEEOL: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const DIFF_HORIZONTAL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const DIFF_VERTICAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const DIFF_HIDDEN_OFF: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const DIFF_INTERNAL: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const DIFF_CLOSE_OFF: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const DIFF_FOLLOWWRAP: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const DIFF_LINEMATCH: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DIFF_INLINE_NONE: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const DIFF_INLINE_SIMPLE: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const DIFF_INLINE_CHAR: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const DIFF_INLINE_WORD: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const DIFF_ANCHOR: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const ALL_WHITE_DIFF: ::core::ffi::c_int = DIFF_IWHITE | DIFF_IWHITEALL | DIFF_IWHITEEOL;
pub const ALL_INLINE: ::core::ffi::c_int =
    DIFF_INLINE_NONE | DIFF_INLINE_SIMPLE | DIFF_INLINE_CHAR | DIFF_INLINE_WORD;
pub const ALL_INLINE_DIFF: ::core::ffi::c_int = DIFF_INLINE_CHAR | DIFF_INLINE_WORD;
static diff_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(
    DIFF_INTERNAL | DIFF_FILLER | DIFF_CLOSE_OFF | DIFF_LINEMATCH | DIFF_INLINE_CHAR,
);
static diff_algorithm: GlobalCell<::core::ffi::c_int> = GlobalCell::new(XDF_INDENT_HEURISTIC);
static diff_word_gap: GlobalCell<::core::ffi::c_int> = GlobalCell::new(5 as ::core::ffi::c_int);
static linematch_lines: GlobalCell<::core::ffi::c_int> = GlobalCell::new(40 as ::core::ffi::c_int);
pub const LBUFLEN: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const MAX_XDIFF_SIZE: ::core::ffi::c_long =
    1024 as ::core::ffi::c_long * 1024 as ::core::ffi::c_long * 1023 as ::core::ffi::c_long;
static diff_a_works: GlobalCell<TriState> = GlobalCell::new(kNone);
unsafe extern "C" fn clear_diffblock(mut dp: *mut diff_T) {
    ga_clear(&raw mut (*dp).df_changes);
    xfree(dp as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn diff_buf_delete(mut buf: *mut buf_T) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut i: ::core::ffi::c_int = diff_buf_idx(buf, tp as *mut tabpage_T);
        if i != DB_COUNT {
            (*tp).tp_diffbuf[i as usize] = ::core::ptr::null_mut::<buf_T>();
            (*tp).tp_diff_invalid = true_0;
            if tp == curtab.get() {
                need_diff_redraw.set(true_0 != 0);
                redraw_later(curwin.get(), UPD_VALID as ::core::ffi::c_int);
            }
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
pub unsafe extern "C" fn diff_buf_adjust(mut win: *mut win_T) {
    if (*win).w_onebuf_opt.wo_diff == 0 {
        let mut found_win: bool = false_0 != 0;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == (*win).w_buffer && (*wp).w_onebuf_opt.wo_diff != 0 {
                found_win = true_0 != 0;
            }
            wp = (*wp).w_next;
        }
        if !found_win {
            let mut i: ::core::ffi::c_int = diff_buf_idx((*win).w_buffer, curtab.get());
            if i != DB_COUNT {
                (*curtab.get()).tp_diffbuf[i as usize] = ::core::ptr::null_mut::<buf_T>();
                (*curtab.get()).tp_diff_invalid = true_0;
                diff_redraw(true_0 != 0);
            }
        }
    } else {
        diff_buf_add((*win).w_buffer);
    };
}
pub unsafe extern "C" fn diff_buf_add(mut buf: *mut buf_T) {
    if diff_buf_idx(buf, curtab.get()) != DB_COUNT {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if (*curtab.get()).tp_diffbuf[i as usize].is_null() {
            (*curtab.get()).tp_diffbuf[i as usize] = buf as *mut buf_T;
            (*curtab.get()).tp_diff_invalid = true_0;
            diff_redraw(true_0 != 0);
            return;
        }
        i += 1;
    }
    semsg(
        gettext(b"E96: Cannot diff more than %d buffers\0".as_ptr() as *const ::core::ffi::c_char),
        DB_COUNT,
    );
}
unsafe extern "C" fn diff_buf_clear() {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
            (*curtab.get()).tp_diffbuf[i as usize] = ::core::ptr::null_mut::<buf_T>();
            (*curtab.get()).tp_diff_invalid = true_0;
            diff_redraw(true_0 != 0);
        }
        i += 1;
    }
}
unsafe extern "C" fn diff_buf_idx(
    mut buf: *mut buf_T,
    mut tp: *mut tabpage_T,
) -> ::core::ffi::c_int {
    let mut idx: ::core::ffi::c_int = 0;
    idx = 0 as ::core::ffi::c_int;
    while idx < DB_COUNT {
        if (*tp).tp_diffbuf[idx as usize] == buf {
            break;
        }
        idx += 1;
    }
    return idx;
}
pub unsafe extern "C" fn diff_invalidate(mut buf: *mut buf_T) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut i: ::core::ffi::c_int = diff_buf_idx(buf, tp as *mut tabpage_T);
        if i != DB_COUNT {
            (*tp).tp_diff_invalid = true_0;
            if tp == curtab.get() {
                diff_redraw(true_0 != 0);
            }
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
pub unsafe extern "C" fn diff_mark_adjust(
    mut buf: *mut buf_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut idx: ::core::ffi::c_int = diff_buf_idx(buf, tp as *mut tabpage_T);
        if idx != DB_COUNT {
            diff_mark_adjust_tp(
                tp as *mut tabpage_T,
                idx,
                line1,
                line2,
                amount,
                amount_after,
            );
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
unsafe extern "C" fn diff_mark_adjust_tp(
    mut tp: *mut tabpage_T,
    mut idx: ::core::ffi::c_int,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) {
    if diff_internal() != 0 {
        (*tp).tp_diff_invalid = true_0;
        (*tp).tp_diff_update = true_0;
    }
    let mut inserted: linenr_T = 0;
    let mut deleted: linenr_T = 0;
    if line2 == MAXLNUM as ::core::ffi::c_int as linenr_T {
        inserted = amount;
        deleted = 0 as ::core::ffi::c_int as linenr_T;
    } else if amount_after > 0 as linenr_T {
        inserted = amount_after;
        deleted = 0 as ::core::ffi::c_int as linenr_T;
    } else {
        inserted = 0 as ::core::ffi::c_int as linenr_T;
        deleted = -amount_after;
    }
    let mut dprev: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut dp: *mut diff_T = (*tp).tp_first_diff;
    let mut lnum_deleted: linenr_T = line1;
    loop {
        if (dp.is_null()
            || (*dp).df_lnum[idx as usize] - 1 as linenr_T > line2
            || line2 == MAXLNUM as ::core::ffi::c_int as linenr_T
                && (*dp).df_lnum[idx as usize] > line1)
            && (dprev.is_null()
                || (*dprev).df_lnum[idx as usize] + (*dprev).df_count[idx as usize] < line1)
            && !diff_busy.get()
        {
            let mut dnext: *mut diff_T = diff_alloc_new(tp, dprev, dp);
            (*dnext).df_lnum[idx as usize] = line1;
            (*dnext).df_count[idx as usize] = inserted;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < DB_COUNT {
                if !(*tp).tp_diffbuf[i as usize].is_null() && i != idx {
                    if dprev.is_null() {
                        (*dnext).df_lnum[i as usize] = line1;
                    } else {
                        (*dnext).df_lnum[i as usize] = line1
                            + ((*dprev).df_lnum[i as usize] + (*dprev).df_count[i as usize])
                            - ((*dprev).df_lnum[idx as usize] + (*dprev).df_count[idx as usize]);
                    }
                    (*dnext).df_count[i as usize] = deleted;
                }
                i += 1;
            }
        }
        if dp.is_null() {
            break;
        }
        let mut last: linenr_T =
            (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] - 1 as linenr_T;
        if last >= line1 - 1 as linenr_T {
            if diff_busy.get() {
                if (*dp).df_lnum[idx as usize] > line2 {
                    (*dp).df_lnum[idx as usize] += amount_after;
                }
                dprev = dp;
                dp = (*dp).df_next;
                continue;
            } else if (*dp).df_lnum[idx as usize]
                - (deleted + inserted != 0 as linenr_T) as ::core::ffi::c_int
                > line2
            {
                if amount_after == 0 as linenr_T {
                    break;
                }
                (*dp).df_lnum[idx as usize] += amount_after;
            } else {
                let mut check_unchanged: bool = false_0 != 0;
                if deleted > 0 as linenr_T {
                    let mut n: linenr_T = 0;
                    let mut off: linenr_T = 0 as linenr_T;
                    if (*dp).df_lnum[idx as usize] >= line1 {
                        if last <= line2 {
                            if !(*dp).df_next.is_null()
                                && (*(*dp).df_next).df_lnum[idx as usize] - 1 as linenr_T <= line2
                            {
                                n = (*(*dp).df_next).df_lnum[idx as usize] - lnum_deleted;
                                deleted -= n;
                                n -= (*dp).df_count[idx as usize];
                                lnum_deleted = (*(*dp).df_next).df_lnum[idx as usize];
                            } else {
                                n = deleted - (*dp).df_count[idx as usize];
                            }
                            (*dp).df_count[idx as usize] = 0 as ::core::ffi::c_int as linenr_T;
                        } else {
                            off = (*dp).df_lnum[idx as usize] - lnum_deleted;
                            n = off;
                            (*dp).df_count[idx as usize] = ((*dp).df_count[idx as usize]
                                as ::core::ffi::c_int
                                - (line2 - (*dp).df_lnum[idx as usize] + 1 as linenr_T)
                                    as ::core::ffi::c_int)
                                as linenr_T;
                            check_unchanged = true_0 != 0;
                        }
                        (*dp).df_lnum[idx as usize] = line1;
                    } else if last < line2 {
                        (*dp).df_count[idx as usize] = ((*dp).df_count[idx as usize]
                            as ::core::ffi::c_int
                            - (last - lnum_deleted + 1 as linenr_T) as ::core::ffi::c_int)
                            as linenr_T;
                        if !(*dp).df_next.is_null()
                            && (*(*dp).df_next).df_lnum[idx as usize] - 1 as linenr_T <= line2
                        {
                            n = (*(*dp).df_next).df_lnum[idx as usize] - 1 as linenr_T - last;
                            deleted -= (*(*dp).df_next).df_lnum[idx as usize] - lnum_deleted;
                            lnum_deleted = (*(*dp).df_next).df_lnum[idx as usize];
                        } else {
                            n = line2 - last;
                        }
                        check_unchanged = true_0 != 0;
                    } else {
                        n = 0 as ::core::ffi::c_int as linenr_T;
                        (*dp).df_count[idx as usize] -= deleted;
                    }
                    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i_0 < DB_COUNT {
                        if !(*tp).tp_diffbuf[i_0 as usize].is_null() && i_0 != idx {
                            if (*dp).df_lnum[i_0 as usize] > off {
                                (*dp).df_lnum[i_0 as usize] -= off;
                            } else {
                                (*dp).df_lnum[i_0 as usize] = 1 as ::core::ffi::c_int as linenr_T;
                            }
                            (*dp).df_count[i_0 as usize] += n;
                        }
                        i_0 += 1;
                    }
                } else if (*dp).df_lnum[idx as usize] <= line1 {
                    (*dp).df_count[idx as usize] += inserted;
                    check_unchanged = true_0 != 0;
                } else {
                    (*dp).df_lnum[idx as usize] += inserted;
                }
                if check_unchanged {
                    diff_check_unchanged(tp, dp);
                }
            }
        }
        if !dprev.is_null()
            && !(*dp).is_linematched
            && !diff_busy.get()
            && (*dprev).df_lnum[idx as usize] + (*dprev).df_count[idx as usize]
                == (*dp).df_lnum[idx as usize]
        {
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < DB_COUNT {
                if !(*tp).tp_diffbuf[i_1 as usize].is_null() {
                    (*dprev).df_count[i_1 as usize] += (*dp).df_count[i_1 as usize];
                }
                i_1 += 1;
            }
            dp = diff_free(tp, dprev, dp);
        } else {
            dprev = dp;
            dp = (*dp).df_next;
        }
    }
    dprev = ::core::ptr::null_mut::<diff_T>();
    dp = (*tp).tp_first_diff;
    while !dp.is_null() {
        let mut i_2: ::core::ffi::c_int = 0;
        i_2 = 0 as ::core::ffi::c_int;
        while i_2 < DB_COUNT {
            if !(*tp).tp_diffbuf[i_2 as usize].is_null()
                && (*dp).df_count[i_2 as usize] != 0 as linenr_T
            {
                break;
            }
            i_2 += 1;
        }
        if i_2 == DB_COUNT {
            dp = diff_free(tp, dprev, dp);
        } else {
            dprev = dp;
            dp = (*dp).df_next;
        }
    }
    if tp == curtab.get() {
        need_diff_redraw.set(true_0 != 0);
        diff_need_scrollbind.set(true_0 != 0);
    }
}
unsafe extern "C" fn diff_alloc_new(
    mut tp: *mut tabpage_T,
    mut dprev: *mut diff_T,
    mut dp: *mut diff_T,
) -> *mut diff_T {
    let mut dnew: *mut diff_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<diff_T>()) as *mut diff_T;
    (*dnew).is_linematched = false_0 != 0;
    (*dnew).df_next = dp;
    if dprev.is_null() {
        (*tp).tp_first_diff = dnew;
    } else {
        (*dprev).df_next = dnew;
    }
    (*dnew).has_changes = false_0 != 0;
    ga_init(
        &raw mut (*dnew).df_changes,
        ::core::mem::size_of::<diffline_change_T>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    return dnew;
}
unsafe extern "C" fn diff_free(
    mut tp: *mut tabpage_T,
    mut dprev: *mut diff_T,
    mut dp: *mut diff_T,
) -> *mut diff_T {
    let mut ret: *mut diff_T = (*dp).df_next;
    clear_diffblock(dp);
    if dprev.is_null() {
        (*tp).tp_first_diff = ret;
    } else {
        (*dprev).df_next = ret;
    }
    return ret;
}
unsafe extern "C" fn diff_check_unchanged(mut tp: *mut tabpage_T, mut dp: *mut diff_T) {
    let mut i_org: ::core::ffi::c_int = 0;
    i_org = 0 as ::core::ffi::c_int;
    while i_org < DB_COUNT {
        if !(*tp).tp_diffbuf[i_org as usize].is_null() {
            break;
        }
        i_org += 1;
    }
    if i_org == DB_COUNT {
        return;
    }
    if diff_check_sanity(tp, dp) == FAIL {
        return;
    }
    let mut off_org: linenr_T = 0 as linenr_T;
    let mut off_new: linenr_T = 0 as linenr_T;
    let mut dir: ::core::ffi::c_int = FORWARD as ::core::ffi::c_int;
    loop {
        while (*dp).df_count[i_org as usize] > 0 as linenr_T {
            if dir == BACKWARD as ::core::ffi::c_int {
                off_org = (*dp).df_count[i_org as usize] - 1 as linenr_T;
            }
            let mut line_org: *mut ::core::ffi::c_char = xstrdup(ml_get_buf(
                (*tp).tp_diffbuf[i_org as usize] as *mut buf_T,
                (*dp).df_lnum[i_org as usize] + off_org,
            ));
            let mut i_new: ::core::ffi::c_int = 0;
            i_new = i_org + 1 as ::core::ffi::c_int;
            while i_new < DB_COUNT {
                if !(*tp).tp_diffbuf[i_new as usize].is_null() {
                    if dir == BACKWARD as ::core::ffi::c_int {
                        off_new = (*dp).df_count[i_new as usize] - 1 as linenr_T;
                    }
                    if off_new < 0 as linenr_T || off_new >= (*dp).df_count[i_new as usize] {
                        break;
                    }
                    if diff_cmp(
                        line_org,
                        ml_get_buf(
                            (*tp).tp_diffbuf[i_new as usize] as *mut buf_T,
                            (*dp).df_lnum[i_new as usize] + off_new,
                        ),
                    ) != 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                }
                i_new += 1;
            }
            xfree(line_org as *mut ::core::ffi::c_void);
            if i_new != DB_COUNT {
                break;
            }
            i_new = i_org;
            while i_new < DB_COUNT {
                if !(*tp).tp_diffbuf[i_new as usize].is_null() {
                    if dir == FORWARD as ::core::ffi::c_int {
                        (*dp).df_lnum[i_new as usize] += 1;
                    }
                    (*dp).df_count[i_new as usize] -= 1;
                }
                i_new += 1;
            }
        }
        if dir == BACKWARD as ::core::ffi::c_int {
            break;
        }
        dir = BACKWARD as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn diff_check_sanity(
    mut tp: *mut tabpage_T,
    mut dp: *mut diff_T,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*tp).tp_diffbuf[i as usize].is_null() {
            if (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize] - 1 as linenr_T
                > (*(*tp).tp_diffbuf[i as usize]).b_ml.ml_line_count
            {
                return FAIL;
            }
        }
        i += 1;
    }
    return OK;
}
pub unsafe extern "C" fn diff_redraw(mut dofold: bool) {
    let mut wp_other: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut used_max_fill_other: bool = false_0 != 0;
    let mut used_max_fill_curwin: bool = false_0 != 0;
    need_diff_redraw.set(false_0 != 0);
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if !((*wp).w_onebuf_opt.wo_diff == 0 || !buf_valid((*wp).w_buffer)) {
            redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
            if wp != curwin.get() {
                wp_other = wp;
            }
            if dofold as ::core::ffi::c_int != 0 && foldmethodIsDiff(wp) as ::core::ffi::c_int != 0
            {
                foldUpdateAll(wp);
            }
            let mut n: ::core::ffi::c_int = diff_check_fill(wp, (*wp).w_topline);
            if wp != curwin.get() && (*wp).w_topfill > 0 as ::core::ffi::c_int
                || n > 0 as ::core::ffi::c_int
            {
                if (*wp).w_topfill > n {
                    (*wp).w_topfill = if n > 0 as ::core::ffi::c_int {
                        n
                    } else {
                        0 as ::core::ffi::c_int
                    };
                } else if n > 0 as ::core::ffi::c_int && n > (*wp).w_topfill {
                    (*wp).w_topfill = n;
                    if wp == curwin.get() {
                        used_max_fill_curwin = true_0 != 0;
                    } else if !wp_other.is_null() {
                        used_max_fill_other = true_0 != 0;
                    }
                }
                check_topfill(wp, false_0 != 0);
            }
        }
        wp = (*wp).w_next;
    }
    if !wp_other.is_null() && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        if used_max_fill_curwin {
            diff_set_topline(wp_other, curwin.get());
        } else if used_max_fill_other {
            diff_set_topline(curwin.get(), wp_other);
        }
    }
}
unsafe extern "C" fn clear_diffin(mut din: *mut diffin_T) {
    if (*din).din_fname.is_null() {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*din).din_mmfile.ptr as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    } else {
        os_remove((*din).din_fname);
    };
}
unsafe extern "C" fn clear_diffout(mut dout: *mut diffout_T) {
    if (*dout).dout_fname.is_null() {
        ga_clear(&raw mut (*dout).dout_ga);
    } else {
        os_remove((*dout).dout_fname);
    };
}
unsafe extern "C" fn diff_write_buffer(
    mut buf: *mut buf_T,
    mut m: *mut mmfile_t,
    mut start: linenr_T,
    mut end: linenr_T,
) -> ::core::ffi::c_int {
    if end < 0 as linenr_T {
        end = (*buf).b_ml.ml_line_count;
    }
    if (*buf).b_ml.ml_flags & ML_EMPTY != 0 || end < start {
        (*m).ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*m).size = 0 as ::core::ffi::c_int;
        return OK;
    }
    let mut len: size_t = 0 as size_t;
    let mut lnum: linenr_T = start;
    while lnum <= end {
        len = len.wrapping_add((ml_get_buf_len(buf, lnum) as size_t).wrapping_add(1 as size_t));
        lnum += 1;
    }
    let mut ptr: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    (*m).ptr = ptr;
    (*m).size = len as ::core::ffi::c_int;
    len = 0 as size_t;
    let mut lnum_0: linenr_T = start;
    while lnum_0 <= end {
        let mut s: *mut ::core::ffi::c_char = ml_get_buf(buf, lnum_0);
        if diff_flags.get() & DIFF_ICASE != 0 {
            while *s as ::core::ffi::c_int != NUL {
                let mut c: ::core::ffi::c_int = 0;
                let mut c_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                let mut cbuf: [::core::ffi::c_char; 22] = [0; 22];
                if *s as ::core::ffi::c_int == NL {
                    c = NUL;
                } else {
                    c = utf_ptr2char(s);
                    c_len = utf_char2len(c);
                    c = utf_fold(c);
                }
                let orig_len: ::core::ffi::c_int = utfc_ptr2len(s);
                if utf_char2bytes(c, &raw mut cbuf as *mut ::core::ffi::c_char) != c_len {
                    memmove(
                        ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                        s as *const ::core::ffi::c_void,
                        orig_len as size_t,
                    );
                } else {
                    memmove(
                        ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                        &raw mut cbuf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        c_len as size_t,
                    );
                    if orig_len > c_len {
                        memmove(
                            ptr.offset(len as isize).offset(c_len as isize)
                                as *mut ::core::ffi::c_void,
                            s.offset(c_len as isize) as *const ::core::ffi::c_void,
                            (orig_len - c_len) as size_t,
                        );
                    }
                }
                s = s.offset(orig_len as isize);
                len = len.wrapping_add(orig_len as size_t);
            }
        } else {
            let mut slen: size_t = strlen(s);
            memmove(
                ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                slen,
            );
            memchrsub(
                ptr.offset(len as isize) as *mut ::core::ffi::c_void,
                NL as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
                slen,
            );
            len = len.wrapping_add(slen);
        }
        let c2rust_fresh8 = len;
        len = len.wrapping_add(1);
        *ptr.offset(c2rust_fresh8 as isize) = NL as ::core::ffi::c_char;
        lnum_0 += 1;
    }
    return OK;
}
unsafe extern "C" fn diff_write(
    mut buf: *mut buf_T,
    mut din: *mut diffin_T,
    mut start: linenr_T,
    mut end: linenr_T,
) -> ::core::ffi::c_int {
    if (*din).din_fname.is_null() {
        return diff_write_buffer(buf, &raw mut (*din).din_mmfile, start, end);
    }
    if frames_locked() {
        return FAIL;
    }
    if end < 0 as linenr_T {
        end = (*buf).b_ml.ml_line_count;
    }
    let mut save_ml_flags: ::core::ffi::c_int = (*buf).b_ml.ml_flags;
    let mut save_ff: *mut ::core::ffi::c_char = (*buf).b_p_ff;
    (*buf).b_p_ff = xstrdup(b"unix\0".as_ptr() as *const ::core::ffi::c_char);
    let save_cmod_flags: bool = (*cmdmod.ptr()).cmod_flags != 0;
    (*cmdmod.ptr()).cmod_flags |= CMOD_LOCKMARKS as ::core::ffi::c_int;
    if end < start {
        end = start;
        (*buf).b_ml.ml_flags |= ML_EMPTY;
    }
    let mut r: ::core::ffi::c_int = buf_write(
        buf,
        (*din).din_fname,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        start,
        end,
        ::core::ptr::null_mut::<exarg_T>(),
        false_0 != 0,
        false_0 != 0,
        false_0 != 0,
        true_0 != 0,
    );
    (*cmdmod.ptr()).cmod_flags = save_cmod_flags as ::core::ffi::c_int;
    free_string_option((*buf).b_p_ff);
    (*buf).b_p_ff = save_ff;
    (*buf).b_ml.ml_flags = (*buf).b_ml.ml_flags & !ML_EMPTY | save_ml_flags & ML_EMPTY;
    return r;
}
unsafe extern "C" fn lnum_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut lnum1: linenr_T = *(s1 as *mut linenr_T);
    let mut lnum2: linenr_T = *(s2 as *mut linenr_T);
    if lnum1 < lnum2 {
        return -1 as ::core::ffi::c_int;
    }
    if lnum1 > lnum2 {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn diff_try_update(
    mut dio: *mut diffio_T,
    mut idx_orig: ::core::ffi::c_int,
    mut eap: *mut exarg_T,
) {
    let mut num_anchors: ::core::ffi::c_int = 0;
    let mut anchors: [[linenr_T; 20]; 8] = [[0; 20]; 8];
    '_theend: {
        if (*dio).dio_internal != 0 {
            ga_init(
                &raw mut (*dio).dio_diff.dout_ga,
                ::core::mem::size_of::<diffhunk_T>() as ::core::ffi::c_int,
                100 as ::core::ffi::c_int,
            );
        } else {
            (*dio).dio_orig.din_fname = vim_tempname();
            (*dio).dio_new.din_fname = vim_tempname();
            (*dio).dio_diff.dout_fname = vim_tempname();
            if (*dio).dio_orig.din_fname.is_null()
                || (*dio).dio_new.din_fname.is_null()
                || (*dio).dio_diff.dout_fname.is_null()
            {
                break '_theend;
            } else if check_external_diff(dio) == FAIL {
                break '_theend;
            }
        }
        if !eap.is_null() && (*eap).forceit != 0 {
            let mut idx_new: ::core::ffi::c_int = idx_orig;
            while idx_new < DB_COUNT {
                let mut buf: *mut buf_T =
                    (*curtab.get()).tp_diffbuf[idx_new as usize] as *mut buf_T;
                if buf_valid(buf) {
                    buf_check_timestamp(buf);
                }
                idx_new += 1;
            }
        }
        num_anchors = INT_MAX;
        anchors = [[0; 20]; 8];
        memset(
            &raw mut anchors as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[[linenr_T; 20]; 8]>(),
        );
        if diff_flags.get() & DIFF_ANCHOR != 0 {
            let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while idx < DB_COUNT {
                if !(*curtab.get()).tp_diffbuf[idx as usize].is_null() {
                    let mut buf_num_anchors: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if parse_diffanchors(
                        false_0 != 0,
                        (*curtab.get()).tp_diffbuf[idx as usize] as *mut buf_T,
                        &raw mut *(&raw mut anchors as *mut [linenr_T; 20]).offset(idx as isize)
                            as *mut linenr_T,
                        &raw mut buf_num_anchors,
                    ) != OK
                    {
                        emsg(gettext(
                            &raw const e_failed_to_find_all_diff_anchors
                                as *const ::core::ffi::c_char,
                        ));
                        num_anchors = 0 as ::core::ffi::c_int;
                        memset(
                            &raw mut anchors as *mut ::core::ffi::c_void,
                            0 as ::core::ffi::c_int,
                            ::core::mem::size_of::<[[linenr_T; 20]; 8]>(),
                        );
                        break;
                    } else {
                        if buf_num_anchors < num_anchors {
                            num_anchors = buf_num_anchors;
                        }
                        if buf_num_anchors > 0 as ::core::ffi::c_int {
                            qsort(
                                &raw mut *(&raw mut anchors as *mut [linenr_T; 20])
                                    .offset(idx as isize)
                                    as *mut linenr_T
                                    as *mut ::core::ffi::c_void,
                                buf_num_anchors as size_t,
                                ::core::mem::size_of::<linenr_T>(),
                                Some(
                                    lnum_compare
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_void,
                                            *const ::core::ffi::c_void,
                                        )
                                            -> ::core::ffi::c_int,
                                ),
                            );
                        }
                    }
                }
                idx += 1;
            }
        }
        if num_anchors == INT_MAX {
            num_anchors = 0 as ::core::ffi::c_int;
        }
        let mut anchor_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            if anchor_i > num_anchors {
                break '_theend;
            }
            let mut orig_diff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
            if anchor_i != 0 as ::core::ffi::c_int {
                orig_diff = (*curtab.get()).tp_first_diff;
                (*curtab.get()).tp_first_diff = ::core::ptr::null_mut::<diff_T>();
            }
            let mut lnum_start: linenr_T = if anchor_i == 0 as ::core::ffi::c_int {
                1 as linenr_T
            } else {
                anchors[idx_orig as usize][(anchor_i - 1 as ::core::ffi::c_int) as usize]
            };
            let mut lnum_end: linenr_T = if anchor_i == num_anchors {
                -1 as linenr_T
            } else {
                anchors[idx_orig as usize][anchor_i as usize] - 1 as linenr_T
            };
            let mut buf_0: *mut buf_T = (*curtab.get()).tp_diffbuf[idx_orig as usize] as *mut buf_T;
            if diff_write(buf_0, &raw mut (*dio).dio_orig, lnum_start, lnum_end) == FAIL {
                if !orig_diff.is_null() {
                    (*curtab.get()).tp_first_diff = orig_diff;
                    diff_clear(curtab.get());
                }
                break '_theend;
            } else {
                let mut idx_new_0: ::core::ffi::c_int = idx_orig + 1 as ::core::ffi::c_int;
                while idx_new_0 < DB_COUNT {
                    buf_0 = (*curtab.get()).tp_diffbuf[idx_new_0 as usize] as *mut buf_T;
                    if !(buf_0.is_null() || (*buf_0).b_ml.ml_mfp.is_null()) {
                        lnum_start = if anchor_i == 0 as ::core::ffi::c_int {
                            1 as linenr_T
                        } else {
                            anchors[idx_new_0 as usize]
                                [(anchor_i - 1 as ::core::ffi::c_int) as usize]
                        };
                        lnum_end = if anchor_i == num_anchors {
                            -1 as linenr_T
                        } else {
                            anchors[idx_new_0 as usize][anchor_i as usize] - 1 as linenr_T
                        };
                        if diff_write(buf_0, &raw mut (*dio).dio_new, lnum_start, lnum_end) != FAIL
                        {
                            if diff_file(dio) != FAIL {
                                diff_read(idx_orig, idx_new_0, dio);
                                clear_diffin(&raw mut (*dio).dio_new);
                                clear_diffout(&raw mut (*dio).dio_diff);
                            }
                        }
                    }
                    idx_new_0 += 1;
                }
                clear_diffin(&raw mut (*dio).dio_orig);
                if anchor_i != 0 as ::core::ffi::c_int {
                    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
                    while !dp.is_null() {
                        let mut idx_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while idx_0 < DB_COUNT {
                            if anchors[idx_0 as usize]
                                [(anchor_i - 1 as ::core::ffi::c_int) as usize]
                                > 0 as linenr_T
                            {
                                (*dp).df_lnum[idx_0 as usize] = ((*dp).df_lnum[idx_0 as usize]
                                    as ::core::ffi::c_int
                                    + (anchors[idx_0 as usize]
                                        [(anchor_i - 1 as ::core::ffi::c_int) as usize]
                                        - 1 as linenr_T)
                                        as ::core::ffi::c_int)
                                    as linenr_T;
                            }
                            idx_0 += 1;
                        }
                        dp = (*dp).df_next;
                    }
                    if !orig_diff.is_null() {
                        let mut last_diff: *mut diff_T = orig_diff;
                        while !(*last_diff).df_next.is_null() {
                            last_diff = (*last_diff).df_next;
                        }
                        (*last_diff).df_next = (*curtab.get()).tp_first_diff;
                        (*curtab.get()).tp_first_diff = orig_diff;
                    }
                }
                anchor_i += 1;
            }
        }
    }
    xfree((*dio).dio_orig.din_fname as *mut ::core::ffi::c_void);
    xfree((*dio).dio_new.din_fname as *mut ::core::ffi::c_void);
    xfree((*dio).dio_diff.dout_fname as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn diff_internal() -> ::core::ffi::c_int {
    return (diff_flags.get() & DIFF_INTERNAL != 0 as ::core::ffi::c_int
        && *p_dex.get() as ::core::ffi::c_int == NUL) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn ex_diffupdate(mut eap: *mut exarg_T) {
    let mut idx_new: ::core::ffi::c_int = 0;
    let mut diffio: diffio_T = diffio_T {
        dio_orig: diffin_T {
            din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            din_mmfile: mmfile_t {
                ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        },
        dio_new: diffin_T {
            din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            din_mmfile: mmfile_t {
                ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        },
        dio_diff: diffout_T {
            dout_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            dout_ga: garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
        },
        dio_internal: 0,
    };
    if diff_busy.get() {
        diff_need_update.set(true_0 != 0);
        return;
    }
    let mut had_diffs: ::core::ffi::c_int =
        !(*curtab.get()).tp_first_diff.is_null() as ::core::ffi::c_int;
    diff_clear(curtab.get());
    (*curtab.get()).tp_diff_invalid = false_0;
    let mut idx_orig: ::core::ffi::c_int = 0;
    idx_orig = 0 as ::core::ffi::c_int;
    while idx_orig < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[idx_orig as usize].is_null() {
            break;
        }
        idx_orig += 1;
    }
    if idx_orig != DB_COUNT {
        idx_new = 0;
        idx_new = idx_orig + 1 as ::core::ffi::c_int;
        while idx_new < DB_COUNT {
            if !(*curtab.get()).tp_diffbuf[idx_new as usize].is_null() {
                break;
            }
            idx_new += 1;
        }
        if idx_new != DB_COUNT {
            diffio = diffio_T {
                dio_orig: diffin_T {
                    din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    din_mmfile: mmfile_t {
                        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        size: 0,
                    },
                },
                dio_new: diffin_T {
                    din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    din_mmfile: mmfile_t {
                        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        size: 0,
                    },
                },
                dio_diff: diffout_T {
                    dout_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    dout_ga: garray_T {
                        ga_len: 0,
                        ga_maxlen: 0,
                        ga_itemsize: 0,
                        ga_growsize: 0,
                        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    },
                },
                dio_internal: 0,
            };
            diffio.dio_internal = diff_internal();
            diff_try_update(&raw mut diffio, idx_orig, eap);
            (*curwin.get()).w_valid_cursor.lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
    }
    if had_diffs != 0 || !(*curtab.get()).tp_first_diff.is_null() {
        diff_redraw(true_0 != 0);
        apply_autocmds(
            EVENT_DIFFUPDATED,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
}
unsafe extern "C" fn check_external_diff(mut diffio: *mut diffio_T) -> ::core::ffi::c_int {
    let mut io_error: bool = false_0 != 0;
    let mut ok: TriState = kFalse;
    loop {
        ok = kFalse;
        let mut fd: *mut FILE = os_fopen(
            (*diffio).dio_orig.din_fname,
            b"w\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if fd.is_null() {
            io_error = true_0 != 0;
        } else {
            if fwrite(
                b"line1\n\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                6 as size_t,
                1 as size_t,
                fd,
            ) != 1 as ::core::ffi::c_ulong
            {
                io_error = true_0 != 0;
            }
            fclose(fd);
            fd = os_fopen(
                (*diffio).dio_new.din_fname,
                b"w\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if fd.is_null() {
                io_error = true_0 != 0;
            } else {
                if fwrite(
                    b"line2\n\0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    6 as size_t,
                    1 as size_t,
                    fd,
                ) != 1 as ::core::ffi::c_ulong
                {
                    io_error = true_0 != 0;
                }
                fclose(fd);
                fd = if diff_file(diffio) == OK {
                    os_fopen(
                        (*diffio).dio_diff.dout_fname,
                        b"r\0".as_ptr() as *const ::core::ffi::c_char,
                    )
                } else {
                    ::core::ptr::null_mut::<FILE>()
                };
                if fd.is_null() {
                    io_error = true_0 != 0;
                } else {
                    let mut linebuf: [::core::ffi::c_char; 50] = [0; 50];
                    while !vim_fgets(&raw mut linebuf as *mut ::core::ffi::c_char, LBUFLEN, fd) {
                        if strncmp(
                            &raw mut linebuf as *mut ::core::ffi::c_char,
                            b"1c1\0".as_ptr() as *const ::core::ffi::c_char,
                            3 as size_t,
                        ) == 0 as ::core::ffi::c_int
                            || strncmp(
                                &raw mut linebuf as *mut ::core::ffi::c_char,
                                b"@@ -1 +1 @@\0".as_ptr() as *const ::core::ffi::c_char,
                                11 as size_t,
                            ) == 0 as ::core::ffi::c_int
                        {
                            ok = kTrue;
                        }
                    }
                    fclose(fd);
                }
                os_remove((*diffio).dio_diff.dout_fname);
                os_remove((*diffio).dio_new.din_fname);
            }
            os_remove((*diffio).dio_orig.din_fname);
        }
        if *p_dex.get() as ::core::ffi::c_int != NUL {
            break;
        }
        if diff_a_works.get() as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            break;
        }
        diff_a_works.set(ok);
        if ok as u64 != 0 {
            break;
        }
    }
    if ok as u64 == 0 {
        if io_error {
            emsg(gettext(
                b"E810: Cannot read or write temp files\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        emsg(gettext(
            b"E97: Cannot create diffs\0".as_ptr() as *const ::core::ffi::c_char
        ));
        diff_a_works.set(kNone);
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn diff_file_internal(mut diffio: *mut diffio_T) -> ::core::ffi::c_int {
    let mut param: xpparam_t = xpparam_t {
        flags: 0,
        anchors: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        anchors_nr: 0,
    };
    let mut emit_cfg: xdemitconf_t = xdemitconf_t {
        ctxlen: 0,
        interhunkctxlen: 0,
        flags: 0,
        find_func: None,
        find_func_priv: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        hunk_func: None,
    };
    let mut emit_cb: xdemitcb_t = xdemitcb_t {
        priv_0: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        out_hunk: None,
        out_line: None,
    };
    memset(
        &raw mut param as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xpparam_t>(),
    );
    memset(
        &raw mut emit_cfg as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xdemitconf_t>(),
    );
    memset(
        &raw mut emit_cb as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xdemitcb_t>(),
    );
    param.flags = diff_algorithm.get() as ::core::ffi::c_ulong;
    if diff_flags.get() & DIFF_IWHITE != 0 {
        param.flags |= XDF_IGNORE_WHITESPACE_CHANGE as ::core::ffi::c_ulong;
    }
    if diff_flags.get() & DIFF_IWHITEALL != 0 {
        param.flags |= XDF_IGNORE_WHITESPACE as ::core::ffi::c_ulong;
    }
    if diff_flags.get() & DIFF_IWHITEEOL != 0 {
        param.flags |= XDF_IGNORE_WHITESPACE_AT_EOL as ::core::ffi::c_ulong;
    }
    if diff_flags.get() & DIFF_IBLANK != 0 {
        param.flags |= XDF_IGNORE_BLANK_LINES as ::core::ffi::c_ulong;
    }
    emit_cfg.ctxlen = 0 as ::core::ffi::c_long;
    emit_cb.priv_0 = &raw mut (*diffio).dio_diff as *mut ::core::ffi::c_void;
    emit_cfg.hunk_func = Some(
        xdiff_out
            as unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                ::core::ffi::c_int,
                *mut ::core::ffi::c_void,
            ) -> ::core::ffi::c_int,
    ) as xdl_emit_hunk_consume_func_t;
    if (*diffio).dio_orig.din_mmfile.size as ::core::ffi::c_long > MAX_XDIFF_SIZE
        || (*diffio).dio_new.din_mmfile.size as ::core::ffi::c_long > MAX_XDIFF_SIZE
    {
        emsg(gettext(
            &raw const e_problem_creating_internal_diff as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if xdl_diff(
        &raw mut (*diffio).dio_orig.din_mmfile,
        &raw mut (*diffio).dio_new.din_mmfile,
        &raw mut param,
        &raw mut emit_cfg,
        &raw mut emit_cb,
    ) < 0 as ::core::ffi::c_int
    {
        emsg(gettext(
            &raw const e_problem_creating_internal_diff as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn diff_file(mut dio: *mut diffio_T) -> ::core::ffi::c_int {
    let mut tmp_orig: *mut ::core::ffi::c_char = (*dio).dio_orig.din_fname;
    let mut tmp_new: *mut ::core::ffi::c_char = (*dio).dio_new.din_fname;
    let mut tmp_diff: *mut ::core::ffi::c_char = (*dio).dio_diff.dout_fname;
    if *p_dex.get() as ::core::ffi::c_int != NUL {
        eval_diff(tmp_orig, tmp_new, tmp_diff);
        return OK;
    }
    if (*dio).dio_internal != 0 {
        return diff_file_internal(dio);
    }
    let len: size_t = strlen(tmp_orig)
        .wrapping_add(strlen(tmp_new))
        .wrapping_add(strlen(tmp_diff))
        .wrapping_add(strlen(p_srr.get()))
        .wrapping_add(27 as size_t);
    let cmd: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    if os_env_exists(
        b"DIFF_OPTIONS\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
    ) {
        os_unsetenv(b"DIFF_OPTIONS\0".as_ptr() as *const ::core::ffi::c_char);
    }
    vim_snprintf(
        cmd,
        len,
        b"diff %s%s%s%s%s%s%s%s %s\0".as_ptr() as *const ::core::ffi::c_char,
        if diff_a_works.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"-a \0".as_ptr() as *const ::core::ffi::c_char
        },
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        if diff_flags.get() & DIFF_IWHITE != 0 {
            b"-b \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if diff_flags.get() & DIFF_IWHITEALL != 0 {
            b"-w \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if diff_flags.get() & DIFF_IWHITEEOL != 0 {
            b"-Z \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if diff_flags.get() & DIFF_IBLANK != 0 {
            b"-B \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        if diff_flags.get() & DIFF_ICASE != 0 {
            b"-i \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
        tmp_orig,
        tmp_new,
    );
    append_redir(cmd, len, p_srr.get(), tmp_diff);
    block_autocmds();
    call_shell(
        cmd,
        kShellOptFilter as ::core::ffi::c_int
            | kShellOptSilent as ::core::ffi::c_int
            | kShellOptDoOut as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    unblock_autocmds();
    xfree(cmd as *mut ::core::ffi::c_void);
    return OK;
}
pub unsafe extern "C" fn ex_diffpatch(mut eap: *mut exarg_T) {
    let mut buflen: size_t = 0;
    let mut dirbuf: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut file_info: FileInfo = FileInfo {
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
    let mut info_ok: bool = false;
    let mut filesize: uint64_t = 0;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut old_curwin: *mut win_T = curwin.get();
    let mut newname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut esc_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fullname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp_orig: *mut ::core::ffi::c_char = vim_tempname();
    let mut tmp_new: *mut ::core::ffi::c_char = vim_tempname();
    if !(tmp_orig.is_null() || tmp_new.is_null()) {
        if buf_write(
            curbuf.get(),
            tmp_orig,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            1 as linenr_T,
            (*curbuf.get()).b_ml.ml_line_count,
            ::core::ptr::null_mut::<exarg_T>(),
            false_0 != 0,
            false_0 != 0,
            false_0 != 0,
            true_0 != 0,
        ) != FAIL
        {
            fullname = FullName_save((*eap).arg, false_0 != 0);
            esc_name = vim_strsave_shellescape(
                if !fullname.is_null() {
                    fullname
                } else {
                    (*eap).arg
                },
                true_0 != 0,
                true_0 != 0,
            );
            buflen = strlen(tmp_orig)
                .wrapping_add(strlen(esc_name))
                .wrapping_add(strlen(tmp_new))
                .wrapping_add(16 as size_t);
            buf = xmalloc(buflen) as *mut ::core::ffi::c_char;
            dirbuf = [0; 4096];
            if os_dirname(
                &raw mut dirbuf as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
            ) != OK
                || os_chdir(&raw mut dirbuf as *mut ::core::ffi::c_char) != 0 as ::core::ffi::c_int
            {
                dirbuf[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
            } else {
                let mut tempdir: *mut ::core::ffi::c_char = vim_gettempdir();
                if tempdir.is_null() {
                    tempdir = b"/tmp\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
                os_chdir(tempdir);
                shorten_fnames(true_0);
            }
            if *p_pex.get() as ::core::ffi::c_int != NUL {
                eval_patch(
                    tmp_orig,
                    if !fullname.is_null() {
                        fullname
                    } else {
                        (*eap).arg
                    },
                    tmp_new,
                );
            } else {
                vim_snprintf(
                    buf,
                    buflen,
                    b"patch -o %s %s < %s\0".as_ptr() as *const ::core::ffi::c_char,
                    tmp_new,
                    tmp_orig,
                    esc_name,
                );
                block_autocmds();
                call_shell(
                    buf,
                    kShellOptFilter as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                unblock_autocmds();
            }
            if dirbuf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL {
                if os_chdir(&raw mut dirbuf as *mut ::core::ffi::c_char) != 0 as ::core::ffi::c_int
                {
                    emsg(gettext(&raw const e_prev_dir as *const ::core::ffi::c_char));
                }
                shorten_fnames(true_0);
            }
            strcpy(buf, tmp_new);
            strcat(buf, b".orig\0".as_ptr() as *const ::core::ffi::c_char);
            os_remove(buf);
            strcpy(buf, tmp_new);
            strcat(buf, b".rej\0".as_ptr() as *const ::core::ffi::c_char);
            os_remove(buf);
            file_info = FileInfo {
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
            info_ok = os_fileinfo(tmp_new, &raw mut file_info);
            filesize = os_fileinfo_size(&raw mut file_info);
            if !info_ok || filesize == 0 as uint64_t {
                emsg(gettext(
                    b"E816: Cannot read patch output\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                if !(*curbuf.get()).b_fname.is_null() {
                    newname = xstrnsave(
                        (*curbuf.get()).b_fname,
                        strlen((*curbuf.get()).b_fname).wrapping_add(4 as size_t),
                    );
                    strcat(newname, b".new\0".as_ptr() as *const ::core::ffi::c_char);
                }
                (*cmdmod.ptr()).cmod_tab = 0 as ::core::ffi::c_int;
                if win_split(
                    0 as ::core::ffi::c_int,
                    if diff_flags.get() & DIFF_VERTICAL != 0 {
                        WSP_VERT as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                ) != FAIL
                {
                    (*eap).cmdidx = CMD_split;
                    (*eap).arg = tmp_new;
                    do_exedit(eap, old_curwin);
                    if curwin.get() != old_curwin
                        && win_valid(old_curwin) as ::core::ffi::c_int != 0
                    {
                        diff_win_options(curwin.get(), true_0 != 0);
                        diff_win_options(old_curwin, true_0 != 0);
                        if !newname.is_null() {
                            (*eap).arg = newname;
                            ex_file(eap);
                            if augroup_exists(
                                b"filetypedetect\0".as_ptr() as *const ::core::ffi::c_char
                            ) {
                                do_cmdline_cmd(b":doau filetypedetect BufRead\0".as_ptr()
                                    as *const ::core::ffi::c_char);
                            }
                        }
                    }
                }
            }
        }
    }
    if !tmp_orig.is_null() {
        os_remove(tmp_orig);
    }
    xfree(tmp_orig as *mut ::core::ffi::c_void);
    if !tmp_new.is_null() {
        os_remove(tmp_new);
    }
    xfree(tmp_new as *mut ::core::ffi::c_void);
    xfree(newname as *mut ::core::ffi::c_void);
    xfree(buf as *mut ::core::ffi::c_void);
    xfree(fullname as *mut ::core::ffi::c_void);
    xfree(esc_name as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn ex_diffsplit(mut eap: *mut exarg_T) {
    let mut old_curwin: *mut win_T = curwin.get();
    let mut old_curbuf: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    set_bufref(&raw mut old_curbuf, curbuf.get());
    validate_cursor(curwin.get());
    set_fraction(curwin.get());
    (*cmdmod.ptr()).cmod_tab = 0 as ::core::ffi::c_int;
    if win_split(
        0 as ::core::ffi::c_int,
        if diff_flags.get() & DIFF_VERTICAL != 0 {
            WSP_VERT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    ) == FAIL
    {
        return;
    }
    (*eap).cmdidx = CMD_split;
    (*curwin.get()).w_onebuf_opt.wo_diff = true_0;
    do_exedit(eap, old_curwin);
    if curwin.get() == old_curwin {
        return;
    }
    diff_win_options(curwin.get(), true_0 != 0);
    if win_valid(old_curwin) {
        diff_win_options(old_curwin, true_0 != 0);
        if bufref_valid(&raw mut old_curbuf) {
            (*curwin.get()).w_cursor.lnum =
                diff_get_corresponding_line(old_curbuf.br_buf, (*old_curwin).w_cursor.lnum);
        }
    }
    scroll_to_fraction(curwin.get(), (*curwin.get()).w_height);
}
pub unsafe extern "C" fn ex_diffthis(mut _eap: *mut exarg_T) {
    diff_win_options(curwin.get(), true_0 != 0);
}
unsafe extern "C" fn set_diff_option(mut wp: *mut win_T, mut value: bool) {
    let mut old_curwin: *mut win_T = curwin.get();
    curwin.set(wp);
    curbuf.set((*curwin.get()).w_buffer);
    (*curbuf.get()).b_ro_locked += 1;
    set_option_value_give_err(
        kOptDiff,
        OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData {
                boolean: value as TriState,
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
    );
    (*curbuf.get()).b_ro_locked -= 1;
    curwin.set(old_curwin);
    curbuf.set((*curwin.get()).w_buffer);
}
pub unsafe extern "C" fn diff_win_options(mut wp: *mut win_T, mut addbuf: bool) {
    let mut old_curwin: *mut win_T = curwin.get();
    curwin.set(wp);
    newFoldLevel();
    curwin.set(old_curwin);
    if (*wp).w_onebuf_opt.wo_diff == 0 {
        (*wp).w_onebuf_opt.wo_scb_save = (*wp).w_onebuf_opt.wo_scb;
    }
    (*wp).w_onebuf_opt.wo_scb = true_0;
    if (*wp).w_onebuf_opt.wo_diff == 0 {
        (*wp).w_onebuf_opt.wo_crb_save = (*wp).w_onebuf_opt.wo_crb;
    }
    (*wp).w_onebuf_opt.wo_crb = true_0;
    if diff_flags.get() & DIFF_FOLLOWWRAP == 0 {
        if (*wp).w_onebuf_opt.wo_diff == 0 {
            (*wp).w_onebuf_opt.wo_wrap_save = (*wp).w_onebuf_opt.wo_wrap;
        }
        (*wp).w_onebuf_opt.wo_wrap = false_0;
        (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
    }
    if (*wp).w_onebuf_opt.wo_diff == 0 {
        if (*wp).w_onebuf_opt.wo_diff_saved != 0 {
            free_string_option((*wp).w_onebuf_opt.wo_fdm_save);
        }
        (*wp).w_onebuf_opt.wo_fdm_save = xstrdup((*wp).w_onebuf_opt.wo_fdm);
    }
    set_option_direct_for(
        kOptFoldmethod,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"diff\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_LOCAL as ::core::ffi::c_int,
        0 as scid_T,
        kOptScopeWin,
        wp as *mut ::core::ffi::c_void,
    );
    if (*wp).w_onebuf_opt.wo_diff == 0 {
        (*wp).w_onebuf_opt.wo_fen_save = (*wp).w_onebuf_opt.wo_fen;
        (*wp).w_onebuf_opt.wo_fdl_save = (*wp).w_onebuf_opt.wo_fdl;
        if (*wp).w_onebuf_opt.wo_diff_saved != 0 {
            free_string_option((*wp).w_onebuf_opt.wo_fdc_save);
        }
        (*wp).w_onebuf_opt.wo_fdc_save = xstrdup((*wp).w_onebuf_opt.wo_fdc);
    }
    free_string_option((*wp).w_onebuf_opt.wo_fdc);
    (*wp).w_onebuf_opt.wo_fdc = xstrdup(b"2\0".as_ptr() as *const ::core::ffi::c_char);
    '_c2rust_label: {
        if diff_foldcolumn.get() >= 0 as ::core::ffi::c_int
            && diff_foldcolumn.get() <= 9 as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"diff_foldcolumn >= 0 && diff_foldcolumn <= 9\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/diff.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1573 as ::core::ffi::c_uint,
                b"void diff_win_options(win_T *, _Bool)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    snprintf(
        (*wp).w_onebuf_opt.wo_fdc,
        strlen((*wp).w_onebuf_opt.wo_fdc).wrapping_add(1 as size_t),
        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
        diff_foldcolumn.get(),
    );
    (*wp).w_onebuf_opt.wo_fen = true_0;
    (*wp).w_onebuf_opt.wo_fdl = 0 as OptInt;
    foldUpdateAll(wp);
    changed_window_setting(wp);
    if vim_strchr(p_sbo.get(), 'h' as ::core::ffi::c_int).is_null() {
        do_cmdline_cmd(b"set sbo+=hor\0".as_ptr() as *const ::core::ffi::c_char);
    }
    (*wp).w_onebuf_opt.wo_diff_saved = true_0;
    set_diff_option(wp, true_0 != 0);
    if addbuf {
        diff_buf_add((*wp).w_buffer);
    }
    redraw_later(wp, UPD_NOT_VALID as ::core::ffi::c_int);
}
pub unsafe extern "C" fn ex_diffoff(mut eap: *mut exarg_T) {
    let mut diffwin: bool = false_0 != 0;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if if (*eap).forceit != 0 {
            (*wp).w_onebuf_opt.wo_diff
        } else {
            (wp == curwin.get()) as ::core::ffi::c_int
        } != 0
        {
            set_diff_option(wp, false_0 != 0);
            if (*wp).w_onebuf_opt.wo_diff_saved != 0 {
                if (*wp).w_onebuf_opt.wo_scb != 0 {
                    (*wp).w_onebuf_opt.wo_scb = (*wp).w_onebuf_opt.wo_scb_save;
                }
                if (*wp).w_onebuf_opt.wo_crb != 0 {
                    (*wp).w_onebuf_opt.wo_crb = (*wp).w_onebuf_opt.wo_crb_save;
                }
                if diff_flags.get() & DIFF_FOLLOWWRAP == 0 {
                    if (*wp).w_onebuf_opt.wo_wrap == 0 && (*wp).w_onebuf_opt.wo_wrap_save != 0 {
                        (*wp).w_onebuf_opt.wo_wrap = true_0;
                        (*wp).w_leftcol = 0 as ::core::ffi::c_int as colnr_T;
                    }
                }
                free_string_option((*wp).w_onebuf_opt.wo_fdm);
                (*wp).w_onebuf_opt.wo_fdm = xstrdup(
                    if *(*wp).w_onebuf_opt.wo_fdm_save as ::core::ffi::c_int != 0 {
                        (*wp).w_onebuf_opt.wo_fdm_save as *const ::core::ffi::c_char
                    } else {
                        b"manual\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
                free_string_option((*wp).w_onebuf_opt.wo_fdc);
                (*wp).w_onebuf_opt.wo_fdc = xstrdup(
                    if *(*wp).w_onebuf_opt.wo_fdc_save as ::core::ffi::c_int != 0 {
                        (*wp).w_onebuf_opt.wo_fdc_save as *const ::core::ffi::c_char
                    } else {
                        b"0\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
                if (*wp).w_onebuf_opt.wo_fdl == 0 as OptInt {
                    (*wp).w_onebuf_opt.wo_fdl = (*wp).w_onebuf_opt.wo_fdl_save;
                }
                if (*wp).w_onebuf_opt.wo_fen != 0 {
                    (*wp).w_onebuf_opt.wo_fen = if foldmethodIsManual(wp) as ::core::ffi::c_int != 0
                    {
                        false_0
                    } else {
                        (*wp).w_onebuf_opt.wo_fen_save
                    };
                }
                foldUpdateAll(wp);
            }
            (*wp).w_topfill = 0 as ::core::ffi::c_int;
            changed_window_setting(wp);
            diff_buf_adjust(wp);
        }
        diffwin = diffwin as ::core::ffi::c_int | (*wp).w_onebuf_opt.wo_diff != 0;
        wp = (*wp).w_next;
    }
    if (*eap).forceit != 0 {
        diff_buf_clear();
    }
    if !diffwin {
        diff_need_update.set(false_0 != 0);
        (*curtab.get()).tp_diff_invalid = false_0;
        (*curtab.get()).tp_diff_update = false_0;
        diff_clear(curtab.get());
    }
    if !diffwin && !vim_strchr(p_sbo.get(), 'h' as ::core::ffi::c_int).is_null() {
        do_cmdline_cmd(b"set sbo-=hor\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
unsafe extern "C" fn extract_hunk_internal(
    mut dout: *mut diffout_T,
    mut hunk: *mut diffhunk_T,
    mut line_idx: *mut ::core::ffi::c_int,
) -> bool {
    let mut eof: bool = *line_idx >= (*dout).dout_ga.ga_len;
    if !eof {
        let c2rust_fresh7 = *line_idx;
        *line_idx = *line_idx + 1;
        *hunk = *((*dout).dout_ga.ga_data as *mut diffhunk_T).offset(c2rust_fresh7 as isize);
    }
    return eof;
}
unsafe extern "C" fn extract_hunk(
    mut fd: *mut FILE,
    mut hunk: *mut diffhunk_T,
    mut diffstyle: *mut diffstyle_T,
) -> bool {
    loop {
        let mut line: [::core::ffi::c_char; 50] = [0; 50];
        if vim_fgets(&raw mut line as *mut ::core::ffi::c_char, LBUFLEN, fd) {
            return true_0 != 0;
        }
        if *diffstyle as ::core::ffi::c_uint
            == DIFF_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if *(*__ctype_b_loc()).offset(
                *(&raw mut line as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int
                    as isize,
            ) as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
            {
                *diffstyle = DIFF_ED;
            } else if strncmp(
                &raw mut line as *mut ::core::ffi::c_char,
                b"@@ \0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                *diffstyle = DIFF_UNIFIED;
            } else {
                if !(strncmp(
                    &raw mut line as *mut ::core::ffi::c_char,
                    b"--- \0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int
                    && vim_fgets(&raw mut line as *mut ::core::ffi::c_char, LBUFLEN, fd)
                        as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    && strncmp(
                        &raw mut line as *mut ::core::ffi::c_char,
                        b"+++ \0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    && vim_fgets(&raw mut line as *mut ::core::ffi::c_char, LBUFLEN, fd)
                        as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    && strncmp(
                        &raw mut line as *mut ::core::ffi::c_char,
                        b"@@ \0".as_ptr() as *const ::core::ffi::c_char,
                        3 as size_t,
                    ) == 0 as ::core::ffi::c_int)
                {
                    continue;
                }
                *diffstyle = DIFF_UNIFIED;
            }
        }
        if *diffstyle as ::core::ffi::c_uint == DIFF_ED as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if *(*__ctype_b_loc()).offset(
                *(&raw mut line as *mut ::core::ffi::c_char) as uint8_t as ::core::ffi::c_int
                    as isize,
            ) as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            {
                continue;
            }
            if parse_diff_ed(&raw mut line as *mut ::core::ffi::c_char, hunk) == FAIL {
                continue;
            }
        } else {
            '_c2rust_label: {
                if *diffstyle as ::core::ffi::c_uint
                    == DIFF_UNIFIED as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                } else {
                    __assert_fail(
                        b"*diffstyle == DIFF_UNIFIED\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/diff.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1726 as ::core::ffi::c_uint,
                        b"_Bool extract_hunk(FILE *, diffhunk_T *, diffstyle_T *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if strncmp(
                &raw mut line as *mut ::core::ffi::c_char,
                b"@@ \0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                continue;
            }
            if parse_diff_unified(&raw mut line as *mut ::core::ffi::c_char, hunk) == FAIL {
                continue;
            }
        }
        return false_0 != 0;
    }
}
unsafe extern "C" fn process_hunk(
    mut dpp: *mut *mut diff_T,
    mut dprevp: *mut *mut diff_T,
    mut idx_orig: ::core::ffi::c_int,
    mut idx_new: ::core::ffi::c_int,
    mut hunk: *mut diffhunk_T,
    mut notsetp: *mut bool,
) {
    let mut dp: *mut diff_T = *dpp;
    let mut dprev: *mut diff_T = *dprevp;
    while !dp.is_null()
        && (*hunk).lnum_orig > (*dp).df_lnum[idx_orig as usize] + (*dp).df_count[idx_orig as usize]
    {
        if *notsetp {
            diff_copy_entry(dprev, dp, idx_orig, idx_new);
        }
        dprev = dp;
        dp = (*dp).df_next;
        *notsetp = true_0 != 0;
    }
    if !dp.is_null()
        && (*hunk).lnum_orig <= (*dp).df_lnum[idx_orig as usize] + (*dp).df_count[idx_orig as usize]
        && (*hunk).lnum_orig + (*hunk).count_orig as linenr_T >= (*dp).df_lnum[idx_orig as usize]
    {
        let mut dpl: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        dpl = dp;
        while !(*dpl).df_next.is_null() {
            if ((*hunk).lnum_orig + (*hunk).count_orig as linenr_T)
                < (*(*dpl).df_next).df_lnum[idx_orig as usize]
            {
                break;
            }
            dpl = (*dpl).df_next;
        }
        let mut off: linenr_T = (*dp).df_lnum[idx_orig as usize] - (*hunk).lnum_orig;
        if off > 0 as linenr_T {
            let mut i: ::core::ffi::c_int = idx_orig;
            while i < idx_new {
                if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                    (*dp).df_lnum[i as usize] -= off;
                    (*dp).df_count[i as usize] += off;
                }
                i += 1;
            }
            (*dp).df_lnum[idx_new as usize] = (*hunk).lnum_new;
            (*dp).df_count[idx_new as usize] = (*hunk).count_new as linenr_T;
        } else if *notsetp {
            (*dp).df_lnum[idx_new as usize] = (*hunk).lnum_new + off;
            (*dp).df_count[idx_new as usize] = (*hunk).count_new as linenr_T - off;
        } else {
            let mut orig_size_in_dp: ::core::ffi::c_int = if ((*hunk).count_orig as linenr_T)
                < (*dp).df_lnum[idx_orig as usize] + (*dp).df_count[idx_orig as usize]
                    - (*hunk).lnum_orig
            {
                (*hunk).count_orig
            } else {
                (*dp).df_lnum[idx_orig as usize] as ::core::ffi::c_int
                    + (*dp).df_count[idx_orig as usize] as ::core::ffi::c_int
                    - (*hunk).lnum_orig as ::core::ffi::c_int
            };
            let mut size_diff: ::core::ffi::c_int = (*hunk).count_new - orig_size_in_dp;
            (*dp).df_count[idx_new as usize] =
                ((*dp).df_count[idx_new as usize] as ::core::ffi::c_int + size_diff) as linenr_T;
            off = (*hunk).lnum_new + (*hunk).count_new as linenr_T
                - ((*dp).df_lnum[idx_new as usize] + (*dp).df_count[idx_new as usize]);
            if off > 0 as linenr_T {
                (*dp).df_count[idx_new as usize] += off;
            }
        }
        off = (*hunk).lnum_orig + (*hunk).count_orig as linenr_T
            - ((*dpl).df_lnum[idx_orig as usize] + (*dpl).df_count[idx_orig as usize]);
        if off < 0 as linenr_T {
            if *notsetp as ::core::ffi::c_int != 0 || dp != dpl {
                (*dp).df_count[idx_new as usize] += -off;
            }
            off = 0 as ::core::ffi::c_int as linenr_T;
        }
        let mut i_0: ::core::ffi::c_int = idx_orig;
        while i_0 < idx_new {
            if !(*curtab.get()).tp_diffbuf[i_0 as usize].is_null() {
                (*dp).df_count[i_0 as usize] = (*dpl).df_lnum[i_0 as usize]
                    + (*dpl).df_count[i_0 as usize]
                    - (*dp).df_lnum[i_0 as usize]
                    + off;
            }
            i_0 += 1;
        }
        let mut dn: *mut diff_T = (*dp).df_next;
        (*dp).df_next = (*dpl).df_next;
        while dn != (*dp).df_next {
            dpl = (*dn).df_next;
            clear_diffblock(dn);
            dn = dpl;
        }
    } else {
        dp = diff_alloc_new(curtab.get(), dprev, dp);
        (*dp).df_lnum[idx_orig as usize] = (*hunk).lnum_orig;
        (*dp).df_count[idx_orig as usize] = (*hunk).count_orig as linenr_T;
        (*dp).df_lnum[idx_new as usize] = (*hunk).lnum_new;
        (*dp).df_count[idx_new as usize] = (*hunk).count_new as linenr_T;
        let mut i_1: ::core::ffi::c_int = idx_orig + 1 as ::core::ffi::c_int;
        while i_1 < idx_new {
            if !(*curtab.get()).tp_diffbuf[i_1 as usize].is_null() {
                diff_copy_entry(dprev, dp, idx_orig, i_1);
            }
            i_1 += 1;
        }
    }
    *notsetp = false_0 != 0;
    *dpp = dp;
    *dprevp = dprev;
}
unsafe extern "C" fn diff_read(
    mut idx_orig: ::core::ffi::c_int,
    mut idx_new: ::core::ffi::c_int,
    mut dio: *mut diffio_T,
) {
    let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut line_hunk_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut dprev: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
    let mut dout: *mut diffout_T = &raw mut (*dio).dio_diff;
    let mut notset: bool = true_0 != 0;
    let mut diffstyle: diffstyle_T = DIFF_NONE;
    if (*dio).dio_internal == 0 {
        fd = os_fopen(
            (*dout).dout_fname,
            b"r\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if fd.is_null() {
            emsg(gettext(
                b"E98: Cannot read diff output\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
    }
    loop {
        let mut hunk: diffhunk_T = diffhunk_T {
            lnum_orig: 0 as linenr_T,
            count_orig: 0,
            lnum_new: 0,
            count_new: 0,
        };
        let mut eof: bool = if (*dio).dio_internal != 0 {
            extract_hunk_internal(dout, &raw mut hunk, &raw mut line_hunk_idx) as ::core::ffi::c_int
        } else {
            extract_hunk(fd, &raw mut hunk, &raw mut diffstyle) as ::core::ffi::c_int
        } != 0;
        if eof {
            break;
        }
        process_hunk(
            &raw mut dp,
            &raw mut dprev,
            idx_orig,
            idx_new,
            &raw mut hunk,
            &raw mut notset,
        );
    }
    while !dp.is_null() {
        if notset {
            diff_copy_entry(dprev, dp, idx_orig, idx_new);
        }
        dprev = dp;
        dp = (*dp).df_next;
        notset = true_0 != 0;
    }
    if !fd.is_null() {
        fclose(fd);
    }
}
unsafe extern "C" fn diff_copy_entry(
    mut dprev: *mut diff_T,
    mut dp: *mut diff_T,
    mut idx_orig: ::core::ffi::c_int,
    mut idx_new: ::core::ffi::c_int,
) {
    let mut off: linenr_T = 0;
    if dprev.is_null() {
        off = 0 as ::core::ffi::c_int as linenr_T;
    } else {
        off = (*dprev).df_lnum[idx_orig as usize] + (*dprev).df_count[idx_orig as usize]
            - ((*dprev).df_lnum[idx_new as usize] + (*dprev).df_count[idx_new as usize]);
    }
    (*dp).df_lnum[idx_new as usize] = (*dp).df_lnum[idx_orig as usize] - off;
    (*dp).df_count[idx_new as usize] = (*dp).df_count[idx_orig as usize];
}
pub unsafe extern "C" fn diff_clear(mut tp: *mut tabpage_T) {
    let mut next_p: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut p: *mut diff_T = (*tp).tp_first_diff;
    while !p.is_null() {
        next_p = (*p).df_next;
        clear_diffblock(p);
        p = next_p;
    }
    (*tp).tp_first_diff = ::core::ptr::null_mut::<diff_T>();
}
pub unsafe extern "C" fn diff_linematch(mut dp: *mut diff_T) -> bool {
    if diff_flags.get() & DIFF_LINEMATCH == 0 {
        return false_0 != 0;
    }
    let mut tsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
            if (*dp).df_count[i as usize] < 0 as linenr_T {
                return false_0 != 0;
            }
            tsize += (*dp).df_count[i as usize] as ::core::ffi::c_int;
        }
        i += 1;
    }
    return tsize <= linematch_lines.get();
}
unsafe extern "C" fn get_max_diff_length(mut dp: *const diff_T) -> ::core::ffi::c_int {
    let mut maxlength: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while k < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[k as usize].is_null() {
            if (*dp).df_count[k as usize] > maxlength as linenr_T {
                maxlength = (*dp).df_count[k as usize] as ::core::ffi::c_int;
            }
        }
        k += 1;
    }
    return maxlength;
}
unsafe extern "C" fn find_top_diff_block(
    mut thistopdiff: *mut *mut diff_T,
    mut next_adjacent_blocks: *mut *mut diff_T,
    mut fromidx: ::core::ffi::c_int,
    mut topline: ::core::ffi::c_int,
) {
    let mut topdiff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut localtopdiff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut topdiffchange: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    topdiff = (*curtab.get()).tp_first_diff;
    while !topdiff.is_null() {
        if localtopdiff.is_null() || topdiffchange != 0 {
            localtopdiff = topdiff;
            topdiffchange = 0 as ::core::ffi::c_int;
        }
        if topline as linenr_T >= (*topdiff).df_lnum[fromidx as usize]
            && topline as linenr_T
                <= (*topdiff).df_lnum[fromidx as usize] + (*topdiff).df_count[fromidx as usize]
        {
            if (*thistopdiff).is_null() {
                *thistopdiff = localtopdiff;
            }
        }
        if !(!(*topdiff).df_next.is_null()
            && (*(*topdiff).df_next).df_lnum[fromidx as usize]
                == (*topdiff).df_lnum[fromidx as usize] + (*topdiff).df_count[fromidx as usize])
        {
            topdiffchange = 1 as ::core::ffi::c_int;
            if !(*thistopdiff).is_null() {
                *next_adjacent_blocks = (*topdiff).df_next;
                break;
            }
        }
        topdiff = (*topdiff).df_next;
    }
}
unsafe extern "C" fn calculate_topfill_and_topline(
    fromidx: ::core::ffi::c_int,
    toidx: ::core::ffi::c_int,
    from_topline: ::core::ffi::c_int,
    from_topfill: ::core::ffi::c_int,
    mut topfill: *mut ::core::ffi::c_int,
    mut topline: *mut linenr_T,
) {
    let mut thistopdiff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut next_adjacent_blocks: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut virtual_lines_passed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    find_top_diff_block(
        &raw mut thistopdiff,
        &raw mut next_adjacent_blocks,
        fromidx,
        from_topline,
    );
    let mut curdif: *mut diff_T = thistopdiff;
    while !curdif.is_null()
        && (*curdif).df_lnum[fromidx as usize] + (*curdif).df_count[fromidx as usize]
            <= from_topline as linenr_T
    {
        virtual_lines_passed += get_max_diff_length(curdif);
        curdif = (*curdif).df_next;
    }
    if curdif != next_adjacent_blocks {
        virtual_lines_passed +=
            (from_topline as linenr_T - (*curdif).df_lnum[fromidx as usize]) as ::core::ffi::c_int;
    }
    virtual_lines_passed -= from_topfill;
    if virtual_lines_passed < 0 as ::core::ffi::c_int {
        virtual_lines_passed = 0 as ::core::ffi::c_int;
    }
    let mut curlinenum_to: ::core::ffi::c_int = if !thistopdiff.is_null() {
        (*thistopdiff).df_lnum[toidx as usize] as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
    let mut virt_lines_left: ::core::ffi::c_int = virtual_lines_passed;
    curdif = thistopdiff;
    while virt_lines_left > 0 as ::core::ffi::c_int
        && !curdif.is_null()
        && curdif != next_adjacent_blocks
    {
        curlinenum_to += (if (virt_lines_left as linenr_T) < (*curdif).df_count[toidx as usize] {
            virt_lines_left as linenr_T
        } else {
            (*curdif).df_count[toidx as usize]
        }) as ::core::ffi::c_int;
        virt_lines_left -= if virt_lines_left < get_max_diff_length(curdif) {
            virt_lines_left
        } else {
            get_max_diff_length(curdif)
        };
        curdif = (*curdif).df_next;
    }
    let mut max_virt_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut dp: *mut diff_T = thistopdiff;
    while !dp.is_null() {
        if (*dp).df_lnum[toidx as usize] + (*dp).df_count[toidx as usize]
            <= curlinenum_to as linenr_T
        {
            max_virt_lines += get_max_diff_length(dp);
            dp = (*dp).df_next;
        } else {
            if (*dp).df_lnum[toidx as usize] <= curlinenum_to as linenr_T {
                max_virt_lines += (curlinenum_to as linenr_T - (*dp).df_lnum[toidx as usize])
                    as ::core::ffi::c_int;
            }
            break;
        }
    }
    if diff_flags.get() & DIFF_FILLER != 0 {
        *topfill = max_virt_lines - virtual_lines_passed;
    }
    *topline = curlinenum_to as linenr_T;
}
unsafe extern "C" fn apply_linematch_results(
    mut dp: *mut diff_T,
    mut decisions_length: size_t,
    mut decisions: *const ::core::ffi::c_int,
) {
    let mut line_numbers: [::core::ffi::c_int; 8] = [0; 8];
    let mut outputmap: [::core::ffi::c_int; 8] = [0; 8];
    let mut ndiffs: size_t = 0 as size_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
            line_numbers[i as usize] = (*dp).df_lnum[i as usize] as ::core::ffi::c_int;
            (*dp).df_count[i as usize] = 0 as ::core::ffi::c_int as linenr_T;
            outputmap[ndiffs as usize] = i;
            ndiffs = ndiffs.wrapping_add(1);
        }
        i += 1;
    }
    let mut dp_s: *mut diff_T = dp;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < decisions_length {
        if i_0 != 0 as size_t
            && *decisions.offset(i_0.wrapping_sub(1 as size_t) as isize)
                != *decisions.offset(i_0 as isize)
        {
            dp_s = diff_alloc_new(curtab.get(), dp_s, (*dp_s).df_next);
            (*dp_s).is_linematched = true_0 != 0;
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < DB_COUNT {
                if !(*curtab.get()).tp_diffbuf[j as usize].is_null() {
                    (*dp_s).df_lnum[j as usize] = line_numbers[j as usize] as linenr_T;
                    (*dp_s).df_count[j as usize] = 0 as ::core::ffi::c_int as linenr_T;
                }
                j += 1;
            }
        }
        let mut j_0: size_t = 0 as size_t;
        while j_0 < ndiffs {
            if *decisions.offset(i_0 as isize) & (1 as ::core::ffi::c_int) << j_0 != 0 {
                (*dp_s).df_count[outputmap[j_0 as usize] as usize] += 1;
                line_numbers[outputmap[j_0 as usize] as usize] += 1;
            }
            j_0 = j_0.wrapping_add(1);
        }
        i_0 = i_0.wrapping_add(1);
    }
    (*dp).is_linematched = true_0 != 0;
}
unsafe extern "C" fn run_linematch_algorithm(mut dp: *mut diff_T) {
    let mut diffbufs_mm: [mmfile_t; 8] = [mmfile_t {
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    }; 8];
    let mut diffbufs: [*const mmfile_t; 8] = [::core::ptr::null::<mmfile_t>(); 8];
    let mut diff_length: [::core::ffi::c_int; 8] = [0; 8];
    let mut ndiffs: size_t = 0 as size_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
            if (*dp).df_count[i as usize] > 0 as linenr_T {
                diff_write_buffer(
                    (*curtab.get()).tp_diffbuf[i as usize] as *mut buf_T,
                    (&raw mut diffbufs_mm as *mut mmfile_t).offset(ndiffs as isize),
                    (*dp).df_lnum[i as usize],
                    (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize] - 1 as linenr_T,
                );
            } else {
                diffbufs_mm[ndiffs as usize].size = 0 as ::core::ffi::c_int;
                diffbufs_mm[ndiffs as usize].ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            diffbufs[ndiffs as usize] =
                (&raw mut diffbufs_mm as *mut mmfile_t).offset(ndiffs as isize);
            diff_length[ndiffs as usize] = (*dp).df_count[i as usize] as ::core::ffi::c_int;
            ndiffs = ndiffs.wrapping_add(1);
        }
        i += 1;
    }
    let mut decisions: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let iwhite: bool = diff_flags.get() & (DIFF_IWHITEALL | DIFF_IWHITE) > 0 as ::core::ffi::c_int;
    let mut decisions_length: size_t = linematch_nbuffers(
        &raw mut diffbufs as *mut *const mmfile_t,
        &raw mut diff_length as *mut ::core::ffi::c_int,
        ndiffs,
        &raw mut decisions,
        iwhite,
    );
    let mut i_0: size_t = 0 as size_t;
    while i_0 < ndiffs {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*(&raw mut diffbufs_mm as *mut mmfile_t).offset(i_0 as isize)).ptr
                as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        i_0 = i_0.wrapping_add(1);
    }
    apply_linematch_results(dp, decisions_length, decisions);
    xfree(decisions as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn diff_check_with_linestatus(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut linestatus: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut buf: *mut buf_T = (*wp).w_buffer;
    if !linestatus.is_null() {
        *linestatus = 0 as ::core::ffi::c_int;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    if (*curtab.get()).tp_first_diff.is_null() || (*wp).w_onebuf_opt.wo_diff == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if lnum < 1 as linenr_T || lnum > (*buf).b_ml.ml_line_count + 1 as linenr_T {
        return 0 as ::core::ffi::c_int;
    }
    let mut idx: ::core::ffi::c_int = diff_buf_idx(buf, curtab.get());
    if idx == DB_COUNT {
        return 0 as ::core::ffi::c_int;
    }
    if hasFolding(
        wp,
        lnum,
        ::core::ptr::null_mut::<linenr_T>(),
        ::core::ptr::null_mut::<linenr_T>(),
    ) as ::core::ffi::c_int
        != 0
        || decor_conceal_line(
            wp,
            lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            false_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    dp = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if lnum <= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            break;
        }
        dp = (*dp).df_next;
    }
    if dp.is_null() || lnum < (*dp).df_lnum[idx as usize] {
        return 0 as ::core::ffi::c_int;
    }
    if lnum >= (*wp).w_topline
        && lnum < (*wp).w_botline
        && !(*dp).is_linematched
        && diff_linematch(dp) as ::core::ffi::c_int != 0
        && diff_check_sanity(curtab.get(), dp) != 0
    {
        run_linematch_algorithm(dp);
    }
    let mut num_fill: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lnum == (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
        if diff_flags.get() & DIFF_FILLER != 0 {
            let mut maxcount: ::core::ffi::c_int = get_max_diff_length(dp);
            num_fill += (maxcount as linenr_T - (*dp).df_count[idx as usize]) as ::core::ffi::c_int;
        }
        if !(!(*dp).df_next.is_null()
            && lnum >= (*(*dp).df_next).df_lnum[idx as usize]
            && lnum
                <= (*(*dp).df_next).df_lnum[idx as usize] + (*(*dp).df_next).df_count[idx as usize])
        {
            break;
        }
        dp = (*dp).df_next;
    }
    if lnum < (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
        let mut zero: bool = false_0 != 0;
        let mut cmp: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if i != idx && !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                if (*dp).df_count[i as usize] == 0 as linenr_T {
                    zero = true_0 != 0;
                } else {
                    if (*dp).df_count[i as usize] != (*dp).df_count[idx as usize] {
                        if !linestatus.is_null() {
                            *linestatus = -1 as ::core::ffi::c_int;
                        }
                        return num_fill;
                    }
                    cmp = true_0 != 0;
                }
            }
            i += 1;
        }
        if cmp {
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < DB_COUNT {
                if i_0 != idx
                    && !(*curtab.get()).tp_diffbuf[i_0 as usize].is_null()
                    && (*dp).df_count[i_0 as usize] != 0 as linenr_T
                {
                    if !diff_equal_entry(dp, idx, i_0) {
                        if !linestatus.is_null() {
                            *linestatus = -1 as ::core::ffi::c_int;
                        }
                        return num_fill;
                    }
                }
                i_0 += 1;
            }
        }
        if !zero {
            return num_fill;
        }
        if !linestatus.is_null() {
            *linestatus = -2 as ::core::ffi::c_int;
        }
        return num_fill;
    }
    return num_fill;
}
pub unsafe extern "C" fn diff_check_fill(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    if diff_flags.get() & DIFF_FILLER == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut n: ::core::ffi::c_int =
        diff_check_with_linestatus(wp, lnum, ::core::ptr::null_mut::<::core::ffi::c_int>());
    return if n > 0 as ::core::ffi::c_int {
        n
    } else {
        0 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn diff_equal_entry(
    mut dp: *mut diff_T,
    mut idx1: ::core::ffi::c_int,
    mut idx2: ::core::ffi::c_int,
) -> bool {
    if (*dp).df_count[idx1 as usize] != (*dp).df_count[idx2 as usize] {
        return false_0 != 0;
    }
    if diff_check_sanity(curtab.get(), dp) == FAIL {
        return false_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i as linenr_T) < (*dp).df_count[idx1 as usize] {
        let mut line: *mut ::core::ffi::c_char = xstrdup(ml_get_buf(
            (*curtab.get()).tp_diffbuf[idx1 as usize] as *mut buf_T,
            (*dp).df_lnum[idx1 as usize] + i as linenr_T,
        ));
        let mut cmp: ::core::ffi::c_int = diff_cmp(
            line,
            ml_get_buf(
                (*curtab.get()).tp_diffbuf[idx2 as usize] as *mut buf_T,
                (*dp).df_lnum[idx2 as usize] + i as linenr_T,
            ),
        );
        xfree(line as *mut ::core::ffi::c_void);
        if cmp != 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        i += 1;
    }
    return true_0 != 0;
}
unsafe extern "C" fn diff_equal_char(
    p1: *const ::core::ffi::c_char,
    p2: *const ::core::ffi::c_char,
    len: *mut ::core::ffi::c_int,
) -> bool {
    let l: ::core::ffi::c_int = utfc_ptr2len(p1);
    if l != utfc_ptr2len(p2) {
        return false_0 != 0;
    }
    if l > 1 as ::core::ffi::c_int {
        if strncmp(p1, p2, l as size_t) != 0 as ::core::ffi::c_int
            && (diff_flags.get() & DIFF_ICASE == 0
                || utf_fold(utf_ptr2char(p1)) != utf_fold(utf_ptr2char(p2)))
        {
            return false_0 != 0;
        }
        *len = l;
    } else {
        if *p1 as ::core::ffi::c_int != *p2 as ::core::ffi::c_int
            && (diff_flags.get() & DIFF_ICASE == 0
                || tolower(*p1 as uint8_t as ::core::ffi::c_int)
                    != tolower(*p2 as uint8_t as ::core::ffi::c_int))
        {
            return false_0 != 0;
        }
        *len = 1 as ::core::ffi::c_int;
    }
    return true_0 != 0;
}
unsafe extern "C" fn diff_cmp(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if diff_flags.get() & DIFF_IBLANK != 0
        && (*skipwhite(s1) as ::core::ffi::c_int == NUL
            || *skipwhite(s2) as ::core::ffi::c_int == NUL)
    {
        return 0 as ::core::ffi::c_int;
    }
    if diff_flags.get() & (DIFF_ICASE | ALL_WHITE_DIFF) == 0 as ::core::ffi::c_int {
        return strcmp(s1, s2);
    }
    if diff_flags.get() & DIFF_ICASE != 0 && diff_flags.get() & ALL_WHITE_DIFF == 0 {
        return mb_stricmp(s1, s2);
    }
    let mut p1: *mut ::core::ffi::c_char = s1;
    let mut p2: *mut ::core::ffi::c_char = s2;
    while *p1 as ::core::ffi::c_int != NUL && *p2 as ::core::ffi::c_int != NUL {
        if diff_flags.get() & DIFF_IWHITE != 0
            && ascii_iswhite(*p1 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            && ascii_iswhite(*p2 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || diff_flags.get() & DIFF_IWHITEALL != 0
                && (ascii_iswhite(*p1 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    || ascii_iswhite(*p2 as ::core::ffi::c_int) as ::core::ffi::c_int != 0)
        {
            p1 = skipwhite(p1);
            p2 = skipwhite(p2);
        } else {
            let mut l: ::core::ffi::c_int = 0;
            if !diff_equal_char(p1, p2, &raw mut l) {
                break;
            }
            p1 = p1.offset(l as isize);
            p2 = p2.offset(l as isize);
        }
    }
    p1 = skipwhite(p1);
    p2 = skipwhite(p2);
    if *p1 as ::core::ffi::c_int != NUL || *p2 as ::core::ffi::c_int != NUL {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn diff_set_topline(mut fromwin: *mut win_T, mut towin: *mut win_T) {
    let mut frombuf: *mut buf_T = (*fromwin).w_buffer;
    let mut fromidx: ::core::ffi::c_int = diff_buf_idx(frombuf, curtab.get());
    if fromidx == DB_COUNT {
        return;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    let mut lnum: linenr_T = (*fromwin).w_topline;
    (*towin).w_topfill = 0 as ::core::ffi::c_int;
    let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    dp = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if lnum <= (*dp).df_lnum[fromidx as usize] + (*dp).df_count[fromidx as usize] {
            break;
        }
        dp = (*dp).df_next;
    }
    if dp.is_null() {
        (*towin).w_topline =
            (*(*towin).w_buffer).b_ml.ml_line_count - ((*frombuf).b_ml.ml_line_count - lnum);
    } else {
        let mut toidx: ::core::ffi::c_int = diff_buf_idx((*towin).w_buffer, curtab.get());
        if toidx == DB_COUNT {
            return;
        }
        (*towin).w_topline =
            lnum + ((*dp).df_lnum[toidx as usize] - (*dp).df_lnum[fromidx as usize]);
        if lnum >= (*dp).df_lnum[fromidx as usize] {
            calculate_topfill_and_topline(
                fromidx,
                toidx,
                (*fromwin).w_topline as ::core::ffi::c_int,
                (*fromwin).w_topfill,
                &raw mut (*towin).w_topfill,
                &raw mut (*towin).w_topline,
            );
        }
    }
    (*towin).w_botfill = false_0 != 0;
    if (*towin).w_topline > (*(*towin).w_buffer).b_ml.ml_line_count {
        (*towin).w_topline = (*(*towin).w_buffer).b_ml.ml_line_count;
        (*towin).w_botfill = true_0 != 0;
    }
    if (*towin).w_topline < 1 as linenr_T {
        (*towin).w_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*towin).w_topfill = 0 as ::core::ffi::c_int;
    }
    invalidate_botline_win(towin);
    changed_line_abv_curs_win(towin);
    check_topfill(towin, false_0 != 0);
    hasFolding(
        towin,
        (*towin).w_topline,
        &raw mut (*towin).w_topline,
        ::core::ptr::null_mut::<linenr_T>(),
    );
}
unsafe extern "C" fn parse_diffanchors(
    mut check_only: bool,
    mut buf: *mut buf_T,
    mut anchors: *mut linenr_T,
    mut num_anchors: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut dia: *mut ::core::ffi::c_char = if *(*buf).b_p_dia as ::core::ffi::c_int == NUL {
        p_dia.get()
    } else {
        (*buf).b_p_dia
    };
    let mut orig_curbuf: *mut buf_T = curbuf.get();
    let mut orig_curwin: *mut win_T = curwin.get();
    let mut bufwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if check_only {
        bufwin = curwin.get();
    } else {
        bufwin = firstwin.get();
        while !bufwin.is_null() {
            if (*bufwin).w_buffer == buf && (*bufwin).w_onebuf_opt.wo_diff != 0 {
                break;
            }
            bufwin = (*bufwin).w_next;
        }
        if bufwin.is_null() && *dia as ::core::ffi::c_int != NUL {
            emsg(gettext(
                &raw const e_diff_anchors_with_hidden_windows as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
    }
    i = 0 as ::core::ffi::c_int;
    while i < MAX_DIFF_ANCHORS as ::core::ffi::c_int && *dia as ::core::ffi::c_int != NUL {
        if *dia as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            return FAIL;
        }
        curbuf.set(buf);
        curwin.set(bufwin);
        let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut lnum: linenr_T = get_address(
            ::core::ptr::null_mut::<exarg_T>(),
            &raw mut dia,
            ADDR_LINES,
            check_only,
            true_0 != 0,
            false_0,
            1 as ::core::ffi::c_int,
            &raw mut errormsg,
        );
        curbuf.set(orig_curbuf);
        curwin.set(orig_curwin);
        if !errormsg.is_null() {
            emsg(errormsg);
        }
        if dia.is_null() {
            return FAIL;
        }
        if *dia as ::core::ffi::c_int != ',' as ::core::ffi::c_int
            && *dia as ::core::ffi::c_int != NUL
        {
            return FAIL;
        }
        if !check_only
            && (lnum == MAXLNUM as ::core::ffi::c_int as linenr_T
                || lnum <= 0 as linenr_T
                || lnum > (*buf).b_ml.ml_line_count + 1 as linenr_T)
        {
            emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
            return FAIL;
        }
        if !anchors.is_null() {
            *anchors.offset(i as isize) = lnum;
        }
        if *dia as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            dia = dia.offset(1);
        }
        i += 1;
    }
    if i == MAX_DIFF_ANCHORS as ::core::ffi::c_int && *dia as ::core::ffi::c_int != NUL {
        semsg(
            gettext(
                &raw const e_cannot_have_more_than_nr_diff_anchors as *const ::core::ffi::c_char,
            ),
            MAX_DIFF_ANCHORS as ::core::ffi::c_int,
        );
        return FAIL;
    }
    if !num_anchors.is_null() {
        *num_anchors = i;
    }
    return OK;
}
pub unsafe extern "C" fn diffanchors_changed(mut buflocal: bool) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = parse_diffanchors(
        true_0 != 0,
        curbuf.get(),
        ::core::ptr::null_mut::<linenr_T>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    );
    if result == OK && diff_flags.get() & DIFF_ANCHOR != 0 {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if !buflocal {
                (*tp).tp_diff_invalid = true_0;
            } else {
                let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while idx < DB_COUNT {
                    if (*tp).tp_diffbuf[idx as usize] == curbuf.get() {
                        (*tp).tp_diff_invalid = true_0;
                        break;
                    } else {
                        idx += 1;
                    }
                }
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
    return result;
}
pub unsafe extern "C" fn diffopt_changed() -> ::core::ffi::c_int {
    let mut diff_context_new: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
    let mut linematch_lines_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut diff_flags_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut diff_foldcolumn_new: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    let mut diff_algorithm_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut diff_indent_heuristic: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = p_dip.get();
    while *p as ::core::ffi::c_int != NUL {
        if strncmp(
            p,
            b"filler\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(6 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_FILLER;
        } else if strncmp(
            p,
            b"anchor\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(6 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_ANCHOR;
        } else if strncmp(
            p,
            b"context:\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
            && ascii_isdigit(*p.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            p = p.offset(8 as ::core::ffi::c_int as isize);
            diff_context_new = getdigits_int(&raw mut p, false_0 != 0, diff_context_new);
        } else if strncmp(
            p,
            b"iblank\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(6 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_IBLANK;
        } else if strncmp(
            p,
            b"icase\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(5 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_ICASE;
        } else if strncmp(
            p,
            b"iwhiteall\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(9 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_IWHITEALL;
        } else if strncmp(
            p,
            b"iwhiteeol\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(9 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_IWHITEEOL;
        } else if strncmp(
            p,
            b"iwhite\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(6 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_IWHITE;
        } else if strncmp(
            p,
            b"horizontal\0".as_ptr() as *const ::core::ffi::c_char,
            10 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(10 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_HORIZONTAL;
        } else if strncmp(
            p,
            b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(8 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_VERTICAL;
        } else if strncmp(
            p,
            b"foldcolumn:\0".as_ptr() as *const ::core::ffi::c_char,
            11 as size_t,
        ) == 0 as ::core::ffi::c_int
            && ascii_isdigit(*p.offset(11 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            p = p.offset(11 as ::core::ffi::c_int as isize);
            diff_foldcolumn_new = getdigits_int(&raw mut p, false_0 != 0, diff_foldcolumn_new);
        } else if strncmp(
            p,
            b"hiddenoff\0".as_ptr() as *const ::core::ffi::c_char,
            9 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(9 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_HIDDEN_OFF;
        } else if strncmp(
            p,
            b"closeoff\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(8 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_CLOSE_OFF;
        } else if strncmp(
            p,
            b"followwrap\0".as_ptr() as *const ::core::ffi::c_char,
            10 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(10 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_FOLLOWWRAP;
        } else if strncmp(
            p,
            b"indent-heuristic\0".as_ptr() as *const ::core::ffi::c_char,
            16 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(16 as ::core::ffi::c_int as isize);
            diff_indent_heuristic = XDF_INDENT_HEURISTIC;
        } else if strncmp(
            p,
            b"internal\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(8 as ::core::ffi::c_int as isize);
            diff_flags_new |= DIFF_INTERNAL;
        } else if strncmp(
            p,
            b"algorithm:\0".as_ptr() as *const ::core::ffi::c_char,
            10 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(10 as ::core::ffi::c_int as isize);
            if strncmp(
                p,
                b"myers\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(5 as ::core::ffi::c_int as isize);
                diff_algorithm_new = 0 as ::core::ffi::c_int;
            } else if strncmp(
                p,
                b"minimal\0".as_ptr() as *const ::core::ffi::c_char,
                7 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(7 as ::core::ffi::c_int as isize);
                diff_algorithm_new = XDF_NEED_MINIMAL;
            } else if strncmp(
                p,
                b"patience\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(8 as ::core::ffi::c_int as isize);
                diff_algorithm_new = XDF_PATIENCE_DIFF;
            } else if strncmp(
                p,
                b"histogram\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(9 as ::core::ffi::c_int as isize);
                diff_algorithm_new = XDF_HISTOGRAM_DIFF;
            } else {
                return FAIL;
            }
        } else if strncmp(
            p,
            b"inline:\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(7 as ::core::ffi::c_int as isize);
            if strncmp(
                p,
                b"none\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(4 as ::core::ffi::c_int as isize);
                diff_flags_new &= !ALL_INLINE;
                diff_flags_new |= DIFF_INLINE_NONE;
            } else if strncmp(
                p,
                b"simple\0".as_ptr() as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(6 as ::core::ffi::c_int as isize);
                diff_flags_new &= !ALL_INLINE;
                diff_flags_new |= DIFF_INLINE_SIMPLE;
            } else if strncmp(
                p,
                b"char\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(4 as ::core::ffi::c_int as isize);
                diff_flags_new &= !ALL_INLINE;
                diff_flags_new |= DIFF_INLINE_CHAR;
            } else if strncmp(
                p,
                b"word\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(4 as ::core::ffi::c_int as isize);
                diff_flags_new &= !ALL_INLINE;
                diff_flags_new |= DIFF_INLINE_WORD;
            } else {
                return FAIL;
            }
        } else if strncmp(
            p,
            b"linematch:\0".as_ptr() as *const ::core::ffi::c_char,
            10 as size_t,
        ) == 0 as ::core::ffi::c_int
            && ascii_isdigit(*p.offset(10 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            p = p.offset(10 as ::core::ffi::c_int as isize);
            linematch_lines_new = getdigits_int(&raw mut p, false_0 != 0, linematch_lines_new);
            diff_flags_new |= DIFF_LINEMATCH;
            diff_flags_new |= DIFF_FILLER;
        }
        if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL
        {
            return FAIL;
        }
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
        }
    }
    diff_algorithm_new |= diff_indent_heuristic;
    if diff_flags_new & DIFF_HORIZONTAL != 0 && diff_flags_new & DIFF_VERTICAL != 0 {
        return FAIL;
    }
    if diff_flags.get() != diff_flags_new || diff_algorithm.get() != diff_algorithm_new {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            (*tp).tp_diff_invalid = true_0;
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
    diff_flags.set(diff_flags_new);
    diff_context.set(if diff_context_new == 0 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else {
        diff_context_new
    });
    linematch_lines.set(linematch_lines_new);
    diff_foldcolumn.set(diff_foldcolumn_new);
    diff_algorithm.set(diff_algorithm_new);
    diff_redraw(true_0 != 0);
    check_scrollbind(0 as linenr_T, 0 as ::core::ffi::c_int);
    return OK;
}
pub unsafe extern "C" fn diffopt_horizontal() -> bool {
    return diff_flags.get() & DIFF_HORIZONTAL != 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn diffopt_hiddenoff() -> bool {
    return diff_flags.get() & DIFF_HIDDEN_OFF != 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn diffopt_closeoff() -> bool {
    return diff_flags.get() & DIFF_CLOSE_OFF != 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn diffopt_filler() -> bool {
    return diff_flags.get() & DIFF_FILLER != 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn diff_update_line(mut lnum: linenr_T) {
    if diff_flags.get() & ALL_INLINE_DIFF == 0 {
        return;
    }
    let mut idx: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
    if idx == DB_COUNT {
        return;
    }
    let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    dp = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if lnum <= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            break;
        }
        dp = (*dp).df_next;
    }
    if !dp.is_null() {
        (*dp).has_changes = false_0 != 0;
        (*dp).df_changes.ga_len = 0 as ::core::ffi::c_int;
    }
}
static simple_diffline_change: GlobalCell<diffline_change_T> = GlobalCell::new(diffline_change_T {
    dc_start: [0; 8],
    dc_end: [0; 8],
    dc_start_lnum_off: [0; 8],
    dc_end_lnum_off: [0; 8],
});
pub unsafe extern "C" fn diff_change_parse(
    mut diffline: *mut diffline_T,
    mut change: *mut diffline_change_T,
    mut change_start: *mut ::core::ffi::c_int,
    mut change_end: *mut ::core::ffi::c_int,
) -> bool {
    if (*change).dc_start_lnum_off[(*diffline).bufidx as usize] < (*diffline).lineoff {
        *change_start = 0 as ::core::ffi::c_int;
    } else {
        *change_start = (*change).dc_start[(*diffline).bufidx as usize] as ::core::ffi::c_int;
    }
    if (*change).dc_end_lnum_off[(*diffline).bufidx as usize] > (*diffline).lineoff {
        *change_end = INT_MAX;
    } else {
        *change_end = (*change).dc_end[(*diffline).bufidx as usize] as ::core::ffi::c_int;
    }
    if change == simple_diffline_change.ptr() {
        return false_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if i != (*diffline).bufidx {
            if (*change).dc_start[i as usize] != (*change).dc_end[i as usize]
                || (*change).dc_end_lnum_off[i as usize] != (*change).dc_start_lnum_off[i as usize]
            {
                return false_0 != 0;
            }
        }
        i += 1;
    }
    return true_0 != 0;
}
unsafe extern "C" fn diff_find_change_simple(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut dp: *const diff_T,
    mut idx: ::core::ffi::c_int,
    mut startp: *mut ::core::ffi::c_int,
    mut endp: *mut ::core::ffi::c_int,
) -> bool {
    let mut line_org: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if diff_flags.get() & DIFF_INLINE_NONE != 0 {
        line_org = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        line_org = xstrdup(ml_get_buf((*wp).w_buffer, lnum));
    }
    let mut si_org: ::core::ffi::c_int = 0;
    let mut si_new: ::core::ffi::c_int = 0;
    let mut ei_org: ::core::ffi::c_int = 0;
    let mut ei_new: ::core::ffi::c_int = 0;
    let mut added: bool = true_0 != 0;
    let mut off: linenr_T = lnum - (*dp).df_lnum[idx as usize];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if !(*curtab.get()).tp_diffbuf[i as usize].is_null() && i != idx {
            if off < (*dp).df_count[i as usize] {
                added = false_0 != 0;
                if diff_flags.get() & DIFF_INLINE_NONE != 0 {
                    break;
                }
                let mut line_new: *mut ::core::ffi::c_char = ml_get_buf(
                    (*curtab.get()).tp_diffbuf[i as usize] as *mut buf_T,
                    (*dp).df_lnum[i as usize] + off,
                );
                si_new = 0 as ::core::ffi::c_int;
                si_org = si_new;
                while *line_org.offset(si_org as isize) as ::core::ffi::c_int != NUL {
                    if diff_flags.get() & DIFF_IWHITE != 0
                        && ascii_iswhite(*line_org.offset(si_org as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                        && ascii_iswhite(*line_new.offset(si_new as isize) as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                        || diff_flags.get() & DIFF_IWHITEALL != 0
                            && (ascii_iswhite(
                                *line_org.offset(si_org as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                                != 0
                                || ascii_iswhite(
                                    *line_new.offset(si_new as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0)
                    {
                        si_org = skipwhite(line_org.offset(si_org as isize)).offset_from(line_org)
                            as ::core::ffi::c_int;
                        si_new = skipwhite(line_new.offset(si_new as isize)).offset_from(line_new)
                            as ::core::ffi::c_int;
                    } else {
                        let mut l: ::core::ffi::c_int = 0;
                        if !diff_equal_char(
                            line_org.offset(si_org as isize),
                            line_new.offset(si_new as isize),
                            &raw mut l,
                        ) {
                            break;
                        }
                        si_org += l;
                        si_new += l;
                    }
                }
                si_org -= utf_head_off(line_org, line_org.offset(si_org as isize));
                si_new -= utf_head_off(line_new, line_new.offset(si_new as isize));
                *startp = if *startp < si_org { *startp } else { si_org };
                if *line_org.offset(si_org as isize) as ::core::ffi::c_int != NUL
                    || *line_new.offset(si_new as isize) as ::core::ffi::c_int != NUL
                {
                    ei_org = strlen(line_org) as ::core::ffi::c_int;
                    ei_new = strlen(line_new) as ::core::ffi::c_int;
                    while ei_org >= *startp
                        && ei_new >= si_new
                        && ei_org >= 0 as ::core::ffi::c_int
                        && ei_new >= 0 as ::core::ffi::c_int
                    {
                        if diff_flags.get() & DIFF_IWHITE != 0
                            && ascii_iswhite(*line_org.offset(ei_org as isize) as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                            && ascii_iswhite(*line_new.offset(ei_new as isize) as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                            || diff_flags.get() & DIFF_IWHITEALL != 0
                                && (ascii_iswhite(
                                    *line_org.offset(ei_org as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0
                                    || ascii_iswhite(
                                        *line_new.offset(ei_new as isize) as ::core::ffi::c_int
                                    ) as ::core::ffi::c_int
                                        != 0)
                        {
                            while ei_org >= *startp
                                && ascii_iswhite(
                                    *line_org.offset(ei_org as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                ei_org -= 1;
                            }
                            while ei_new >= si_new
                                && ascii_iswhite(
                                    *line_new.offset(ei_new as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                ei_new -= 1;
                            }
                        } else {
                            let mut p1: *const ::core::ffi::c_char =
                                line_org.offset(ei_org as isize);
                            let mut p2: *const ::core::ffi::c_char =
                                line_new.offset(ei_new as isize);
                            p1 = p1.offset(-(utf_head_off(line_org, p1) as isize));
                            p2 = p2.offset(-(utf_head_off(line_new, p2) as isize));
                            let mut l_0: ::core::ffi::c_int = 0;
                            if !diff_equal_char(p1, p2, &raw mut l_0) {
                                break;
                            }
                            ei_org -= l_0;
                            ei_new -= l_0;
                        }
                    }
                    *endp = if *endp > ei_org { *endp } else { ei_org };
                }
            }
        }
        i += 1;
    }
    xfree(line_org as *mut ::core::ffi::c_void);
    return added;
}
unsafe extern "C" fn diff_refine_inline_char_highlight(
    mut dp_orig: *mut diff_T,
    mut linemap: *mut garray_T,
    mut idx1: ::core::ffi::c_int,
) {
    let mut pass: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    loop {
        let mut has_unmerged_gaps: bool = false_0 != 0;
        let mut has_merged_gaps: bool = false_0 != 0;
        let mut dp: *mut diff_T = dp_orig;
        while !dp.is_null() && !(*dp).df_next.is_null() {
            if (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize] - 1 as linenr_T
                >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
                || (*(*dp).df_next).df_lnum[idx1 as usize] - 1 as linenr_T
                    >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
            {
                dp = (*dp).df_next;
            } else {
                let mut entry1: *mut linemap_entry_T =
                    ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                        (*(&raw mut (*dp).df_lnum as *mut linenr_T).offset(idx1 as isize)
                            + *(&raw mut (*dp).df_count as *mut linenr_T).offset(idx1 as isize)
                            - 1 as linenr_T) as isize,
                    );
                let mut entry2: *mut linemap_entry_T =
                    ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                        (*(&raw mut (*(*dp).df_next).df_lnum as *mut linenr_T)
                            .offset(idx1 as isize)
                            - 1 as linenr_T) as isize,
                    );
                if (*entry1).lineoff != (*entry2).lineoff {
                    dp = (*dp).df_next;
                } else {
                    let mut gap: linenr_T = (*(*dp).df_next).df_lnum[idx1 as usize]
                        - ((*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize]);
                    if gap <= 3 as linenr_T {
                        let mut max_df_count: linenr_T = 0 as linenr_T;
                        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while i < DB_COUNT {
                            max_df_count = if max_df_count
                                > (*dp).df_count[i as usize] + (*(*dp).df_next).df_count[i as usize]
                            {
                                max_df_count
                            } else {
                                (*dp).df_count[i as usize] + (*(*dp).df_next).df_count[i as usize]
                            };
                            i += 1;
                        }
                        if max_df_count >= gap * 4 as linenr_T {
                            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i_0 < DB_COUNT {
                                (*dp).df_count[i_0 as usize] = (*(*dp).df_next).df_lnum
                                    [i_0 as usize]
                                    + (*(*dp).df_next).df_count[i_0 as usize]
                                    - (*dp).df_lnum[i_0 as usize];
                                i_0 += 1;
                            }
                            let mut dp_next: *mut diff_T = (*dp).df_next;
                            (*dp).df_next = (*dp_next).df_next;
                            clear_diffblock(dp_next);
                            has_merged_gaps = true_0 != 0;
                            continue;
                        } else {
                            has_unmerged_gaps = true_0 != 0;
                        }
                    }
                    dp = (*dp).df_next;
                }
            }
        }
        if !has_unmerged_gaps || !has_merged_gaps {
            break;
        }
        let c2rust_fresh9 = pass;
        pass = pass + 1;
        if c2rust_fresh9 >= 4 as ::core::ffi::c_int {
            break;
        }
    }
}
unsafe extern "C" fn diff_refine_inline_word_highlight(
    mut dp_orig: *mut diff_T,
    mut linemap: *mut garray_T,
    mut idx1: ::core::ffi::c_int,
    mut start_lnum: linenr_T,
) {
    let mut pass: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    loop {
        let mut dp: *mut diff_T = dp_orig;
        while !dp.is_null() && !(*dp).df_next.is_null() {
            if (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize] - 1 as linenr_T
                >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
                || (*(*dp).df_next).df_lnum[idx1 as usize] - 1 as linenr_T
                    >= (*linemap.offset(idx1 as isize)).ga_len as linenr_T
            {
                dp = (*dp).df_next;
            } else {
                let mut entry1: *mut linemap_entry_T =
                    ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                        (*(&raw mut (*dp).df_lnum as *mut linenr_T).offset(idx1 as isize)
                            + *(&raw mut (*dp).df_count as *mut linenr_T).offset(idx1 as isize)
                            - 2 as linenr_T) as isize,
                    );
                let mut entry2: *mut linemap_entry_T =
                    ((*linemap.offset(idx1 as isize)).ga_data as *mut linemap_entry_T).offset(
                        (*(&raw mut (*(*dp).df_next).df_lnum as *mut linenr_T)
                            .offset(idx1 as isize)
                            - 1 as linenr_T) as isize,
                    );
                if (*entry1).lineoff != (*entry2).lineoff {
                    dp = (*dp).df_next;
                } else {
                    let mut gap_start: ::core::ffi::c_int = (*entry1).byte_start
                        as ::core::ffi::c_int
                        + (*entry1).num_bytes as ::core::ffi::c_int;
                    let mut gap_end: ::core::ffi::c_int =
                        (*entry2).byte_start as ::core::ffi::c_int;
                    let mut gap_size: ::core::ffi::c_int = gap_end - gap_start;
                    if gap_size <= 0 as ::core::ffi::c_int || gap_size > diff_word_gap.get() {
                        dp = (*dp).df_next;
                    } else {
                        let mut line: *mut ::core::ffi::c_char = ml_get_buf(
                            (*curtab.get()).tp_diffbuf[idx1 as usize] as *mut buf_T,
                            start_lnum + (*entry1).lineoff as linenr_T,
                        );
                        let mut gap_text: *mut ::core::ffi::c_char =
                            line.offset(gap_start as isize);
                        let mut only_non_word: bool = true_0 != 0;
                        let mut has_content: bool = false_0 != 0;
                        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while i < gap_size
                            && *gap_text.offset(i as isize) as ::core::ffi::c_int != NUL
                        {
                            has_content = true_0 != 0;
                            let mut char_class: ::core::ffi::c_int = mb_get_class_tab(
                                gap_text.offset(i as isize),
                                &raw mut (**(&raw mut (*curtab.get()).tp_diffbuf
                                    as *mut *mut buf_T)
                                    .offset(idx1 as isize))
                                .b_chartab as *mut uint64_t,
                            );
                            if char_class == 2 as ::core::ffi::c_int {
                                only_non_word = false_0 != 0;
                                break;
                            } else {
                                i += 1;
                            }
                        }
                        if has_content as ::core::ffi::c_int != 0
                            && only_non_word as ::core::ffi::c_int != 0
                        {
                            let mut total_change_bytes: ::core::ffi::c_long =
                                0 as ::core::ffi::c_long;
                            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i_0 < DB_COUNT {
                                if !(*curtab.get()).tp_diffbuf[i_0 as usize].is_null() {
                                    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while (k as linenr_T) < (*dp).df_count[i_0 as usize] {
                                        let mut idx: ::core::ffi::c_int =
                                            (*dp).df_lnum[i_0 as usize] as ::core::ffi::c_int + k
                                                - 1 as ::core::ffi::c_int;
                                        if idx < (*linemap.offset(i_0 as isize)).ga_len {
                                            total_change_bytes +=
                                                (*((*linemap.offset(i_0 as isize)).ga_data
                                                    as *mut linemap_entry_T)
                                                    .offset(idx as isize))
                                                .num_bytes
                                                    as ::core::ffi::c_long;
                                        }
                                        k += 1;
                                    }
                                    let mut k_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while (k_0 as linenr_T)
                                        < (*(*dp).df_next).df_count[i_0 as usize]
                                    {
                                        let mut idx_0: ::core::ffi::c_int = (*(*dp).df_next).df_lnum
                                            [i_0 as usize]
                                            as ::core::ffi::c_int
                                            + k_0
                                            - 1 as ::core::ffi::c_int;
                                        if idx_0 < (*linemap.offset(i_0 as isize)).ga_len {
                                            total_change_bytes +=
                                                (*((*linemap.offset(i_0 as isize)).ga_data
                                                    as *mut linemap_entry_T)
                                                    .offset(idx_0 as isize))
                                                .num_bytes
                                                    as ::core::ffi::c_long;
                                        }
                                        k_0 += 1;
                                    }
                                }
                                i_0 += 1;
                            }
                            if total_change_bytes
                                >= (gap_size * 2 as ::core::ffi::c_int) as ::core::ffi::c_long
                            {
                                let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                while i_1 < DB_COUNT {
                                    if !(*curtab.get()).tp_diffbuf[i_1 as usize].is_null() {
                                        (*dp).df_count[i_1 as usize] = (*(*dp).df_next).df_lnum
                                            [i_1 as usize]
                                            + (*(*dp).df_next).df_count[i_1 as usize]
                                            - (*dp).df_lnum[i_1 as usize];
                                    }
                                    i_1 += 1;
                                }
                                let mut dp_next: *mut diff_T = (*dp).df_next;
                                (*dp).df_next = (*dp_next).df_next;
                                clear_diffblock(dp_next);
                                continue;
                            }
                        }
                        dp = (*dp).df_next;
                    }
                }
            }
        }
        let c2rust_fresh10 = pass;
        pass = pass + 1;
        if c2rust_fresh10 >= 4 as ::core::ffi::c_int {
            break;
        }
    }
}
unsafe extern "C" fn diff_find_change_inline_diff(mut dp: *mut diff_T) {
    let mut new_diff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let save_diff_algorithm: ::core::ffi::c_int = diff_algorithm.get();
    let mut dio: diffio_T = diffio_T {
        dio_orig: diffin_T {
            din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            din_mmfile: mmfile_t {
                ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        },
        dio_new: diffin_T {
            din_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            din_mmfile: mmfile_t {
                ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
        },
        dio_diff: diffout_T {
            dout_fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            dout_ga: garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
        },
        dio_internal: 0,
    };
    ga_init(
        &raw mut dio.dio_diff.dout_ga,
        ::core::mem::size_of::<diffhunk_T>() as ::core::ffi::c_int,
        1000 as ::core::ffi::c_int,
    );
    dio.dio_internal = true_0;
    (*diff_algorithm.ptr()) |= XDF_INDENT_HEURISTIC;
    let mut orig_diff: *mut diff_T = (*curtab.get()).tp_first_diff;
    (*curtab.get()).tp_first_diff = ::core::ptr::null_mut::<diff_T>();
    let mut orig_diffbuf: [*mut buf_T; 8] = [::core::ptr::null_mut::<buf_T>(); 8];
    memcpy(
        &raw mut orig_diffbuf as *mut *mut buf_T as *mut ::core::ffi::c_void,
        &raw mut (*curtab.get()).tp_diffbuf as *mut *mut buf_T as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[*mut buf_T; 8]>(),
    );
    let mut linemap: [garray_T; 8] = [garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    }; 8];
    let mut file1_str: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut file2_str: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut file1_str,
        1 as ::core::ffi::c_int,
        1024 as ::core::ffi::c_int,
    );
    ga_init(
        &raw mut file2_str,
        1 as ::core::ffi::c_int,
        1024 as ::core::ffi::c_int,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        ga_init(
            (&raw mut linemap as *mut garray_T).offset(i as isize),
            ::core::mem::size_of::<linemap_entry_T>() as ::core::ffi::c_int,
            128 as ::core::ffi::c_int,
        );
        i += 1;
    }
    let mut file1_idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_done: {
        while i_0 < DB_COUNT {
            dio.dio_diff.dout_ga.ga_len = 0 as ::core::ffi::c_int;
            let mut buf: *mut buf_T = (*curtab.get()).tp_diffbuf[i_0 as usize] as *mut buf_T;
            if !(buf.is_null() || (*buf).b_ml.ml_mfp.is_null()) {
                if (*dp).df_count[i_0 as usize] == 0 as linenr_T {
                    (*curtab.get()).tp_diffbuf[i_0 as usize] = ::core::ptr::null_mut::<buf_T>();
                } else {
                    if file1_idx == -1 as ::core::ffi::c_int {
                        file1_idx = i_0;
                    }
                    let mut curstr: *mut garray_T = if file1_idx != i_0 {
                        &raw mut file2_str
                    } else {
                        &raw mut file1_str
                    };
                    let mut numlines: linenr_T = 0 as linenr_T;
                    (*curstr).ga_len = 0 as ::core::ffi::c_int;
                    let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while (off as linenr_T) < (*dp).df_count[i_0 as usize] {
                        let mut curline: *mut ::core::ffi::c_char = ml_get_buf(
                            (*curtab.get()).tp_diffbuf[i_0 as usize] as *mut buf_T,
                            (*dp).df_lnum[i_0 as usize] + off as linenr_T,
                        );
                        let mut in_keyword: bool = false_0 != 0;
                        let mut last_white: bool = false_0 != 0;
                        let mut eol_ga_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                        let mut eol_linemap_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                        let mut eol_numlines: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                        let mut s: *mut ::core::ffi::c_char = curline;
                        while *s as ::core::ffi::c_int != NUL {
                            let mut new_in_keyword: bool = false_0 != 0;
                            if diff_flags.get() & DIFF_INLINE_WORD != 0 {
                                new_in_keyword = mb_get_class_tab(
                                    s,
                                    &raw mut (**(&raw mut (*curtab.get()).tp_diffbuf
                                        as *mut *mut buf_T)
                                        .offset(file1_idx as isize))
                                    .b_chartab as *mut uint64_t,
                                ) == 2 as ::core::ffi::c_int;
                            }
                            if in_keyword as ::core::ffi::c_int != 0 && !new_in_keyword {
                                ga_append(curstr, NL as uint8_t);
                                numlines += 1;
                            }
                            if ascii_iswhite(*s as ::core::ffi::c_int) {
                                if diff_flags.get() & DIFF_IWHITEALL != 0 {
                                    in_keyword = false_0 != 0;
                                    s = skipwhite(s);
                                    continue;
                                } else if diff_flags.get() & DIFF_IWHITEEOL != 0
                                    || diff_flags.get() & DIFF_IWHITE != 0
                                {
                                    if !last_white {
                                        eol_ga_len = (*curstr).ga_len;
                                        eol_linemap_len = linemap[i_0 as usize].ga_len;
                                        eol_numlines = numlines as ::core::ffi::c_int;
                                        last_white = true_0 != 0;
                                    }
                                }
                            } else if diff_flags.get() & DIFF_IWHITEEOL != 0
                                || diff_flags.get() & DIFF_IWHITE != 0
                            {
                                last_white = false_0 != 0;
                                eol_ga_len = -1 as ::core::ffi::c_int;
                                eol_linemap_len = -1 as ::core::ffi::c_int;
                                eol_numlines = -1 as ::core::ffi::c_int;
                            }
                            let mut char_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            if *s as ::core::ffi::c_int == NL {
                                ga_append(curstr, NUL as uint8_t);
                            } else {
                                char_len = utfc_ptr2len(s);
                                if ascii_iswhite(*s as ::core::ffi::c_int) as ::core::ffi::c_int
                                    != 0
                                    && diff_flags.get() & DIFF_IWHITE != 0
                                {
                                    char_len = skipwhite(s).offset_from(s) as ::core::ffi::c_int;
                                }
                                if diff_flags.get() & DIFF_ICASE != 0 {
                                    let mut c: ::core::ffi::c_int = utf_ptr2char(s);
                                    let mut c_len: ::core::ffi::c_int = utf_char2len(c);
                                    c = utf_fold(c);
                                    let mut cbuf: [::core::ffi::c_char; 22] = [0; 22];
                                    let mut c_fold_len: ::core::ffi::c_int = utf_char2bytes(
                                        c,
                                        &raw mut cbuf as *mut ::core::ffi::c_char,
                                    );
                                    ga_concat_len(
                                        curstr,
                                        &raw mut cbuf as *mut ::core::ffi::c_char,
                                        c_fold_len as size_t,
                                    );
                                    if char_len > c_len {
                                        ga_concat_len(
                                            curstr,
                                            s.offset(c_len as isize),
                                            (char_len - c_len) as size_t,
                                        );
                                    }
                                } else {
                                    ga_concat_len(curstr, s, char_len as size_t);
                                }
                            }
                            if !new_in_keyword {
                                ga_append(curstr, NL as uint8_t);
                                numlines += 1;
                            }
                            if !new_in_keyword
                                || new_in_keyword as ::core::ffi::c_int != 0 && !in_keyword
                            {
                                let mut linemap_entry: linemap_entry_T = linemap_entry_T {
                                    byte_start: s.offset_from(curline) as colnr_T,
                                    num_bytes: char_len as colnr_T,
                                    lineoff: off,
                                };
                                ga_grow(
                                    (&raw mut linemap as *mut garray_T).offset(i_0 as isize),
                                    1 as ::core::ffi::c_int,
                                );
                                *(linemap[i_0 as usize].ga_data as *mut linemap_entry_T)
                                    .offset(linemap[i_0 as usize].ga_len as isize) = linemap_entry;
                                linemap[i_0 as usize].ga_len += 1;
                            } else {
                                (*(linemap[i_0 as usize].ga_data as *mut linemap_entry_T)
                                    .offset(
                                        (linemap[i_0 as usize].ga_len - 1 as ::core::ffi::c_int)
                                            as isize,
                                    ))
                                .num_bytes += char_len;
                            }
                            in_keyword = new_in_keyword;
                            s = s.offset(char_len as isize);
                        }
                        if in_keyword {
                            ga_append(curstr, NL as uint8_t);
                            numlines += 1;
                        }
                        if diff_flags.get() & DIFF_IWHITEEOL != 0
                            || diff_flags.get() & DIFF_IWHITE != 0
                        {
                            if eol_ga_len != -1 as ::core::ffi::c_int {
                                (*curstr).ga_len = eol_ga_len;
                                linemap[i_0 as usize].ga_len = eol_linemap_len;
                                numlines = eol_numlines as linenr_T;
                            }
                        }
                        if diff_flags.get() & DIFF_IWHITEALL == 0 {
                            ga_append(curstr, NL as uint8_t);
                            numlines += 1;
                            let mut linemap_entry_0: linemap_entry_T = linemap_entry_T {
                                byte_start: s.offset_from(curline) as colnr_T,
                                num_bytes: ::core::mem::size_of::<::core::ffi::c_int>() as colnr_T,
                                lineoff: off,
                            };
                            ga_grow(
                                (&raw mut linemap as *mut garray_T).offset(i_0 as isize),
                                1 as ::core::ffi::c_int,
                            );
                            *(linemap[i_0 as usize].ga_data as *mut linemap_entry_T)
                                .offset(linemap[i_0 as usize].ga_len as isize) = linemap_entry_0;
                            linemap[i_0 as usize].ga_len += 1;
                        }
                        off += 1;
                    }
                    if file1_idx != i_0 {
                        dio.dio_new.din_mmfile.ptr = (*curstr).ga_data as *mut ::core::ffi::c_char;
                        dio.dio_new.din_mmfile.size = (*curstr).ga_len;
                    } else {
                        dio.dio_orig.din_mmfile.ptr = (*curstr).ga_data as *mut ::core::ffi::c_char;
                        dio.dio_orig.din_mmfile.size = (*curstr).ga_len;
                    }
                    if file1_idx != i_0 {
                        let mut diff_status: ::core::ffi::c_int = diff_file_internal(&raw mut dio);
                        if diff_status == FAIL {
                            break '_done;
                        }
                        diff_read(0 as ::core::ffi::c_int, i_0, &raw mut dio);
                        clear_diffout(&raw mut dio.dio_diff);
                    }
                }
            }
            i_0 += 1;
        }
        new_diff = (*curtab.get()).tp_first_diff;
        if diff_flags.get() & DIFF_INLINE_WORD != 0 && file1_idx != -1 as ::core::ffi::c_int {
            diff_refine_inline_word_highlight(
                new_diff,
                &raw mut linemap as *mut garray_T,
                file1_idx,
                (*dp).df_lnum[file1_idx as usize],
            );
        } else if diff_flags.get() & DIFF_INLINE_CHAR != 0 && file1_idx != -1 as ::core::ffi::c_int
        {
            diff_refine_inline_char_highlight(
                new_diff,
                &raw mut linemap as *mut garray_T,
                file1_idx,
            );
        }
        (*dp).df_changes.ga_len = 0 as ::core::ffi::c_int;
        while !new_diff.is_null() {
            let mut change: diffline_change_T = diffline_change_S {
                dc_start: [0 as colnr_T; 8],
                dc_end: [0; 8],
                dc_start_lnum_off: [0; 8],
                dc_end_lnum_off: [0; 8],
            };
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < DB_COUNT {
                if (*new_diff).df_lnum[i_1 as usize] > 0 as linenr_T {
                    let mut diff_lnum: linenr_T = (*new_diff).df_lnum[i_1 as usize] - 1 as linenr_T;
                    let mut diff_lnum_end: linenr_T =
                        diff_lnum + (*new_diff).df_count[i_1 as usize];
                    if diff_lnum >= linemap[i_1 as usize].ga_len as linenr_T {
                        change.dc_start[i_1 as usize] = MAXCOL as ::core::ffi::c_int as colnr_T;
                        change.dc_start_lnum_off[i_1 as usize] = INT_MAX;
                    } else {
                        change.dc_start[i_1 as usize] = (*(linemap[i_1 as usize].ga_data
                            as *mut linemap_entry_T)
                            .offset(diff_lnum as isize))
                        .byte_start;
                        change.dc_start_lnum_off[i_1 as usize] = (*(linemap[i_1 as usize].ga_data
                            as *mut linemap_entry_T)
                            .offset(diff_lnum as isize))
                        .lineoff;
                    }
                    if diff_lnum == diff_lnum_end {
                        change.dc_end[i_1 as usize] = change.dc_start[i_1 as usize];
                        change.dc_end_lnum_off[i_1 as usize] =
                            change.dc_start_lnum_off[i_1 as usize];
                    } else if diff_lnum_end - 1 as linenr_T
                        >= linemap[i_1 as usize].ga_len as linenr_T
                    {
                        change.dc_end[i_1 as usize] = MAXCOL as ::core::ffi::c_int as colnr_T;
                        change.dc_end_lnum_off[i_1 as usize] = INT_MAX;
                    } else {
                        change.dc_end[i_1 as usize] = (*(linemap[i_1 as usize].ga_data
                            as *mut linemap_entry_T)
                            .offset((diff_lnum_end - 1 as linenr_T) as isize))
                        .byte_start
                            + (*(linemap[i_1 as usize].ga_data as *mut linemap_entry_T)
                                .offset((diff_lnum_end - 1 as linenr_T) as isize))
                            .num_bytes;
                        change.dc_end_lnum_off[i_1 as usize] = (*(linemap[i_1 as usize].ga_data
                            as *mut linemap_entry_T)
                            .offset((diff_lnum_end - 1 as linenr_T) as isize))
                        .lineoff;
                    }
                }
                i_1 += 1;
            }
            ga_grow(&raw mut (*dp).df_changes, 1 as ::core::ffi::c_int);
            *((*dp).df_changes.ga_data as *mut diffline_change_T)
                .offset((*dp).df_changes.ga_len as isize) = change;
            (*dp).df_changes.ga_len += 1;
            new_diff = (*new_diff).df_next;
        }
    }
    diff_algorithm.set(save_diff_algorithm);
    (*dp).has_changes = true_0 != 0;
    diff_clear(curtab.get());
    (*curtab.get()).tp_first_diff = orig_diff;
    memcpy(
        &raw mut (*curtab.get()).tp_diffbuf as *mut *mut buf_T as *mut ::core::ffi::c_void,
        &raw mut orig_diffbuf as *mut *mut buf_T as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[*mut buf_T; 8]>(),
    );
    ga_clear(&raw mut file1_str);
    ga_clear(&raw mut file2_str);
    clear_diffout(&raw mut dio.dio_diff);
    let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_2 < DB_COUNT {
        ga_clear((&raw mut linemap as *mut garray_T).offset(i_2 as isize));
        i_2 += 1;
    }
}
pub unsafe extern "C" fn diff_find_change(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut diffline: *mut diffline_T,
) -> bool {
    let mut idx: ::core::ffi::c_int = diff_buf_idx((*wp).w_buffer, curtab.get());
    if idx == DB_COUNT {
        return false_0 != 0;
    }
    let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    dp = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if lnum < (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            break;
        }
        dp = (*dp).df_next;
    }
    if dp.is_null() || diff_check_sanity(curtab.get(), dp) == FAIL {
        return false_0 != 0;
    }
    let mut off: ::core::ffi::c_int =
        lnum as ::core::ffi::c_int - (*dp).df_lnum[idx as usize] as ::core::ffi::c_int;
    if diff_flags.get() & ALL_INLINE_DIFF == 0 {
        let mut change_start: ::core::ffi::c_int = MAXCOL as ::core::ffi::c_int;
        let mut change_end: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut ret: ::core::ffi::c_int = diff_find_change_simple(
            wp,
            lnum,
            dp,
            idx,
            &raw mut change_start,
            &raw mut change_end,
        ) as ::core::ffi::c_int;
        change_end += 1 as ::core::ffi::c_int;
        memset(
            simple_diffline_change.ptr() as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<diffline_change_T>(),
        );
        (*diffline).changes = simple_diffline_change.ptr();
        (*diffline).num_changes = 1 as ::core::ffi::c_int;
        (*diffline).bufidx = idx;
        (*diffline).lineoff = (lnum - (*dp).df_lnum[idx as usize]) as ::core::ffi::c_int;
        (*simple_diffline_change.ptr()).dc_start[idx as usize] = change_start as colnr_T;
        (*simple_diffline_change.ptr()).dc_end[idx as usize] = change_end as colnr_T;
        (*simple_diffline_change.ptr()).dc_start_lnum_off[idx as usize] = off;
        (*simple_diffline_change.ptr()).dc_end_lnum_off[idx as usize] = off;
        return ret != 0;
    }
    if !(*dp).has_changes {
        diff_find_change_inline_diff(dp);
    }
    let mut changes: *mut garray_T = &raw mut (*dp).df_changes;
    let mut num_changes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut change_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*diffline).changes = ::core::ptr::null_mut::<diffline_change_T>();
    change_idx = 0 as ::core::ffi::c_int;
    while change_idx < (*changes).ga_len {
        let mut change: *mut diffline_change_T =
            ((*dp).df_changes.ga_data as *mut diffline_change_T).offset(change_idx as isize);
        if (*change).dc_end_lnum_off[idx as usize] >= off {
            if (*change).dc_start_lnum_off[idx as usize] > off {
                break;
            }
            if (*diffline).changes.is_null() {
                (*diffline).changes = change;
            }
            num_changes += 1;
        }
        change_idx += 1;
    }
    (*diffline).num_changes = num_changes;
    (*diffline).bufidx = idx;
    (*diffline).lineoff = off;
    let mut added: bool = false_0 != 0;
    if num_changes == 1 as ::core::ffi::c_int && change_idx == (*dp).df_changes.ga_len {
        added = true_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if idx != i {
                if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                    let mut change_0: *mut diffline_change_T = ((*dp).df_changes.ga_data
                        as *mut diffline_change_T)
                        .offset(((*dp).df_changes.ga_len - 1 as ::core::ffi::c_int) as isize);
                    if (*change_0).dc_start_lnum_off[i as usize] != INT_MAX {
                        added = false_0 != 0;
                        break;
                    }
                }
            }
            i += 1;
        }
    }
    return added;
}
pub unsafe extern "C" fn diff_infold(mut wp: *mut win_T, mut lnum: linenr_T) -> bool {
    if (*wp).w_onebuf_opt.wo_diff == 0 {
        return false_0 != 0;
    }
    let mut idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut other: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < DB_COUNT {
        if (*curtab.get()).tp_diffbuf[i as usize] == (*wp).w_buffer {
            idx = i;
        } else if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
            other = true_0 != 0;
        }
        i += 1;
    }
    if idx == -1 as ::core::ffi::c_int || !other {
        return false_0 != 0;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    if (*curtab.get()).tp_first_diff.is_null() {
        return true_0 != 0;
    }
    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if (*dp).df_lnum[idx as usize] - diff_context.get() as linenr_T > lnum {
            break;
        }
        if (*dp).df_lnum[idx as usize]
            + (*dp).df_count[idx as usize]
            + diff_context.get() as linenr_T
            > lnum
        {
            return false_0 != 0;
        }
        dp = (*dp).df_next;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn nv_diffgetput(mut put: bool, mut count: size_t) {
    if bt_prompt(curbuf.get()) {
        vim_beep(kOptBoFlagOperator as ::core::ffi::c_int as ::core::ffi::c_uint);
        return;
    }
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    let mut buf: [::core::ffi::c_char; 30] = [0; 30];
    if count == 0 as size_t {
        ea.arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else {
        vim_snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
            b"%zu\0".as_ptr() as *const ::core::ffi::c_char,
            count,
        );
        ea.arg = &raw mut buf as *mut ::core::ffi::c_char;
    }
    if put {
        ea.cmdidx = CMD_diffput;
    } else {
        ea.cmdidx = CMD_diffget;
    }
    ea.addr_count = 0 as ::core::ffi::c_int;
    ea.line1 = (*curwin.get()).w_cursor.lnum;
    ea.line2 = (*curwin.get()).w_cursor.lnum;
    ex_diffgetput(&raw mut ea);
}
unsafe extern "C" fn valid_diff(mut diff: *mut diff_T) -> bool {
    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if dp == diff {
            return true_0 != 0;
        }
        dp = (*dp).df_next;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn ex_diffgetput(mut eap: *mut exarg_T) {
    let mut idx_other: ::core::ffi::c_int = 0;
    let mut idx_cur: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
    if idx_cur == DB_COUNT {
        emsg(gettext(
            b"E99: Current buffer is not in diff mode\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return;
    }
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        let mut found_not_ma: bool = false_0 != 0;
        idx_other = 0 as ::core::ffi::c_int;
        while idx_other < DB_COUNT {
            if (*curtab.get()).tp_diffbuf[idx_other as usize] != curbuf.get()
                && !(*curtab.get()).tp_diffbuf[idx_other as usize].is_null()
            {
                if (*eap).cmdidx as ::core::ffi::c_int != CMD_diffput as ::core::ffi::c_int
                    || (*(*curtab.get()).tp_diffbuf[idx_other as usize]).b_p_ma != 0
                {
                    break;
                }
                found_not_ma = true_0 != 0;
            }
            idx_other += 1;
        }
        if idx_other == DB_COUNT {
            if found_not_ma {
                emsg(gettext(
                    b"E793: No other buffer in diff mode is modifiable\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            } else {
                emsg(gettext(
                    b"E100: No other buffer in diff mode\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            return;
        }
        let mut i: ::core::ffi::c_int = idx_other + 1 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if (*curtab.get()).tp_diffbuf[i as usize] != curbuf.get()
                && !(*curtab.get()).tp_diffbuf[i as usize].is_null()
                && ((*eap).cmdidx as ::core::ffi::c_int != CMD_diffput as ::core::ffi::c_int
                    || (*(*curtab.get()).tp_diffbuf[i as usize]).b_p_ma != 0)
            {
                emsg(gettext(
                    b"E101: More than two buffers in diff mode, don't know which one to use\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                ));
                return;
            }
            i += 1;
        }
    } else {
        let mut p: *mut ::core::ffi::c_char = (*eap).arg.offset(strlen((*eap).arg) as isize);
        while p > (*eap).arg
            && ascii_iswhite(*p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            p = p.offset(-1);
        }
        let mut i_0: ::core::ffi::c_int = 0;
        i_0 = 0 as ::core::ffi::c_int;
        while ascii_isdigit(*(*eap).arg.offset(i_0 as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
            && (*eap).arg.offset(i_0 as isize) < p
        {
            i_0 += 1;
        }
        if (*eap).arg.offset(i_0 as isize) == p {
            i_0 = atol((*eap).arg) as ::core::ffi::c_int;
        } else {
            i_0 = buflist_findpat((*eap).arg, p, false_0 != 0, true_0 != 0, false_0 != 0);
            if i_0 < 0 as ::core::ffi::c_int {
                return;
            }
        }
        let mut buf: *mut buf_T = buflist_findnr(i_0);
        if buf.is_null() {
            semsg(
                gettext(b"E102: Can't find buffer \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                (*eap).arg,
            );
            return;
        }
        if buf == curbuf.get() {
            return;
        }
        idx_other = diff_buf_idx(buf, curtab.get());
        if idx_other == DB_COUNT {
            semsg(
                gettext(b"E103: Buffer \"%s\" is not in diff mode\0".as_ptr()
                    as *const ::core::ffi::c_char),
                (*eap).arg,
            );
            return;
        }
    }
    diff_busy.set(true_0 != 0);
    if (*eap).addr_count == 0 as ::core::ffi::c_int {
        let mut linestatus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*eap).line1 == (*curbuf.get()).b_ml.ml_line_count
            && (diff_check_with_linestatus(curwin.get(), (*eap).line1, &raw mut linestatus)
                == 0 as ::core::ffi::c_int
                && linestatus == 0 as ::core::ffi::c_int)
            && ((*eap).line1 == 1 as linenr_T
                || diff_check_with_linestatus(
                    curwin.get(),
                    (*eap).line1 - 1 as linenr_T,
                    &raw mut linestatus,
                ) >= 0 as ::core::ffi::c_int
                    && linestatus == 0 as ::core::ffi::c_int)
        {
            (*eap).line2 += 1;
        } else if (*eap).line1 > 0 as linenr_T {
            (*eap).line1 -= 1;
        }
    }
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_diffget as ::core::ffi::c_int {
        aucmd_prepbuf(
            &raw mut aco,
            (*curtab.get()).tp_diffbuf[idx_other as usize] as *mut buf_T,
        );
    }
    let idx_from: ::core::ffi::c_int =
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_diffget as ::core::ffi::c_int {
            idx_other
        } else {
            idx_cur
        };
    let idx_to: ::core::ffi::c_int =
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_diffget as ::core::ffi::c_int {
            idx_cur
        } else {
            idx_other
        };
    '_theend: {
        if (*curbuf.get()).b_changed == 0 {
            change_warning(curbuf.get(), 0 as ::core::ffi::c_int);
            if diff_buf_idx(curbuf.get(), curtab.get()) != idx_to {
                emsg(gettext(
                    b"E787: Buffer changed unexpectedly\0".as_ptr() as *const ::core::ffi::c_char
                ));
                break '_theend;
            }
        }
        diffgetput(
            (*eap).addr_count,
            idx_cur,
            idx_from,
            idx_to,
            (*eap).line1,
            (*eap).line2,
        );
        if (*eap).cmdidx as ::core::ffi::c_int != CMD_diffget as ::core::ffi::c_int {
            if KeyTyped.get() {
                u_sync(false_0 != 0);
            }
            aucmd_restbuf(&raw mut aco);
        }
    }
    diff_busy.set(false_0 != 0);
    if diff_need_update.get() {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    check_cursor(curwin.get());
    changed_line_abv_curs();
    if (*curtab.get()).tp_first_diff.is_null() {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_diff != 0
                && *(*wp)
                    .w_onebuf_opt
                    .wo_fdm
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == 'd' as ::core::ffi::c_int
                && (*wp).w_onebuf_opt.wo_fen != 0
            {
                foldUpdateAll(wp);
            }
            wp = (*wp).w_next;
        }
    }
    if diff_need_update.get() {
        diff_need_update.set(false_0 != 0);
    } else {
        diff_redraw(false_0 != 0);
        apply_autocmds(
            EVENT_DIFFUPDATED,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    };
}
unsafe extern "C" fn diffgetput(
    addr_count: ::core::ffi::c_int,
    idx_cur: ::core::ffi::c_int,
    idx_from: ::core::ffi::c_int,
    idx_to: ::core::ffi::c_int,
    line1: linenr_T,
    line2: linenr_T,
) {
    let mut off: linenr_T = 0 as linenr_T;
    let mut dprev: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if addr_count == 0 {
            while !(*dp).df_next.is_null()
                && (*(*dp).df_next).df_lnum[idx_cur as usize]
                    == (*dp).df_lnum[idx_cur as usize] + (*dp).df_count[idx_cur as usize]
                && (*(*dp).df_next).df_lnum[idx_cur as usize] == line1 + off + 1 as linenr_T
            {
                dprev = dp;
                dp = (*dp).df_next;
            }
        }
        if (*dp).df_lnum[idx_cur as usize] > line2 + off {
            break;
        }
        let mut dfree: diff_T = diffblock_S {
            df_next: ::core::ptr::null_mut::<diff_T>(),
            df_lnum: [0; 8],
            df_count: [0; 8],
            is_linematched: false,
            has_changes: false,
            df_changes: garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
        };
        let mut did_free: bool = false_0 != 0;
        let mut lnum: linenr_T = (*dp).df_lnum[idx_to as usize];
        let mut count: linenr_T = (*dp).df_count[idx_to as usize];
        if (*dp).df_lnum[idx_cur as usize] + (*dp).df_count[idx_cur as usize] > line1 + off
            && u_save(lnum - 1 as linenr_T, lnum + count) != FAIL
        {
            let mut start_skip: linenr_T = 0 as linenr_T;
            let mut end_skip: linenr_T = 0 as linenr_T;
            if addr_count > 0 as ::core::ffi::c_int {
                start_skip = line1 + off - (*dp).df_lnum[idx_cur as usize];
                if start_skip > 0 as linenr_T {
                    if start_skip > count {
                        lnum += count;
                        count = 0 as ::core::ffi::c_int as linenr_T;
                    } else {
                        count -= start_skip;
                        lnum += start_skip;
                    }
                } else {
                    start_skip = 0 as ::core::ffi::c_int as linenr_T;
                }
                end_skip = (*dp).df_lnum[idx_cur as usize] + (*dp).df_count[idx_cur as usize]
                    - 1 as linenr_T
                    - (line2 + off);
                if end_skip > 0 as linenr_T {
                    if idx_cur == idx_from {
                        count = if count < (*dp).df_count[idx_cur as usize] - start_skip - end_skip
                        {
                            count
                        } else {
                            (*dp).df_count[idx_cur as usize] - start_skip - end_skip
                        };
                    } else {
                        count -= end_skip;
                        end_skip = if (*dp).df_count[idx_from as usize] - start_skip - count
                            > 0 as linenr_T
                        {
                            (*dp).df_count[idx_from as usize] - start_skip - count
                        } else {
                            0 as linenr_T
                        };
                    }
                } else {
                    end_skip = 0 as ::core::ffi::c_int as linenr_T;
                }
            }
            let mut buf_empty: bool = buf_is_empty(curbuf.get());
            let mut added: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while (i as linenr_T) < count {
                buf_empty = (*curbuf.get()).b_ml.ml_line_count == 1 as linenr_T;
                if ml_delete(lnum) == OK {
                    added -= 1;
                }
                i += 1;
            }
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while (i_0 as linenr_T) < (*dp).df_count[idx_from as usize] - start_skip - end_skip {
                let mut nr: linenr_T =
                    (*dp).df_lnum[idx_from as usize] + start_skip + i_0 as linenr_T;
                if nr
                    > (*(*curtab.get()).tp_diffbuf[idx_from as usize])
                        .b_ml
                        .ml_line_count
                {
                    break;
                }
                let mut p: *mut ::core::ffi::c_char = xstrdup(ml_get_buf(
                    (*curtab.get()).tp_diffbuf[idx_from as usize] as *mut buf_T,
                    nr,
                ));
                ml_append(
                    lnum + i_0 as linenr_T - 1 as linenr_T,
                    p,
                    0 as colnr_T,
                    false_0 != 0,
                );
                xfree(p as *mut ::core::ffi::c_void);
                added += 1;
                if buf_empty as ::core::ffi::c_int != 0
                    && (*curbuf.get()).b_ml.ml_line_count == 2 as linenr_T
                {
                    buf_empty = false_0 != 0;
                    ml_delete(2 as linenr_T);
                }
                i_0 += 1;
            }
            let mut new_count: linenr_T = (*dp).df_count[idx_to as usize] + added as linenr_T;
            (*dp).df_count[idx_to as usize] = new_count;
            if start_skip == 0 as linenr_T && end_skip == 0 as linenr_T {
                let mut i_1: ::core::ffi::c_int = 0;
                i_1 = 0 as ::core::ffi::c_int;
                while i_1 < DB_COUNT {
                    if !(*curtab.get()).tp_diffbuf[i_1 as usize].is_null()
                        && i_1 != idx_from
                        && i_1 != idx_to
                        && !diff_equal_entry(dp, idx_from, i_1)
                    {
                        break;
                    }
                    i_1 += 1;
                }
                if i_1 == DB_COUNT {
                    dfree = *dp;
                    did_free = true_0 != 0;
                    dp = diff_free(curtab.get(), dprev, dp);
                }
            }
            if added != 0 as ::core::ffi::c_int {
                mark_adjust(
                    lnum,
                    lnum + count - 1 as linenr_T,
                    MAXLNUM as ::core::ffi::c_int as linenr_T,
                    added as linenr_T,
                    kExtmarkNOOP,
                );
                if (*curwin.get()).w_cursor.lnum >= lnum {
                    if (*curwin.get()).w_cursor.lnum >= lnum + count {
                        (*curwin.get()).w_cursor.lnum =
                            ((*curwin.get()).w_cursor.lnum as ::core::ffi::c_int + added)
                                as linenr_T;
                        (*curwin.get()).w_cursor.lnum =
                            if (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count {
                                (*curwin.get()).w_cursor.lnum
                            } else {
                                (*curbuf.get()).b_ml.ml_line_count
                            };
                    } else if added < 0 as ::core::ffi::c_int {
                        (*curwin.get()).w_cursor.lnum = lnum;
                    }
                }
            }
            extmark_adjust(
                curbuf.get(),
                lnum,
                lnum + count - 1 as linenr_T,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                added as linenr_T,
                kExtmarkUndo,
            );
            changed_lines(
                curbuf.get(),
                lnum,
                0 as colnr_T,
                lnum + count,
                added as linenr_T,
                true_0 != 0,
            );
            if did_free {
                diff_fold_update(&raw mut dfree, idx_to);
            }
            if added != 0 as ::core::ffi::c_int && !valid_diff(dp) {
                break;
            }
            if !did_free {
                (*dp).df_count[idx_to as usize] = new_count;
            }
            if idx_cur == idx_to {
                off = (off as ::core::ffi::c_int + added) as linenr_T;
            }
        }
        if !did_free {
            dprev = dp;
            dp = (*dp).df_next;
        }
    }
}
unsafe extern "C" fn diff_fold_update(mut dp: *mut diff_T, mut skip_idx: ::core::ffi::c_int) {
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if (*curtab.get()).tp_diffbuf[i as usize] == (*wp).w_buffer && i != skip_idx {
                foldUpdate(
                    wp,
                    (*dp).df_lnum[i as usize],
                    (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize],
                );
            }
            i += 1;
        }
        wp = (*wp).w_next;
    }
}
pub unsafe extern "C" fn diff_mode_buf(mut buf: *mut buf_T) -> bool {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        if diff_buf_idx(buf, tp as *mut tabpage_T) != DB_COUNT {
            return true_0 != 0;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn diff_move_to(
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut idx: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
    if idx == DB_COUNT || (*curtab.get()).tp_first_diff.is_null() {
        return FAIL;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    if (*curtab.get()).tp_first_diff.is_null() {
        return FAIL;
    }
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if dir == BACKWARD as ::core::ffi::c_int
            && lnum <= (*(*curtab.get()).tp_first_diff).df_lnum[idx as usize]
        {
            break;
        }
        let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        dp = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if dir == FORWARD as ::core::ffi::c_int && lnum < (*dp).df_lnum[idx as usize]
                || dir == BACKWARD as ::core::ffi::c_int
                    && ((*dp).df_next.is_null() || lnum <= (*(*dp).df_next).df_lnum[idx as usize])
            {
                lnum = (*dp).df_lnum[idx as usize];
                break;
            } else {
                dp = (*dp).df_next;
            }
        }
    }
    lnum = if lnum < (*curbuf.get()).b_ml.ml_line_count {
        lnum
    } else {
        (*curbuf.get()).b_ml.ml_line_count
    };
    if lnum == (*curwin.get()).w_cursor.lnum {
        return FAIL;
    }
    setpcmark();
    (*curwin.get()).w_cursor.lnum = lnum;
    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
    return OK;
}
unsafe extern "C" fn diff_get_corresponding_line_int(
    mut buf1: *mut buf_T,
    mut lnum1: linenr_T,
) -> linenr_T {
    let mut baseline: linenr_T = 0 as linenr_T;
    let mut idx1: ::core::ffi::c_int = diff_buf_idx(buf1, curtab.get());
    let mut idx2: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
    if idx1 == DB_COUNT || idx2 == DB_COUNT || (*curtab.get()).tp_first_diff.is_null() {
        return lnum1;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    if (*curtab.get()).tp_first_diff.is_null() {
        return lnum1;
    }
    let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if (*dp).df_lnum[idx1 as usize] > lnum1 {
            return lnum1 - baseline;
        }
        if (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize] > lnum1 {
            baseline = lnum1 - (*dp).df_lnum[idx1 as usize];
            baseline = if baseline < (*dp).df_count[idx2 as usize] {
                baseline
            } else {
                (*dp).df_count[idx2 as usize]
            };
            return (*dp).df_lnum[idx2 as usize] + baseline;
        }
        if (*dp).df_lnum[idx1 as usize] == lnum1
            && (*dp).df_count[idx1 as usize] == 0 as linenr_T
            && (*dp).df_lnum[idx2 as usize] <= (*curwin.get()).w_cursor.lnum
            && (*dp).df_lnum[idx2 as usize] + (*dp).df_count[idx2 as usize]
                > (*curwin.get()).w_cursor.lnum
        {
            return (*curwin.get()).w_cursor.lnum;
        }
        baseline = (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize]
            - ((*dp).df_lnum[idx2 as usize] + (*dp).df_count[idx2 as usize]);
        dp = (*dp).df_next;
    }
    return lnum1 - baseline;
}
pub unsafe extern "C" fn diff_get_corresponding_line(
    mut buf1: *mut buf_T,
    mut lnum1: linenr_T,
) -> linenr_T {
    let mut lnum: linenr_T = diff_get_corresponding_line_int(buf1, lnum1);
    return if lnum < (*curbuf.get()).b_ml.ml_line_count {
        lnum
    } else {
        (*curbuf.get()).b_ml.ml_line_count
    };
}
pub unsafe extern "C" fn diff_lnum_win(mut lnum: linenr_T, mut wp: *mut win_T) -> linenr_T {
    let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
    let mut idx: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
    if idx == DB_COUNT {
        return 0 as linenr_T;
    }
    if (*curtab.get()).tp_diff_invalid != 0 {
        ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
    }
    dp = (*curtab.get()).tp_first_diff;
    while !dp.is_null() {
        if lnum <= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            break;
        }
        dp = (*dp).df_next;
    }
    if dp.is_null() {
        return (*(*wp).w_buffer).b_ml.ml_line_count - ((*curbuf.get()).b_ml.ml_line_count - lnum);
    }
    let mut i: ::core::ffi::c_int = diff_buf_idx((*wp).w_buffer, curtab.get());
    if i == DB_COUNT {
        return 0 as linenr_T;
    }
    let mut n: linenr_T = lnum + ((*dp).df_lnum[i as usize] - (*dp).df_lnum[idx as usize]);
    return if n < (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize] {
        n
    } else {
        (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize]
    };
}
unsafe extern "C" fn parse_diff_ed(
    mut line: *mut ::core::ffi::c_char,
    mut hunk: *mut diffhunk_T,
) -> ::core::ffi::c_int {
    let mut l1: ::core::ffi::c_int = 0;
    let mut l2: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = line;
    let mut f1: linenr_T = getdigits_int32(&raw mut p, true_0 != 0, 0 as int32_t);
    if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        p = p.offset(1);
        l1 = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
    } else {
        l1 = f1 as ::core::ffi::c_int;
    }
    if *p as ::core::ffi::c_int != 'a' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != 'c' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != 'd' as ::core::ffi::c_int
    {
        return FAIL;
    }
    let c2rust_fresh6 = p;
    p = p.offset(1);
    let mut difftype: ::core::ffi::c_int = *c2rust_fresh6 as uint8_t as ::core::ffi::c_int;
    let mut f2: ::core::ffi::c_int =
        getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
    if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        p = p.offset(1);
        l2 = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
    } else {
        l2 = f2;
    }
    if (l1 as linenr_T) < f1 || l2 < f2 {
        return FAIL;
    }
    if difftype == 'a' as ::core::ffi::c_int {
        (*hunk).lnum_orig = f1 + 1 as linenr_T;
        (*hunk).count_orig = 0 as ::core::ffi::c_int;
    } else {
        (*hunk).lnum_orig = f1;
        (*hunk).count_orig = (l1 as linenr_T - f1 + 1 as linenr_T) as ::core::ffi::c_int;
    }
    if difftype == 'd' as ::core::ffi::c_int {
        (*hunk).lnum_new = f2 as linenr_T + 1 as linenr_T;
        (*hunk).count_new = 0 as ::core::ffi::c_int;
    } else {
        (*hunk).lnum_new = f2 as linenr_T;
        (*hunk).count_new = l2 - f2 + 1 as ::core::ffi::c_int;
    }
    return OK;
}
unsafe extern "C" fn parse_diff_unified(
    mut line: *mut ::core::ffi::c_char,
    mut hunk: *mut diffhunk_T,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = line;
    let c2rust_fresh0 = p;
    p = p.offset(1);
    if *c2rust_fresh0 as ::core::ffi::c_int == '@' as ::core::ffi::c_int
        && {
            let c2rust_fresh1 = p;
            p = p.offset(1);
            *c2rust_fresh1 as ::core::ffi::c_int == '@' as ::core::ffi::c_int
        }
        && {
            let c2rust_fresh2 = p;
            p = p.offset(1);
            *c2rust_fresh2 as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
        }
        && {
            let c2rust_fresh3 = p;
            p = p.offset(1);
            *c2rust_fresh3 as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        }
    {
        let mut oldcount: ::core::ffi::c_int = 0;
        let mut newline: linenr_T = 0;
        let mut newcount: ::core::ffi::c_int = 0;
        let mut oldline: linenr_T = getdigits_int32(&raw mut p, true_0 != 0, 0 as int32_t);
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            p = p.offset(1);
            oldcount = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
        } else {
            oldcount = 1 as ::core::ffi::c_int;
        }
        let c2rust_fresh4 = p;
        p = p.offset(1);
        if *c2rust_fresh4 as ::core::ffi::c_int == ' ' as ::core::ffi::c_int && {
            let c2rust_fresh5 = p;
            p = p.offset(1);
            *c2rust_fresh5 as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        } {
            newline = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int) as linenr_T;
            if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                p = p.offset(1);
                newcount = getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int);
            } else {
                newcount = 1 as ::core::ffi::c_int;
            }
        } else {
            return FAIL;
        }
        if oldcount == 0 as ::core::ffi::c_int {
            oldline = (oldline as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as linenr_T;
        }
        if newcount == 0 as ::core::ffi::c_int {
            newline = (newline as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as linenr_T;
        }
        if newline == 0 as linenr_T {
            newline = 1 as ::core::ffi::c_int as linenr_T;
        }
        (*hunk).lnum_orig = oldline;
        (*hunk).count_orig = oldcount;
        (*hunk).lnum_new = newline;
        (*hunk).count_new = newcount;
        return OK;
    }
    return FAIL;
}
unsafe extern "C" fn xdiff_out(
    mut start_a: ::core::ffi::c_int,
    mut count_a: ::core::ffi::c_int,
    mut start_b: ::core::ffi::c_int,
    mut count_b: ::core::ffi::c_int,
    mut priv_0: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut dout: *mut diffout_T = priv_0 as *mut diffout_T;
    ga_grow(&raw mut (*dout).dout_ga, 1 as ::core::ffi::c_int);
    *((*dout).dout_ga.ga_data as *mut diffhunk_T).offset((*dout).dout_ga.ga_len as isize) =
        diffhunk_T {
            lnum_orig: start_a as linenr_T + 1 as linenr_T,
            count_orig: count_a,
            lnum_new: start_b as linenr_T + 1 as linenr_T,
            count_new: count_b,
        };
    (*dout).dout_ga.ga_len += 1;
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn f_diff_filler(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number =
        (if 0 as ::core::ffi::c_int > diff_check_fill(curwin.get(), tv_get_lnum(argvars)) {
            0 as ::core::ffi::c_int
        } else {
            diff_check_fill(curwin.get(), tv_get_lnum(argvars))
        }) as varnumber_T;
}
pub unsafe extern "C" fn f_diff_hlID(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    static prev_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
    static changedtick: GlobalCell<varnumber_T> = GlobalCell::new(0 as varnumber_T);
    static fnum: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static prev_diff_flags: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static change_start: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static change_end: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    static hlID: GlobalCell<hlf_T> = GlobalCell::new(HLF_NONE);
    let mut diffline: diffline_T = diffline_S {
        changes: ::core::ptr::null_mut::<diffline_change_T>(),
        num_changes: 0,
        bufidx: 0,
        lineoff: 0,
    };
    let cache_results: bool = diff_flags.get() & ALL_INLINE_DIFF == 0;
    let mut lnum: linenr_T = tv_get_lnum(argvars);
    if lnum < 0 as linenr_T {
        lnum = 0 as ::core::ffi::c_int as linenr_T;
    }
    if !cache_results
        || lnum != prev_lnum.get()
        || changedtick.get() != buf_get_changedtick(curbuf.get())
        || fnum.get() != (*curbuf.get()).handle
        || diff_flags.get() != prev_diff_flags.get()
    {
        let mut linestatus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        diff_check_with_linestatus(curwin.get(), lnum, &raw mut linestatus);
        if linestatus < 0 as ::core::ffi::c_int {
            if linestatus == -1 as ::core::ffi::c_int {
                change_start.set(MAXCOL as ::core::ffi::c_int);
                change_end.set(-1 as ::core::ffi::c_int);
                if diff_find_change(curwin.get(), lnum, &raw mut diffline) {
                    hlID.set(HLF_ADD);
                } else {
                    hlID.set(HLF_CHD);
                    if diffline.num_changes > 0 as ::core::ffi::c_int
                        && cache_results as ::core::ffi::c_int != 0
                    {
                        change_start.set(
                            (*diffline.changes.offset(0 as ::core::ffi::c_int as isize)).dc_start
                                [diffline.bufidx as usize]
                                as ::core::ffi::c_int,
                        );
                        change_end.set(
                            (*diffline.changes.offset(0 as ::core::ffi::c_int as isize)).dc_end
                                [diffline.bufidx as usize]
                                as ::core::ffi::c_int,
                        );
                    }
                }
            } else {
                hlID.set(HLF_ADD);
            }
        } else {
            hlID.set(HLF_NONE);
        }
        if cache_results {
            prev_lnum.set(lnum);
            changedtick.set(buf_get_changedtick(curbuf.get()));
            fnum.set((*curbuf.get()).handle as ::core::ffi::c_int);
            prev_diff_flags.set(diff_flags.get());
        }
    }
    if hlID.get() as ::core::ffi::c_uint == HLF_CHD as ::core::ffi::c_int as ::core::ffi::c_uint
        || hlID.get() as ::core::ffi::c_uint == HLF_TXD as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut col: ::core::ffi::c_int =
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int;
        if cache_results {
            if col >= change_start.get() && col < change_end.get() {
                hlID.set(HLF_TXD);
            } else {
                hlID.set(HLF_CHD);
            }
        } else {
            hlID.set(HLF_CHD);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < diffline.num_changes {
                let mut added: bool = diff_change_parse(
                    &raw mut diffline,
                    diffline.changes.offset(i as isize),
                    change_start.ptr(),
                    change_end.ptr(),
                );
                if col >= change_start.get() && col < change_end.get() {
                    hlID.set(
                        (if added as ::core::ffi::c_int != 0 {
                            HLF_TXA as ::core::ffi::c_int
                        } else {
                            HLF_TXD as ::core::ffi::c_int
                        }) as hlf_T,
                    );
                    break;
                } else {
                    if col < change_start.get() {
                        break;
                    }
                    i += 1;
                }
            }
        }
    }
    (*rettv).vval.v_number = hlID.get() as varnumber_T;
}
pub const XDF_NEED_MINIMAL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE_CHANGE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE_AT_EOL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int;
pub const XDF_IGNORE_BLANK_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const XDF_PATIENCE_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
pub const XDF_HISTOGRAM_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int;
pub const XDF_INDENT_HEURISTIC: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 23 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
