extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn object_to_vim(obj: Object, tv: *mut typval_T, err: *mut Error);
    fn exec_impl(
        channel_id: uint64_t,
        src: String_0,
        opts: *mut KeyDict_exec_opts,
        err: *mut Error,
    ) -> String_0;
    fn encode_vim_list_to_buf(
        list: *const list_T,
        ret_len: *mut size_t,
        ret_buf: *mut *mut ::core::ffi::c_char,
    ) -> bool;
    static mut hash_removed: ::core::ffi::c_char;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn string_to_array(input: String_0, crlf: bool, arena: *mut Arena) -> Array;
    fn api_free_string(value: String_0);
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn api_free_array(value: Array);
    fn api_clear_error(value: *mut Error);
    fn copy_array(array: Array, arena: *mut Arena) -> Array;
    fn copy_object(obj: Object, arena: *mut Arena) -> Object;
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn tv_clear(tv: *mut typval_T);
    fn func_tbl_get() -> *mut hashtab_T;
    fn do_cmdline_cmd(cmd: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn optval_free(o: OptVal);
    fn get_option_value(opt_idx: OptIndex, opt_flags: ::core::ffi::c_int) -> OptVal;
    fn set_option_value(
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: ::core::ffi::c_int,
    ) -> *const ::core::ffi::c_char;
    fn shada_encode_regs() -> String_0;
    fn shada_encode_jumps() -> String_0;
    fn shada_encode_buflist() -> String_0;
    fn shada_encode_gvars() -> String_0;
    fn shada_read_string(string: String_0, flags: ::core::ffi::c_int);
}
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint64_t = u64;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
pub type proftime_T = uint64_t;
pub type TriState = ::core::ffi::c_int;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub type OptInt = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub type linenr_T = int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashtab_T {
    pub ht_mask: hash_T,
    pub ht_used: size_t,
    pub ht_filled: size_t,
    pub ht_changed: ::core::ffi::c_int,
    pub ht_locked: ::core::ffi::c_int,
    pub ht_array: *mut hashitem_T,
    pub ht_smallarray: [hashitem_T; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
}
pub type hash_T = size_t;
pub type partial_T = partial_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct partial_S {
    pub pt_refcount: ::core::ffi::c_int,
    pub pt_copyID: ::core::ffi::c_int,
    pub pt_name: *mut ::core::ffi::c_char,
    pub pt_func: *mut ufunc_T,
    pub pt_auto: bool,
    pub pt_argc: ::core::ffi::c_int,
    pub pt_argv: *mut typval_T,
    pub pt_dict: *mut dict_T,
}
pub type dict_T = dictvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictvar_S {
    pub dv_lock: VarLockStatus,
    pub dv_scope: ScopeType,
    pub dv_refcount: ::core::ffi::c_int,
    pub dv_copyID: ::core::ffi::c_int,
    pub dv_hashtab: hashtab_T,
    pub dv_copydict: *mut dict_T,
    pub dv_used_next: *mut dict_T,
    pub dv_used_prev: *mut dict_T,
    pub watchers: QUEUE,
    pub lua_table_ref: LuaRef,
}
pub type QUEUE = queue;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
pub type ScopeType = ::core::ffi::c_uint;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub type VarLockStatus = ::core::ffi::c_uint;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct typval_T {
    pub v_type: VarType,
    pub v_lock: VarLockStatus,
    pub vval: typval_vval_union,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union typval_vval_union {
    pub v_number: varnumber_T,
    pub v_bool: BoolVarValue,
    pub v_special: SpecialVarValue,
    pub v_float: float_T,
    pub v_string: *mut ::core::ffi::c_char,
    pub v_list: *mut list_T,
    pub v_dict: *mut dict_T,
    pub v_partial: *mut partial_T,
    pub v_blob: *mut blob_T,
}
pub type blob_T = blobvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct blobvar_S {
    pub bv_ga: garray_T,
    pub bv_refcount: ::core::ffi::c_int,
    pub bv_lock: VarLockStatus,
}
pub type list_T = listvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listvar_S {
    pub lv_first: *mut listitem_T,
    pub lv_last: *mut listitem_T,
    pub lv_watch: *mut listwatch_T,
    pub lv_idx_item: *mut listitem_T,
    pub lv_copylist: *mut list_T,
    pub lv_used_next: *mut list_T,
    pub lv_used_prev: *mut list_T,
    pub lv_refcount: ::core::ffi::c_int,
    pub lv_len: ::core::ffi::c_int,
    pub lv_idx: ::core::ffi::c_int,
    pub lv_copyID: ::core::ffi::c_int,
    pub lv_lock: VarLockStatus,
    pub lua_table_ref: LuaRef,
}
pub type listitem_T = listitem_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listitem_S {
    pub li_next: *mut listitem_T,
    pub li_prev: *mut listitem_T,
    pub li_tv: typval_T,
}
pub type listwatch_T = listwatch_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct listwatch_S {
    pub lw_item: *mut listitem_T,
    pub lw_next: *mut listwatch_T,
}
pub type SpecialVarValue = ::core::ffi::c_uint;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub type BoolVarValue = ::core::ffi::c_uint;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub type varnumber_T = int64_t;
pub type VarType = ::core::ffi::c_uint;
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
pub type ufunc_T = ufunc_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ufunc_S {
    pub uf_varargs: ::core::ffi::c_int,
    pub uf_flags: ::core::ffi::c_int,
    pub uf_calls: ::core::ffi::c_int,
    pub uf_cleared: bool,
    pub uf_args: garray_T,
    pub uf_def_args: garray_T,
    pub uf_lines: garray_T,
    pub uf_profiling: ::core::ffi::c_int,
    pub uf_prof_initialized: ::core::ffi::c_int,
    pub uf_luaref: LuaRef,
    pub uf_tm_count: ::core::ffi::c_int,
    pub uf_tm_total: proftime_T,
    pub uf_tm_self: proftime_T,
    pub uf_tm_children: proftime_T,
    pub uf_tml_count: *mut ::core::ffi::c_int,
    pub uf_tml_total: *mut proftime_T,
    pub uf_tml_self: *mut proftime_T,
    pub uf_tml_start: proftime_T,
    pub uf_tml_children: proftime_T,
    pub uf_tml_wait: proftime_T,
    pub uf_tml_idx: ::core::ffi::c_int,
    pub uf_tml_execed: ::core::ffi::c_int,
    pub uf_script_ctx: sctx_T,
    pub uf_refcount: ::core::ffi::c_int,
    pub uf_scoped: *mut funccall_T,
    pub uf_name_exp: *mut ::core::ffi::c_char,
    pub uf_namelen: size_t,
    pub uf_name: [::core::ffi::c_char; 0],
}
pub type funccall_T = funccall_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funccall_S {
    pub fc_func: *mut ufunc_T,
    pub fc_linenr: ::core::ffi::c_int,
    pub fc_returned: ::core::ffi::c_int,
    pub fc_fixvar: [C2Rust_Unnamed; 12],
    pub fc_l_vars: dict_T,
    pub fc_l_vars_var: ScopeDictDictItem,
    pub fc_l_avars: dict_T,
    pub fc_l_avars_var: ScopeDictDictItem,
    pub fc_l_varlist: list_T,
    pub fc_l_listitems: [listitem_T; 20],
    pub fc_rettv: *mut typval_T,
    pub fc_breakpoint: linenr_T,
    pub fc_dbg_tick: ::core::ffi::c_int,
    pub fc_level: ::core::ffi::c_int,
    pub fc_defer: garray_T,
    pub fc_prof_child: proftime_T,
    pub fc_caller: *mut funccall_T,
    pub fc_refcount: ::core::ffi::c_int,
    pub fc_copyID: ::core::ffi::c_int,
    pub fc_ufuncs: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScopeDictDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 21],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sctx_T {
    pub sc_sid: scid_T,
    pub sc_seq: ::core::ffi::c_int,
    pub sc_lnum: linenr_T,
    pub sc_chan: uint64_t,
}
pub type scid_T = ::core::ffi::c_int;
pub type ErrorType = ::core::ffi::c_int;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error {
    pub type_0: ErrorType,
    pub msg: *mut ::core::ffi::c_char,
}
pub type Boolean = bool;
pub type Integer = int64_t;
pub type Float = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct object {
    pub type_0: ObjectType,
    pub data: C2Rust_Unnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
    pub boolean: Boolean,
    pub integer: Integer,
    pub floating: Float,
    pub string: String_0,
    pub array: Array,
    pub dict: Dict,
    pub luaref: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dict {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut KeyValuePair,
}
pub type KeyValuePair = key_value_pair;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_value_pair {
    pub key: String_0,
    pub value: Object,
}
pub type Object = object;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Array {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
}
pub type ObjectType = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_exec_opts {
    pub output: Boolean,
}
pub type OptIndex = ::core::ffi::c_int;
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
pub type OptValType = ::core::ffi::c_int;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub union OptValData {
    pub boolean: TriState,
    pub number: OptInt,
    pub string: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OptVal {
    pub type_0: OptValType,
    pub data: OptValData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Context {
    pub regs: String_0,
    pub jumps: String_0,
    pub bufs: String_0,
    pub gvars: String_0,
    pub funcs: Array,
}
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
#[no_mangle]
pub static mut kCtxAll: ::core::ffi::c_int = kCtxRegs as ::core::ffi::c_int
    | kCtxJumps as ::core::ffi::c_int
    | kCtxBufs as ::core::ffi::c_int
    | kCtxGVars as ::core::ffi::c_int
    | kCtxSFuncs as ::core::ffi::c_int
    | kCtxFuncs as ::core::ffi::c_int;
static mut ctx_stack: ContextVec = ContextVec {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Context>(),
};
#[no_mangle]
pub unsafe extern "C" fn ctx_free_all() {
    let mut i: size_t = 0 as size_t;
    while i < ctx_stack.size {
        ctx_free(ctx_stack.items.offset(i as isize));
        i = i.wrapping_add(1);
    }
    xfree(ctx_stack.items as *mut ::core::ffi::c_void);
    ctx_stack.capacity = 0 as size_t;
    ctx_stack.size = ctx_stack.capacity;
    ctx_stack.items = ::core::ptr::null_mut::<Context>();
}
#[no_mangle]
pub unsafe extern "C" fn ctx_size() -> size_t {
    return ctx_stack.size;
}
#[no_mangle]
pub unsafe extern "C" fn ctx_get(mut index: size_t) -> *mut Context {
    if index < ctx_stack.size {
        return ctx_stack
            .items
            .offset(ctx_stack.size.wrapping_sub(index).wrapping_sub(1 as size_t) as isize);
    }
    return ::core::ptr::null_mut::<Context>();
}
#[no_mangle]
pub unsafe extern "C" fn ctx_free(mut ctx: *mut Context) {
    api_free_string((*ctx).regs);
    api_free_string((*ctx).jumps);
    api_free_string((*ctx).bufs);
    api_free_string((*ctx).gvars);
    api_free_array((*ctx).funcs);
}
#[no_mangle]
pub unsafe extern "C" fn ctx_save(mut ctx: *mut Context, flags: ::core::ffi::c_int) {
    if ctx.is_null() {
        if ctx_stack.size == ctx_stack.capacity {
            ctx_stack.capacity = if ctx_stack.capacity != 0 {
                ctx_stack.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            ctx_stack.items = xrealloc(
                ctx_stack.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Context>().wrapping_mul(ctx_stack.capacity),
            ) as *mut Context;
        } else {
        };
        let c2rust_fresh0 = ctx_stack.size;
        ctx_stack.size = ctx_stack.size.wrapping_add(1);
        *ctx_stack.items.offset(c2rust_fresh0 as isize) = Context {
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
        ctx = ctx_stack.items.offset(
            ctx_stack
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
#[no_mangle]
pub unsafe extern "C" fn ctx_restore(mut ctx: *mut Context, flags: ::core::ffi::c_int) -> bool {
    let mut free_ctx: bool = false_0 != 0;
    if ctx.is_null() {
        if ctx_stack.size == 0 as size_t {
            return false_0 != 0;
        }
        ctx_stack.size = ctx_stack.size.wrapping_sub(1);
        ctx = ctx_stack.items.offset(ctx_stack.size as isize);
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
        if !((*hi).hi_key.is_null() || (*hi).hi_key == &raw mut hash_removed) {
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
                b"/home/overlord/projects/neovim/neovim/src/nvim/context.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
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
#[no_mangle]
pub unsafe extern "C" fn ctx_to_dict(mut ctx: *mut Context, mut arena: *mut Arena) -> Dict {
    '_c2rust_label: {
        if !ctx.is_null() {
        } else {
            __assert_fail(
                b"ctx != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/context.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
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
#[no_mangle]
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
                b"/home/overlord/projects/neovim/neovim/src/nvim/context.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
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
