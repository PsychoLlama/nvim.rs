use crate::src::nvim::api::private::helpers::{
    cbuf_to_string, copy_string, cstr_as_string, cstr_to_string,
};
use crate::src::nvim::autocmd::{apply_autocmds, has_event};
use crate::src::nvim::charset::{getdigits_int32, getdigits_long, skipwhite, vim_isfilec};
use crate::src::nvim::cursor::get_cursor_line_ptr;
use crate::src::nvim::eval::typval::{
    tv_dict_add_bool, tv_dict_add_str, tv_dict_set_keys_readonly,
};
use crate::src::nvim::eval::vars::set_vim_var_string;
use crate::src::nvim::eval_1::{eval_to_string_safe, get_v_event, restore_v_event};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    curbuf, current_sctx, curwin, e_cant_find_directory_str_in_cdpath,
    e_cant_find_file_str_in_path, e_no_more_directory_str_found_in_cdpath,
    e_no_more_file_str_found_in_path, got_int, line_msg, p_cdpath, p_cpo, p_fic, p_path, NameBuff,
    VIsual_active,
};
use crate::src::nvim::mbyte::{mb_tolower, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemcpyz, xmemdupz, xrealloc, xstrlcpy};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::normal::get_visual_text;
use crate::src::nvim::option::{copy_option_part, was_set_insecurely};
use crate::src::nvim::os::env::expand_env_esc;
use crate::src::nvim::os::fs::{
    os_chdir, os_dirname, os_fileid, os_fileid_equal, os_isdir, os_path_exists,
};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, abort, gettext, memmove, snprintf, strcpy, strlen, strncmp,
    strtol,
};
use crate::src::nvim::path::{
    after_pathsep, expand_wildcards, path_fnamecmp, path_fnamencmp, path_has_drive_letter,
    path_is_url, path_shorten_fname, path_tail, path_tail_with_sep, path_with_url, pathcmp,
    simplify_filename, vim_isAbsName, vim_ispathsep, FreeWild, FullName_save,
};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, xstrnsave};
pub use crate::src::nvim::types::{
    __time_t, alist_T, auto_event, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T,
    chunksize_T, cmdarg_T, colnr_T, dict_T, dictvar_S, disptick_T, event_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, oparg_T, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t,
    qf_info_S, qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T,
    save_v_event_T, schar_T, scid_T, sctx_T, size_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, taggy_T, terminal,
    time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, AdditionalData, AlignTextPos, Arena,
    BoolVarValue, BufUpdateCallbacks, Callback, CallbackType, Callback_data as C2Rust_Unnamed_5,
    CdCause, CdScope, ChangedtickDictItem, DecorExt, DecorHighlightInline, DecorInlineData,
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, ExtmarkUndoObject,
    FileID, FloatAnchor, FloatRelative, GridView, Intersection, LuaRef, MTKey, MTNode, MTPos,
    MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t,
    MarkTree, MotionType, OptIndex, OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t,
    Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, String_0, Terminal, Timestamp, VarLockStatus,
    VarType, VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo,
    WinSplit, WinStyle, Window, QUEUE,
};
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
pub const kCdScopeGlobal: CdScope = 2;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdScopeInvalid: CdScope = -1;
pub const kCdCauseAuto: CdCause = 2;
pub const kCdCauseWindow: CdCause = 1;
pub const kCdCauseManual: CdCause = 0;
pub const kCdCauseOther: CdCause = -1;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const kBufOptWrapmargin: C2Rust_Unnamed_13 = 91;
pub const kBufOptVartabstop: C2Rust_Unnamed_13 = 90;
pub const kBufOptVarsofttabstop: C2Rust_Unnamed_13 = 89;
pub const kBufOptUndolevels: C2Rust_Unnamed_13 = 88;
pub const kBufOptUndofile: C2Rust_Unnamed_13 = 87;
pub const kBufOptThesaurusfunc: C2Rust_Unnamed_13 = 86;
pub const kBufOptThesaurus: C2Rust_Unnamed_13 = 85;
pub const kBufOptTextwidth: C2Rust_Unnamed_13 = 84;
pub const kBufOptTags: C2Rust_Unnamed_13 = 83;
pub const kBufOptTagfunc: C2Rust_Unnamed_13 = 82;
pub const kBufOptTagcase: C2Rust_Unnamed_13 = 81;
pub const kBufOptTabstop: C2Rust_Unnamed_13 = 80;
pub const kBufOptSyntax: C2Rust_Unnamed_13 = 79;
pub const kBufOptSynmaxcol: C2Rust_Unnamed_13 = 78;
pub const kBufOptSwapfile: C2Rust_Unnamed_13 = 77;
pub const kBufOptSuffixesadd: C2Rust_Unnamed_13 = 76;
pub const kBufOptSpelloptions: C2Rust_Unnamed_13 = 75;
pub const kBufOptSpelllang: C2Rust_Unnamed_13 = 74;
pub const kBufOptSpellfile: C2Rust_Unnamed_13 = 73;
pub const kBufOptSpellcapcheck: C2Rust_Unnamed_13 = 72;
pub const kBufOptSofttabstop: C2Rust_Unnamed_13 = 71;
pub const kBufOptSmartindent: C2Rust_Unnamed_13 = 70;
pub const kBufOptShiftwidth: C2Rust_Unnamed_13 = 69;
pub const kBufOptScrollback: C2Rust_Unnamed_13 = 68;
pub const kBufOptReadonly: C2Rust_Unnamed_13 = 67;
pub const kBufOptQuoteescape: C2Rust_Unnamed_13 = 66;
pub const kBufOptPreserveindent: C2Rust_Unnamed_13 = 65;
pub const kBufOptPath: C2Rust_Unnamed_13 = 64;
pub const kBufOptOmnifunc: C2Rust_Unnamed_13 = 63;
pub const kBufOptNrformats: C2Rust_Unnamed_13 = 62;
pub const kBufOptModified: C2Rust_Unnamed_13 = 61;
pub const kBufOptModifiable: C2Rust_Unnamed_13 = 60;
pub const kBufOptModeline: C2Rust_Unnamed_13 = 59;
pub const kBufOptMatchpairs: C2Rust_Unnamed_13 = 58;
pub const kBufOptMakeprg: C2Rust_Unnamed_13 = 57;
pub const kBufOptMakeencoding: C2Rust_Unnamed_13 = 56;
pub const kBufOptLispwords: C2Rust_Unnamed_13 = 55;
pub const kBufOptLispoptions: C2Rust_Unnamed_13 = 54;
pub const kBufOptLisp: C2Rust_Unnamed_13 = 53;
pub const kBufOptKeywordprg: C2Rust_Unnamed_13 = 52;
pub const kBufOptKeymap: C2Rust_Unnamed_13 = 51;
pub const kBufOptIskeyword: C2Rust_Unnamed_13 = 50;
pub const kBufOptInfercase: C2Rust_Unnamed_13 = 49;
pub const kBufOptIndentkeys: C2Rust_Unnamed_13 = 48;
pub const kBufOptIndentexpr: C2Rust_Unnamed_13 = 47;
pub const kBufOptIncludeexpr: C2Rust_Unnamed_13 = 46;
pub const kBufOptInclude: C2Rust_Unnamed_13 = 45;
pub const kBufOptImsearch: C2Rust_Unnamed_13 = 44;
pub const kBufOptIminsert: C2Rust_Unnamed_13 = 43;
pub const kBufOptGrepprg: C2Rust_Unnamed_13 = 42;
pub const kBufOptGrepformat: C2Rust_Unnamed_13 = 41;
pub const kBufOptFsync: C2Rust_Unnamed_13 = 40;
pub const kBufOptFormatprg: C2Rust_Unnamed_13 = 39;
pub const kBufOptFormatoptions: C2Rust_Unnamed_13 = 38;
pub const kBufOptFormatlistpat: C2Rust_Unnamed_13 = 37;
pub const kBufOptFormatexpr: C2Rust_Unnamed_13 = 36;
pub const kBufOptFixendofline: C2Rust_Unnamed_13 = 35;
pub const kBufOptFindfunc: C2Rust_Unnamed_13 = 34;
pub const kBufOptFiletype: C2Rust_Unnamed_13 = 33;
pub const kBufOptFileformat: C2Rust_Unnamed_13 = 32;
pub const kBufOptFileencoding: C2Rust_Unnamed_13 = 31;
pub const kBufOptExpandtab: C2Rust_Unnamed_13 = 30;
pub const kBufOptErrorformat: C2Rust_Unnamed_13 = 29;
pub const kBufOptEqualprg: C2Rust_Unnamed_13 = 28;
pub const kBufOptEndofline: C2Rust_Unnamed_13 = 27;
pub const kBufOptEndoffile: C2Rust_Unnamed_13 = 26;
pub const kBufOptDiffanchors: C2Rust_Unnamed_13 = 25;
pub const kBufOptDictionary: C2Rust_Unnamed_13 = 24;
pub const kBufOptDefine: C2Rust_Unnamed_13 = 23;
pub const kBufOptCopyindent: C2Rust_Unnamed_13 = 22;
pub const kBufOptCompleteslash: C2Rust_Unnamed_13 = 21;
pub const kBufOptCompleteopt: C2Rust_Unnamed_13 = 20;
pub const kBufOptCompletefunc: C2Rust_Unnamed_13 = 19;
pub const kBufOptComplete: C2Rust_Unnamed_13 = 18;
pub const kBufOptCommentstring: C2Rust_Unnamed_13 = 17;
pub const kBufOptComments: C2Rust_Unnamed_13 = 16;
pub const kBufOptCinwords: C2Rust_Unnamed_13 = 15;
pub const kBufOptCinscopedecls: C2Rust_Unnamed_13 = 14;
pub const kBufOptCinoptions: C2Rust_Unnamed_13 = 13;
pub const kBufOptCinkeys: C2Rust_Unnamed_13 = 12;
pub const kBufOptCindent: C2Rust_Unnamed_13 = 11;
pub const kBufOptChannel: C2Rust_Unnamed_13 = 10;
pub const kBufOptBusy: C2Rust_Unnamed_13 = 9;
pub const kBufOptBuftype: C2Rust_Unnamed_13 = 8;
pub const kBufOptBuflisted: C2Rust_Unnamed_13 = 7;
pub const kBufOptBufhidden: C2Rust_Unnamed_13 = 6;
pub const kBufOptBomb: C2Rust_Unnamed_13 = 5;
pub const kBufOptBinary: C2Rust_Unnamed_13 = 4;
pub const kBufOptBackupcopy: C2Rust_Unnamed_13 = 3;
pub const kBufOptAutoread: C2Rust_Unnamed_13 = 2;
pub const kBufOptAutoindent: C2Rust_Unnamed_13 = 1;
pub const kBufOptAutocomplete: C2Rust_Unnamed_13 = 0;
pub const kBufOptInvalid: C2Rust_Unnamed_13 = -1;
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
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_STARTTIME: VimVarIndex = 104;
pub const VV_VIRTNUM: VimVarIndex = 103;
pub const VV_RELNUM: VimVarIndex = 102;
pub const VV_LUA: VimVarIndex = 101;
pub const VV__NULL_BLOB: VimVarIndex = 100;
pub const VV__NULL_DICT: VimVarIndex = 99;
pub const VV__NULL_LIST: VimVarIndex = 98;
pub const VV__NULL_STRING: VimVarIndex = 97;
pub const VV_MSGPACK_TYPES: VimVarIndex = 96;
pub const VV_STDERR: VimVarIndex = 95;
pub const VV_VIM_DID_INIT: VimVarIndex = 94;
pub const VV_STACKTRACE: VimVarIndex = 93;
pub const VV_MAXCOL: VimVarIndex = 92;
pub const VV_EXITING: VimVarIndex = 91;
pub const VV_COLLATE: VimVarIndex = 90;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_ARGF: VimVarIndex = 88;
pub const VV_ECHOSPACE: VimVarIndex = 87;
pub const VV_VERSIONLONG: VimVarIndex = 86;
pub const VV_EVENT: VimVarIndex = 85;
pub const VV_TYPE_BLOB: VimVarIndex = 84;
pub const VV_TYPE_BOOL: VimVarIndex = 83;
pub const VV_TYPE_FLOAT: VimVarIndex = 82;
pub const VV_TYPE_DICT: VimVarIndex = 81;
pub const VV_TYPE_LIST: VimVarIndex = 80;
pub const VV_TYPE_FUNC: VimVarIndex = 79;
pub const VV_TYPE_STRING: VimVarIndex = 78;
pub const VV_TYPE_NUMBER: VimVarIndex = 77;
pub const VV_TESTING: VimVarIndex = 76;
pub const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub const VV_NUMBERSIZE: VimVarIndex = 74;
pub const VV_NUMBERMIN: VimVarIndex = 73;
pub const VV_NUMBERMAX: VimVarIndex = 72;
pub const VV_NULL: VimVarIndex = 71;
pub const VV_TRUE: VimVarIndex = 70;
pub const VV_FALSE: VimVarIndex = 69;
pub const VV_ERRORS: VimVarIndex = 68;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_COMPLETED_ITEM: VimVarIndex = 61;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_WINDOWID: VimVarIndex = 59;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub const VV_OP: VimVarIndex = 55;
pub const VV_MOUSE_COL: VimVarIndex = 54;
pub const VV_MOUSE_LNUM: VimVarIndex = 53;
pub const VV_MOUSE_WINID: VimVarIndex = 52;
pub const VV_MOUSE_WIN: VimVarIndex = 51;
pub const VV_CHAR: VimVarIndex = 50;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub const VV_SWAPCHOICE: VimVarIndex = 48;
pub const VV_SWAPNAME: VimVarIndex = 47;
pub const VV_SCROLLSTART: VimVarIndex = 46;
pub const VV_BEVAL_TEXT: VimVarIndex = 45;
pub const VV_BEVAL_COL: VimVarIndex = 44;
pub const VV_BEVAL_LNUM: VimVarIndex = 43;
pub const VV_BEVAL_WINID: VimVarIndex = 42;
pub const VV_BEVAL_WINNR: VimVarIndex = 41;
pub const VV_BEVAL_BUFNR: VimVarIndex = 40;
pub const VV_FCS_CHOICE: VimVarIndex = 39;
pub const VV_FCS_REASON: VimVarIndex = 38;
pub const VV_PROFILING: VimVarIndex = 37;
pub const VV_KEY: VimVarIndex = 36;
pub const VV_VAL: VimVarIndex = 35;
pub const VV_INSERTMODE: VimVarIndex = 34;
pub const VV_CMDBANG: VimVarIndex = 33;
pub const VV_REG: VimVarIndex = 32;
pub const VV_THROWPOINT: VimVarIndex = 31;
pub const VV_EXCEPTION: VimVarIndex = 30;
pub const VV_DYING: VimVarIndex = 29;
pub const VV_SEND_SERVER: VimVarIndex = 28;
pub const VV_PROGNAME: VimVarIndex = 27;
pub const VV_FOLDLEVEL: VimVarIndex = 26;
pub const VV_FOLDDASHES: VimVarIndex = 25;
pub const VV_FOLDEND: VimVarIndex = 24;
pub const VV_FOLDSTART: VimVarIndex = 23;
pub const VV_CMDARG: VimVarIndex = 22;
pub const VV_FNAME_DIFF: VimVarIndex = 21;
pub const VV_FNAME_NEW: VimVarIndex = 20;
pub const VV_FNAME_OUT: VimVarIndex = 19;
pub const VV_FNAME_IN: VimVarIndex = 18;
pub const VV_CC_TO: VimVarIndex = 17;
pub const VV_CC_FROM: VimVarIndex = 16;
pub const VV_CTYPE: VimVarIndex = 15;
pub const VV_LC_TIME: VimVarIndex = 14;
pub const VV_LANG: VimVarIndex = 13;
pub const VV_FNAME: VimVarIndex = 12;
pub const VV_TERMRESPONSE: VimVarIndex = 11;
pub const VV_TERMREQUEST: VimVarIndex = 10;
pub const VV_LNUM: VimVarIndex = 9;
pub const VV_VERSION: VimVarIndex = 8;
pub const VV_THIS_SESSION: VimVarIndex = 7;
pub const VV_SHELL_ERROR: VimVarIndex = 6;
pub const VV_STATUSMSG: VimVarIndex = 5;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const VV_ERRMSG: VimVarIndex = 3;
pub const VV_PREVCOUNT: VimVarIndex = 2;
pub const VV_COUNT1: VimVarIndex = 1;
pub const VV_COUNT: VimVarIndex = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const FINDFILE_BOTH: C2Rust_Unnamed_14 = 2;
pub const FINDFILE_DIR: C2Rust_Unnamed_14 = 1;
pub const FINDFILE_FILE: C2Rust_Unnamed_14 = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const FNAME_UNESC: C2Rust_Unnamed_15 = 32;
pub const FNAME_REL: C2Rust_Unnamed_15 = 16;
pub const FNAME_INCL: C2Rust_Unnamed_15 = 8;
pub const FNAME_HYP: C2Rust_Unnamed_15 = 4;
pub const FNAME_EXP: C2Rust_Unnamed_15 = 2;
pub const FNAME_MESS: C2Rust_Unnamed_15 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_search_ctx_T {
    pub ffsc_stack_ptr: *mut ff_stack_T,
    pub ffsc_visited_list: *mut ff_visited_list_hdr_T,
    pub ffsc_dir_visited_list: *mut ff_visited_list_hdr_T,
    pub ffsc_visited_lists_list: *mut ff_visited_list_hdr_T,
    pub ffsc_dir_visited_lists_list: *mut ff_visited_list_hdr_T,
    pub ffsc_file_to_search: String_0,
    pub ffsc_start_dir: String_0,
    pub ffsc_fix_path: String_0,
    pub ffsc_wc_path: String_0,
    pub ffsc_level: ::core::ffi::c_int,
    pub ffsc_stopdirs_v: *mut String_0,
    pub ffsc_find_what: ::core::ffi::c_int,
    pub ffsc_tagfile: ::core::ffi::c_int,
}
pub type ff_visited_list_hdr_T = ff_visited_list_hdr;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_visited_list_hdr {
    pub ffvl_next: *mut ff_visited_list_hdr,
    pub ffvl_filename: *mut ::core::ffi::c_char,
    pub ffvl_visited_list: *mut ff_visited_T,
}
pub type ff_visited_T = ff_visited;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_visited {
    pub ffv_next: *mut ff_visited,
    pub ffv_wc_path: *mut ::core::ffi::c_char,
    pub file_id_valid: bool,
    pub file_id: FileID,
    pub ffv_fname: [::core::ffi::c_char; 0],
}
pub type ff_stack_T = ff_stack;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_stack {
    pub ffs_prev: *mut ff_stack,
    pub ffs_fix_path: String_0,
    pub ffs_wc_path: String_0,
    pub ffs_filearray: *mut *mut ::core::ffi::c_char,
    pub ffs_filearray_size: ::core::ffi::c_int,
    pub ffs_filearray_cur: ::core::ffi::c_int,
    pub ffs_stage: ::core::ffi::c_int,
    pub ffs_level: ::core::ffi::c_int,
    pub ffs_star_star_empty: ::core::ffi::c_int,
}
pub const EW_NOTWILD: C2Rust_Unnamed_17 = 1024;
pub const EW_SILENT: C2Rust_Unnamed_17 = 32;
pub const EW_ADDSLASH: C2Rust_Unnamed_17 = 8;
pub const EW_DIR: C2Rust_Unnamed_17 = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_16 = 2;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_16 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_16 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_16 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_16 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_16 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_16 = 4;
pub const OPT_GLOBAL: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_17 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_17 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_17 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_17 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_17 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_17 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_17 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_17 = 2048;
pub const EW_NOERROR: C2Rust_Unnamed_17 = 512;
pub const EW_ICASE: C2Rust_Unnamed_17 = 256;
pub const EW_PATH: C2Rust_Unnamed_17 = 128;
pub const EW_EXEC: C2Rust_Unnamed_17 = 64;
pub const EW_KEEPALL: C2Rust_Unnamed_17 = 16;
pub const EW_NOTFOUND: C2Rust_Unnamed_17 = 4;
pub const EW_FILE: C2Rust_Unnamed_17 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const PATHSEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"/\0") };
pub const CPO_DOTTAG: ::core::ffi::c_int = 'd' as ::core::ffi::c_int;
static ff_expand_buffer: GlobalCell<String_0> = GlobalCell::new(String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
});
pub const FF_MAX_STAR_STAR_EXPAND: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
static e_path_too_long_for_completion: GlobalCell<[::core::ffi::c_char; 35]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
            *b"E854: Path too long for completion\0",
        )
    });
