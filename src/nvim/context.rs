use crate::src::nvim::api::private::converter::object_to_vim;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_array, api_free_string, api_set_error, arena_dict, copy_array,
    copy_object, cstr_as_string, string_to_array,
};
use crate::src::nvim::api::vimscript::exec_impl;
use crate::src::nvim::eval::encode::encode_vim_list_to_buf;
use crate::src::nvim::eval::typval::tv_clear;
use crate::src::nvim::eval::userfunc::func_tbl_get;
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::memory::{strequal, xfree, xmalloc, xrealloc};
use crate::src::nvim::option::{get_option_value, optval_free, set_option_value};
use crate::src::nvim::os::libc::{__assert_fail, snprintf, strlen, strncmp};
use crate::src::nvim::shada::{
    shada_encode_buflist, shada_encode_gvars, shada_encode_jumps, shada_encode_regs,
    shada_read_string,
};
pub use crate::src::nvim::types::{
    blob_T, blobvar_S, dict_T, dictvar_S, float_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed, funccall_T, garray_T, hash_T, hashitem_T, hashtab_T,
    int32_t, int64_t, key_value_pair, linenr_T, list_T, listitem_S, listitem_T, listvar_S,
    listwatch_S, listwatch_T, object, object_data as C2Rust_Unnamed_0, partial_S, partial_T,
    proftime_T, queue, scid_T, sctx_T, size_t, typval_T, typval_vval_union, ufunc_S, ufunc_T,
    uint64_t, uint8_t, varnumber_T, Arena, Array, BoolVarValue, Boolean, Context, Dict, Error,
    ErrorType, Float, Integer, KeyDict_exec_opts, KeyValuePair, LuaRef, Object, ObjectType,
    OptIndex, OptInt, OptVal, OptValData, OptValType, ScopeDictDictItem, ScopeType,
    SpecialVarValue, String_0, TriState, VarLockStatus, VarType, QUEUE,
};
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ContextVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Context,
}
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const kCtxFuncs: C2Rust_Unnamed_1 = 32;
pub const kCtxSFuncs: C2Rust_Unnamed_1 = 16;
pub const kCtxGVars: C2Rust_Unnamed_1 = 8;
pub const kCtxBufs: C2Rust_Unnamed_1 = 4;
pub const kCtxJumps: C2Rust_Unnamed_1 = 2;
pub const kCtxRegs: C2Rust_Unnamed_1 = 1;
pub const OPT_GLOBAL: C2Rust_Unnamed_2 = 1;
pub const kShaDaForceit: C2Rust_Unnamed_3 = 4;
pub const kShaDaWantInfo: C2Rust_Unnamed_3 = 1;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_2 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_2 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_2 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_2 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_2 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_2 = 4;
pub const OPT_LOCAL: C2Rust_Unnamed_2 = 2;
pub type C2Rust_Unnamed_3 = ::core::ffi::c_uint;
pub const kShaDaMissingError: C2Rust_Unnamed_3 = 16;
pub const kShaDaGetOldfiles: C2Rust_Unnamed_3 = 8;
pub const kShaDaWantMarks: C2Rust_Unnamed_3 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub static kCtxAll: GlobalCell<::core::ffi::c_int> = GlobalCell::new(
    kCtxRegs as ::core::ffi::c_int
        | kCtxJumps as ::core::ffi::c_int
        | kCtxBufs as ::core::ffi::c_int
        | kCtxGVars as ::core::ffi::c_int
        | kCtxSFuncs as ::core::ffi::c_int
        | kCtxFuncs as ::core::ffi::c_int,
);
static ctx_stack: GlobalCell<ContextVec> = GlobalCell::new(ContextVec {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Context>(),
});
pub unsafe extern "C" fn ctx_free_all() {
    let mut i: size_t = 0 as size_t;
    while i < (*ctx_stack.ptr()).size {
        ctx_free((*ctx_stack.ptr()).items.offset(i as isize));
        i = i.wrapping_add(1);
    }
    xfree((*ctx_stack.ptr()).items as *mut ::core::ffi::c_void);
    (*ctx_stack.ptr()).capacity = 0 as size_t;
    (*ctx_stack.ptr()).size = (*ctx_stack.ptr()).capacity;
    (*ctx_stack.ptr()).items = ::core::ptr::null_mut::<Context>();
}
pub unsafe extern "C" fn ctx_size() -> size_t {
    return (*ctx_stack.ptr()).size;
}
pub unsafe extern "C" fn ctx_get(mut index: size_t) -> *mut Context {
    if index < (*ctx_stack.ptr()).size {
        return (*ctx_stack.ptr()).items.offset(
            (*ctx_stack.ptr())
                .size
                .wrapping_sub(index)
                .wrapping_sub(1 as size_t) as isize,
        );
    }
    return ::core::ptr::null_mut::<Context>();
}
pub unsafe extern "C" fn ctx_free(mut ctx: *mut Context) {
    api_free_string((*ctx).regs);
    api_free_string((*ctx).jumps);
    api_free_string((*ctx).bufs);
    api_free_string((*ctx).gvars);
    api_free_array((*ctx).funcs);
}
pub unsafe extern "C" fn ctx_save(mut ctx: *mut Context, flags: ::core::ffi::c_int) {
    if ctx.is_null() {
        if (*ctx_stack.ptr()).size == (*ctx_stack.ptr()).capacity {
            (*ctx_stack.ptr()).capacity = if (*ctx_stack.ptr()).capacity != 0 {
                (*ctx_stack.ptr()).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*ctx_stack.ptr()).items = xrealloc(
                (*ctx_stack.ptr()).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Context>().wrapping_mul((*ctx_stack.ptr()).capacity),
            ) as *mut Context;
        } else {
        };
        let c2rust_fresh0 = (*ctx_stack.ptr()).size;
        (*ctx_stack.ptr()).size = (*ctx_stack.ptr()).size.wrapping_add(1);
        *(*ctx_stack.ptr()).items.offset(c2rust_fresh0 as isize) = Context {
            regs: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            },
            jumps: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            },
            bufs: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            },
            gvars: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            },
            funcs: Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            },
        };
        ctx = (*ctx_stack.ptr()).items.offset(
            (*ctx_stack.ptr())
                .size
                .wrapping_sub(0 as size_t)
                .wrapping_sub(1 as size_t) as isize,
        );
    }
    if flags & kCtxRegs as ::core::ffi::c_int != 0 {
        (*ctx).regs = shada_encode_regs();
    }
    if flags & kCtxJumps as ::core::ffi::c_int != 0 {
        (*ctx).jumps = shada_encode_jumps();
    }
    if flags & kCtxBufs as ::core::ffi::c_int != 0 {
        (*ctx).bufs = shada_encode_buflist();
    }
    if flags & kCtxGVars as ::core::ffi::c_int != 0 {
        (*ctx).gvars = shada_encode_gvars();
    }
    if flags & kCtxFuncs as ::core::ffi::c_int != 0 {
        ctx_save_funcs(ctx, false_0 != 0);
    } else if flags & kCtxSFuncs as ::core::ffi::c_int != 0 {
        ctx_save_funcs(ctx, true_0 != 0);
    }
}
pub unsafe extern "C" fn ctx_restore(mut ctx: *mut Context, flags: ::core::ffi::c_int) -> bool {
    let mut free_ctx: bool = false_0 != 0;
    if ctx.is_null() {
        if (*ctx_stack.ptr()).size == 0 as size_t {
            return false_0 != 0;
        }
        (*ctx_stack.ptr()).size = (*ctx_stack.ptr()).size.wrapping_sub(1);
        ctx = (*ctx_stack.ptr())
            .items
            .offset((*ctx_stack.ptr()).size as isize);
        free_ctx = true_0 != 0;
    }
    let mut op_shada: OptVal = get_option_value(kOptShada, OPT_GLOBAL as ::core::ffi::c_int);
    set_option_value(
        kOptShada,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: String_0 {
                    data: b"!,'100,%\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        },
        OPT_GLOBAL as ::core::ffi::c_int,
    );
    if flags & kCtxRegs as ::core::ffi::c_int != 0 {
        ctx_restore_regs(ctx);
    }
    if flags & kCtxJumps as ::core::ffi::c_int != 0 {
        ctx_restore_jumps(ctx);
    }
    if flags & kCtxBufs as ::core::ffi::c_int != 0 {
        ctx_restore_bufs(ctx);
    }
    if flags & kCtxGVars as ::core::ffi::c_int != 0 {
        ctx_restore_gvars(ctx);
    }
    if flags & kCtxFuncs as ::core::ffi::c_int != 0 {
        ctx_restore_funcs(ctx);
    }
    if free_ctx {
        ctx_free(ctx);
    }
    set_option_value(kOptShada, op_shada, OPT_GLOBAL as ::core::ffi::c_int);
    optval_free(op_shada);
    return true_0 != 0;
}
#[inline]
unsafe extern "C" fn ctx_restore_regs(mut ctx: *mut Context) {
    shada_read_string(
        (*ctx).regs,
        kShaDaWantInfo as ::core::ffi::c_int | kShaDaForceit as ::core::ffi::c_int,
    );
}
#[inline]
unsafe extern "C" fn ctx_restore_jumps(mut ctx: *mut Context) {
    shada_read_string(
        (*ctx).jumps,
        kShaDaWantInfo as ::core::ffi::c_int | kShaDaForceit as ::core::ffi::c_int,
    );
}
#[inline]
unsafe extern "C" fn ctx_restore_bufs(mut ctx: *mut Context) {
    shada_read_string(
        (*ctx).bufs,
        kShaDaWantInfo as ::core::ffi::c_int | kShaDaForceit as ::core::ffi::c_int,
    );
}
#[inline]
unsafe extern "C" fn ctx_restore_gvars(mut ctx: *mut Context) {
    shada_read_string(
        (*ctx).gvars,
        kShaDaWantInfo as ::core::ffi::c_int | kShaDaForceit as ::core::ffi::c_int,
    );
}
#[inline]
unsafe extern "C" fn ctx_save_funcs(mut ctx: *mut Context, mut scriptonly: bool) {
    (*ctx).funcs = ARRAY_DICT_INIT;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let hiht_: *mut hashtab_T = func_tbl_get();
    let mut hitodo_: size_t = (*hiht_).ht_used;
    let mut hi: *mut hashitem_T = (*hiht_).ht_array;
    while hitodo_ != 0 {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            hitodo_ = hitodo_.wrapping_sub(1);
            let name: *const ::core::ffi::c_char = (*hi).hi_key;
            let mut islambda: bool = strncmp(
                name,
                b"<lambda>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int;
            let mut isscript: bool = *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t
                as ::core::ffi::c_int
                == 0x80 as ::core::ffi::c_int;
            if !islambda && (!scriptonly || isscript as ::core::ffi::c_int != 0) {
                let mut cmd_len: size_t =
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_add(strlen(name));
                let mut cmd: *mut ::core::ffi::c_char =
                    xmalloc(cmd_len) as *mut ::core::ffi::c_char;
                snprintf(
                    cmd,
                    cmd_len,
                    b"func! %s\0".as_ptr() as *const ::core::ffi::c_char,
                    name,
                );
                let mut opts: KeyDict_exec_opts = KeyDict_exec_opts { output: true };
                let mut func_body: String_0 = exec_impl(
                    (1 as ::core::ffi::c_int as uint64_t)
                        << ::core::mem::size_of::<uint64_t>()
                            .wrapping_mul(8 as usize)
                            .wrapping_sub(1 as usize),
                    cstr_as_string(cmd),
                    &raw mut opts,
                    &raw mut err,
                );
                xfree(cmd as *mut ::core::ffi::c_void);
                if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                    if (*ctx).funcs.size == (*ctx).funcs.capacity {
                        (*ctx).funcs.capacity = if (*ctx).funcs.capacity != 0 {
                            (*ctx).funcs.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        (*ctx).funcs.items = xrealloc(
                            (*ctx).funcs.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul((*ctx).funcs.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh1 = (*ctx).funcs.size;
                    (*ctx).funcs.size = (*ctx).funcs.size.wrapping_add(1);
                    *(*ctx).funcs.items.offset(c2rust_fresh1 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_0 { string: func_body },
                    };
                }
                api_clear_error(&raw mut err);
            }
        }
        hi = hi.offset(1);
    }
}
#[inline]
unsafe extern "C" fn ctx_restore_funcs(mut ctx: *mut Context) {
    let mut i: size_t = 0 as size_t;
    while i < (*ctx).funcs.size {
        do_cmdline_cmd((*(*ctx).funcs.items.offset(i as isize)).data.string.data);
        i = i.wrapping_add(1);
    }
}
#[inline]
unsafe extern "C" fn array_to_string(mut array: Array, mut err: *mut Error) -> String_0 {
    let mut sbuf: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    let mut list_tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    object_to_vim(
        object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 { array: array },
        },
        &raw mut list_tv,
        err,
    );
    '_c2rust_label: {
        if list_tv.v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"list_tv.v_type == VAR_LIST\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/context.rs\0".as_ptr() as *const ::core::ffi::c_char,
                257 as ::core::ffi::c_uint,
                b"String array_to_string(Array, Error *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if !encode_vim_list_to_buf(list_tv.vval.v_list, &raw mut sbuf.size, &raw mut sbuf.data) {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            b"E474: Failed to convert list to msgpack string buffer\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
    tv_clear(&raw mut list_tv);
    return sbuf;
}
pub unsafe extern "C" fn ctx_to_dict(mut ctx: *mut Context, mut arena: *mut Arena) -> Dict {
    '_c2rust_label: {
        if !ctx.is_null() {
        } else {
            __assert_fail(
                b"ctx != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/context.rs\0".as_ptr() as *const ::core::ffi::c_char,
                275 as ::core::ffi::c_uint,
                b"Dict ctx_to_dict(Context *, Arena *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut rv: Dict = arena_dict(arena, 5 as size_t);
    let c2rust_fresh2 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh2 as isize) = key_value_pair {
        key: cstr_as_string(b"regs\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 {
                array: string_to_array((*ctx).regs, false, arena),
            },
        },
    };
    let c2rust_fresh3 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh3 as isize) = key_value_pair {
        key: cstr_as_string(b"jumps\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 {
                array: string_to_array((*ctx).jumps, false, arena),
            },
        },
    };
    let c2rust_fresh4 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh4 as isize) = key_value_pair {
        key: cstr_as_string(b"bufs\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 {
                array: string_to_array((*ctx).bufs, false, arena),
            },
        },
    };
    let c2rust_fresh5 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh5 as isize) = key_value_pair {
        key: cstr_as_string(b"gvars\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 {
                array: string_to_array((*ctx).gvars, false, arena),
            },
        },
    };
    let c2rust_fresh6 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh6 as isize) = key_value_pair {
        key: cstr_as_string(b"funcs\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed_0 {
                array: copy_array((*ctx).funcs, arena),
            },
        },
    };
    return rv;
}
pub unsafe extern "C" fn ctx_from_dict(
    mut dict: Dict,
    mut ctx: *mut Context,
    mut err: *mut Error,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if !ctx.is_null() {
        } else {
            __assert_fail(
                b"ctx != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/context.rs\0".as_ptr() as *const ::core::ffi::c_char,
                298 as ::core::ffi::c_uint,
                b"int ctx_from_dict(Dict, Context *, Error *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut types: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    while i < dict.size
        && !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
    {
        let mut item: KeyValuePair = *dict.items.offset(i as isize);
        if item.value.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if strequal(
                item.key.data,
                b"regs\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                types |= kCtxRegs as ::core::ffi::c_int;
                (*ctx).regs = array_to_string(item.value.data.array, err);
            } else if strequal(
                item.key.data,
                b"jumps\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                types |= kCtxJumps as ::core::ffi::c_int;
                (*ctx).jumps = array_to_string(item.value.data.array, err);
            } else if strequal(
                item.key.data,
                b"bufs\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                types |= kCtxBufs as ::core::ffi::c_int;
                (*ctx).bufs = array_to_string(item.value.data.array, err);
            } else if strequal(
                item.key.data,
                b"gvars\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                types |= kCtxGVars as ::core::ffi::c_int;
                (*ctx).gvars = array_to_string(item.value.data.array, err);
            } else if strequal(
                item.key.data,
                b"funcs\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                types |= kCtxFuncs as ::core::ffi::c_int;
                (*ctx).funcs = copy_object(item.value, ::core::ptr::null_mut::<Arena>())
                    .data
                    .array;
            }
        }
        i = i.wrapping_add(1);
    }
    return types;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
