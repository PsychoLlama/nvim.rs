use crate::src::nvim::api::buffer::{nvim_buf_get_lines, nvim_buf_set_lines};
use crate::src::nvim::api::extmark::{
    nvim_buf_clear_namespace, nvim_create_namespace, parse_virt_text,
};
use crate::src::nvim::api::private::dispatch::msgpack_rpc_get_handler_for;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_object, api_set_error, api_set_sctx, api_typename, arena_array,
    copy_object, copy_string, cstr_as_string, dict_set_var, find_buffer_by_handle,
    find_tab_by_handle, find_window_by_handle,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid};
use crate::src::nvim::api::vimscript::exec_impl;
use crate::src::nvim::decoration::{clear_virttext, decor_find_virttext};
use crate::src::nvim::eval::vars::get_globvar_dict;
use crate::src::nvim::extmark::extmark_set;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight::hl_get_attr_by_id;
use crate::src::nvim::highlight_group::{
    syn_check_group, syn_get_final_id, syn_id2attr, syn_name2id,
};
use crate::src::nvim::lua::executor::nlua_exec;
use crate::src::nvim::main::{
    curbuf, current_sctx, curwin, got_int, msg_didout, msg_silent, no_wait_return,
};
use crate::src::nvim::memory::{xmalloc, xrealloc};
use crate::src::nvim::message::{emsg, msg, msg_end};
use crate::src::nvim::option::{
    find_option, get_option_value_for, get_vimoption, object_as_optval, option_has_scope,
    optval_as_object, set_option_value_for,
};
pub use crate::src::nvim::types::{
    __time_t, alist_T, bcount_t, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T,
    chunksize_T, colnr_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T, extmark_undo_vec_t,
    fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_12, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, key_value_pair, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    lua_State, mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T,
    mfdirty_T, mtnode_inner_s, mtnode_s, object, object_data as C2Rust_Unnamed, partial_S,
    partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S, qf_info_T, queue,
    reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t,
    syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T,
    tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry,
    u_entry_T, u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_9,
    u_header_uh_alt_prev as C2Rust_Unnamed_8, u_header_uh_next as C2Rust_Unnamed_11,
    u_header_uh_prev as C2Rust_Unnamed_10, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, undo_object_data as C2Rust_Unnamed_7, varnumber_T, virt_line, visualinfo_T, win_T,
    window_S, wininfo_S, winopt_T, wline_T, xfmark_T, AdditionalData, AlignTextPos,
    ApiDispatchWrapper, Arena, Array, BoolVarValue, Boolean, BufUpdateCallbacks, Buffer, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, ErrorType, ExtmarkMove, ExtmarkSavePos,
    ExtmarkSplice, ExtmarkUndoObject, FileID, Float, FloatAnchor, FloatRelative, GridView, Integer,
    Intersection, KeyDict_empty, KeyDict_exec_opts, KeyValuePair, LuaRef, LuaRetMode, MTKey,
    MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MsgpackRpcRequestHandler, Object, ObjectType, OptIndex, OptInt,
    OptScope, OptVal, OptValData, OptValType, OptionalKeys, ScopeDictDictItem, ScopeType,
    ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_13, StringBuilder, String_0, Tabpage, Terminal,
    Timestamp, TriState, UndoObjectType, VarLockStatus, VarType, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit, WinStyle, Window, QUEUE,
};
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
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const kHlModeUnknown: C2Rust_Unnamed_16 = 0;
pub const OPT_GLOBAL: C2Rust_Unnamed_17 = 1;
pub const kOptScopeBuf: OptScope = 2;
pub const kOptScopeWin: OptScope = 1;
pub const kOptScopeGlobal: OptScope = 0;
pub const OPT_LOCAL: C2Rust_Unnamed_17 = 2;
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
pub const LINE_BUFFER_MIN_SIZE: C2Rust_Unnamed_18 = 4096;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kHlModeBlend: C2Rust_Unnamed_16 = 3;
pub const kHlModeCombine: C2Rust_Unnamed_16 = 2;
pub const kHlModeReplace: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_17 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_17 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_17 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_17 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_17 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_17 = 4;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub unsafe extern "C" fn nvim_exec(
    mut channel_id: uint64_t,
    mut src: String_0,
    mut output: Boolean,
    mut err: *mut Error,
) -> String_0 {
    let mut opts: KeyDict_exec_opts = KeyDict_exec_opts { output: output };
    return exec_impl(channel_id, src, &raw mut opts, err);
}
pub unsafe extern "C" fn nvim_command_output(
    mut channel_id: uint64_t,
    mut command: String_0,
    mut err: *mut Error,
) -> String_0 {
    let mut opts: KeyDict_exec_opts = KeyDict_exec_opts {
        output: true_0 != 0,
    };
    return exec_impl(channel_id, command, &raw mut opts, err);
}
pub unsafe extern "C" fn nvim_execute_lua(
    mut code: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return nlua_exec(
        code,
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetObject,
        arena,
        err,
    );
}
pub unsafe extern "C" fn nvim_buf_get_number(mut buffer: Buffer, mut err: *mut Error) -> Integer {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return 0 as Integer;
    }
    return (*buf).handle as Integer;
}
unsafe extern "C" fn src2ns(mut src_id: *mut Integer) -> uint32_t {
    if *src_id == 0 as Integer {
        *src_id = nvim_create_namespace(String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        });
    }
    if *src_id < 0 as Integer {
        return ((1 as ::core::ffi::c_int as uint32_t) << 31 as ::core::ffi::c_int)
            .wrapping_sub(1 as uint32_t);
    }
    return *src_id as uint32_t;
}
pub unsafe extern "C" fn nvim_buf_clear_highlight(
    mut buffer: Buffer,
    mut ns_id: Integer,
    mut line_start: Integer,
    mut line_end: Integer,
    mut err: *mut Error,
) {
    nvim_buf_clear_namespace(buffer, ns_id, line_start, line_end, err);
}
pub unsafe extern "C" fn nvim_buf_add_highlight(
    mut buffer: Buffer,
    mut ns_id: Integer,
    mut hl_group: String_0,
    mut line: Integer,
    mut col_start: Integer,
    mut col_end: Integer,
    mut err: *mut Error,
) -> Integer {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return 0 as Integer;
    }
    if !(line >= 0 as Integer && line < MAXLNUM as ::core::ffi::c_int as Integer) {
        api_err_invalid(
            err,
            b"line number\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return 0 as Integer;
    }
    if !(col_start >= 0 as Integer && col_start <= MAXCOL as ::core::ffi::c_int as Integer) {
        api_err_invalid(
            err,
            b"column\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return 0 as Integer;
    }
    if col_end < 0 as Integer || col_end > MAXCOL as ::core::ffi::c_int as Integer {
        col_end = MAXCOL as ::core::ffi::c_int as Integer;
    }
    let mut ns: uint32_t = src2ns(&raw mut ns_id);
    if !(line < (*buf).b_ml.ml_line_count as Integer) {
        return ns_id;
    }
    let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if hl_group.size > 0 as size_t {
        hl_id = syn_check_group(hl_group.data, hl_group.size);
    } else {
        return ns_id;
    }
    let mut end_line: ::core::ffi::c_int = line as ::core::ffi::c_int;
    if col_end == MAXCOL as ::core::ffi::c_int as Integer {
        col_end = 0 as Integer;
        end_line += 1;
    }
    let mut decor: DecorInline = DECOR_INLINE_INIT;
    decor.data.hl.hl_id = hl_id;
    extmark_set(
        buf,
        ns,
        ::core::ptr::null_mut::<uint32_t>(),
        line as ::core::ffi::c_int,
        col_start as colnr_T,
        end_line,
        col_end as colnr_T,
        decor,
        MT_FLAG_DECOR_HL as uint16_t,
        true_0 != 0,
        false_0 != 0,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Error>(),
    );
    return ns_id;
}
pub unsafe extern "C" fn nvim_buf_set_virtual_text(
    mut buffer: Buffer,
    mut src_id: Integer,
    mut line: Integer,
    mut chunks: Array,
    mut _opts: *mut KeyDict_empty,
    mut err: *mut Error,
) -> Integer {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return 0 as Integer;
    }
    if line < 0 as Integer || line >= MAXLNUM as ::core::ffi::c_int as Integer {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Line number outside range\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 0 as Integer;
    }
    let mut ns_id: uint32_t = src2ns(&raw mut src_id);
    let mut width: ::core::ffi::c_int = 0;
    let mut virt_text: VirtText = parse_virt_text(chunks, err, &raw mut width);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return 0 as Integer;
    }
    let mut existing: *mut DecorVirtText =
        decor_find_virttext(buf, line as ::core::ffi::c_int, ns_id as uint64_t);
    if !existing.is_null() {
        clear_virttext(&raw mut (*existing).data.virt_text);
        (*existing).data.virt_text = virt_text;
        (*existing).width = width;
        return src_id;
    }
    let mut vt: *mut DecorVirtText =
        xmalloc(::core::mem::size_of::<DecorVirtText>()) as *mut DecorVirtText;
    *vt = DecorVirtText {
        flags: 0 as uint8_t,
        hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
        priority: DECOR_PRIORITY_BASE as DecorPriority,
        width: 0 as ::core::ffi::c_int,
        col: 0 as ::core::ffi::c_int,
        pos: kVPosEndOfLine,
        data: C2Rust_Unnamed_2 {
            virt_text: VirtText {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
        },
        next: ::core::ptr::null_mut::<DecorVirtText>(),
    };
    (*vt).data.virt_text = virt_text;
    (*vt).width = width;
    (*vt).priority = 0 as DecorPriority;
    let mut decor: DecorInline = DecorInline {
        ext: true_0 != 0,
        data: DecorInlineData {
            ext: DecorExt {
                sh_idx: DECOR_ID_INVALID as uint32_t,
                vt: vt,
            },
        },
    };
    extmark_set(
        buf,
        ns_id,
        ::core::ptr::null_mut::<uint32_t>(),
        line as ::core::ffi::c_int,
        0 as colnr_T,
        -1 as ::core::ffi::c_int,
        -1 as colnr_T,
        decor,
        0 as uint16_t,
        true_0 != 0,
        false_0 != 0,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Error>(),
    );
    return src_id;
}
pub unsafe extern "C" fn nvim_get_hl_by_id(
    mut hl_id: Integer,
    mut rgb: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut dic: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    if !(syn_get_final_id(hl_id as ::core::ffi::c_int) != 0 as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"highlight id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            hl_id as int64_t,
            false_0 != 0,
        );
        return dic;
    }
    let mut attrcode: ::core::ffi::c_int = syn_id2attr(hl_id as ::core::ffi::c_int);
    return hl_get_attr_by_id(attrcode as Integer, rgb, arena, err);
}
pub unsafe extern "C" fn nvim_get_hl_by_name(
    mut name: String_0,
    mut rgb: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut result: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut id: ::core::ffi::c_int = syn_name2id(name.data);
    if !(id != 0 as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"highlight name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return result;
    }
    return nvim_get_hl_by_id(id as Integer, rgb, arena, err);
}
pub unsafe extern "C" fn buffer_insert(
    mut buffer: Buffer,
    mut lnum: Integer,
    mut lines: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    nvim_buf_set_lines(
        0 as uint64_t,
        buffer,
        lnum,
        lnum,
        true_0 != 0,
        lines,
        arena,
        err,
    );
}
pub unsafe extern "C" fn buffer_get_line(
    mut buffer: Buffer,
    mut index: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    let mut rv: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    index = convert_index(index as int64_t) as Integer;
    let mut slice: Array = nvim_buf_get_lines(
        0 as uint64_t,
        buffer,
        index,
        index + 1 as Integer,
        true_0 != 0,
        arena,
        ::core::ptr::null_mut::<lua_State>(),
        err,
    );
    if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
        && slice.size != 0
    {
        rv = (*slice.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .string;
    }
    return rv;
}
pub unsafe extern "C" fn buffer_set_line(
    mut buffer: Buffer,
    mut index: Integer,
    mut line: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut l: Object = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: line },
    };
    let mut array: Array = Array {
        size: 1 as size_t,
        capacity: 0,
        items: &raw mut l,
    };
    index = convert_index(index as int64_t) as Integer;
    nvim_buf_set_lines(
        0 as uint64_t,
        buffer,
        index,
        index + 1 as Integer,
        true_0 != 0,
        array,
        arena,
        err,
    );
}
pub unsafe extern "C" fn buffer_del_line(
    mut buffer: Buffer,
    mut index: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut array: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    index = convert_index(index as int64_t) as Integer;
    nvim_buf_set_lines(
        0 as uint64_t,
        buffer,
        index,
        index + 1 as Integer,
        true_0 != 0,
        array,
        arena,
        err,
    );
}
pub unsafe extern "C" fn buffer_get_line_slice(
    mut buffer: Buffer,
    mut start: Integer,
    mut end: Integer,
    mut include_start: Boolean,
    mut include_end: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    start = (convert_index(start as int64_t) + !include_start as ::core::ffi::c_int as int64_t)
        as Integer;
    end = (convert_index(end as int64_t) + include_end as int64_t) as Integer;
    return nvim_buf_get_lines(
        0 as uint64_t,
        buffer,
        start,
        end,
        false_0 != 0,
        arena,
        ::core::ptr::null_mut::<lua_State>(),
        err,
    );
}
pub unsafe extern "C" fn buffer_set_line_slice(
    mut buffer: Buffer,
    mut start: Integer,
    mut end: Integer,
    mut include_start: Boolean,
    mut include_end: Boolean,
    mut replacement: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    start = (convert_index(start as int64_t) + !include_start as ::core::ffi::c_int as int64_t)
        as Integer;
    end = (convert_index(end as int64_t) + include_end as int64_t) as Integer;
    nvim_buf_set_lines(
        0 as uint64_t,
        buffer,
        start,
        end,
        false_0 != 0,
        replacement,
        arena,
        err,
    );
}
pub unsafe extern "C" fn buffer_set_var(
    mut buffer: Buffer,
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*buf).b_vars,
        name,
        value,
        false_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn buffer_del_var(
    mut buffer: Buffer,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*buf).b_vars,
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn window_set_var(
    mut window: Window,
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut win: *mut win_T = find_window_by_handle(window, err);
    if win.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*win).w_vars,
        name,
        value,
        false_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn window_del_var(
    mut window: Window,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut win: *mut win_T = find_window_by_handle(window, err);
    if win.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*win).w_vars,
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn tabpage_set_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*tab).tp_vars,
        name,
        value,
        false_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn tabpage_del_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_set_var(
        (*tab).tp_vars,
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn vim_set_var(
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return dict_set_var(
        get_globvar_dict(),
        name,
        value,
        false_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
pub unsafe extern "C" fn vim_del_var(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return dict_set_var(
        get_globvar_dict(),
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        true_0 != 0,
        arena,
        err,
    );
}
unsafe extern "C" fn convert_index(mut index: int64_t) -> int64_t {
    return if index < 0 as int64_t {
        index - 1 as int64_t
    } else {
        index
    };
}
pub unsafe extern "C" fn nvim_get_option_info(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    return get_vimoption(
        name,
        OPT_GLOBAL as ::core::ffi::c_int,
        curbuf.get(),
        curwin.get(),
        arena,
        err,
    );
}
pub unsafe extern "C" fn nvim_set_option(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    set_option_to(channel_id, NULL, kOptScopeGlobal, name, value, err);
}
pub unsafe extern "C" fn nvim_get_option(mut name: String_0, mut err: *mut Error) -> Object {
    return get_option_from(NULL, kOptScopeGlobal, name, err);
}
pub unsafe extern "C" fn nvim_buf_get_option(
    mut buffer: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) -> Object {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return get_option_from(buf as *mut ::core::ffi::c_void, kOptScopeBuf, name, err);
}
pub unsafe extern "C" fn nvim_buf_set_option(
    mut channel_id: uint64_t,
    mut buffer: Buffer,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
    if buf.is_null() {
        return;
    }
    set_option_to(
        channel_id,
        buf as *mut ::core::ffi::c_void,
        kOptScopeBuf,
        name,
        value,
        err,
    );
}
pub unsafe extern "C" fn nvim_win_get_option(
    mut window: Window,
    mut name: String_0,
    mut err: *mut Error,
) -> Object {
    let mut win: *mut win_T = find_window_by_handle(window, err);
    if win.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return get_option_from(win as *mut ::core::ffi::c_void, kOptScopeWin, name, err);
}
pub unsafe extern "C" fn nvim_win_set_option(
    mut channel_id: uint64_t,
    mut window: Window,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    let mut win: *mut win_T = find_window_by_handle(window, err);
    if win.is_null() {
        return;
    }
    set_option_to(
        channel_id,
        win as *mut ::core::ffi::c_void,
        kOptScopeWin,
        name,
        value,
        err,
    );
}
unsafe extern "C" fn get_option_from(
    mut from: *mut ::core::ffi::c_void,
    mut scope: OptScope,
    mut name: String_0,
    mut err: *mut Error,
) -> Object {
    if !(name.size > 0 as size_t) {
        api_err_invalid(
            err,
            b"option name\0".as_ptr() as *const ::core::ffi::c_char,
            b"<empty>\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            true_0 != 0,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    let mut opt_idx: OptIndex = find_option(name.data);
    if !(opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"option name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    let mut value: OptVal = OptVal {
        type_0: kOptValTypeNil,
        data: OptValData { boolean: kFalse },
    };
    if option_has_scope(opt_idx, scope) {
        value = get_option_value_for(
            opt_idx,
            if scope as ::core::ffi::c_uint
                == kOptScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                OPT_GLOBAL as ::core::ffi::c_int
            } else {
                OPT_LOCAL as ::core::ffi::c_int
            },
            scope,
            from,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
    }
    if !(value.type_0 as ::core::ffi::c_int != kOptValTypeNil as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"option name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return optval_as_object(value);
}
unsafe extern "C" fn set_option_to(
    mut channel_id: uint64_t,
    mut to: *mut ::core::ffi::c_void,
    mut scope: OptScope,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    if !(name.size > 0 as size_t) {
        api_err_invalid(
            err,
            b"option name\0".as_ptr() as *const ::core::ffi::c_char,
            b"<empty>\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    let mut opt_idx: OptIndex = find_option(name.data);
    if !(opt_idx as ::core::ffi::c_int != kOptInvalid as ::core::ffi::c_int) {
        api_err_invalid(
            err,
            b"option name\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
    let mut error: bool = false_0 != 0;
    let mut optval: OptVal = object_as_optval(value, &raw mut error);
    if error {
        api_err_exp(
            err,
            b"value\0".as_ptr() as *const ::core::ffi::c_char,
            b"valid option type\0".as_ptr() as *const ::core::ffi::c_char,
            api_typename(value.type_0),
        );
        return;
    }
    let opt_flags: ::core::ffi::c_int = if scope as ::core::ffi::c_uint
        == kOptScopeWin as ::core::ffi::c_int as ::core::ffi::c_uint
        && !option_has_scope(opt_idx, kOptScopeGlobal)
    {
        0 as ::core::ffi::c_int
    } else if scope as ::core::ffi::c_uint
        == kOptScopeGlobal as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        OPT_GLOBAL as ::core::ffi::c_int
    } else {
        OPT_LOCAL as ::core::ffi::c_int
    };
    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
    set_option_value_for(name.data, opt_idx, optval, opt_flags, scope, to, err);
    current_sctx.set(save_current_sctx);
}
pub unsafe extern "C" fn nvim_call_atomic(
    mut channel_id: uint64_t,
    mut calls: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = arena_array(arena, 2 as size_t);
    let mut results: Array = arena_array(arena, calls.size);
    let mut nested_error: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut i: size_t = 0;
    i = 0 as size_t;
    '_theend: {
        while i < calls.size {
            if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                != (*calls.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
            {
                api_err_exp(
                    err,
                    b"'calls' item\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(kObjectTypeArray),
                    api_typename((*calls.items.offset(i as isize)).type_0),
                );
                break '_theend;
            } else {
                let mut call: Array = (*calls.items.offset(i as isize)).data.array;
                if !(call.size == 2 as size_t) {
                    api_err_exp(
                        err,
                        b"'calls' item\0".as_ptr() as *const ::core::ffi::c_char,
                        b"2-item Array\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_theend;
                } else if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    != (*call.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                        as ::core::ffi::c_uint
                {
                    api_err_exp(
                        err,
                        b"name\0".as_ptr() as *const ::core::ffi::c_char,
                        api_typename(kObjectTypeString),
                        api_typename((*call.items.offset(0 as ::core::ffi::c_int as isize)).type_0),
                    );
                    break '_theend;
                } else {
                    let mut name: String_0 = (*call.items.offset(0 as ::core::ffi::c_int as isize))
                        .data
                        .string;
                    if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                        != (*call.items.offset(1 as ::core::ffi::c_int as isize)).type_0
                            as ::core::ffi::c_uint
                    {
                        api_err_exp(
                            err,
                            b"call args\0".as_ptr() as *const ::core::ffi::c_char,
                            api_typename(kObjectTypeArray),
                            api_typename(
                                (*call.items.offset(1 as ::core::ffi::c_int as isize)).type_0,
                            ),
                        );
                        break '_theend;
                    } else {
                        let mut args: Array =
                            (*call.items.offset(1 as ::core::ffi::c_int as isize))
                                .data
                                .array;
                        let mut handler: MsgpackRpcRequestHandler = msgpack_rpc_get_handler_for(
                            name.data,
                            name.size,
                            &raw mut nested_error,
                        );
                        if nested_error.type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break;
                        }
                        let mut result: Object = handler.fn_0.expect("non-null function pointer")(
                            channel_id,
                            args,
                            arena,
                            &raw mut nested_error,
                        );
                        if nested_error.type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break;
                        }
                        let c2rust_fresh0 = results.size;
                        results.size = results.size.wrapping_add(1);
                        *results.items.offset(c2rust_fresh0 as isize) = copy_object(result, arena);
                        if handler.ret_alloc {
                            api_free_object(result);
                        }
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
        let c2rust_fresh1 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: results },
        };
        if nested_error.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            let mut errval: Array = arena_array(arena, 3 as size_t);
            let c2rust_fresh2 = errval.size;
            errval.size = errval.size.wrapping_add(1);
            *errval.items.offset(c2rust_fresh2 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: i as Integer,
                },
            };
            let c2rust_fresh3 = errval.size;
            errval.size = errval.size.wrapping_add(1);
            *errval.items.offset(c2rust_fresh3 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: nested_error.type_0 as Integer,
                },
            };
            let c2rust_fresh4 = errval.size;
            errval.size = errval.size.wrapping_add(1);
            *errval.items.offset(c2rust_fresh4 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: copy_string(cstr_as_string(nested_error.msg), arena),
                },
            };
            let c2rust_fresh5 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh5 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: errval },
            };
        } else {
            let c2rust_fresh6 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh6 as isize) = object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
    }
    api_clear_error(&raw mut nested_error);
    return rv;
}
pub unsafe extern "C" fn nvim_subscribe(mut _channel_id: uint64_t, mut _event: String_0) {}
pub unsafe extern "C" fn nvim_unsubscribe(mut _channel_id: uint64_t, mut _event: String_0) {}
unsafe extern "C" fn write_msg(mut message: String_0, mut to_err: bool, mut writeln: bool) {
    static out_line_buf: GlobalCell<StringBuilder> = GlobalCell::new(StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    });
    static err_line_buf: GlobalCell<StringBuilder> = GlobalCell::new(StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    });
    let mut line_buf: *mut StringBuilder = if to_err as ::core::ffi::c_int != 0 {
        err_line_buf.ptr()
    } else {
        out_line_buf.ptr()
    };
    (*no_wait_return.ptr()) += 1;
    let mut i: uint32_t = 0 as uint32_t;
    while (i as size_t) < message.size {
        if got_int.get() {
            break;
        }
        if (*line_buf).capacity == 0 as size_t {
            (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
            (*line_buf).items = xrealloc(
                (*line_buf).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*line_buf).capacity),
            ) as *mut ::core::ffi::c_char;
        }
        if *message.data.offset(i as isize) as ::core::ffi::c_int == NL {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh7 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh7 as isize) = '\0' as ::core::ffi::c_char;
            if to_err {
                emsg((*line_buf).items);
            } else {
                msg((*line_buf).items, 0 as ::core::ffi::c_int);
            }
            if msg_silent.get() == 0 as ::core::ffi::c_int {
                msg_didout.set(true_0 != 0);
            }
            (*line_buf).size = (*line_buf).size.wrapping_sub((*line_buf).size);
            (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
            (*line_buf).items = xrealloc(
                (*line_buf).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*line_buf).capacity),
            ) as *mut ::core::ffi::c_char;
        } else if *message.data.offset(i as isize) as ::core::ffi::c_int == NUL {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh8 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh8 as isize) = '\n' as ::core::ffi::c_char;
        } else {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh9 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh9 as isize) = *message.data.offset(i as isize);
        }
        i = i.wrapping_add(1);
    }
    if writeln {
        if (*line_buf).capacity == 0 as size_t {
            (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
            (*line_buf).items = xrealloc(
                (*line_buf).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*line_buf).capacity),
            ) as *mut ::core::ffi::c_char;
        }
        if '\n' as ::core::ffi::c_int == NL {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh10 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh10 as isize) = '\0' as ::core::ffi::c_char;
            if to_err {
                emsg((*line_buf).items);
            } else {
                msg((*line_buf).items, 0 as ::core::ffi::c_int);
            }
            if msg_silent.get() == 0 as ::core::ffi::c_int {
                msg_didout.set(true_0 != 0);
            }
            (*line_buf).size = (*line_buf).size.wrapping_sub((*line_buf).size);
            (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
            (*line_buf).items = xrealloc(
                (*line_buf).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*line_buf).capacity),
            ) as *mut ::core::ffi::c_char;
        } else if '\n' as ::core::ffi::c_int == NUL {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh11 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh11 as isize) = '\n' as ::core::ffi::c_char;
        } else {
            if (*line_buf).size == (*line_buf).capacity {
                (*line_buf).capacity = if (*line_buf).capacity != 0 {
                    (*line_buf).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh12 = (*line_buf).size;
            (*line_buf).size = (*line_buf).size.wrapping_add(1);
            *(*line_buf).items.offset(c2rust_fresh12 as isize) = '\n' as ::core::ffi::c_char;
        }
    }
    (*no_wait_return.ptr()) -= 1;
    msg_end();
}
pub unsafe extern "C" fn nvim_out_write(mut str: String_0) {
    write_msg(str, false_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn nvim_err_write(mut str: String_0) {
    write_msg(str, true_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn nvim_err_writeln(mut str: String_0) {
    write_msg(str, true_0 != 0, true_0 != 0);
}
pub unsafe extern "C" fn nvim_notify(
    mut msg_0: String_0,
    mut log_level: Integer,
    mut opts: Dict,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 3] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 3];
    args.capacity = 3 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh13 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh13 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: msg_0 },
    };
    let c2rust_fresh14 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh14 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: log_level },
    };
    let c2rust_fresh15 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh15 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: opts },
    };
    return nlua_exec(
        String_0 {
            data: b"return vim.notify(...)\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetObject,
        arena,
        err,
    );
}
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    conceal_char: 0 as schar_T,
};
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false_0 != 0,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_HL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 8 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