pub unsafe extern "C" fn vim_findfile_init(
    mut path: *mut ::core::ffi::c_char,
    mut filename: *mut ::core::ffi::c_char,
    mut filenamelen: size_t,
    mut stopdirs: *mut ::core::ffi::c_char,
    mut level: ::core::ffi::c_int,
    mut free_visited: ::core::ffi::c_int,
    mut find_what: ::core::ffi::c_int,
    mut search_ctx_arg: *mut ::core::ffi::c_void,
    mut tagfile: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_void {
    let mut wc_part: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut add_sep: bool = false;
    let mut sptr: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
    let mut search_ctx: *mut ff_search_ctx_T = ::core::ptr::null_mut::<ff_search_ctx_T>();
    if !search_ctx_arg.is_null() {
        search_ctx = search_ctx_arg as *mut ff_search_ctx_T;
    } else {
        search_ctx =
            xcalloc(1 as size_t, ::core::mem::size_of::<ff_search_ctx_T>()) as *mut ff_search_ctx_T;
    }
    (*search_ctx).ffsc_find_what = find_what;
    (*search_ctx).ffsc_tagfile = tagfile;
    ff_clear(search_ctx);
    '_error_return: {
        if free_visited == true_0 {
            vim_findfile_free_visited(search_ctx as *mut ::core::ffi::c_void);
        } else {
            (*search_ctx).ffsc_visited_list = ff_get_visited_list(
                filename,
                filenamelen,
                &raw mut (*search_ctx).ffsc_visited_lists_list,
            );
            if (*search_ctx).ffsc_visited_list.is_null() {
                break '_error_return;
            } else {
                (*search_ctx).ffsc_dir_visited_list = ff_get_visited_list(
                    filename,
                    filenamelen,
                    &raw mut (*search_ctx).ffsc_dir_visited_lists_list,
                );
                if (*search_ctx).ffsc_dir_visited_list.is_null() {
                    break '_error_return;
                }
            }
        }
        if (*ff_expand_buffer.ptr()).data.is_null() {
            (*ff_expand_buffer.ptr()).size = 0 as size_t;
            (*ff_expand_buffer.ptr()).data =
                xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        }
        if *path.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (vim_ispathsep(*path.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *path.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
            && (tagfile == 0 || vim_strchr(p_cpo.get(), CPO_DOTTAG).is_null())
            && !rel_fname.is_null()
        {
            let mut len: size_t = path_tail(rel_fname).offset_from(rel_fname) as size_t;
            if !vim_isAbsName(rel_fname) && len.wrapping_add(1 as size_t) < MAXPATHL as size_t {
                xmemcpyz(
                    (*ff_expand_buffer.ptr()).data as *mut ::core::ffi::c_void,
                    rel_fname as *const ::core::ffi::c_void,
                    len,
                );
                (*ff_expand_buffer.ptr()).size = len;
                (*search_ctx).ffsc_start_dir =
                    cstr_as_string(FullName_save((*ff_expand_buffer.ptr()).data, false_0 != 0));
            } else {
                (*search_ctx).ffsc_start_dir = cbuf_to_string(rel_fname, len);
            }
            path = path.offset(1);
            if *path as ::core::ffi::c_int != NUL {
                path = path.offset(1);
            }
        } else if *path as ::core::ffi::c_int == NUL || !vim_isAbsName(path) {
            if os_dirname((*ff_expand_buffer.ptr()).data, MAXPATHL as size_t) == FAIL {
                break '_error_return;
            } else {
                (*ff_expand_buffer.ptr()).size = strlen((*ff_expand_buffer.ptr()).data);
                (*search_ctx).ffsc_start_dir =
                    copy_string(ff_expand_buffer.get(), ::core::ptr::null_mut::<Arena>());
            }
        }
        if !stopdirs.is_null() {
            let mut walker: *mut ::core::ffi::c_char = stopdirs;
            while *walker as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                walker = walker.offset(1);
            }
            let mut dircount: size_t = 1 as size_t;
            (*search_ctx).ffsc_stopdirs_v =
                xmalloc(::core::mem::size_of::<String_0>()) as *mut String_0;
            loop {
                let mut helper: *mut ::core::ffi::c_char = walker;
                let mut ptr: *mut ::core::ffi::c_void = xrealloc(
                    (*search_ctx).ffsc_stopdirs_v as *mut ::core::ffi::c_void,
                    dircount
                        .wrapping_add(1 as size_t)
                        .wrapping_mul(::core::mem::size_of::<String_0>()),
                );
                (*search_ctx).ffsc_stopdirs_v = ptr as *mut String_0;
                walker = vim_strchr(walker, ';' as ::core::ffi::c_int);
                '_c2rust_label: {
                    if walker.is_null() || walker.offset_from(helper) >= 0 as isize {
                    } else {
                        __assert_fail(
                            b"!walker || walker - helper >= 0\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/file_search.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            359 as ::core::ffi::c_uint,
                            b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                let mut len_0: size_t = if !walker.is_null() {
                    walker.offset_from(helper) as size_t
                } else {
                    strlen(helper)
                };
                if *helper as ::core::ffi::c_int != NUL
                    && !vim_isAbsName(helper)
                    && len_0.wrapping_add(1 as size_t) < MAXPATHL as size_t
                {
                    xmemcpyz(
                        (*ff_expand_buffer.ptr()).data as *mut ::core::ffi::c_void,
                        helper as *const ::core::ffi::c_void,
                        len_0,
                    );
                    (*ff_expand_buffer.ptr()).size = len_0;
                    *(*search_ctx)
                        .ffsc_stopdirs_v
                        .offset(dircount.wrapping_sub(1 as size_t) as isize) =
                        cstr_as_string(FullName_save(helper, len_0 != 0));
                } else {
                    *(*search_ctx)
                        .ffsc_stopdirs_v
                        .offset(dircount.wrapping_sub(1 as size_t) as isize) =
                        cbuf_to_string(helper, len_0);
                }
                if !walker.is_null() {
                    walker = walker.offset(1);
                }
                dircount = dircount.wrapping_add(1);
                if walker.is_null() {
                    break;
                }
            }
            *(*search_ctx)
                .ffsc_stopdirs_v
                .offset(dircount.wrapping_sub(1 as size_t) as isize) = NULL_STRING;
        }
        (*search_ctx).ffsc_level = level;
        wc_part = vim_strchr(path, '*' as ::core::ffi::c_int);
        if !wc_part.is_null() {
            let mut llevel: int64_t = 0;
            let mut errpt: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            '_c2rust_label_0: {
                if wc_part.offset_from(path) >= 0 as isize {
                } else {
                    __assert_fail(
                        b"wc_part - path >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/file_search.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        390 as ::core::ffi::c_uint,
                        b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            (*search_ctx).ffsc_fix_path = cbuf_to_string(path, wc_part.offset_from(path) as size_t);
            (*ff_expand_buffer.ptr()).size = 0 as size_t;
            while *wc_part as ::core::ffi::c_int != NUL {
                if (*ff_expand_buffer.ptr()).size.wrapping_add(5 as size_t) >= MAXPATHL as size_t {
                    emsg(gettext(
                        (e_path_too_long_for_completion.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                    break;
                } else if strncmp(
                    wc_part,
                    b"**\0".as_ptr() as *const ::core::ffi::c_char,
                    2 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    let c2rust_fresh0 = wc_part;
                    wc_part = wc_part.offset(1);
                    let c2rust_fresh1 = (*ff_expand_buffer.ptr()).size;
                    (*ff_expand_buffer.ptr()).size = (*ff_expand_buffer.ptr()).size.wrapping_add(1);
                    *(*ff_expand_buffer.ptr())
                        .data
                        .offset(c2rust_fresh1 as isize) = *c2rust_fresh0;
                    let c2rust_fresh2 = wc_part;
                    wc_part = wc_part.offset(1);
                    let c2rust_fresh3 = (*ff_expand_buffer.ptr()).size;
                    (*ff_expand_buffer.ptr()).size = (*ff_expand_buffer.ptr()).size.wrapping_add(1);
                    *(*ff_expand_buffer.ptr())
                        .data
                        .offset(c2rust_fresh3 as isize) = *c2rust_fresh2;
                    llevel = strtol(wc_part, &raw mut errpt, 10 as ::core::ffi::c_int) as int64_t;
                    if errpt != wc_part && llevel > 0 as int64_t && llevel < 255 as int64_t {
                        let c2rust_fresh4 = (*ff_expand_buffer.ptr()).size;
                        (*ff_expand_buffer.ptr()).size =
                            (*ff_expand_buffer.ptr()).size.wrapping_add(1);
                        *(*ff_expand_buffer.ptr())
                            .data
                            .offset(c2rust_fresh4 as isize) = llevel as ::core::ffi::c_char;
                    } else if errpt != wc_part && llevel == 0 as int64_t {
                        (*ff_expand_buffer.ptr()).size =
                            (*ff_expand_buffer.ptr()).size.wrapping_sub(2 as size_t);
                    } else {
                        let c2rust_fresh5 = (*ff_expand_buffer.ptr()).size;
                        (*ff_expand_buffer.ptr()).size =
                            (*ff_expand_buffer.ptr()).size.wrapping_add(1);
                        *(*ff_expand_buffer.ptr())
                            .data
                            .offset(c2rust_fresh5 as isize) =
                            FF_MAX_STAR_STAR_EXPAND as ::core::ffi::c_char;
                    }
                    wc_part = errpt;
                    if !(*wc_part as ::core::ffi::c_int != NUL
                        && !vim_ispathsep(*wc_part as ::core::ffi::c_int))
                    {
                        continue;
                    }
                    semsg(
                        gettext(
                            b"E343: Invalid path: '**[number]' must be at the end of the path or be followed by '%s'.\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        PATHSEPSTR.as_ptr(),
                    );
                    break '_error_return;
                } else {
                    let c2rust_fresh6 = wc_part;
                    wc_part = wc_part.offset(1);
                    let c2rust_fresh7 = (*ff_expand_buffer.ptr()).size;
                    (*ff_expand_buffer.ptr()).size = (*ff_expand_buffer.ptr()).size.wrapping_add(1);
                    *(*ff_expand_buffer.ptr())
                        .data
                        .offset(c2rust_fresh7 as isize) = *c2rust_fresh6;
                }
            }
            *(*ff_expand_buffer.ptr())
                .data
                .offset((*ff_expand_buffer.ptr()).size as isize) = NUL as ::core::ffi::c_char;
            (*search_ctx).ffsc_wc_path =
                copy_string(ff_expand_buffer.get(), ::core::ptr::null_mut::<Arena>());
        } else {
            (*search_ctx).ffsc_fix_path = cstr_to_string(path);
        }
        if (*search_ctx).ffsc_start_dir.data.is_null() {
            (*search_ctx).ffsc_start_dir = copy_string(
                (*search_ctx).ffsc_fix_path,
                ::core::ptr::null_mut::<Arena>(),
            );
            *(*search_ctx)
                .ffsc_fix_path
                .data
                .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            (*search_ctx).ffsc_fix_path.size = 0 as size_t;
        }
        if (*search_ctx)
            .ffsc_start_dir
            .size
            .wrapping_add((*search_ctx).ffsc_fix_path.size)
            .wrapping_add(3 as size_t)
            >= MAXPATHL as size_t
        {
            emsg(gettext(
                (e_path_too_long_for_completion.ptr() as *const _) as *const ::core::ffi::c_char,
            ));
        } else {
            add_sep = after_pathsep(
                (*search_ctx).ffsc_start_dir.data,
                (*search_ctx)
                    .ffsc_start_dir
                    .data
                    .offset((*search_ctx).ffsc_start_dir.size as isize),
            ) == 0;
            (*ff_expand_buffer.ptr()).size = vim_snprintf(
                (*ff_expand_buffer.ptr()).data,
                MAXPATHL as size_t,
                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*search_ctx).ffsc_start_dir.data,
                if add_sep as ::core::ffi::c_int != 0 {
                    PATHSEPSTR.as_ptr()
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                },
            ) as size_t;
            '_c2rust_label_1: {
                if (*ff_expand_buffer.ptr()).size < 4096 as size_t {
                } else {
                    __assert_fail(
                        b"ff_expand_buffer.size < MAXPATHL\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/file_search.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        458 as ::core::ffi::c_uint,
                        b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut bufsize: size_t = (*ff_expand_buffer.ptr())
                .size
                .wrapping_add((*search_ctx).ffsc_fix_path.size)
                .wrapping_add(1 as size_t);
            let mut buf: *mut ::core::ffi::c_char = xmalloc(bufsize) as *mut ::core::ffi::c_char;
            vim_snprintf(
                buf,
                bufsize,
                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*ff_expand_buffer.ptr()).data,
                (*search_ctx).ffsc_fix_path.data,
            );
            if os_isdir(buf) {
                if (*search_ctx).ffsc_fix_path.size > 0 as size_t {
                    add_sep = after_pathsep(
                        (*search_ctx).ffsc_fix_path.data,
                        (*search_ctx)
                            .ffsc_fix_path
                            .data
                            .offset((*search_ctx).ffsc_fix_path.size as isize),
                    ) == 0;
                    (*ff_expand_buffer.ptr()).size = (*ff_expand_buffer.ptr()).size.wrapping_add(
                        vim_snprintf(
                            (*ff_expand_buffer.ptr())
                                .data
                                .offset((*ff_expand_buffer.ptr()).size as isize),
                            (MAXPATHL as size_t).wrapping_sub((*ff_expand_buffer.ptr()).size),
                            b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                            (*search_ctx).ffsc_fix_path.data,
                            if add_sep as ::core::ffi::c_int != 0 {
                                PATHSEPSTR.as_ptr()
                            } else {
                                b"\0".as_ptr() as *const ::core::ffi::c_char
                            },
                        ) as size_t,
                    );
                    '_c2rust_label_2: {
                        if (*ff_expand_buffer.ptr()).size < 4096 as size_t {
                        } else {
                            __assert_fail(
                                b"ff_expand_buffer.size < MAXPATHL\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/file_search.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                478 as ::core::ffi::c_uint,
                                b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
            } else {
                let mut p: *mut ::core::ffi::c_char = path_tail((*search_ctx).ffsc_fix_path.data);
                let mut len_1: ::core::ffi::c_int =
                    (*search_ctx).ffsc_fix_path.size as ::core::ffi::c_int;
                if p > (*search_ctx).ffsc_fix_path.data {
                    len_1 = p.offset_from((*search_ctx).ffsc_fix_path.data) as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int;
                    if len_1 >= 2 as ::core::ffi::c_int
                        && strncmp(
                            (*search_ctx).ffsc_fix_path.data,
                            b"..\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        && (len_1 == 2 as ::core::ffi::c_int
                            || *(*search_ctx)
                                .ffsc_fix_path
                                .data
                                .offset(2 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == PATHSEP)
                    {
                        xfree(buf as *mut ::core::ffi::c_void);
                        break '_error_return;
                    } else {
                        add_sep = after_pathsep(
                            (*search_ctx).ffsc_fix_path.data,
                            (*search_ctx)
                                .ffsc_fix_path
                                .data
                                .offset((*search_ctx).ffsc_fix_path.size as isize),
                        ) == 0;
                        (*ff_expand_buffer.ptr()).size = (*ff_expand_buffer.ptr())
                            .size
                            .wrapping_add(vim_snprintf(
                                (*ff_expand_buffer.ptr())
                                    .data
                                    .offset((*ff_expand_buffer.ptr()).size as isize),
                                (MAXPATHL as size_t).wrapping_sub((*ff_expand_buffer.ptr()).size),
                                b"%.*s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                len_1,
                                (*search_ctx).ffsc_fix_path.data,
                                if add_sep as ::core::ffi::c_int != 0 {
                                    PATHSEPSTR.as_ptr()
                                } else {
                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                },
                            ) as size_t);
                        '_c2rust_label_3: {
                            if (*ff_expand_buffer.ptr()).size < 4096 as size_t {
                            } else {
                                __assert_fail(
                                    b"ff_expand_buffer.size < MAXPATHL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/file_search.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    501 as ::core::ffi::c_uint,
                                    b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                    }
                }
                if !(*search_ctx).ffsc_wc_path.data.is_null() {
                    let mut tempsize: size_t = (*search_ctx)
                        .ffsc_fix_path
                        .size
                        .wrapping_sub(len_1 as size_t)
                        .wrapping_add((*search_ctx).ffsc_wc_path.size)
                        .wrapping_add(1 as size_t);
                    let mut temp: *mut ::core::ffi::c_char =
                        xmalloc(tempsize) as *mut ::core::ffi::c_char;
                    (*search_ctx).ffsc_wc_path.size = vim_snprintf(
                        temp,
                        tempsize,
                        b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                        (*search_ctx).ffsc_fix_path.data.offset(len_1 as isize),
                        (*search_ctx).ffsc_wc_path.data,
                    ) as size_t;
                    '_c2rust_label_4: {
                        if (*search_ctx).ffsc_wc_path.size < tempsize {
                        } else {
                            __assert_fail(
                                b"search_ctx->ffsc_wc_path.size < tempsize\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/file_search.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                513 as ::core::ffi::c_uint,
                                b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    xfree((*search_ctx).ffsc_wc_path.data as *mut ::core::ffi::c_void);
                    (*search_ctx).ffsc_wc_path.data = temp;
                }
            }
            xfree(buf as *mut ::core::ffi::c_void);
            sptr = ff_create_stack_element(
                (*ff_expand_buffer.ptr()).data,
                (*ff_expand_buffer.ptr()).size,
                (*search_ctx).ffsc_wc_path.data,
                (*search_ctx).ffsc_wc_path.size,
                level,
                0 as ::core::ffi::c_int,
            );
            ff_push(search_ctx, sptr);
            (*search_ctx).ffsc_file_to_search = cbuf_to_string(filename, filenamelen);
            return search_ctx as *mut ::core::ffi::c_void;
        }
    }
    vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
    return NULL;
}
pub unsafe extern "C" fn vim_findfile_stopdir(
    mut buf: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    while *buf as ::core::ffi::c_int != NUL
        && *buf as ::core::ffi::c_int != ';' as ::core::ffi::c_int
        && (*buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '\\' as ::core::ffi::c_int
            || *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ';' as ::core::ffi::c_int)
    {
        buf = buf.offset(1);
    }
    let mut dst: *mut ::core::ffi::c_char = buf;
    's_91: {
        '_is_semicolon: {
            if *buf as ::core::ffi::c_int != ';' as ::core::ffi::c_int {
                if *buf as ::core::ffi::c_int != NUL {
                    's_61: loop {
                        let c2rust_fresh8 = dst;
                        dst = dst.offset(1);
                        *c2rust_fresh8 = ';' as ::core::ffi::c_char;
                        buf = buf.offset(2 as ::core::ffi::c_int as isize);
                        loop {
                            if !(*buf as ::core::ffi::c_int != NUL
                                && *buf as ::core::ffi::c_int != ';' as ::core::ffi::c_int)
                            {
                                break 's_61;
                            }
                            if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '\\' as ::core::ffi::c_int
                                && *buf.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ';' as ::core::ffi::c_int
                            {
                                break;
                            }
                            let c2rust_fresh9 = buf;
                            buf = buf.offset(1);
                            let c2rust_fresh10 = dst;
                            dst = dst.offset(1);
                            *c2rust_fresh10 = *c2rust_fresh9;
                        }
                    }
                    '_c2rust_label: {
                        if dst < buf {
                        } else {
                            __assert_fail(
                                b"dst < buf\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/file_search.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                561 as ::core::ffi::c_uint,
                                b"char *vim_findfile_stopdir(char *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    *dst = NUL as ::core::ffi::c_char;
                    if *buf as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                        break '_is_semicolon;
                    }
                }
                buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                break 's_91;
            }
        }
        *buf = NUL as ::core::ffi::c_char;
        buf = buf.offset(1);
    }
    return buf;
}
pub unsafe extern "C" fn vim_findfile_cleanup(mut ctx: *mut ::core::ffi::c_void) {
    if ctx.is_null() {
        return;
    }
    vim_findfile_free_visited(ctx);
    ff_clear(ctx as *mut ff_search_ctx_T);
    xfree(ctx);
}
pub unsafe extern "C" fn vim_findfile(
    mut search_ctx_arg: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    let mut rest_of_wildcards: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut path_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut stackp: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
    if search_ctx_arg.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut search_ctx: *mut ff_search_ctx_T = search_ctx_arg as *mut ff_search_ctx_T;
    let mut file_path: String_0 = String_0 {
        data: xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char,
        size: 0,
    };
    if !(*search_ctx).ffsc_start_dir.data.is_null() {
        path_end = (*search_ctx)
            .ffsc_start_dir
            .data
            .offset((*search_ctx).ffsc_start_dir.size as isize);
    }
    '_fail: loop {
        os_breakcheck();
        if !got_int.get() {
            stackp = ff_pop(search_ctx);
            if !stackp.is_null() {
                if (*stackp).ffs_filearray.is_null()
                    && ff_check_visited(
                        &raw mut (*(*search_ctx).ffsc_dir_visited_list).ffvl_visited_list,
                        (*stackp).ffs_fix_path.data,
                        (*stackp).ffs_fix_path.size,
                        (*stackp).ffs_wc_path.data,
                        (*stackp).ffs_wc_path.size,
                    ) == FAIL
                {
                    ff_free_stack_element(stackp);
                    continue;
                } else if (*stackp).ffs_level <= 0 as ::core::ffi::c_int {
                    ff_free_stack_element(stackp);
                    continue;
                } else {
                    *file_path.data.offset(0 as ::core::ffi::c_int as isize) =
                        NUL as ::core::ffi::c_char;
                    file_path.size = 0 as size_t;
                    if (*stackp).ffs_filearray.is_null() {
                        let mut dirptrs: [*mut ::core::ffi::c_char; 2] =
                            [::core::ptr::null_mut::<::core::ffi::c_char>(); 2];
                        dirptrs[0 as ::core::ffi::c_int as usize] = file_path.data;
                        dirptrs[1 as ::core::ffi::c_int as usize] =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        if !vim_isAbsName((*stackp).ffs_fix_path.data)
                            && !(*search_ctx).ffsc_start_dir.data.is_null()
                        {
                            if (*search_ctx).ffsc_start_dir.size.wrapping_add(1 as size_t)
                                >= MAXPATHL as size_t
                            {
                                ff_free_stack_element(stackp);
                                break;
                            } else {
                                let mut add_sep: bool = after_pathsep(
                                    (*search_ctx).ffsc_start_dir.data,
                                    (*search_ctx)
                                        .ffsc_start_dir
                                        .data
                                        .offset((*search_ctx).ffsc_start_dir.size as isize),
                                ) == 0;
                                file_path.size = vim_snprintf(
                                    file_path.data,
                                    MAXPATHL as size_t,
                                    b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    (*search_ctx).ffsc_start_dir.data,
                                    if add_sep as ::core::ffi::c_int != 0 {
                                        PATHSEPSTR.as_ptr()
                                    } else {
                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                    },
                                ) as size_t;
                                if file_path.size >= MAXPATHL as size_t {
                                    ff_free_stack_element(stackp);
                                    break;
                                }
                            }
                        }
                        if file_path
                            .size
                            .wrapping_add((*stackp).ffs_fix_path.size)
                            .wrapping_add(1 as size_t)
                            >= MAXPATHL as size_t
                        {
                            ff_free_stack_element(stackp);
                            break;
                        } else {
                            let mut add_sep_0: bool = after_pathsep(
                                (*stackp).ffs_fix_path.data,
                                (*stackp)
                                    .ffs_fix_path
                                    .data
                                    .offset((*stackp).ffs_fix_path.size as isize),
                            ) == 0;
                            file_path.size = file_path.size.wrapping_add(vim_snprintf(
                                file_path.data.offset(file_path.size as isize),
                                (MAXPATHL as size_t).wrapping_sub(file_path.size),
                                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                (*stackp).ffs_fix_path.data,
                                if add_sep_0 as ::core::ffi::c_int != 0 {
                                    PATHSEPSTR.as_ptr()
                                } else {
                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                },
                            )
                                as size_t);
                            if file_path.size >= MAXPATHL as size_t {
                                ff_free_stack_element(stackp);
                                break;
                            } else {
                                rest_of_wildcards = (*stackp).ffs_wc_path;
                                if *rest_of_wildcards.data as ::core::ffi::c_int != NUL {
                                    if strncmp(
                                        rest_of_wildcards.data,
                                        b"**\0".as_ptr() as *const ::core::ffi::c_char,
                                        2 as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        let mut p: *mut ::core::ffi::c_char = rest_of_wildcards
                                            .data
                                            .offset(2 as ::core::ffi::c_int as isize);
                                        if *p as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                                            *p -= 1;
                                            if file_path.size.wrapping_add(1 as size_t)
                                                >= MAXPATHL as size_t
                                            {
                                                ff_free_stack_element(stackp);
                                                break;
                                            } else {
                                                let c2rust_fresh11 = file_path.size;
                                                file_path.size = file_path.size.wrapping_add(1);
                                                *file_path.data.offset(c2rust_fresh11 as isize) =
                                                    '*' as ::core::ffi::c_char;
                                            }
                                        }
                                        if *p as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                                            memmove(
                                                rest_of_wildcards.data as *mut ::core::ffi::c_void,
                                                rest_of_wildcards
                                                    .data
                                                    .offset(3 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                rest_of_wildcards
                                                    .size
                                                    .wrapping_sub(3 as size_t)
                                                    .wrapping_add(1 as size_t),
                                            );
                                            rest_of_wildcards.size =
                                                rest_of_wildcards.size.wrapping_sub(3 as size_t);
                                            (*stackp).ffs_wc_path.size = rest_of_wildcards.size;
                                        } else {
                                            rest_of_wildcards.data = rest_of_wildcards
                                                .data
                                                .offset(3 as ::core::ffi::c_int as isize);
                                            rest_of_wildcards.size =
                                                rest_of_wildcards.size.wrapping_sub(3 as size_t);
                                        }
                                        if (*stackp).ffs_star_star_empty == 0 as ::core::ffi::c_int
                                        {
                                            (*stackp).ffs_star_star_empty = 1 as ::core::ffi::c_int;
                                            dirptrs[1 as ::core::ffi::c_int as usize] =
                                                (*stackp).ffs_fix_path.data;
                                        }
                                    }
                                    while *rest_of_wildcards.data as ::core::ffi::c_int != 0
                                        && !vim_ispathsep(
                                            *rest_of_wildcards.data as ::core::ffi::c_int,
                                        )
                                    {
                                        if file_path.size.wrapping_add(1 as size_t)
                                            >= MAXPATHL as size_t
                                        {
                                            ff_free_stack_element(stackp);
                                            break '_fail;
                                        } else {
                                            let c2rust_fresh12 = rest_of_wildcards.data;
                                            rest_of_wildcards.data =
                                                rest_of_wildcards.data.offset(1);
                                            let c2rust_fresh13 = file_path.size;
                                            file_path.size = file_path.size.wrapping_add(1);
                                            *file_path.data.offset(c2rust_fresh13 as isize) =
                                                *c2rust_fresh12;
                                            rest_of_wildcards.size =
                                                rest_of_wildcards.size.wrapping_sub(1);
                                        }
                                    }
                                    *file_path.data.offset(file_path.size as isize) =
                                        NUL as ::core::ffi::c_char;
                                    if vim_ispathsep(*rest_of_wildcards.data as ::core::ffi::c_int)
                                    {
                                        rest_of_wildcards.data = rest_of_wildcards.data.offset(1);
                                        rest_of_wildcards.size =
                                            rest_of_wildcards.size.wrapping_sub(1);
                                    }
                                }
                                if path_with_url(dirptrs[0 as ::core::ffi::c_int as usize]) != 0 {
                                    (*stackp).ffs_filearray =
                                        xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                                            as *mut *mut ::core::ffi::c_char;
                                    *(*stackp)
                                        .ffs_filearray
                                        .offset(0 as ::core::ffi::c_int as isize) = xmemdupz(
                                        dirptrs[0 as ::core::ffi::c_int as usize]
                                            as *const ::core::ffi::c_void,
                                        file_path.size,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    (*stackp).ffs_filearray_size = 1 as ::core::ffi::c_int;
                                } else {
                                    expand_wildcards(
                                        if dirptrs[1 as ::core::ffi::c_int as usize].is_null() {
                                            1 as ::core::ffi::c_int
                                        } else {
                                            2 as ::core::ffi::c_int
                                        },
                                        &raw mut dirptrs as *mut *mut ::core::ffi::c_char,
                                        &raw mut (*stackp).ffs_filearray_size,
                                        &raw mut (*stackp).ffs_filearray,
                                        EW_DIR as ::core::ffi::c_int
                                            | EW_ADDSLASH as ::core::ffi::c_int
                                            | EW_SILENT as ::core::ffi::c_int
                                            | EW_NOTWILD as ::core::ffi::c_int,
                                    );
                                }
                                (*stackp).ffs_filearray_cur = 0 as ::core::ffi::c_int;
                                (*stackp).ffs_stage = 0 as ::core::ffi::c_int;
                            }
                        }
                    } else {
                        rest_of_wildcards.data = (*stackp)
                            .ffs_wc_path
                            .data
                            .offset((*stackp).ffs_wc_path.size as isize);
                        rest_of_wildcards.size = 0 as size_t;
                    }
                    if (*stackp).ffs_stage == 0 as ::core::ffi::c_int {
                        's_500: {
                            if *rest_of_wildcards.data as ::core::ffi::c_int == NUL {
                                let mut i: ::core::ffi::c_int = (*stackp).ffs_filearray_cur;
                                loop {
                                    if i >= (*stackp).ffs_filearray_size {
                                        break 's_500;
                                    }
                                    if !(path_with_url(*(*stackp).ffs_filearray.offset(i as isize))
                                        == 0
                                        && !os_isdir(*(*stackp).ffs_filearray.offset(i as isize)))
                                    {
                                        let mut len: size_t =
                                            strlen(*(*stackp).ffs_filearray.offset(i as isize));
                                        if len
                                            .wrapping_add(1 as size_t)
                                            .wrapping_add((*search_ctx).ffsc_file_to_search.size)
                                            >= MAXPATHL as size_t
                                        {
                                            ff_free_stack_element(stackp);
                                            break '_fail;
                                        } else {
                                            let mut add_sep_1: bool = after_pathsep(
                                                *(*stackp).ffs_filearray.offset(i as isize),
                                                (*(*stackp).ffs_filearray.offset(i as isize))
                                                    .offset(len as isize),
                                            ) == 0;
                                            file_path.size = vim_snprintf(
                                                file_path.data,
                                                MAXPATHL as size_t,
                                                b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                                *(*stackp).ffs_filearray.offset(i as isize),
                                                if add_sep_1 as ::core::ffi::c_int != 0 {
                                                    PATHSEPSTR.as_ptr()
                                                } else {
                                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                                },
                                                (*search_ctx).ffsc_file_to_search.data,
                                            )
                                                as size_t;
                                            if file_path.size >= MAXPATHL as size_t {
                                                ff_free_stack_element(stackp);
                                                break '_fail;
                                            } else {
                                                len = file_path.size;
                                                let mut suf: *mut ::core::ffi::c_char =
                                                    (if (*search_ctx).ffsc_tagfile != 0 {
                                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                                    } else {
                                                        (*curbuf.get()).b_p_sua
                                                            as *const ::core::ffi::c_char
                                                    })
                                                        as *mut ::core::ffi::c_char;
                                                loop {
                                                    if (path_with_url(file_path.data) != 0
                                                        || os_path_exists(file_path.data)
                                                            as ::core::ffi::c_int
                                                            != 0
                                                            && ((*search_ctx).ffsc_find_what
                                                                == FINDFILE_BOTH
                                                                    as ::core::ffi::c_int
                                                                || ((*search_ctx).ffsc_find_what
                                                                    == FINDFILE_DIR
                                                                        as ::core::ffi::c_int)
                                                                    as ::core::ffi::c_int
                                                                    == os_isdir(file_path.data)
                                                                        as ::core::ffi::c_int))
                                                        && ff_check_visited(
                                                            &raw mut (*(*search_ctx)
                                                                .ffsc_visited_list)
                                                                .ffvl_visited_list,
                                                            file_path.data,
                                                            file_path.size,
                                                            b"\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                                as *mut ::core::ffi::c_char,
                                                            0 as size_t,
                                                        ) == OK
                                                    {
                                                        '_c2rust_label: {
                                                            if i < 2147483647 as ::core::ffi::c_int
                                                            {
                                                            } else {
                                                                __assert_fail(
                                                                    b"i < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                                                    b"src/nvim/file_search.rs\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                    875 as ::core::ffi::c_uint,
                                                                    b"char *vim_findfile(void *)\0".as_ptr()
                                                                        as *const ::core::ffi::c_char,
                                                                );
                                                            }
                                                        };
                                                        (*stackp).ffs_filearray_cur =
                                                            i + 1 as ::core::ffi::c_int;
                                                        ff_push(search_ctx, stackp);
                                                        if path_with_url(file_path.data) == 0 {
                                                            file_path.size =
                                                                simplify_filename(file_path.data);
                                                        }
                                                        if os_dirname(
                                                            (*ff_expand_buffer.ptr()).data,
                                                            MAXPATHL as size_t,
                                                        ) == OK
                                                        {
                                                            (*ff_expand_buffer.ptr()).size = strlen(
                                                                (*ff_expand_buffer.ptr()).data,
                                                            );
                                                            let mut p_0: *mut ::core::ffi::c_char =
                                                                path_shorten_fname(
                                                                    file_path.data,
                                                                    (*ff_expand_buffer.ptr()).data,
                                                                );
                                                            if !p_0.is_null() {
                                                                memmove(
                                                                    file_path.data as *mut ::core::ffi::c_void,
                                                                    p_0 as *const ::core::ffi::c_void,
                                                                    (file_path
                                                                        .data
                                                                        .offset(file_path.size as isize)
                                                                        .offset_from(p_0) as size_t)
                                                                        .wrapping_add(1 as size_t),
                                                                );
                                                                file_path.size =
                                                                    file_path.size.wrapping_sub(
                                                                        p_0.offset_from(
                                                                            file_path.data,
                                                                        )
                                                                            as size_t,
                                                                    );
                                                            }
                                                        }
                                                        return file_path.data;
                                                    }
                                                    if *suf as ::core::ffi::c_int == NUL {
                                                        break;
                                                    }
                                                    '_c2rust_label_0: {
                                                        if 4096 as size_t >= file_path.size {
                                                        } else {
                                                            __assert_fail(
                                                                b"MAXPATHL >= file_path.size\0"
                                                                    .as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                                b"src/nvim/file_search.rs\0"
                                                                    .as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                                907 as ::core::ffi::c_uint,
                                                                b"char *vim_findfile(void *)\0"
                                                                    .as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                        }
                                                    };
                                                    file_path.size =
                                                        len.wrapping_add(copy_option_part(
                                                            &raw mut suf,
                                                            file_path.data.offset(len as isize),
                                                            (MAXPATHL as size_t).wrapping_sub(len),
                                                            b",\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                                as *mut ::core::ffi::c_char,
                                                        ));
                                                }
                                            }
                                        }
                                    }
                                    i += 1;
                                }
                            } else {
                                let mut i_0: ::core::ffi::c_int = (*stackp).ffs_filearray_cur;
                                while i_0 < (*stackp).ffs_filearray_size {
                                    if os_isdir(*(*stackp).ffs_filearray.offset(i_0 as isize)) {
                                        ff_push(
                                            search_ctx,
                                            ff_create_stack_element(
                                                *(*stackp).ffs_filearray.offset(i_0 as isize),
                                                strlen(
                                                    *(*stackp).ffs_filearray.offset(i_0 as isize),
                                                ),
                                                rest_of_wildcards.data,
                                                rest_of_wildcards.size,
                                                (*stackp).ffs_level - 1 as ::core::ffi::c_int,
                                                0 as ::core::ffi::c_int,
                                            ),
                                        );
                                    }
                                    i_0 += 1;
                                }
                            }
                        }
                        (*stackp).ffs_filearray_cur = 0 as ::core::ffi::c_int;
                        (*stackp).ffs_stage = 1 as ::core::ffi::c_int;
                    }
                    if strncmp(
                        (*stackp).ffs_wc_path.data,
                        b"**\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        let mut i_1: ::core::ffi::c_int = (*stackp).ffs_filearray_cur;
                        while i_1 < (*stackp).ffs_filearray_size {
                            if path_fnamecmp(
                                *(*stackp).ffs_filearray.offset(i_1 as isize),
                                (*stackp).ffs_fix_path.data,
                            ) != 0 as ::core::ffi::c_int
                            {
                                if os_isdir(*(*stackp).ffs_filearray.offset(i_1 as isize)) {
                                    ff_push(
                                        search_ctx,
                                        ff_create_stack_element(
                                            *(*stackp).ffs_filearray.offset(i_1 as isize),
                                            strlen(*(*stackp).ffs_filearray.offset(i_1 as isize)),
                                            (*stackp).ffs_wc_path.data,
                                            (*stackp).ffs_wc_path.size,
                                            (*stackp).ffs_level - 1 as ::core::ffi::c_int,
                                            1 as ::core::ffi::c_int,
                                        ),
                                    );
                                }
                            }
                            i_1 += 1;
                        }
                    }
                    ff_free_stack_element(stackp);
                    continue;
                }
            }
        }
        if !(!(*search_ctx).ffsc_start_dir.data.is_null()
            && !(*search_ctx).ffsc_stopdirs_v.is_null()
            && !got_int.get())
        {
            break;
        }
        let mut sptr: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
        let mut plen: ptrdiff_t = path_end.offset_from((*search_ctx).ffsc_start_dir.data)
            + (*path_end as ::core::ffi::c_int != NUL) as ::core::ffi::c_int as ptrdiff_t;
        if ff_path_in_stoplist(
            (*search_ctx).ffsc_start_dir.data,
            plen as size_t,
            (*search_ctx).ffsc_stopdirs_v,
        ) {
            break;
        }
        while path_end > (*search_ctx).ffsc_start_dir.data
            && vim_ispathsep(*path_end as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            path_end = path_end.offset(-1);
        }
        while path_end > (*search_ctx).ffsc_start_dir.data
            && !vim_ispathsep(
                *path_end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            )
        {
            path_end = path_end.offset(-1);
        }
        *path_end = NUL as ::core::ffi::c_char;
        (*search_ctx).ffsc_start_dir.size =
            path_end.offset_from((*search_ctx).ffsc_start_dir.data) as size_t;
        path_end = path_end.offset(-1);
        if *(*search_ctx).ffsc_start_dir.data as ::core::ffi::c_int == NUL {
            break;
        }
        if (*search_ctx)
            .ffsc_start_dir
            .size
            .wrapping_add(1 as size_t)
            .wrapping_add((*search_ctx).ffsc_fix_path.size)
            >= MAXPATHL as size_t
        {
            break;
        }
        let mut add_sep_2: bool = after_pathsep(
            (*search_ctx).ffsc_start_dir.data,
            (*search_ctx)
                .ffsc_start_dir
                .data
                .offset((*search_ctx).ffsc_start_dir.size as isize),
        ) == 0;
        file_path.size = vim_snprintf(
            file_path.data,
            MAXPATHL as size_t,
            b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            (*search_ctx).ffsc_start_dir.data,
            if add_sep_2 as ::core::ffi::c_int != 0 {
                PATHSEPSTR.as_ptr()
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            (*search_ctx).ffsc_fix_path.data,
        ) as size_t;
        if file_path.size >= MAXPATHL as size_t {
            break;
        }
        sptr = ff_create_stack_element(
            file_path.data,
            file_path.size,
            (*search_ctx).ffsc_wc_path.data,
            (*search_ctx).ffsc_wc_path.size,
            (*search_ctx).ffsc_level,
            0 as ::core::ffi::c_int,
        );
        ff_push(search_ctx, sptr);
    }
    xfree(file_path.data as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn vim_findfile_free_visited(mut search_ctx_arg: *mut ::core::ffi::c_void) {
    if search_ctx_arg.is_null() {
        return;
    }
    let mut search_ctx: *mut ff_search_ctx_T = search_ctx_arg as *mut ff_search_ctx_T;
    vim_findfile_free_visited_list(&raw mut (*search_ctx).ffsc_visited_lists_list);
    vim_findfile_free_visited_list(&raw mut (*search_ctx).ffsc_dir_visited_lists_list);
}
unsafe extern "C" fn vim_findfile_free_visited_list(
    mut list_headp: *mut *mut ff_visited_list_hdr_T,
) {
    let mut vp: *mut ff_visited_list_hdr_T = ::core::ptr::null_mut::<ff_visited_list_hdr_T>();
    while !(*list_headp).is_null() {
        vp = (**list_headp).ffvl_next as *mut ff_visited_list_hdr_T;
        ff_free_visited_list((**list_headp).ffvl_visited_list);
        xfree((**list_headp).ffvl_filename as *mut ::core::ffi::c_void);
        xfree(*list_headp as *mut ::core::ffi::c_void);
        *list_headp = vp;
    }
    *list_headp = ::core::ptr::null_mut::<ff_visited_list_hdr_T>();
}
unsafe extern "C" fn ff_free_visited_list(mut vl: *mut ff_visited_T) {
    let mut vp: *mut ff_visited_T = ::core::ptr::null_mut::<ff_visited_T>();
    while !vl.is_null() {
        vp = (*vl).ffv_next as *mut ff_visited_T;
        xfree((*vl).ffv_wc_path as *mut ::core::ffi::c_void);
        xfree(vl as *mut ::core::ffi::c_void);
        vl = vp;
    }
    vl = ::core::ptr::null_mut::<ff_visited_T>();
}
unsafe extern "C" fn ff_get_visited_list(
    mut filename: *mut ::core::ffi::c_char,
    mut filenamelen: size_t,
    mut list_headp: *mut *mut ff_visited_list_hdr_T,
) -> *mut ff_visited_list_hdr_T {
    let mut retptr: *mut ff_visited_list_hdr_T = ::core::ptr::null_mut::<ff_visited_list_hdr_T>();
    if !(*list_headp).is_null() {
        retptr = *list_headp;
        while !retptr.is_null() {
            if path_fnamecmp(filename, (*retptr).ffvl_filename) == 0 as ::core::ffi::c_int {
                return retptr;
            }
            retptr = (*retptr).ffvl_next as *mut ff_visited_list_hdr_T;
        }
    }
    retptr = xmalloc(::core::mem::size_of::<ff_visited_list_hdr_T>()) as *mut ff_visited_list_hdr_T;
    (*retptr).ffvl_visited_list = ::core::ptr::null_mut::<ff_visited_T>();
    (*retptr).ffvl_filename =
        xmemdupz(filename as *const ::core::ffi::c_void, filenamelen) as *mut ::core::ffi::c_char;
    (*retptr).ffvl_next = *list_headp as *mut ff_visited_list_hdr;
    *list_headp = retptr;
    return retptr;
}
unsafe extern "C" fn ff_wc_equal(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
) -> bool {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut prev1: ::core::ffi::c_int = NUL;
    let mut prev2: ::core::ffi::c_int = NUL;
    if s1 == s2 {
        return true_0 != 0;
    }
    if s1.is_null() || s2.is_null() {
        return false_0 != 0;
    }
    i = 0 as ::core::ffi::c_int;
    j = 0 as ::core::ffi::c_int;
    while *s1.offset(i as isize) as ::core::ffi::c_int != NUL
        && *s2.offset(j as isize) as ::core::ffi::c_int != NUL
    {
        let mut c1: ::core::ffi::c_int = utf_ptr2char(s1.offset(i as isize));
        let mut c2: ::core::ffi::c_int = utf_ptr2char(s2.offset(j as isize));
        if (if p_fic.get() != 0 {
            (mb_tolower(c1) != mb_tolower(c2)) as ::core::ffi::c_int
        } else {
            (c1 != c2) as ::core::ffi::c_int
        }) != 0
            && (prev1 != '*' as ::core::ffi::c_int || prev2 != '*' as ::core::ffi::c_int)
        {
            return false_0 != 0;
        }
        prev2 = prev1;
        prev1 = c1;
        i += utfc_ptr2len(s1.offset(i as isize));
        j += utfc_ptr2len(s2.offset(j as isize));
    }
    return *s1.offset(i as isize) as ::core::ffi::c_int
        == *s2.offset(j as isize) as ::core::ffi::c_int;
}
unsafe extern "C" fn ff_check_visited(
    mut visited_list: *mut *mut ff_visited_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fnamelen: size_t,
    mut wc_path: *mut ::core::ffi::c_char,
    mut wc_pathlen: size_t,
) -> ::core::ffi::c_int {
    let mut vp: *mut ff_visited_T = ::core::ptr::null_mut::<ff_visited_T>();
    let mut url: bool = false_0 != 0;
    let mut file_id: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    if path_with_url(fname) != 0 {
        xmemcpyz(
            (*ff_expand_buffer.ptr()).data as *mut ::core::ffi::c_void,
            fname as *const ::core::ffi::c_void,
            fnamelen,
        );
        (*ff_expand_buffer.ptr()).size = fnamelen;
        url = true_0 != 0;
    } else {
        *(*ff_expand_buffer.ptr())
            .data
            .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        (*ff_expand_buffer.ptr()).size = 0 as size_t;
        if !os_fileid(fname, &raw mut file_id) {
            return FAIL;
        }
    }
    vp = *visited_list;
    while !vp.is_null() {
        if url as ::core::ffi::c_int != 0
            && path_fnamecmp(
                &raw mut (*vp).ffv_fname as *mut ::core::ffi::c_char,
                (*ff_expand_buffer.ptr()).data,
            ) == 0 as ::core::ffi::c_int
            || !url
                && (*vp).file_id_valid as ::core::ffi::c_int != 0
                && os_fileid_equal(&raw mut (*vp).file_id, &raw mut file_id) as ::core::ffi::c_int
                    != 0
        {
            if ff_wc_equal((*vp).ffv_wc_path, wc_path) {
                return FAIL;
            }
        }
        vp = (*vp).ffv_next as *mut ff_visited_T;
    }
    vp = xmalloc(
        (40 as size_t)
            .wrapping_add((*ff_expand_buffer.ptr()).size)
            .wrapping_add(1 as size_t),
    ) as *mut ff_visited_T;
    if !url {
        (*vp).file_id_valid = true_0 != 0;
        (*vp).file_id = file_id;
        *(&raw mut (*vp).ffv_fname as *mut ::core::ffi::c_char)
            .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    } else {
        (*vp).file_id_valid = false_0 != 0;
        strcpy(
            &raw mut (*vp).ffv_fname as *mut ::core::ffi::c_char,
            (*ff_expand_buffer.ptr()).data,
        );
    }
    if !wc_path.is_null() {
        (*vp).ffv_wc_path =
            xmemdupz(wc_path as *const ::core::ffi::c_void, wc_pathlen) as *mut ::core::ffi::c_char;
    } else {
        (*vp).ffv_wc_path = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*vp).ffv_next = *visited_list as *mut ff_visited;
    *visited_list = vp;
    return OK;
}
unsafe extern "C" fn ff_create_stack_element(
    mut fix_part: *mut ::core::ffi::c_char,
    mut fix_partlen: size_t,
    mut wc_part: *mut ::core::ffi::c_char,
    mut wc_partlen: size_t,
    mut level: ::core::ffi::c_int,
    mut star_star_empty: ::core::ffi::c_int,
) -> *mut ff_stack_T {
    let mut stack: *mut ff_stack_T =
        xmalloc(::core::mem::size_of::<ff_stack_T>()) as *mut ff_stack_T;
    (*stack).ffs_prev = ::core::ptr::null_mut::<ff_stack>();
    (*stack).ffs_filearray = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    (*stack).ffs_filearray_size = 0 as ::core::ffi::c_int;
    (*stack).ffs_filearray_cur = 0 as ::core::ffi::c_int;
    (*stack).ffs_stage = 0 as ::core::ffi::c_int;
    (*stack).ffs_level = level;
    (*stack).ffs_star_star_empty = star_star_empty;
    if fix_part.is_null() {
        fix_part = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        fix_partlen = 0 as size_t;
    }
    (*stack).ffs_fix_path = cbuf_to_string(fix_part, fix_partlen);
    if wc_part.is_null() {
        wc_part = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        wc_partlen = 0 as size_t;
    }
    (*stack).ffs_wc_path = cbuf_to_string(wc_part, wc_partlen);
    return stack;
}
unsafe extern "C" fn ff_push(mut search_ctx: *mut ff_search_ctx_T, mut stack_ptr: *mut ff_stack_T) {
    if stack_ptr.is_null() {
        return;
    }
    (*stack_ptr).ffs_prev = (*search_ctx).ffsc_stack_ptr as *mut ff_stack;
    (*search_ctx).ffsc_stack_ptr = stack_ptr;
}
unsafe extern "C" fn ff_pop(mut search_ctx: *mut ff_search_ctx_T) -> *mut ff_stack_T {
    let mut sptr: *mut ff_stack_T = (*search_ctx).ffsc_stack_ptr;
    if !(*search_ctx).ffsc_stack_ptr.is_null() {
        (*search_ctx).ffsc_stack_ptr = (*(*search_ctx).ffsc_stack_ptr).ffs_prev as *mut ff_stack_T;
    }
    return sptr;
}
unsafe extern "C" fn ff_free_stack_element(stack_ptr: *mut ff_stack_T) {
    if stack_ptr.is_null() {
        return;
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*stack_ptr).ffs_fix_path.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*stack_ptr).ffs_fix_path.size = 0 as size_t;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*stack_ptr).ffs_wc_path.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
    (*stack_ptr).ffs_wc_path.size = 0 as size_t;
    if !(*stack_ptr).ffs_filearray.is_null() {
        FreeWild((*stack_ptr).ffs_filearray_size, (*stack_ptr).ffs_filearray);
    }
    xfree(stack_ptr as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn ff_clear(mut search_ctx: *mut ff_search_ctx_T) {
    let mut sptr: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
    loop {
        sptr = ff_pop(search_ctx);
        if sptr.is_null() {
            break;
        }
        ff_free_stack_element(sptr);
    }
    if !(*search_ctx).ffsc_stopdirs_v.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !(*(*search_ctx).ffsc_stopdirs_v.offset(i as isize))
            .data
            .is_null()
        {
            xfree(
                (*(*search_ctx).ffsc_stopdirs_v.offset(i as isize)).data
                    as *mut ::core::ffi::c_void,
            );
            i += 1;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*search_ctx).ffsc_stopdirs_v as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*search_ctx).ffsc_file_to_search.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
    (*search_ctx).ffsc_file_to_search.size = 0 as size_t;
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*search_ctx).ffsc_start_dir.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL;
    let _ = *ptr__1;
    (*search_ctx).ffsc_start_dir.size = 0 as size_t;
    let mut ptr__2: *mut *mut ::core::ffi::c_void =
        &raw mut (*search_ctx).ffsc_fix_path.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__2);
    *ptr__2 = NULL;
    let _ = *ptr__2;
    (*search_ctx).ffsc_fix_path.size = 0 as size_t;
    let mut ptr__3: *mut *mut ::core::ffi::c_void =
        &raw mut (*search_ctx).ffsc_wc_path.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__3);
    *ptr__3 = NULL;
    let _ = *ptr__3;
    (*search_ctx).ffsc_wc_path.size = 0 as size_t;
    (*search_ctx).ffsc_level = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn ff_path_in_stoplist(
    mut path: *mut ::core::ffi::c_char,
    mut path_len: size_t,
    mut stopdirs_v: *mut String_0,
) -> bool {
    while path_len > 1 as size_t
        && vim_ispathsep(
            *path.offset(path_len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
        ) as ::core::ffi::c_int
            != 0
    {
        path_len = path_len.wrapping_sub(1);
    }
    if path_len == 0 as size_t {
        return true_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !(*stopdirs_v.offset(i as isize)).data.is_null() {
        if path_fnamencmp((*stopdirs_v.offset(i as isize)).data, path, path_len)
            == 0 as ::core::ffi::c_int
            && ((*stopdirs_v.offset(i as isize)).size <= path_len
                || vim_ispathsep(
                    *(*stopdirs_v.offset(i as isize))
                        .data
                        .offset(path_len as isize) as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
        {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn find_file_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut first: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut file_to_find: *mut *mut ::core::ffi::c_char,
    mut search_ctx: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return find_file_in_path_option(
        ptr,
        len,
        options,
        first,
        if *(*curbuf.get()).b_p_path as ::core::ffi::c_int == NUL {
            p_path.get()
        } else {
            (*curbuf.get()).b_p_path
        },
        FINDFILE_BOTH as ::core::ffi::c_int,
        rel_fname,
        (*curbuf.get()).b_p_sua,
        file_to_find,
        search_ctx,
    );
}
pub unsafe extern "C" fn find_directory_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut file_to_find: *mut *mut ::core::ffi::c_char,
    mut search_ctx: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return find_file_in_path_option(
        ptr,
        len,
        options,
        true_0,
        p_cdpath.get(),
        FINDFILE_DIR as ::core::ffi::c_int,
        rel_fname,
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        file_to_find,
        search_ctx,
    );
}
pub unsafe extern "C" fn find_file_in_path_option(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut first: ::core::ffi::c_int,
    mut path_option: *mut ::core::ffi::c_char,
    mut find_what: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut suffixes: *mut ::core::ffi::c_char,
    mut file_to_find: *mut *mut ::core::ffi::c_char,
    mut search_ctx_arg: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut search_ctx: *mut *mut ff_search_ctx_T = search_ctx_arg as *mut *mut ff_search_ctx_T;
    static dir: GlobalCell<*mut ::core::ffi::c_char> =
        GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
    static did_findfile_init: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut file_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static file_to_findlen: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
    if !rel_fname.is_null() && path_with_url(rel_fname) != 0 {
        rel_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if first == true_0 {
        if len == 0 as size_t {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut save_char: ::core::ffi::c_char = *ptr.offset(len as isize);
        *ptr.offset(len as isize) = NUL as ::core::ffi::c_char;
        file_to_findlen.set(expand_env_esc(
            ptr,
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            MAXPATHL,
            false_0 != 0,
            true_0 != 0,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ));
        *ptr.offset(len as isize) = save_char;
        xfree(*file_to_find as *mut ::core::ffi::c_void);
        *file_to_find = xmemdupz(
            NameBuff.ptr() as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            file_to_findlen.get(),
        ) as *mut ::core::ffi::c_char;
        if options & FNAME_UNESC as ::core::ffi::c_int != 0 {
            ptr = *file_to_find;
            while *ptr as ::core::ffi::c_int != NUL {
                if *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int
                {
                    memmove(
                        ptr as *mut ::core::ffi::c_void,
                        ptr.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                        ((*file_to_find)
                            .offset(file_to_findlen.get() as isize)
                            .offset_from(ptr.offset(1 as ::core::ffi::c_int as isize))
                            as size_t)
                            .wrapping_add(1 as size_t),
                    );
                    file_to_findlen.set((*file_to_findlen.ptr()).wrapping_sub(1));
                }
                ptr = ptr.offset(1);
            }
        }
    }
    let mut rel_to_curdir: bool = *(*file_to_find).offset(0 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        && (*(*file_to_find).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || vim_ispathsep(
                *(*file_to_find).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) as ::core::ffi::c_int
                != 0
            || *(*file_to_find).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && (*(*file_to_find).offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == NUL
                    || vim_ispathsep(*(*file_to_find).offset(2 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int) as ::core::ffi::c_int
                        != 0));
    '_theend: {
        's_300: {
            if vim_isAbsName(*file_to_find) as ::core::ffi::c_int != 0
                || rel_to_curdir as ::core::ffi::c_int != 0
            {
                if first == true_0 {
                    if path_with_url(*file_to_find) != 0 {
                        file_name = xmemdupz(
                            *file_to_find as *const ::core::ffi::c_void,
                            file_to_findlen.get(),
                        ) as *mut ::core::ffi::c_char;
                        break '_theend;
                    } else {
                        let mut rel_fnamelen: size_t = if !rel_fname.is_null() {
                            strlen(rel_fname)
                        } else {
                            0 as size_t
                        };
                        let mut run: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                        loop {
                            if run > 2 as ::core::ffi::c_int {
                                break 's_300;
                            }
                            let mut l: size_t = file_to_findlen.get();
                            if run == 1 as ::core::ffi::c_int
                                && rel_to_curdir as ::core::ffi::c_int != 0
                                && options & FNAME_REL as ::core::ffi::c_int != 0
                                && !rel_fname.is_null()
                                && rel_fnamelen.wrapping_add(l) < MAXPATHL as size_t
                            {
                                l = vim_snprintf(
                                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                                    MAXPATHL as size_t,
                                    b"%.*s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    path_tail(rel_fname).offset_from(rel_fname)
                                        as ::core::ffi::c_int,
                                    rel_fname,
                                    *file_to_find,
                                ) as size_t;
                                '_c2rust_label: {
                                    if l < 4096 as size_t {
                                    } else {
                                        __assert_fail(
                                            b"l < MAXPATHL\0".as_ptr() as *const ::core::ffi::c_char,
                                            b"src/nvim/file_search.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            1499 as ::core::ffi::c_uint,
                                            b"char *find_file_in_path_option(char *, size_t, int, int, char *, int, char *, char *, char **, char **)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                            } else {
                                strcpy(NameBuff.ptr() as *mut ::core::ffi::c_char, *file_to_find);
                                run = 2 as ::core::ffi::c_int;
                            }
                            let mut NameBufflen: size_t = l;
                            let mut suffix: *mut ::core::ffi::c_char = suffixes;
                            loop {
                                if os_path_exists(NameBuff.ptr() as *mut ::core::ffi::c_char)
                                    as ::core::ffi::c_int
                                    != 0
                                    && (find_what == FINDFILE_BOTH as ::core::ffi::c_int
                                        || (find_what == FINDFILE_DIR as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                            == os_isdir(NameBuff.ptr() as *mut ::core::ffi::c_char)
                                                as ::core::ffi::c_int)
                                {
                                    file_name = xmemdupz(
                                        NameBuff.ptr() as *mut ::core::ffi::c_char
                                            as *const ::core::ffi::c_void,
                                        NameBufflen,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    break '_theend;
                                } else {
                                    if *suffix as ::core::ffi::c_int == NUL {
                                        break;
                                    }
                                    '_c2rust_label_0: {
                                        if 4096 as size_t >= l {
                                        } else {
                                            __assert_fail(
                                                b"MAXPATHL >= l\0".as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/file_search.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                1518 as ::core::ffi::c_uint,
                                                b"char *find_file_in_path_option(char *, size_t, int, int, char *, int, char *, char *, char **, char **)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    NameBufflen = l.wrapping_add(copy_option_part(
                                        &raw mut suffix,
                                        (NameBuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(l as isize),
                                        (MAXPATHL as size_t).wrapping_sub(l),
                                        b",\0".as_ptr() as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char,
                                    ));
                                }
                            }
                            run += 1;
                        }
                    }
                }
            } else {
                if first == true_0 {
                    vim_findfile_free_visited(*search_ctx as *mut ::core::ffi::c_void);
                    dir.set(path_option);
                    did_findfile_init.set(false_0 != 0);
                }
                loop {
                    if did_findfile_init.get() {
                        file_name = vim_findfile(*search_ctx as *mut ::core::ffi::c_void);
                        if !file_name.is_null() {
                            break;
                        }
                        did_findfile_init.set(false_0 != 0);
                    } else {
                        let mut r_ptr: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        if (*dir.ptr()).is_null() || *dir.get() as ::core::ffi::c_int == NUL {
                            vim_findfile_cleanup(*search_ctx as *mut ::core::ffi::c_void);
                            *search_ctx = ::core::ptr::null_mut::<ff_search_ctx_T>();
                            break;
                        } else {
                            let mut buf: *mut ::core::ffi::c_char =
                                xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
                            *buf.offset(0 as ::core::ffi::c_int as isize) =
                                NUL as ::core::ffi::c_char;
                            copy_option_part(
                                dir.ptr(),
                                buf,
                                MAXPATHL as size_t,
                                b" ,\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            );
                            r_ptr = vim_findfile_stopdir(buf);
                            *search_ctx = vim_findfile_init(
                                buf,
                                *file_to_find,
                                file_to_findlen.get(),
                                r_ptr,
                                100 as ::core::ffi::c_int,
                                false_0,
                                find_what,
                                *search_ctx as *mut ::core::ffi::c_void,
                                false_0,
                                rel_fname,
                            ) as *mut ff_search_ctx_T;
                            if !(*search_ctx).is_null() {
                                did_findfile_init.set(true_0 != 0);
                            }
                            xfree(buf as *mut ::core::ffi::c_void);
                        }
                    }
                }
            }
        }
        if file_name.is_null() && options & FNAME_MESS as ::core::ffi::c_int != 0 {
            if first == true_0 {
                if find_what == FINDFILE_DIR as ::core::ffi::c_int {
                    semsg(
                        gettext(
                            &raw const e_cant_find_directory_str_in_cdpath
                                as *const ::core::ffi::c_char,
                        ),
                        *file_to_find,
                    );
                } else {
                    semsg(
                        gettext(
                            &raw const e_cant_find_file_str_in_path as *const ::core::ffi::c_char,
                        ),
                        *file_to_find,
                    );
                }
            } else if find_what == FINDFILE_DIR as ::core::ffi::c_int {
                semsg(
                    gettext(
                        &raw const e_no_more_directory_str_found_in_cdpath
                            as *const ::core::ffi::c_char,
                    ),
                    *file_to_find,
                );
            } else {
                semsg(
                    gettext(
                        &raw const e_no_more_file_str_found_in_path as *const ::core::ffi::c_char,
                    ),
                    *file_to_find,
                );
            }
        }
    }
    return file_name;
}
pub unsafe extern "C" fn grab_file_name(
    mut count: ::core::ffi::c_int,
    mut file_lnum: *mut linenr_T,
) -> *mut ::core::ffi::c_char {
    let mut options: ::core::ffi::c_int = FNAME_MESS as ::core::ffi::c_int
        | FNAME_EXP as ::core::ffi::c_int
        | FNAME_REL as ::core::ffi::c_int
        | FNAME_UNESC as ::core::ffi::c_int;
    if VIsual_active.get() {
        let mut len: size_t = 0;
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if get_visual_text(
            ::core::ptr::null_mut::<cmdarg_T>(),
            &raw mut ptr,
            &raw mut len,
        ) as ::core::ffi::c_int
            == FAIL
        {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !file_lnum.is_null()
            && *ptr.offset(len as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *(*__ctype_b_loc()).offset(*ptr.offset(len.wrapping_add(1 as size_t) as isize)
                as uint8_t as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
        {
            let mut p: *mut ::core::ffi::c_char = ptr
                .offset(len as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            *file_lnum = getdigits_int32(&raw mut p, false_0 != 0, 0 as int32_t) as linenr_T;
        }
        return find_file_name_in_path(
            ptr,
            len,
            options,
            count as ::core::ffi::c_long,
            (*curbuf.get()).b_ffname,
        );
    }
    return file_name_at_cursor(options | FNAME_HYP as ::core::ffi::c_int, count, file_lnum);
}
pub unsafe extern "C" fn file_name_at_cursor(
    mut options: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut file_lnum: *mut linenr_T,
) -> *mut ::core::ffi::c_char {
    return file_name_in_line(
        get_cursor_line_ptr(),
        (*curwin.get()).w_cursor.col as ::core::ffi::c_int,
        options,
        count,
        (*curbuf.get()).b_ffname,
        file_lnum,
    );
}
pub unsafe extern "C" fn file_name_in_line(
    mut line: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut file_lnum: *mut linenr_T,
) -> *mut ::core::ffi::c_char {
    let mut ptr: *mut ::core::ffi::c_char = line.offset(col as isize);
    while *ptr as ::core::ffi::c_int != NUL && !vim_isfilec(*ptr as uint8_t as ::core::ffi::c_int) {
        ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
    }
    if *ptr as ::core::ffi::c_int == NUL {
        if options & FNAME_MESS as ::core::ffi::c_int != 0 {
            emsg(gettext(
                b"E446: No file name under cursor\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut len: size_t = 0;
    let mut in_type: bool = true_0 != 0;
    let mut is_url: bool = false_0 != 0;
    while ptr > line {
        len = utf_head_off(line, ptr.offset(-(1 as ::core::ffi::c_int as isize))) as size_t;
        if len > 0 as size_t {
            ptr = ptr.offset(-(len.wrapping_add(1 as size_t) as isize));
        } else {
            if !(vim_isfilec(
                *ptr.offset(-1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            ) as ::core::ffi::c_int
                != 0
                || options & FNAME_HYP as ::core::ffi::c_int != 0
                    && path_is_url(ptr.offset(-(1 as ::core::ffi::c_int as isize))) != 0)
            {
                break;
            }
            ptr = ptr.offset(-1);
        }
    }
    len = (if path_has_drive_letter(ptr, strlen(ptr)) as ::core::ffi::c_int != 0 {
        2 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as size_t;
    while vim_isfilec(*ptr.offset(len as isize) as uint8_t as ::core::ffi::c_int)
        as ::core::ffi::c_int
        != 0
        || *ptr.offset(len as isize) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *ptr.offset(len.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == ' ' as ::core::ffi::c_int
        || options & FNAME_HYP as ::core::ffi::c_int != 0
            && path_is_url(ptr.offset(len as isize)) != 0
        || is_url as ::core::ffi::c_int != 0
            && !vim_strchr(
                b":?&=\0".as_ptr() as *const ::core::ffi::c_char,
                *ptr.offset(len as isize) as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
    {
        if *ptr.offset(len as isize) as ::core::ffi::c_int >= 'A' as ::core::ffi::c_int
            && *ptr.offset(len as isize) as ::core::ffi::c_int <= 'Z' as ::core::ffi::c_int
            || *ptr.offset(len as isize) as ::core::ffi::c_int >= 'a' as ::core::ffi::c_int
                && *ptr.offset(len as isize) as ::core::ffi::c_int <= 'z' as ::core::ffi::c_int
        {
            if in_type as ::core::ffi::c_int != 0
                && path_is_url(
                    ptr.offset(len as isize)
                        .offset(1 as ::core::ffi::c_int as isize),
                ) != 0
            {
                is_url = true_0 != 0;
            }
        } else {
            in_type = false_0 != 0;
        }
        if *ptr.offset(len as isize) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *ptr.offset(len.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == ' ' as ::core::ffi::c_int
        {
            len = len.wrapping_add(1);
        }
        len = len.wrapping_add(utfc_ptr2len(ptr.offset(len as isize)) as size_t);
    }
    if len > 2 as size_t
        && !vim_strchr(
            b".,:;!\0".as_ptr() as *const ::core::ffi::c_char,
            *ptr.offset(len.wrapping_sub(1 as size_t) as isize) as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        && *ptr.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
            != '.' as ::core::ffi::c_int
    {
        len = len.wrapping_sub(1);
    }
    if !file_lnum.is_null() {
        let mut match_text: *const ::core::ffi::c_char =
            b" line \0".as_ptr() as *const ::core::ffi::c_char;
        let mut match_textlen: size_t = 6 as size_t;
        let mut p: *mut ::core::ffi::c_char = ptr.offset(len as isize);
        if strncmp(p, match_text, match_textlen) == 0 as ::core::ffi::c_int {
            p = p.offset(match_textlen as isize);
        } else {
            match_text = gettext(&raw const line_msg as *const ::core::ffi::c_char);
            match_textlen = strlen(match_text);
            if strncmp(p, match_text, match_textlen) == 0 as ::core::ffi::c_int {
                p = p.offset(match_textlen as isize);
            } else {
                p = skipwhite(p);
            }
        }
        if *p as ::core::ffi::c_int != NUL {
            if *(*__ctype_b_loc()).offset(*p as uint8_t as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            {
                p = p.offset(1);
            }
            p = skipwhite(p);
            if *(*__ctype_b_loc()).offset(*p as uint8_t as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
            {
                *file_lnum =
                    getdigits_long(&raw mut p, false_0 != 0, 0 as ::core::ffi::c_long) as linenr_T;
            }
        }
    }
    return find_file_name_in_path(ptr, len, options, count as ::core::ffi::c_long, rel_fname);
}
unsafe extern "C" fn eval_includeexpr(
    ptr: *const ::core::ffi::c_char,
    len: size_t,
) -> *mut ::core::ffi::c_char {
    let save_sctx: sctx_T = current_sctx.get();
    set_vim_var_string(VV_FNAME, ptr, len as ptrdiff_t);
    current_sctx
        .set((*curbuf.get()).b_p_script_ctx[kBufOptIncludeexpr as ::core::ffi::c_int as usize]);
    let mut res: *mut ::core::ffi::c_char = eval_to_string_safe(
        (*curbuf.get()).b_p_inex,
        was_set_insecurely(
            curwin.get(),
            kOptIncludeexpr,
            OPT_LOCAL as ::core::ffi::c_int,
        ) != 0,
        true_0 != 0,
    );
    set_vim_var_string(
        VV_FNAME,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ptrdiff_t,
    );
    current_sctx.set(save_sctx);
    return res;
}
pub unsafe extern "C" fn find_file_name_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut count: ::core::ffi::c_long,
    mut rel_fname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut file_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if len == 0 as size_t {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if options & FNAME_HYP as ::core::ffi::c_int != 0
        && len > 6 as size_t
        && strncmp(
            ptr,
            b"file:/\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        && !vim_ispathsep(*ptr.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
    {
        let mut off: size_t = (if path_has_drive_letter(
            ptr.offset(6 as ::core::ffi::c_int as isize),
            len.wrapping_sub(6 as size_t),
        ) as ::core::ffi::c_int
            != 0
        {
            6 as ::core::ffi::c_int
        } else {
            5 as ::core::ffi::c_int
        }) as size_t;
        ptr = ptr.offset(off as isize);
        len = len.wrapping_sub(off);
    }
    if options & FNAME_INCL as ::core::ffi::c_int != 0
        && *(*curbuf.get()).b_p_inex as ::core::ffi::c_int != NUL
    {
        tofree = eval_includeexpr(ptr, len);
        if !tofree.is_null() {
            ptr = tofree;
            len = strlen(ptr);
        }
    }
    if options & FNAME_EXP as ::core::ffi::c_int != 0 {
        let mut file_to_find: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut search_ctx: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        file_name = find_file_in_path(
            ptr,
            len,
            options & !(FNAME_MESS as ::core::ffi::c_int),
            true_0,
            rel_fname,
            &raw mut file_to_find,
            &raw mut search_ctx,
        );
        if file_name.is_null()
            && options & FNAME_INCL as ::core::ffi::c_int == 0
            && *(*curbuf.get()).b_p_inex as ::core::ffi::c_int != NUL
        {
            tofree = eval_includeexpr(ptr, len);
            if !tofree.is_null() {
                ptr = tofree;
                len = strlen(ptr);
                file_name = find_file_in_path(
                    ptr,
                    len,
                    options & !(FNAME_MESS as ::core::ffi::c_int),
                    true_0,
                    rel_fname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
            }
        }
        if file_name.is_null() && options & FNAME_MESS as ::core::ffi::c_int != 0 {
            let mut c: ::core::ffi::c_char = *ptr.offset(len as isize);
            *ptr.offset(len as isize) = NUL as ::core::ffi::c_char;
            semsg(
                gettext(b"E447: Can't find file \"%s\" in path\0".as_ptr()
                    as *const ::core::ffi::c_char),
                ptr,
            );
            *ptr.offset(len as isize) = c;
        }
        while !file_name.is_null() && {
            count -= 1;
            count > 0 as ::core::ffi::c_long
        } {
            xfree(file_name as *mut ::core::ffi::c_void);
            file_name = find_file_in_path(
                ptr,
                len,
                options,
                false_0,
                rel_fname,
                &raw mut file_to_find,
                &raw mut search_ctx,
            );
        }
        xfree(file_to_find as *mut ::core::ffi::c_void);
        vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
    } else {
        file_name = xstrnsave(ptr, len);
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    return file_name;
}
pub unsafe extern "C" fn do_autocmd_dirchanged(
    mut new_dir: *mut ::core::ffi::c_char,
    mut scope: CdScope,
    mut cause: CdCause,
    mut pre: bool,
) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut event: event_T = (if pre as ::core::ffi::c_int != 0 {
        EVENT_DIRCHANGEDPRE as ::core::ffi::c_int
    } else {
        EVENT_DIRCHANGED as ::core::ffi::c_int
    }) as event_T;
    if recursive.get() as ::core::ffi::c_int != 0 || !has_event(event) {
        return;
    }
    recursive.set(true_0 != 0);
    let mut save_v_event: save_v_event_T = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: hashtab_T {
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
    };
    let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
    let mut buf: [::core::ffi::c_char; 8] = [0; 8];
    match scope as ::core::ffi::c_int {
        2 => {
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                b"global\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        1 => {
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                b"tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        0 => {
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                b"window\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        -1 => {
            abort();
        }
        _ => {}
    }
    if pre {
        tv_dict_add_str(
            dict,
            b"directory\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            new_dir,
        );
    } else {
        tv_dict_add_str(
            dict,
            b"cwd\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            new_dir,
        );
    }
    tv_dict_add_str(
        dict,
        b"scope\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    tv_dict_add_bool(
        dict,
        b"changed_window\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 15]>().wrapping_sub(1 as size_t),
        (cause as ::core::ffi::c_int == kCdCauseWindow as ::core::ffi::c_int) as ::core::ffi::c_int
            as BoolVarValue,
    );
    tv_dict_set_keys_readonly(dict);
    match cause as ::core::ffi::c_int {
        2 => {
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                b"auto\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        -1 => {
            abort();
        }
        0 | 1 | _ => {}
    }
    apply_autocmds(
        event,
        &raw mut buf as *mut ::core::ffi::c_char,
        new_dir,
        false_0 != 0,
        curbuf.get(),
    );
    restore_v_event(dict, &raw mut save_v_event);
    recursive.set(false_0 != 0);
}
pub unsafe extern "C" fn vim_chdirfile(
    mut fname: *mut ::core::ffi::c_char,
    mut cause: CdCause,
) -> ::core::ffi::c_int {
    let mut dir: [::core::ffi::c_char; 4096] = [0; 4096];
    xstrlcpy(
        &raw mut dir as *mut ::core::ffi::c_char,
        fname,
        MAXPATHL as size_t,
    );
    *path_tail_with_sep(&raw mut dir as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
    if os_dirname(
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
    ) != OK
    {
        (*NameBuff.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    }
    if pathcmp(
        &raw mut dir as *mut ::core::ffi::c_char,
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        -1 as ::core::ffi::c_int,
    ) == 0 as ::core::ffi::c_int
    {
        return OK;
    }
    if cause as ::core::ffi::c_int != kCdCauseOther as ::core::ffi::c_int {
        do_autocmd_dirchanged(
            &raw mut dir as *mut ::core::ffi::c_char,
            kCdScopeWindow,
            cause,
            true_0 != 0,
        );
    }
    if os_chdir(&raw mut dir as *mut ::core::ffi::c_char) != 0 as ::core::ffi::c_int {
        return FAIL;
    }
    if cause as ::core::ffi::c_int != kCdCauseOther as ::core::ffi::c_int {
        do_autocmd_dirchanged(
            &raw mut dir as *mut ::core::ffi::c_char,
            kCdScopeWindow,
            cause,
            false_0 != 0,
        );
    }
    return OK;
}
pub unsafe extern "C" fn vim_chdir(mut new_dir: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut file_to_find: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut search_ctx: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dir_name: *mut ::core::ffi::c_char = find_directory_in_path(
        new_dir,
        strlen(new_dir),
        FNAME_MESS as ::core::ffi::c_int,
        (*curbuf.get()).b_ffname,
        &raw mut file_to_find,
        &raw mut search_ctx,
    );
    xfree(file_to_find as *mut ::core::ffi::c_void);
    vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
    if dir_name.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = os_chdir(dir_name);
    xfree(dir_name as *mut ::core::ffi::c_void);
    return r;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
