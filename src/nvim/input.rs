use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type multiqueue;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn atoi(__nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getcmdline_prompt(
        firstc: ::core::ffi::c_int,
        prompt: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        xp_context: ::core::ffi::c_int,
        xp_arg: *const ::core::ffi::c_char,
        highlight_callback: Callback,
        one_key: bool,
        mouse_used: *mut bool,
    ) -> *mut ::core::ffi::c_char;
    fn merge_modifiers(
        c_arg: ::core::ffi::c_int,
        modifiers: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn fix_input_buffer(buf: *mut uint8_t, len: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mod_mask: GlobalCell<::core::ffi::c_int>;
    static cmdline_row: GlobalCell<::core::ffi::c_int>;
    static msg_row: GlobalCell<::core::ffi::c_int>;
    static msg_scrolled: GlobalCell<::core::ffi::c_int>;
    static keep_msg: GlobalCell<*mut ::core::ffi::c_char>;
    static keep_msg_hl_id: GlobalCell<::core::ffi::c_int>;
    static no_wait_return: GlobalCell<::core::ffi::c_int>;
    static need_wait_return: GlobalCell<bool>;
    static State: GlobalCell<::core::ffi::c_int>;
    static no_mapping: GlobalCell<::core::ffi::c_int>;
    static allow_keys: GlobalCell<::core::ffi::c_int>;
    static mapped_ctrl_c: GlobalCell<::core::ffi::c_int>;
    static IObuff: GlobalCell<[::core::ffi::c_char; 1025]>;
    static utf8len_tab: [uint8_t; 256];
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn set_keep_msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int);
    fn msg_putchar(c: ::core::ffi::c_int);
    fn is_mouse_key(c: ::core::ffi::c_int) -> bool;
    fn setmouse();
    fn ui_flush();
    fn ui_has(ext: UIExtension) -> bool;
    fn input_get(
        buf: *mut uint8_t,
        maxlen: ::core::ffi::c_int,
        ms: ::core::ffi::c_int,
        tb_change_cnt: ::core::ffi::c_int,
        events: *mut MultiQueue,
    ) -> ::core::ffi::c_int;
}
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint64_t = u64;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub type hash_T = size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
}
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
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
pub type QUEUE = queue;
pub type linenr_T = int32_t;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
pub type proftime_T = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Callback {
    pub data: C2Rust_Unnamed,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed {
    pub funcref: *mut ::core::ffi::c_char,
    pub partial: *mut partial_T,
    pub luaref: LuaRef,
}
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
    pub fc_fixvar: [C2Rust_Unnamed_0; 12],
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
pub struct C2Rust_Unnamed_0 {
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
pub type MultiQueue = multiqueue;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_1 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_1 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_1 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_1 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_1 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_1 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_1 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_1 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_1 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_1 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_1 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_1 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_1 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_1 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_1 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_1 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_1 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_1 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_1 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_1 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_1 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_1 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_1 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_1 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_1 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_1 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_1 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_1 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_1 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_1 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_1 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_1 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_1 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_1 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_1 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_1 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_1 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_1 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_1 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_1 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_1 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_1 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_1 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_1 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_1 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_1 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_1 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_1 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_1 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_1 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_1 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_1 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_1 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_1 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_1 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_1 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_1 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_1 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_1 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_1 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_1 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_1 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_1 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_1 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_1 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_1 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_1 = -2;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_2 = 76;
pub const HLF_PRE: C2Rust_Unnamed_2 = 75;
pub const HLF_OK: C2Rust_Unnamed_2 = 74;
pub const HLF_SO: C2Rust_Unnamed_2 = 73;
pub const HLF_SE: C2Rust_Unnamed_2 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_2 = 71;
pub const HLF_TS: C2Rust_Unnamed_2 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_2 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_2 = 68;
pub const HLF_CU: C2Rust_Unnamed_2 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_2 = 66;
pub const HLF_WBR: C2Rust_Unnamed_2 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_2 = 64;
pub const HLF_MSG: C2Rust_Unnamed_2 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_2 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_2 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_2 = 60;
pub const HLF_0: C2Rust_Unnamed_2 = 59;
pub const HLF_QFL: C2Rust_Unnamed_2 = 58;
pub const HLF_MC: C2Rust_Unnamed_2 = 57;
pub const HLF_CUL: C2Rust_Unnamed_2 = 56;
pub const HLF_CUC: C2Rust_Unnamed_2 = 55;
pub const HLF_TPF: C2Rust_Unnamed_2 = 54;
pub const HLF_TPS: C2Rust_Unnamed_2 = 53;
pub const HLF_TP: C2Rust_Unnamed_2 = 52;
pub const HLF_PBR: C2Rust_Unnamed_2 = 51;
pub const HLF_PST: C2Rust_Unnamed_2 = 50;
pub const HLF_PSB: C2Rust_Unnamed_2 = 49;
pub const HLF_PSX: C2Rust_Unnamed_2 = 48;
pub const HLF_PNX: C2Rust_Unnamed_2 = 47;
pub const HLF_PSK: C2Rust_Unnamed_2 = 46;
pub const HLF_PNK: C2Rust_Unnamed_2 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_2 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_2 = 43;
pub const HLF_PSI: C2Rust_Unnamed_2 = 42;
pub const HLF_PNI: C2Rust_Unnamed_2 = 41;
pub const HLF_SPL: C2Rust_Unnamed_2 = 40;
pub const HLF_SPR: C2Rust_Unnamed_2 = 39;
pub const HLF_SPC: C2Rust_Unnamed_2 = 38;
pub const HLF_SPB: C2Rust_Unnamed_2 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_2 = 36;
pub const HLF_SC: C2Rust_Unnamed_2 = 35;
pub const HLF_TXA: C2Rust_Unnamed_2 = 34;
pub const HLF_TXD: C2Rust_Unnamed_2 = 33;
pub const HLF_DED: C2Rust_Unnamed_2 = 32;
pub const HLF_CHD: C2Rust_Unnamed_2 = 31;
pub const HLF_ADD: C2Rust_Unnamed_2 = 30;
pub const HLF_FC: C2Rust_Unnamed_2 = 29;
pub const HLF_FL: C2Rust_Unnamed_2 = 28;
pub const HLF_WM: C2Rust_Unnamed_2 = 27;
pub const HLF_W: C2Rust_Unnamed_2 = 26;
pub const HLF_VNC: C2Rust_Unnamed_2 = 25;
pub const HLF_V: C2Rust_Unnamed_2 = 24;
pub const HLF_T: C2Rust_Unnamed_2 = 23;
pub const HLF_VSP: C2Rust_Unnamed_2 = 22;
pub const HLF_C: C2Rust_Unnamed_2 = 21;
pub const HLF_SNC: C2Rust_Unnamed_2 = 20;
pub const HLF_S: C2Rust_Unnamed_2 = 19;
pub const HLF_R: C2Rust_Unnamed_2 = 18;
pub const HLF_CLF: C2Rust_Unnamed_2 = 17;
pub const HLF_CLS: C2Rust_Unnamed_2 = 16;
pub const HLF_CLN: C2Rust_Unnamed_2 = 15;
pub const HLF_LNB: C2Rust_Unnamed_2 = 14;
pub const HLF_LNA: C2Rust_Unnamed_2 = 13;
pub const HLF_N: C2Rust_Unnamed_2 = 12;
pub const HLF_CM: C2Rust_Unnamed_2 = 11;
pub const HLF_M: C2Rust_Unnamed_2 = 10;
pub const HLF_LC: C2Rust_Unnamed_2 = 9;
pub const HLF_L: C2Rust_Unnamed_2 = 8;
pub const HLF_I: C2Rust_Unnamed_2 = 7;
pub const HLF_E: C2Rust_Unnamed_2 = 6;
pub const HLF_D: C2Rust_Unnamed_2 = 5;
pub const HLF_AT: C2Rust_Unnamed_2 = 4;
pub const HLF_TERM: C2Rust_Unnamed_2 = 3;
pub const HLF_EOB: C2Rust_Unnamed_2 = 2;
pub const HLF_8: C2Rust_Unnamed_2 = 1;
pub const HLF_NONE: C2Rust_Unnamed_2 = 0;
pub type UIExtension = ::core::ffi::c_uint;
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
pub const KE_LEFTMOUSE: key_extra = 44;
pub const KE_IGNORE: key_extra = 53;
pub type key_extra = ::core::ffi::c_uint;
pub const KE_WILD: key_extra = 108;
pub const KE_COMMAND: key_extra = 104;
pub const KE_LUA: key_extra = 103;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_NOP: key_extra = 97;
pub const KE_DROP: key_extra = 95;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2DRAG: key_extra = 93;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1RELEASE: key_extra = 91;
pub const KE_X1DRAG: key_extra = 90;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_C_END: key_extra = 88;
pub const KE_C_HOME: key_extra = 87;
pub const KE_C_RIGHT: key_extra = 86;
pub const KE_C_LEFT: key_extra = 85;
pub const KE_CMDWIN: key_extra = 84;
pub const KE_PLUG: key_extra = 83;
pub const KE_SNR: key_extra = 82;
pub const KE_KDEL: key_extra = 80;
pub const KE_KINS: key_extra = 79;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSELEFT: key_extra = 77;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_S_XF4: key_extra = 74;
pub const KE_S_XF3: key_extra = 73;
pub const KE_S_XF2: key_extra = 72;
pub const KE_S_XF1: key_extra = 71;
pub const KE_LEFTRELEASE_NM: key_extra = 70;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_XRIGHT: key_extra = 68;
pub const KE_XLEFT: key_extra = 67;
pub const KE_XDOWN: key_extra = 66;
pub const KE_XUP: key_extra = 65;
pub const KE_ZHOME: key_extra = 64;
pub const KE_XHOME: key_extra = 63;
pub const KE_ZEND: key_extra = 62;
pub const KE_XEND: key_extra = 61;
pub const KE_XF4: key_extra = 60;
pub const KE_XF3: key_extra = 59;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const KE_S_TAB_OLD: key_extra = 55;
pub const KE_TAB: key_extra = 54;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLERELEASE: key_extra = 49;
pub const KE_MIDDLEDRAG: key_extra = 48;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTRELEASE: key_extra = 46;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_MOUSE: key_extra = 43;
pub const KE_S_F37: key_extra = 42;
pub const KE_S_F36: key_extra = 41;
pub const KE_S_F35: key_extra = 40;
pub const KE_S_F34: key_extra = 39;
pub const KE_S_F33: key_extra = 38;
pub const KE_S_F32: key_extra = 37;
pub const KE_S_F31: key_extra = 36;
pub const KE_S_F30: key_extra = 35;
pub const KE_S_F29: key_extra = 34;
pub const KE_S_F28: key_extra = 33;
pub const KE_S_F27: key_extra = 32;
pub const KE_S_F26: key_extra = 31;
pub const KE_S_F25: key_extra = 30;
pub const KE_S_F24: key_extra = 29;
pub const KE_S_F23: key_extra = 28;
pub const KE_S_F22: key_extra = 27;
pub const KE_S_F21: key_extra = 26;
pub const KE_S_F20: key_extra = 25;
pub const KE_S_F19: key_extra = 24;
pub const KE_S_F18: key_extra = 23;
pub const KE_S_F17: key_extra = 22;
pub const KE_S_F16: key_extra = 21;
pub const KE_S_F15: key_extra = 20;
pub const KE_S_F14: key_extra = 19;
pub const KE_S_F13: key_extra = 18;
pub const KE_S_F12: key_extra = 17;
pub const KE_S_F11: key_extra = 16;
pub const KE_S_F10: key_extra = 15;
pub const KE_S_F9: key_extra = 14;
pub const KE_S_F8: key_extra = 13;
pub const KE_S_F7: key_extra = 12;
pub const KE_S_F6: key_extra = 11;
pub const KE_S_F5: key_extra = 10;
pub const KE_S_F4: key_extra = 9;
pub const KE_S_F3: key_extra = 8;
pub const KE_S_F2: key_extra = 7;
pub const KE_S_F1: key_extra = 6;
pub const KE_S_DOWN: key_extra = 5;
pub const KE_S_UP: key_extra = 4;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn ask_yesno(str: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let save_State: ::core::ffi::c_int = State.get();
    (*no_wait_return.ptr()) += 1;
    snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        gettext(b"%s (y/n)?\0".as_ptr() as *const ::core::ffi::c_char),
        str,
    );
    let mut prompt: *mut ::core::ffi::c_char = xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
    let mut r: ::core::ffi::c_int = ' ' as ::core::ffi::c_int;
    while r != 'y' as ::core::ffi::c_int && r != 'n' as ::core::ffi::c_int {
        r = prompt_for_input(
            prompt,
            HLF_R as ::core::ffi::c_int,
            true_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        if r == Ctrl_C || r == ESC {
            r = 'n' as ::core::ffi::c_int;
            if !ui_has(kUIMessages) {
                msg_putchar(r);
            }
        }
    }
    need_wait_return.set(msg_scrolled.get() != 0);
    (*no_wait_return.ptr()) -= 1;
    State.set(save_State);
    setmouse();
    xfree(prompt as *mut ::core::ffi::c_void);
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn get_keystroke(mut events: *mut MultiQueue) -> ::core::ffi::c_int {
    let mut buf: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut buflen: ::core::ffi::c_int = 150 as ::core::ffi::c_int;
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = 0;
    let mut save_mapped_ctrl_c: ::core::ffi::c_int = mapped_ctrl_c.get();
    mod_mask.set(0 as ::core::ffi::c_int);
    mapped_ctrl_c.set(0 as ::core::ffi::c_int);
    loop {
        ui_flush();
        let mut maxlen: ::core::ffi::c_int =
            (buflen - 6 as ::core::ffi::c_int - len) / 3 as ::core::ffi::c_int;
        if buf.is_null() {
            buf = xmalloc(buflen as size_t) as *mut uint8_t;
        } else if maxlen < 10 as ::core::ffi::c_int {
            buflen += 100 as ::core::ffi::c_int;
            buf = xrealloc(buf as *mut ::core::ffi::c_void, buflen as size_t) as *mut uint8_t;
            maxlen = (buflen - 6 as ::core::ffi::c_int - len) / 3 as ::core::ffi::c_int;
        }
        n = input_get(
            buf.offset(len as isize),
            maxlen,
            if len == 0 as ::core::ffi::c_int {
                -1 as ::core::ffi::c_int
            } else {
                100 as ::core::ffi::c_int
            },
            0 as ::core::ffi::c_int,
            events,
        );
        if n > 0 as ::core::ffi::c_int {
            n = fix_input_buffer(buf.offset(len as isize), n);
            len += n;
        }
        if n > 0 as ::core::ffi::c_int {
            len = n;
        }
        if len == 0 as ::core::ffi::c_int {
            continue;
        }
        n = *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
        if n == K_SPECIAL {
            n = if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_SPECIAL
            {
                K_SPECIAL
            } else if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_ZERO
            {
                K_ZERO
            } else {
                -(*buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ((*buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int))
            };
            if !(*buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
                || n == -(253 as ::core::ffi::c_int
                    + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || is_mouse_key(n) as ::core::ffi::c_int != 0
                    && n != -(253 as ::core::ffi::c_int
                        + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
            {
                break;
            }
            if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER {
                mod_mask.set(*buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int);
            }
            len -= 3 as ::core::ffi::c_int;
            if len > 0 as ::core::ffi::c_int {
                memmove(
                    buf as *mut ::core::ffi::c_void,
                    buf.offset(3 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    len as size_t,
                );
            }
        } else {
            if utf8len_tab[n as usize] as ::core::ffi::c_int > len {
                continue;
            }
            *buf.offset(
                (if len >= buflen {
                    buflen - 1 as ::core::ffi::c_int
                } else {
                    len
                }) as isize,
            ) = NUL as uint8_t;
            n = utf_ptr2char(buf as *mut ::core::ffi::c_char);
            break;
        }
    }
    xfree(buf as *mut ::core::ffi::c_void);
    mapped_ctrl_c.set(save_mapped_ctrl_c);
    return merge_modifiers(n, mod_mask.ptr());
}
#[no_mangle]
pub unsafe extern "C" fn prompt_for_input(
    mut prompt: *mut ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut one_key: bool,
    mut mouse_used: *mut bool,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = if one_key as ::core::ffi::c_int != 0 {
        ESC
    } else {
        0 as ::core::ffi::c_int
    };
    let mut kmsg: *mut ::core::ffi::c_char = if !(*keep_msg.ptr()).is_null() {
        xstrdup(keep_msg.get())
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    if prompt.is_null() {
        if !mouse_used.is_null() {
            prompt = gettext(
                b"Type number and <Enter> or click with the mouse (q or empty cancels): \0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        } else {
            prompt = gettext(b"Type number and <Enter> (q or empty cancels): \0".as_ptr()
                as *const ::core::ffi::c_char);
        }
    }
    cmdline_row.set(msg_row.get());
    ui_flush();
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    let mut resp: *mut ::core::ffi::c_char = getcmdline_prompt(
        -1 as ::core::ffi::c_int,
        prompt,
        hl_id,
        EXPAND_NOTHING as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
        Callback {
            data: C2Rust_Unnamed {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        one_key,
        mouse_used,
    );
    (*allow_keys.ptr()) -= 1;
    (*no_mapping.ptr()) -= 1;
    if !resp.is_null() {
        ret = if one_key as ::core::ffi::c_int != 0 {
            *resp as ::core::ffi::c_int
        } else {
            atoi(resp)
        };
        xfree(resp as *mut ::core::ffi::c_void);
    }
    if !kmsg.is_null() {
        set_keep_msg(kmsg, keep_msg_hl_id.get());
        xfree(kmsg as *mut ::core::ffi::c_void);
    }
    return ret;
}
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const K_ZERO: ::core::ffi::c_int =
    -(255 as ::core::ffi::c_int + (('X' as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
