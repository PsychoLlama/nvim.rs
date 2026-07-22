use crate::src::nvim::change::{
    del_bytes, del_char, get_leader_len, ins_bytes, ins_str, open_line,
};
use crate::src::nvim::charset::{char2cells, getwhitecols_curline, skipwhite};
use crate::src::nvim::cursor::{
    check_cursor, check_cursor_col, coladvance, dec_cursor, gchar_cursor, get_cursor_line_len,
    get_cursor_line_ptr, get_cursor_pos_len, get_cursor_pos_ptr, inc_cursor, pchar_cursor,
};
use crate::src::nvim::drawscreen::redraw_curbuf_later;
use crate::src::nvim::edit::{
    backspace_until_column, beginline, get_nolist_virtcol, insertchar, set_can_cindent,
    undisplay_dollar,
};
use crate::src::nvim::eval::vars::{set_vim_var_char, set_vim_var_nr, set_vim_var_string};
use crate::src::nvim::eval_1::eval_to_number;
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::{
    change_indent, get_expr_indent, get_indent, get_indent_lnum, get_lisp_indent,
    get_number_indent, set_indent,
};
use crate::src::nvim::indent_c::{cindent_on, get_c_indent};
use crate::src::nvim::main::{
    can_si, can_si_back, cmdmod, cmdwin_buf, curbuf, current_sctx, curtab, curwin, did_ai, did_si,
    firstwin, got_int, old_indent, p_paste, p_smd, replace_offset, sandbox, saved_cursor, Insstart,
    State,
};
use crate::src::nvim::mark::mark_col_adjust;
use crate::src::nvim::mbyte::{
    utf_allow_break, utf_allow_break_before, utf_iscomposing_first, utf_ptr2char,
};
use crate::src::nvim::memline::{ml_get, ml_get_len, ml_replace};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::msgmore;
use crate::src::nvim::ops::do_join;
use crate::src::nvim::option::was_set_insecurely;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::strncmp;
use crate::src::nvim::r#move::update_topline;
use crate::src::nvim::search::check_linecomment;
use crate::src::nvim::strings::{vim_strchr, xstrnsave};
use crate::src::nvim::textobject::startPS;
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2,
    ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection, LuaRef, MTKey,
    MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t,
    Map_uint64_t_ptr_t, MarkTree, MotionType, OptIndex, OptInt, ScopeDictDictItem, ScopeType,
    ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal, Timestamp, VarLockStatus, VarType,
    VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit,
    WinStyle, Window, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufstate_T,
    chunksize_T, cmdmod_T, colnr_T, dict_T, dictvar_S, diff_T, diffblock_S, disptick_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, intptr_t, lcs_chars_T, linenr_T,
    list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, oparg_T, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t,
    qf_info_S, qf_info_T, queue, reg_extmatch_T, regmatch_T, regmmatch_T, regprog, regprog_T,
    sattr_T, schar_T, scid_T, sctx_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4,
    syn_time_T, synblock_T, synstate_T, tabpage_S, tabpage_T, taggy_T, terminal, time_t, typval_T,
    typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uint16_t, uint32_t, uint64_t, uint8_t, undo_object, varnumber_T, virt_line, visualinfo_T,
    win_T, window_S, wininfo_S, winopt_T, wline_T, xfmark_T, QUEUE,
};
use crate::src::nvim::ui::ui_cursor_shape;
use crate::src::nvim::undo::{u_save, u_save_cursor};
use crate::src::nvim::window::win_fdccol_count;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_13 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_13 = 3;
pub const BACKWARD: C2Rust_Unnamed_13 = -1;
pub const FORWARD: C2Rust_Unnamed_13 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_13 = 0;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const kBufOptWrapmargin: C2Rust_Unnamed_14 = 91;
pub const kBufOptVartabstop: C2Rust_Unnamed_14 = 90;
pub const kBufOptVarsofttabstop: C2Rust_Unnamed_14 = 89;
pub const kBufOptUndolevels: C2Rust_Unnamed_14 = 88;
pub const kBufOptUndofile: C2Rust_Unnamed_14 = 87;
pub const kBufOptThesaurusfunc: C2Rust_Unnamed_14 = 86;
pub const kBufOptThesaurus: C2Rust_Unnamed_14 = 85;
pub const kBufOptTextwidth: C2Rust_Unnamed_14 = 84;
pub const kBufOptTags: C2Rust_Unnamed_14 = 83;
pub const kBufOptTagfunc: C2Rust_Unnamed_14 = 82;
pub const kBufOptTagcase: C2Rust_Unnamed_14 = 81;
pub const kBufOptTabstop: C2Rust_Unnamed_14 = 80;
pub const kBufOptSyntax: C2Rust_Unnamed_14 = 79;
pub const kBufOptSynmaxcol: C2Rust_Unnamed_14 = 78;
pub const kBufOptSwapfile: C2Rust_Unnamed_14 = 77;
pub const kBufOptSuffixesadd: C2Rust_Unnamed_14 = 76;
pub const kBufOptSpelloptions: C2Rust_Unnamed_14 = 75;
pub const kBufOptSpelllang: C2Rust_Unnamed_14 = 74;
pub const kBufOptSpellfile: C2Rust_Unnamed_14 = 73;
pub const kBufOptSpellcapcheck: C2Rust_Unnamed_14 = 72;
pub const kBufOptSofttabstop: C2Rust_Unnamed_14 = 71;
pub const kBufOptSmartindent: C2Rust_Unnamed_14 = 70;
pub const kBufOptShiftwidth: C2Rust_Unnamed_14 = 69;
pub const kBufOptScrollback: C2Rust_Unnamed_14 = 68;
pub const kBufOptReadonly: C2Rust_Unnamed_14 = 67;
pub const kBufOptQuoteescape: C2Rust_Unnamed_14 = 66;
pub const kBufOptPreserveindent: C2Rust_Unnamed_14 = 65;
pub const kBufOptPath: C2Rust_Unnamed_14 = 64;
pub const kBufOptOmnifunc: C2Rust_Unnamed_14 = 63;
pub const kBufOptNrformats: C2Rust_Unnamed_14 = 62;
pub const kBufOptModified: C2Rust_Unnamed_14 = 61;
pub const kBufOptModifiable: C2Rust_Unnamed_14 = 60;
pub const kBufOptModeline: C2Rust_Unnamed_14 = 59;
pub const kBufOptMatchpairs: C2Rust_Unnamed_14 = 58;
pub const kBufOptMakeprg: C2Rust_Unnamed_14 = 57;
pub const kBufOptMakeencoding: C2Rust_Unnamed_14 = 56;
pub const kBufOptLispwords: C2Rust_Unnamed_14 = 55;
pub const kBufOptLispoptions: C2Rust_Unnamed_14 = 54;
pub const kBufOptLisp: C2Rust_Unnamed_14 = 53;
pub const kBufOptKeywordprg: C2Rust_Unnamed_14 = 52;
pub const kBufOptKeymap: C2Rust_Unnamed_14 = 51;
pub const kBufOptIskeyword: C2Rust_Unnamed_14 = 50;
pub const kBufOptInfercase: C2Rust_Unnamed_14 = 49;
pub const kBufOptIndentkeys: C2Rust_Unnamed_14 = 48;
pub const kBufOptIndentexpr: C2Rust_Unnamed_14 = 47;
pub const kBufOptIncludeexpr: C2Rust_Unnamed_14 = 46;
pub const kBufOptInclude: C2Rust_Unnamed_14 = 45;
pub const kBufOptImsearch: C2Rust_Unnamed_14 = 44;
pub const kBufOptIminsert: C2Rust_Unnamed_14 = 43;
pub const kBufOptGrepprg: C2Rust_Unnamed_14 = 42;
pub const kBufOptGrepformat: C2Rust_Unnamed_14 = 41;
pub const kBufOptFsync: C2Rust_Unnamed_14 = 40;
pub const kBufOptFormatprg: C2Rust_Unnamed_14 = 39;
pub const kBufOptFormatoptions: C2Rust_Unnamed_14 = 38;
pub const kBufOptFormatlistpat: C2Rust_Unnamed_14 = 37;
pub const kBufOptFormatexpr: C2Rust_Unnamed_14 = 36;
pub const kBufOptFixendofline: C2Rust_Unnamed_14 = 35;
pub const kBufOptFindfunc: C2Rust_Unnamed_14 = 34;
pub const kBufOptFiletype: C2Rust_Unnamed_14 = 33;
pub const kBufOptFileformat: C2Rust_Unnamed_14 = 32;
pub const kBufOptFileencoding: C2Rust_Unnamed_14 = 31;
pub const kBufOptExpandtab: C2Rust_Unnamed_14 = 30;
pub const kBufOptErrorformat: C2Rust_Unnamed_14 = 29;
pub const kBufOptEqualprg: C2Rust_Unnamed_14 = 28;
pub const kBufOptEndofline: C2Rust_Unnamed_14 = 27;
pub const kBufOptEndoffile: C2Rust_Unnamed_14 = 26;
pub const kBufOptDiffanchors: C2Rust_Unnamed_14 = 25;
pub const kBufOptDictionary: C2Rust_Unnamed_14 = 24;
pub const kBufOptDefine: C2Rust_Unnamed_14 = 23;
pub const kBufOptCopyindent: C2Rust_Unnamed_14 = 22;
pub const kBufOptCompleteslash: C2Rust_Unnamed_14 = 21;
pub const kBufOptCompleteopt: C2Rust_Unnamed_14 = 20;
pub const kBufOptCompletefunc: C2Rust_Unnamed_14 = 19;
pub const kBufOptComplete: C2Rust_Unnamed_14 = 18;
pub const kBufOptCommentstring: C2Rust_Unnamed_14 = 17;
pub const kBufOptComments: C2Rust_Unnamed_14 = 16;
pub const kBufOptCinwords: C2Rust_Unnamed_14 = 15;
pub const kBufOptCinscopedecls: C2Rust_Unnamed_14 = 14;
pub const kBufOptCinoptions: C2Rust_Unnamed_14 = 13;
pub const kBufOptCinkeys: C2Rust_Unnamed_14 = 12;
pub const kBufOptCindent: C2Rust_Unnamed_14 = 11;
pub const kBufOptChannel: C2Rust_Unnamed_14 = 10;
pub const kBufOptBusy: C2Rust_Unnamed_14 = 9;
pub const kBufOptBuftype: C2Rust_Unnamed_14 = 8;
pub const kBufOptBuflisted: C2Rust_Unnamed_14 = 7;
pub const kBufOptBufhidden: C2Rust_Unnamed_14 = 6;
pub const kBufOptBomb: C2Rust_Unnamed_14 = 5;
pub const kBufOptBinary: C2Rust_Unnamed_14 = 4;
pub const kBufOptBackupcopy: C2Rust_Unnamed_14 = 3;
pub const kBufOptAutoread: C2Rust_Unnamed_14 = 2;
pub const kBufOptAutoindent: C2Rust_Unnamed_14 = 1;
pub const kBufOptAutocomplete: C2Rust_Unnamed_14 = 0;
pub const kBufOptInvalid: C2Rust_Unnamed_14 = -1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const OPENLINE_FORCE_INDENT: C2Rust_Unnamed_15 = 64;
pub const OPENLINE_FORMAT: C2Rust_Unnamed_15 = 32;
pub const OPENLINE_COM_LIST: C2Rust_Unnamed_15 = 16;
pub const OPENLINE_MARKFIX: C2Rust_Unnamed_15 = 8;
pub const OPENLINE_KEEPTRAIL: C2Rust_Unnamed_15 = 4;
pub const OPENLINE_DO_COM: C2Rust_Unnamed_15 = 2;
pub const OPENLINE_DELSPACES: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_16 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_16 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_16 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_16 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_16 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_16 = 20;
pub const UPD_VALID: C2Rust_Unnamed_16 = 10;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_17 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_17 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_17 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_17 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_17 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_17 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_17 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_17 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_17 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_17 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_17 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_17 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_17 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const INDENT_DEC: C2Rust_Unnamed_18 = 3;
pub const INDENT_INC: C2Rust_Unnamed_18 = 2;
pub const INDENT_SET: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_19 = 4;
pub const BL_SOL: C2Rust_Unnamed_19 = 2;
pub const BL_WHITE: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const INSCHAR_COM_LIST: C2Rust_Unnamed_20 = 16;
pub const INSCHAR_NO_FEX: C2Rust_Unnamed_20 = 8;
pub const INSCHAR_CTRLV: C2Rust_Unnamed_20 = 4;
pub const INSCHAR_DO_COM: C2Rust_Unnamed_20 = 2;
pub const INSCHAR_FORMAT: C2Rust_Unnamed_20 = 1;
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
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const MODE_SHOWMATCH: C2Rust_Unnamed_21 = 24592;
pub const MODE_EXTERNCMD: C2Rust_Unnamed_21 = 20480;
pub const MODE_SETWSIZE: C2Rust_Unnamed_21 = 16384;
pub const MODE_ASKMORE: C2Rust_Unnamed_21 = 12288;
pub const MODE_HITRETURN: C2Rust_Unnamed_21 = 8193;
pub const MODE_NORMAL_BUSY: C2Rust_Unnamed_21 = 4097;
pub const MODE_LREPLACE: C2Rust_Unnamed_21 = 288;
pub const MODE_VREPLACE: C2Rust_Unnamed_21 = 784;
pub const VREPLACE_FLAG: C2Rust_Unnamed_21 = 512;
pub const MODE_REPLACE: C2Rust_Unnamed_21 = 272;
pub const REPLACE_FLAG: C2Rust_Unnamed_21 = 256;
pub const MAP_ALL_MODES: C2Rust_Unnamed_21 = 255;
pub const MODE_TERMINAL: C2Rust_Unnamed_21 = 128;
pub const MODE_SELECT: C2Rust_Unnamed_21 = 64;
pub const MODE_LANGMAP: C2Rust_Unnamed_21 = 32;
pub const MODE_INSERT: C2Rust_Unnamed_21 = 16;
pub const MODE_CMDLINE: C2Rust_Unnamed_21 = 8;
pub const MODE_OP_PENDING: C2Rust_Unnamed_21 = 4;
pub const MODE_VISUAL: C2Rust_Unnamed_21 = 2;
pub const MODE_NORMAL: C2Rust_Unnamed_21 = 1;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const SIN_NOMARK: C2Rust_Unnamed_22 = 8;
pub const SIN_UNDO: C2Rust_Unnamed_22 = 4;
pub const SIN_INSERT: C2Rust_Unnamed_22 = 2;
pub const SIN_CHANGED: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_23 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_23 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_23 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_23 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_23 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_23 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_23 = 2;
pub const OPT_GLOBAL: C2Rust_Unnamed_23 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FO_WRAP: ::core::ffi::c_int = 't' as ::core::ffi::c_int;
pub const FO_WRAP_COMS: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
pub const FO_Q_COMS: ::core::ffi::c_int = 'q' as ::core::ffi::c_int;
pub const FO_Q_NUMBER: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const FO_Q_SECOND: ::core::ffi::c_int = '2' as ::core::ffi::c_int;
pub const FO_INS_VI: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const FO_INS_BLANK: ::core::ffi::c_int = 'b' as ::core::ffi::c_int;
pub const FO_MBYTE_BREAK: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const FO_ONE_LETTER: ::core::ffi::c_int = '1' as ::core::ffi::c_int;
pub const FO_WHITE_PAR: ::core::ffi::c_int = 'w' as ::core::ffi::c_int;
pub const FO_AUTO: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const FO_RIGOROUS_TW: ::core::ffi::c_int = ']' as ::core::ffi::c_int;
pub const FO_PERIOD_ABBR: ::core::ffi::c_int = 'p' as ::core::ffi::c_int;
pub const COM_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const COM_FIRST: ::core::ffi::c_int = 'f' as ::core::ffi::c_int;
static did_add_space: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn has_format_option(mut x: ::core::ffi::c_int) -> bool {
    if p_paste.get() != 0 {
        return false_0 != 0;
    }
    return !vim_strchr((*curbuf.get()).b_p_fo, x).is_null();
}
pub unsafe extern "C" fn internal_format(
    mut textwidth: ::core::ffi::c_int,
    mut second_indent: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut format_only: bool,
    mut c: ::core::ffi::c_int,
) {
    let mut cc: ::core::ffi::c_int = 0;
    let mut save_char: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    let mut haveto_redraw: bool = false_0 != 0;
    let fo_ins_blank: bool = has_format_option(FO_INS_BLANK);
    let fo_multibyte: bool = has_format_option(FO_MBYTE_BREAK);
    let fo_rigor_tw: bool = has_format_option(FO_RIGOROUS_TW);
    let fo_white_par: bool = has_format_option(FO_WHITE_PAR);
    let mut first_line: bool = true_0 != 0;
    let mut leader_len: colnr_T = 0;
    let mut no_leader: bool = false_0 != 0;
    let mut do_comments: bool = flags & INSCHAR_DO_COM as ::core::ffi::c_int != 0;
    let mut has_lbr: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_lbr;
    (*curwin.get()).w_onebuf_opt.wo_lbr = false_0;
    if (*curbuf.get()).b_p_ai == 0 && State.get() & VREPLACE_FLAG as ::core::ffi::c_int == 0 {
        cc = gchar_cursor();
        if ascii_iswhite(cc) {
            save_char = cc as ::core::ffi::c_char;
            pchar_cursor('x' as ::core::ffi::c_char);
        }
    }
    while !got_int.get() {
        let mut startcol: ::core::ffi::c_int = 0;
        let mut wantcol: ::core::ffi::c_int = 0;
        let mut foundcol: ::core::ffi::c_int = 0;
        let mut end_foundcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut orig_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut saved_text: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut col: colnr_T = 0;
        let mut did_do_comment: bool = false_0 != 0;
        let mut virtcol: colnr_T =
            get_nolist_virtcol() + char2cells(if c != NUL { c } else { gchar_cursor() });
        if virtcol <= textwidth {
            break;
        }
        if no_leader {
            do_comments = false_0 != 0;
        } else if flags & INSCHAR_FORMAT as ::core::ffi::c_int == 0
            && has_format_option(FO_WRAP_COMS) as ::core::ffi::c_int != 0
        {
            do_comments = true_0 != 0;
        }
        if do_comments {
            let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            leader_len = get_leader_len(
                line,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                false_0 != 0,
                true_0 != 0,
            ) as colnr_T;
            if leader_len == 0 as ::core::ffi::c_int && (*curbuf.get()).b_p_cin != 0 {
                let mut comment_start: ::core::ffi::c_int = check_linecomment(line);
                if comment_start != MAXCOL as ::core::ffi::c_int {
                    leader_len = get_leader_len(
                        line.offset(comment_start as isize),
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        false_0 != 0,
                        true_0 != 0,
                    ) as colnr_T;
                    if leader_len != 0 as ::core::ffi::c_int {
                        leader_len += comment_start;
                    }
                }
            }
        } else {
            leader_len = 0 as ::core::ffi::c_int as colnr_T;
        }
        if leader_len == 0 as ::core::ffi::c_int {
            no_leader = true_0 != 0;
        }
        if flags & INSCHAR_FORMAT as ::core::ffi::c_int == 0
            && leader_len == 0 as ::core::ffi::c_int
            && !has_format_option(FO_WRAP)
        {
            break;
        }
        startcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        if startcol == 0 as ::core::ffi::c_int {
            break;
        }
        coladvance(curwin.get(), textwidth);
        wantcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        (*curwin.get()).w_cursor.col = startcol as colnr_T;
        foundcol = 0 as ::core::ffi::c_int;
        let mut skip_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !fo_ins_blank && !has_format_option(FO_INS_VI)
            || flags & INSCHAR_FORMAT as ::core::ffi::c_int != 0
            || (*curwin.get()).w_cursor.lnum != (*Insstart.ptr()).lnum
            || (*curwin.get()).w_cursor.col >= (*Insstart.ptr()).col
        {
            if (*curwin.get()).w_cursor.col == startcol && c != NUL {
                cc = c;
            } else {
                cc = gchar_cursor();
            }
            if ascii_iswhite(cc) as ::core::ffi::c_int != 0
                && !utf_iscomposing_first(utf_ptr2char(
                    get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                ))
            {
                let mut end_col: colnr_T = (*curwin.get()).w_cursor.col;
                let mut wcc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
                    && (ascii_iswhite(cc) as ::core::ffi::c_int != 0
                        && !utf_iscomposing_first(utf_ptr2char(
                            get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                        )))
                {
                    dec_cursor();
                    cc = gchar_cursor();
                    if wcc < 2 as ::core::ffi::c_int {
                        wcc += 1;
                    }
                }
                if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                    && (ascii_iswhite(cc) as ::core::ffi::c_int != 0
                        && !utf_iscomposing_first(utf_ptr2char(
                            get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                        )))
                {
                    break;
                } else {
                    if has_format_option(FO_PERIOD_ABBR) as ::core::ffi::c_int != 0
                        && cc == '.' as ::core::ffi::c_int
                        && wcc < 2 as ::core::ffi::c_int
                    {
                        continue;
                    }
                    if (*curwin.get()).w_cursor.col < leader_len {
                        break;
                    }
                    if has_format_option(FO_ONE_LETTER) {
                        if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
                            break;
                        }
                        if (*curwin.get()).w_cursor.col <= leader_len {
                            break;
                        }
                        col = (*curwin.get()).w_cursor.col;
                        dec_cursor();
                        cc = gchar_cursor();
                        if ascii_iswhite(cc) as ::core::ffi::c_int != 0
                            && !utf_iscomposing_first(utf_ptr2char(
                                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                            ))
                        {
                            continue;
                        } else {
                            (*curwin.get()).w_cursor.col = col;
                        }
                    }
                    inc_cursor();
                    end_foundcol = end_col as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                    foundcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                    if (*curwin.get()).w_cursor.col <= wantcol {
                        break;
                    }
                }
            } else if (cc >= 0x100 as ::core::ffi::c_int || !utf_allow_break_before(cc))
                && fo_multibyte as ::core::ffi::c_int != 0
            {
                let mut ncc: ::core::ffi::c_int = 0;
                let mut allow_break: bool = false;
                if (*curwin.get()).w_cursor.col != startcol {
                    if (*curwin.get()).w_cursor.col < leader_len {
                        break;
                    }
                    col = (*curwin.get()).w_cursor.col;
                    inc_cursor();
                    ncc = gchar_cursor();
                    allow_break = utf_allow_break(cc, ncc);
                    if (*curwin.get()).w_cursor.col != skip_pos
                        && allow_break as ::core::ffi::c_int != 0
                    {
                        foundcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                        end_foundcol = foundcol;
                        if (*curwin.get()).w_cursor.col <= wantcol {
                            break;
                        }
                    }
                    (*curwin.get()).w_cursor.col = col;
                }
                if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
                    break;
                }
                ncc = cc;
                col = (*curwin.get()).w_cursor.col;
                dec_cursor();
                cc = gchar_cursor();
                if ascii_iswhite(cc) as ::core::ffi::c_int != 0
                    && !utf_iscomposing_first(utf_ptr2char(
                        get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                    ))
                {
                    continue;
                } else {
                    if (*curwin.get()).w_cursor.col < leader_len {
                        break;
                    }
                    (*curwin.get()).w_cursor.col = col;
                    skip_pos = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                    allow_break = utf_allow_break(cc, ncc);
                    if allow_break {
                        foundcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                        end_foundcol = foundcol;
                    }
                    if (*curwin.get()).w_cursor.col <= wantcol {
                        let ncc_allow_break: bool = utf_allow_break_before(ncc);
                        if allow_break {
                            break;
                        }
                        if !ncc_allow_break && !fo_rigor_tw {
                            if (*curwin.get()).w_cursor.col == startcol {
                                foundcol = 0 as ::core::ffi::c_int;
                                end_foundcol = foundcol;
                                break;
                            } else {
                                col = (*curwin.get()).w_cursor.col;
                                inc_cursor();
                                cc = ncc;
                                ncc = gchar_cursor();
                                ncc = if ncc != NUL { ncc } else { c };
                                allow_break = utf_allow_break(cc, ncc);
                                if allow_break {
                                    foundcol = if ncc == NUL {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        (*curwin.get()).w_cursor.col as ::core::ffi::c_int
                                    };
                                    end_foundcol = foundcol;
                                    break;
                                } else {
                                    (*curwin.get()).w_cursor.col = col;
                                }
                            }
                        }
                    }
                }
            }
            if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
                break;
            }
            dec_cursor();
        }
        if foundcol == 0 as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.col = startcol as colnr_T;
            break;
        } else {
            undisplay_dollar();
            if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                orig_col = startcol;
            } else {
                replace_offset.set(startcol - end_foundcol);
            }
            (*curwin.get()).w_cursor.col = foundcol as colnr_T;
            loop {
                cc = gchar_cursor();
                if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
                    && !utf_iscomposing_first(utf_ptr2char(
                        get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                    ))
                    && (!fo_white_par || (*curwin.get()).w_cursor.col < startcol))
                {
                    break;
                }
                inc_cursor();
            }
            startcol -= (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
            startcol = if startcol > 0 as ::core::ffi::c_int {
                startcol
            } else {
                0 as ::core::ffi::c_int
            };
            if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                saved_text = xstrnsave(get_cursor_pos_ptr(), get_cursor_pos_len() as size_t);
                (*curwin.get()).w_cursor.col = orig_col as colnr_T;
                *saved_text.offset(startcol as isize) = NUL as ::core::ffi::c_char;
                if !fo_white_par {
                    backspace_until_column(foundcol);
                }
            } else if !fo_white_par {
                (*curwin.get()).w_cursor.col = foundcol as colnr_T;
            }
            open_line(
                FORWARD as ::core::ffi::c_int,
                OPENLINE_DELSPACES as ::core::ffi::c_int
                    + OPENLINE_MARKFIX as ::core::ffi::c_int
                    + (if fo_white_par as ::core::ffi::c_int != 0 {
                        OPENLINE_KEEPTRAIL as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
                    + (if do_comments as ::core::ffi::c_int != 0 {
                        OPENLINE_DO_COM as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
                    + OPENLINE_FORMAT as ::core::ffi::c_int
                    + (if flags & INSCHAR_COM_LIST as ::core::ffi::c_int != 0 {
                        OPENLINE_COM_LIST as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
                if flags & INSCHAR_COM_LIST as ::core::ffi::c_int != 0 {
                    second_indent
                } else {
                    old_indent.get()
                },
                &raw mut did_do_comment,
            );
            if flags & INSCHAR_COM_LIST as ::core::ffi::c_int == 0 {
                old_indent.set(0 as ::core::ffi::c_int);
            }
            if did_do_comment {
                no_leader = false_0 != 0;
            }
            replace_offset.set(0 as ::core::ffi::c_int);
            if first_line {
                if flags & INSCHAR_COM_LIST as ::core::ffi::c_int == 0 {
                    if second_indent < 0 as ::core::ffi::c_int
                        && has_format_option(FO_Q_NUMBER) as ::core::ffi::c_int != 0
                    {
                        second_indent =
                            get_number_indent((*curwin.get()).w_cursor.lnum - 1 as linenr_T);
                    }
                    if second_indent >= 0 as ::core::ffi::c_int {
                        if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                            change_indent(
                                INDENT_SET as ::core::ffi::c_int,
                                second_indent,
                                false_0,
                                true_0 != 0,
                            );
                        } else if leader_len > 0 as ::core::ffi::c_int
                            && second_indent as colnr_T - leader_len > 0 as ::core::ffi::c_int
                        {
                            let mut padding: ::core::ffi::c_int =
                                second_indent - leader_len as ::core::ffi::c_int;
                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i < padding {
                                ins_str(
                                    b" \0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                        .wrapping_sub(1 as size_t),
                                );
                                i += 1;
                            }
                        } else {
                            set_indent(second_indent, SIN_CHANGED as ::core::ffi::c_int);
                        }
                    }
                }
                first_line = false_0 != 0;
            }
            if State.get() & VREPLACE_FLAG as ::core::ffi::c_int != 0 {
                ins_bytes(saved_text);
                xfree(saved_text as *mut ::core::ffi::c_void);
            } else {
                (*curwin.get()).w_cursor.col += startcol;
                let mut len: colnr_T = get_cursor_line_len();
                (*curwin.get()).w_cursor.col = if (*curwin.get()).w_cursor.col < len {
                    (*curwin.get()).w_cursor.col
                } else {
                    len
                };
            }
            haveto_redraw = true_0 != 0;
            set_can_cindent(true_0 != 0);
            did_ai.set(false_0 != 0);
            did_si.set(false_0 != 0);
            can_si.set(false_0 != 0);
            can_si_back.set(false_0 != 0);
            line_breakcheck();
        }
    }
    if save_char as ::core::ffi::c_int != NUL {
        pchar_cursor(save_char);
    }
    (*curwin.get()).w_onebuf_opt.wo_lbr = has_lbr;
    if !format_only && haveto_redraw as ::core::ffi::c_int != 0 {
        update_topline(curwin.get());
        redraw_curbuf_later(UPD_VALID as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn fmt_check_par(
    mut lnum: linenr_T,
    mut leader_len: *mut ::core::ffi::c_int,
    mut leader_flags: *mut *mut ::core::ffi::c_char,
    mut do_comments: bool,
) -> ::core::ffi::c_int {
    let mut flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptr: *mut ::core::ffi::c_char = ml_get(lnum);
    if do_comments {
        *leader_len = get_leader_len(ptr, leader_flags, false_0 != 0, true_0 != 0);
    } else {
        *leader_len = 0 as ::core::ffi::c_int;
    }
    if *leader_len > 0 as ::core::ffi::c_int {
        flags = *leader_flags;
        while *flags as ::core::ffi::c_int != 0
            && *flags as ::core::ffi::c_int != ':' as ::core::ffi::c_int
            && *flags as ::core::ffi::c_int != COM_END
        {
            flags = flags.offset(1);
        }
    }
    return (*skipwhite(ptr.offset(*leader_len as isize)) as ::core::ffi::c_int == NUL
        || *leader_len > 0 as ::core::ffi::c_int && *flags as ::core::ffi::c_int == COM_END
        || startPS(lnum, NUL, false_0 != 0) as ::core::ffi::c_int != 0)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn ends_in_white(mut lnum: linenr_T) -> bool {
    let mut s: *mut ::core::ffi::c_char = ml_get(lnum);
    if *s as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    let mut l: colnr_T = ml_get_len(lnum) - 1 as colnr_T;
    return ascii_iswhite(*s.offset(l as isize) as uint8_t as ::core::ffi::c_int);
}
unsafe extern "C" fn same_leader(
    mut lnum: linenr_T,
    mut leader1_len: ::core::ffi::c_int,
    mut leader1_flags: *mut ::core::ffi::c_char,
    mut leader2_len: ::core::ffi::c_int,
    mut leader2_flags: *mut ::core::ffi::c_char,
) -> bool {
    let mut idx1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut idx2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if leader1_len == 0 as ::core::ffi::c_int {
        return leader2_len == 0 as ::core::ffi::c_int;
    }
    if !leader1_flags.is_null() {
        let mut p: *mut ::core::ffi::c_char = leader1_flags;
        while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
        {
            if *p as ::core::ffi::c_int == COM_FIRST {
                return leader2_len == 0 as ::core::ffi::c_int;
            }
            if *p as ::core::ffi::c_int == COM_END {
                return false_0 != 0;
            }
            if *p as ::core::ffi::c_int == COM_START {
                let mut line_len: ::core::ffi::c_int = ml_get_len(lnum);
                if line_len <= leader1_len {
                    return false_0 != 0;
                }
                if leader2_flags.is_null() || leader2_len == 0 as ::core::ffi::c_int {
                    return false_0 != 0;
                }
                p = leader2_flags;
                while *p as ::core::ffi::c_int != 0
                    && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                {
                    if *p as ::core::ffi::c_int == COM_MIDDLE {
                        return true_0 != 0;
                    }
                    p = p.offset(1);
                }
                return false_0 != 0;
            }
            p = p.offset(1);
        }
    }
    let mut line1: *mut ::core::ffi::c_char = xstrnsave(ml_get(lnum), ml_get_len(lnum) as size_t);
    idx1 = 0 as ::core::ffi::c_int;
    while ascii_iswhite(*line1.offset(idx1 as isize) as ::core::ffi::c_int) {
        idx1 += 1;
    }
    let mut line2: *mut ::core::ffi::c_char = ml_get(lnum + 1 as linenr_T);
    idx2 = 0 as ::core::ffi::c_int;
    while idx2 < leader2_len {
        if !ascii_iswhite(*line2.offset(idx2 as isize) as ::core::ffi::c_int) {
            let c2rust_fresh0 = idx1;
            idx1 = idx1 + 1;
            if *line1.offset(c2rust_fresh0 as isize) as ::core::ffi::c_int
                != *line2.offset(idx2 as isize) as ::core::ffi::c_int
            {
                break;
            }
        } else {
            while ascii_iswhite(*line1.offset(idx1 as isize) as ::core::ffi::c_int) {
                idx1 += 1;
            }
        }
        idx2 += 1;
    }
    xfree(line1 as *mut ::core::ffi::c_void);
    return idx2 == leader2_len && idx1 == leader1_len;
}
unsafe extern "C" fn paragraph_start(mut lnum: linenr_T) -> bool {
    let mut leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut leader_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next_leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_leader_flags: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lnum <= 1 as linenr_T {
        return true_0 != 0;
    }
    let mut p: *mut ::core::ffi::c_char = ml_get(lnum - 1 as linenr_T);
    if *p as ::core::ffi::c_int == NUL {
        return true_0 != 0;
    }
    let do_comments: bool = has_format_option(FO_Q_COMS);
    if fmt_check_par(
        lnum - 1 as linenr_T,
        &raw mut leader_len,
        &raw mut leader_flags,
        do_comments,
    ) != 0
    {
        return true_0 != 0;
    }
    if fmt_check_par(
        lnum,
        &raw mut next_leader_len,
        &raw mut next_leader_flags,
        do_comments,
    ) != 0
    {
        return true_0 != 0;
    }
    if has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0
        && !ends_in_white(lnum - 1 as linenr_T)
    {
        return true_0 != 0;
    }
    if has_format_option(FO_Q_NUMBER) as ::core::ffi::c_int != 0
        && get_number_indent(lnum) > 0 as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    if !same_leader(
        lnum - 1 as linenr_T,
        leader_len,
        leader_flags,
        next_leader_len,
        next_leader_flags,
    ) {
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn auto_format(mut trailblank: bool, mut prev_line: bool) {
    if !has_format_option(FO_AUTO) {
        return;
    }
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut old: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    check_auto_format(false_0 != 0);
    let mut wasatend: bool = pos.col == get_cursor_line_len();
    if *old as ::core::ffi::c_int != NUL && !trailblank && wasatend as ::core::ffi::c_int != 0 {
        dec_cursor();
        let mut cc: ::core::ffi::c_int = gchar_cursor();
        if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            )))
            && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
            && has_format_option(FO_ONE_LETTER) as ::core::ffi::c_int != 0
        {
            dec_cursor();
        }
        cc = gchar_cursor();
        if ascii_iswhite(cc) as ::core::ffi::c_int != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            ))
        {
            (*curwin.get()).w_cursor = pos;
            return;
        }
        (*curwin.get()).w_cursor = pos;
    }
    if *old as ::core::ffi::c_int != NUL
        && !trailblank
        && !wasatend
        && pos.col > 0 as ::core::ffi::c_int
        && State.get() & MODE_INSERT as ::core::ffi::c_int != 0
    {
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        if ascii_iswhite(
            *line.offset((pos.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            ))
        {
            (*curwin.get()).w_cursor = pos;
            return;
        }
    }
    if has_format_option(FO_WRAP_COMS) as ::core::ffi::c_int != 0
        && !has_format_option(FO_WRAP)
        && get_leader_len(
            old,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            false_0 != 0,
            true_0 != 0,
        ) == 0 as ::core::ffi::c_int
    {
        return;
    }
    if prev_line as ::core::ffi::c_int != 0 && !paragraph_start((*curwin.get()).w_cursor.lnum) {
        (*curwin.get()).w_cursor.lnum -= 1;
        if u_save_cursor() == FAIL {
            return;
        }
    }
    saved_cursor.set(pos);
    format_lines(-1 as linenr_T, false_0 != 0);
    (*curwin.get()).w_cursor = saved_cursor.get();
    (*saved_cursor.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
    if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
    } else {
        check_cursor_col(curwin.get());
    }
    if !wasatend && has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0 {
        let mut linep: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        let mut len: colnr_T = get_cursor_line_len();
        if (*curwin.get()).w_cursor.col == len {
            let mut plinep: *mut ::core::ffi::c_char =
                xstrnsave(linep, (len as size_t).wrapping_add(2 as size_t));
            *plinep.offset(len as isize) = ' ' as ::core::ffi::c_char;
            *plinep.offset((len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                NUL as ::core::ffi::c_char;
            ml_replace((*curwin.get()).w_cursor.lnum, plinep, false_0 != 0);
            did_add_space.set(true_0 != 0);
        } else {
            check_auto_format(false_0 != 0);
        }
    }
    check_cursor(curwin.get());
}
pub unsafe extern "C" fn check_auto_format(mut end_insert: bool) {
    if !did_add_space.get() {
        return;
    }
    let mut cc: ::core::ffi::c_int = gchar_cursor();
    if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
        && !utf_iscomposing_first(utf_ptr2char(
            get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
        )))
    {
        did_add_space.set(false_0 != 0);
    } else {
        let mut c: ::core::ffi::c_int = ' ' as ::core::ffi::c_int;
        if !end_insert {
            inc_cursor();
            c = gchar_cursor();
            dec_cursor();
        }
        if c != NUL {
            del_char(false_0 != 0);
            did_add_space.set(false_0 != 0);
        }
    };
}
pub unsafe extern "C" fn comp_textwidth(mut ff: bool) -> ::core::ffi::c_int {
    let mut textwidth: ::core::ffi::c_int = (*curbuf.get()).b_p_tw as ::core::ffi::c_int;
    if textwidth == 0 as ::core::ffi::c_int && (*curbuf.get()).b_p_wm != 0 {
        textwidth = (*curwin.get()).w_view_width - (*curbuf.get()).b_p_wm as ::core::ffi::c_int;
        if curbuf.get() == cmdwin_buf.get() {
            textwidth -= 1 as ::core::ffi::c_int;
        }
        textwidth -= win_fdccol_count(curwin.get());
        textwidth -= (*curwin.get()).w_scwidth;
        if (*curwin.get()).w_onebuf_opt.wo_nu != 0 || (*curwin.get()).w_onebuf_opt.wo_rnu != 0 {
            textwidth -= 8 as ::core::ffi::c_int;
        }
    }
    textwidth = if textwidth > 0 as ::core::ffi::c_int {
        textwidth
    } else {
        0 as ::core::ffi::c_int
    };
    if ff as ::core::ffi::c_int != 0 && textwidth == 0 as ::core::ffi::c_int {
        textwidth = if ((*curwin.get()).w_view_width - 1 as ::core::ffi::c_int)
            < 79 as ::core::ffi::c_int
        {
            (*curwin.get()).w_view_width - 1 as ::core::ffi::c_int
        } else {
            79 as ::core::ffi::c_int
        };
    }
    return textwidth;
}
pub unsafe extern "C" fn op_format(mut oap: *mut oparg_T, mut keep_cursor: bool) {
    let mut old_line_count: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
    (*curwin.get()).w_cursor = (*oap).cursor_start;
    if u_save(
        (*oap).start.lnum - 1 as linenr_T,
        (*oap).end.lnum + 1 as linenr_T,
    ) == FAIL
    {
        return;
    }
    (*curwin.get()).w_cursor = (*oap).start;
    if (*oap).is_VIsual {
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_start = (*oap).start;
    }
    if keep_cursor {
        saved_cursor.set((*oap).cursor_start);
    }
    format_lines((*oap).line_count, keep_cursor);
    if (*oap).end_adjusted as ::core::ffi::c_int != 0
        && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
    {
        (*curwin.get()).w_cursor.lnum += 1;
    }
    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    old_line_count = (*curbuf.get()).b_ml.ml_line_count - old_line_count;
    msgmore(old_line_count as ::core::ffi::c_int);
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
    }
    if keep_cursor {
        (*curwin.get()).w_cursor = saved_cursor.get();
        (*saved_cursor.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
        check_cursor(curwin.get());
    }
    if (*oap).is_VIsual {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_old_cursor_lnum != 0 as linenr_T {
                if (*wp).w_old_cursor_lnum > (*wp).w_old_visual_lnum {
                    (*wp).w_old_cursor_lnum += old_line_count;
                } else {
                    (*wp).w_old_visual_lnum += old_line_count;
                }
            }
            wp = (*wp).w_next;
        }
    }
}
pub unsafe extern "C" fn op_formatexpr(mut oap: *mut oparg_T) {
    if (*oap).is_VIsual {
        redraw_curbuf_later(UPD_INVERTED as ::core::ffi::c_int);
    }
    if fex_format(
        (*oap).start.lnum,
        (*oap).line_count as ::core::ffi::c_long,
        NUL,
    ) != 0 as ::core::ffi::c_int
    {
        op_format(oap, false_0 != 0);
    }
}
pub unsafe extern "C" fn fex_format(
    mut lnum: linenr_T,
    mut count: ::core::ffi::c_long,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut use_sandbox: bool = was_set_insecurely(
        curwin.get(),
        kOptFormatexpr,
        OPT_LOCAL as ::core::ffi::c_int,
    ) != 0;
    let save_sctx: sctx_T = current_sctx.get();
    set_vim_var_nr(VV_LNUM, lnum as varnumber_T);
    set_vim_var_nr(VV_COUNT, count as varnumber_T);
    set_vim_var_char(c);
    let mut fex: *mut ::core::ffi::c_char = xstrdup((*curbuf.get()).b_p_fex);
    current_sctx
        .set((*curbuf.get()).b_p_script_ctx[kBufOptFormatexpr as ::core::ffi::c_int as usize]);
    if use_sandbox {
        (*sandbox.ptr()) += 1;
    }
    let mut r: ::core::ffi::c_int = eval_to_number(fex, true_0 != 0) as ::core::ffi::c_int;
    if use_sandbox {
        (*sandbox.ptr()) -= 1;
    }
    set_vim_var_string(
        VV_CHAR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    xfree(fex as *mut ::core::ffi::c_void);
    current_sctx.set(save_sctx);
    return r;
}
pub unsafe extern "C" fn format_lines(mut line_count: linenr_T, mut avoid_fex: bool) {
    let mut is_not_par: bool = false;
    let mut next_is_not_par: bool = false;
    let mut is_end_par: bool = false;
    let mut prev_is_end_par: bool = false_0 != 0;
    let mut next_is_start_par: bool = false_0 != 0;
    let mut leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_leader_len: ::core::ffi::c_int = 0;
    let mut leader_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next_leader_flags: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut advance: bool = true_0 != 0;
    let mut second_indent: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut first_par_line: bool = true_0 != 0;
    let mut smd_save: ::core::ffi::c_int = 0;
    let mut count: ::core::ffi::c_long = 0;
    let mut need_set_indent: bool = true_0 != 0;
    let mut first_line: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut force_format: bool = false_0 != 0;
    let old_State: ::core::ffi::c_int = State.get();
    let max_len: ::core::ffi::c_int = comp_textwidth(true_0 != 0) * 3 as ::core::ffi::c_int;
    let do_comments: bool = has_format_option(FO_Q_COMS);
    let mut do_comments_list: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let do_second_indent: bool = has_format_option(FO_Q_SECOND);
    let do_number_indent: bool = has_format_option(FO_Q_NUMBER);
    let do_trail_white: bool = has_format_option(FO_WHITE_PAR);
    if (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
        is_not_par = fmt_check_par(
            (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
            &raw mut leader_len,
            &raw mut leader_flags,
            do_comments,
        ) != 0;
    } else {
        is_not_par = true_0 != 0;
    }
    next_is_not_par = fmt_check_par(
        (*curwin.get()).w_cursor.lnum,
        &raw mut next_leader_len,
        &raw mut next_leader_flags,
        do_comments,
    ) != 0;
    is_end_par =
        is_not_par as ::core::ffi::c_int != 0 || next_is_not_par as ::core::ffi::c_int != 0;
    if !is_end_par && do_trail_white as ::core::ffi::c_int != 0 {
        is_end_par = !ends_in_white((*curwin.get()).w_cursor.lnum - 1 as linenr_T);
    }
    (*curwin.get()).w_cursor.lnum -= 1;
    count = line_count as ::core::ffi::c_long;
    while count != 0 as ::core::ffi::c_long && !got_int.get() {
        if advance {
            (*curwin.get()).w_cursor.lnum += 1;
            prev_is_end_par = is_end_par;
            is_not_par = next_is_not_par;
            leader_len = next_leader_len;
            leader_flags = next_leader_flags;
        }
        if count == 1 as ::core::ffi::c_long
            || (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_ml.ml_line_count
        {
            next_is_not_par = true_0 != 0;
            next_leader_len = 0 as ::core::ffi::c_int;
            next_leader_flags = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            next_is_not_par = fmt_check_par(
                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                &raw mut next_leader_len,
                &raw mut next_leader_flags,
                do_comments,
            ) != 0;
            if do_number_indent {
                next_is_start_par =
                    get_number_indent((*curwin.get()).w_cursor.lnum + 1 as linenr_T)
                        > 0 as ::core::ffi::c_int;
            }
        }
        advance = true_0 != 0;
        is_end_par = is_not_par as ::core::ffi::c_int != 0
            || next_is_not_par as ::core::ffi::c_int != 0
            || next_is_start_par as ::core::ffi::c_int != 0;
        if !is_end_par && do_trail_white as ::core::ffi::c_int != 0 {
            is_end_par = !ends_in_white((*curwin.get()).w_cursor.lnum);
        }
        if is_not_par {
            if line_count < 0 as linenr_T {
                break;
            }
        } else {
            if first_par_line as ::core::ffi::c_int != 0
                && (do_second_indent as ::core::ffi::c_int != 0
                    || do_number_indent as ::core::ffi::c_int != 0)
                && prev_is_end_par as ::core::ffi::c_int != 0
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
            {
                if do_second_indent as ::core::ffi::c_int != 0
                    && !(*ml_get((*curwin.get()).w_cursor.lnum + 1 as linenr_T)
                        as ::core::ffi::c_int
                        == NUL)
                {
                    if leader_len == 0 as ::core::ffi::c_int
                        && next_leader_len == 0 as ::core::ffi::c_int
                    {
                        second_indent =
                            get_indent_lnum((*curwin.get()).w_cursor.lnum + 1 as linenr_T);
                    } else {
                        second_indent = next_leader_len;
                        do_comments_list = 1 as ::core::ffi::c_int;
                    }
                } else if do_number_indent {
                    if leader_len == 0 as ::core::ffi::c_int
                        && next_leader_len == 0 as ::core::ffi::c_int
                    {
                        second_indent = get_number_indent((*curwin.get()).w_cursor.lnum);
                    } else {
                        second_indent = get_number_indent((*curwin.get()).w_cursor.lnum);
                        do_comments_list = 1 as ::core::ffi::c_int;
                    }
                }
            }
            if (*curwin.get()).w_cursor.lnum >= (*curbuf.get()).b_ml.ml_line_count
                || !same_leader(
                    (*curwin.get()).w_cursor.lnum,
                    leader_len,
                    leader_flags,
                    next_leader_len,
                    next_leader_flags,
                )
            {
                if next_leader_flags.is_null()
                    || strncmp(
                        next_leader_flags,
                        b"://\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as size_t,
                    ) != 0 as ::core::ffi::c_int
                    || check_linecomment(get_cursor_line_ptr()) == MAXCOL as ::core::ffi::c_int
                {
                    is_end_par = true_0 != 0;
                }
            }
            if is_end_par as ::core::ffi::c_int != 0 || force_format as ::core::ffi::c_int != 0 {
                if need_set_indent {
                    let mut indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if (*curwin.get()).w_cursor.lnum == first_line {
                        indent = get_indent();
                    } else if (*curbuf.get()).b_p_lisp != 0 {
                        indent = get_lisp_indent();
                    } else if cindent_on() {
                        indent = if *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL {
                            get_expr_indent()
                        } else {
                            get_c_indent()
                        };
                    } else {
                        indent = get_indent();
                    }
                    set_indent(indent, SIN_CHANGED as ::core::ffi::c_int);
                }
                State.set(MODE_NORMAL as ::core::ffi::c_int);
                coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
                while (*curwin.get()).w_cursor.col != 0
                    && ascii_isspace(gchar_cursor()) as ::core::ffi::c_int != 0
                {
                    dec_cursor();
                }
                State.set(MODE_INSERT as ::core::ffi::c_int);
                smd_save = p_smd.get();
                p_smd.set(false_0);
                insertchar(
                    NUL,
                    INSCHAR_FORMAT as ::core::ffi::c_int
                        + (if do_comments as ::core::ffi::c_int != 0 {
                            INSCHAR_DO_COM as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        })
                        + (if do_comments as ::core::ffi::c_int != 0 && do_comments_list != 0 {
                            INSCHAR_COM_LIST as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        })
                        + (if avoid_fex as ::core::ffi::c_int != 0 {
                            INSCHAR_NO_FEX as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }),
                    second_indent,
                );
                State.set(old_State);
                p_smd.set(smd_save);
                ui_cursor_shape();
                second_indent = -1 as ::core::ffi::c_int;
                need_set_indent = is_end_par;
                if is_end_par {
                    if line_count < 0 as linenr_T {
                        break;
                    }
                    first_par_line = true_0 != 0;
                }
                force_format = false_0 != 0;
            }
            if !is_end_par {
                advance = false_0 != 0;
                (*curwin.get()).w_cursor.lnum += 1;
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                if line_count < 0 as linenr_T && u_save_cursor() == FAIL {
                    break;
                }
                if next_leader_len > 0 as ::core::ffi::c_int {
                    del_bytes(next_leader_len as colnr_T, false_0 != 0, false_0 != 0);
                    mark_col_adjust(
                        (*curwin.get()).w_cursor.lnum,
                        0 as colnr_T,
                        0 as linenr_T,
                        -(next_leader_len as colnr_T),
                        0 as ::core::ffi::c_int,
                    );
                } else if second_indent > 0 as ::core::ffi::c_int {
                    let mut indent_0: ::core::ffi::c_int =
                        getwhitecols_curline() as ::core::ffi::c_int;
                    if indent_0 > 0 as ::core::ffi::c_int {
                        del_bytes(indent_0 as colnr_T, false_0 != 0, false_0 != 0);
                        mark_col_adjust(
                            (*curwin.get()).w_cursor.lnum,
                            0 as colnr_T,
                            0 as linenr_T,
                            -(indent_0 as colnr_T),
                            0 as ::core::ffi::c_int,
                        );
                    }
                }
                (*curwin.get()).w_cursor.lnum -= 1;
                if do_join(
                    2 as size_t,
                    true_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                ) == FAIL
                {
                    beep_flush();
                    break;
                } else {
                    first_par_line = false_0 != 0;
                    force_format = get_cursor_line_len() > max_len;
                }
            }
        }
        line_breakcheck();
        count -= 1;
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
