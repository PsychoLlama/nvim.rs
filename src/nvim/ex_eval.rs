use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type regprog;
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
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static empty_string_option: GlobalCell<[::core::ffi::c_char; 0]>;
    static p_cpo: GlobalCell<*mut ::core::ffi::c_char>;
    static p_verbose: GlobalCell<OptInt>;
    static p_vfile: GlobalCell<*mut ::core::ffi::c_char>;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn concat_str(
        str1: *const ::core::ffi::c_char,
        str2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn vim_snprintf_safelen(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> size_t;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn dbg_check_skipped(eap: *mut exarg_T) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_argreq: [::core::ffi::c_char; 0];
    static e_endif: [::core::ffi::c_char; 0];
    static e_endtry: [::core::ffi::c_char; 0];
    static e_endwhile: [::core::ffi::c_char; 0];
    static e_endfor: [::core::ffi::c_char; 0];
    static e_while: [::core::ffi::c_char; 0];
    static e_for: [::core::ffi::c_char; 0];
    static e_interr: [::core::ffi::c_char; 0];
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invexpr2: [::core::ffi::c_char; 0];
    static e_outofmem: [::core::ffi::c_char; 0];
    static e_trailing_arg: [::core::ffi::c_char; 0];
    static e_str_not_inside_function: [::core::ffi::c_char; 0];
    fn fill_evalarg_from_eap(evalarg: *mut evalarg_T, eap: *mut exarg_T, skip: bool);
    fn eval_to_bool(
        arg: *mut ::core::ffi::c_char,
        error: *mut bool,
        eap: *mut exarg_T,
        skip: bool,
        use_simple_function: bool,
    ) -> bool;
    fn eval_to_string_skip(
        arg: *mut ::core::ffi::c_char,
        eap: *mut exarg_T,
        skip: bool,
    ) -> *mut ::core::ffi::c_char;
    fn eval_for_line(
        arg: *const ::core::ffi::c_char,
        errp: *mut bool,
        eap: *mut exarg_T,
        evalarg: *mut evalarg_T,
    ) -> *mut ::core::ffi::c_void;
    fn next_for_item(fi_void: *mut ::core::ffi::c_void, arg: *mut ::core::ffi::c_char) -> bool;
    fn free_for_info(fi_void: *mut ::core::ffi::c_void);
    fn clear_evalarg(evalarg: *mut evalarg_T, eap: *mut exarg_T);
    fn eval0(
        arg: *mut ::core::ffi::c_char,
        rettv: *mut typval_T,
        eap: *mut exarg_T,
        evalarg: *mut evalarg_T,
    ) -> ::core::ffi::c_int;
    fn smsg(hl_id: ::core::ffi::c_int, s: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn internal_error(where_0: *const ::core::ffi::c_char);
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn verbose_enter();
    fn verbose_leave();
    fn tv_list_unref(l: *mut list_T);
    fn tv_clear(tv: *mut typval_T);
    fn tv_free(tv: *mut typval_T);
    fn do_return(
        eap: *mut exarg_T,
        reanimate: bool,
        is_cmd: bool,
        rettv: *mut ::core::ffi::c_void,
    ) -> bool;
    fn get_return_cmd(rettv: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_char;
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    fn set_vim_var_list(idx: VimVarIndex, val: *mut list_T);
    fn handle_did_throw();
    fn modifier_len(cmd: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ends_excmd(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn find_nextcmd(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static cmdline_row: GlobalCell<::core::ffi::c_int>;
    static msg_row: GlobalCell<::core::ffi::c_int>;
    static msg_scroll: GlobalCell<::core::ffi::c_int>;
    static emsg_off: GlobalCell<::core::ffi::c_int>;
    static did_endif: GlobalCell<bool>;
    static did_emsg: GlobalCell<::core::ffi::c_int>;
    static no_wait_return: GlobalCell<::core::ffi::c_int>;
    static debug_break_level: GlobalCell<::core::ffi::c_int>;
    static current_exception: GlobalCell<*mut except_T>;
    static did_throw: GlobalCell<bool>;
    static need_rethrow: GlobalCell<bool>;
    static trylevel: GlobalCell<::core::ffi::c_int>;
    static force_abort: GlobalCell<bool>;
    static msg_list: GlobalCell<*mut *mut msglist_T>;
    static suppress_errthrow: GlobalCell<bool>;
    static caught_stack: GlobalCell<*mut except_T>;
    static msg_silent: GlobalCell<::core::ffi::c_int>;
    static emsg_silent: GlobalCell<::core::ffi::c_int>;
    static IObuff: GlobalCell<[::core::ffi::c_char; 1025]>;
    static got_int: GlobalCell<bool>;
    static exestack: GlobalCell<garray_T>;
    fn estack_sfile(which: estack_arg_T) -> *mut ::core::ffi::c_char;
    fn stacktrace_create() -> *mut list_T;
    fn do_finish(eap: *mut exarg_T, reanimate: bool);
    fn skip_regexp_err(
        startp: *mut ::core::ffi::c_char,
        delim: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_nl(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T)
        -> bool;
}
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
pub type ptrdiff_t = isize;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Array {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
}
pub type Object = object;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct object {
    pub type_0: ObjectType,
    pub data: C2Rust_Unnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
}
pub type Float = ::core::ffi::c_double;
pub type Integer = int64_t;
pub type Boolean = bool;
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
pub type proftime_T = uint64_t;
pub type OptInt = int64_t;
pub type regprog_T = regprog;
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
pub type colnr_T = ::core::ffi::c_int;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct eslist_elem {
    pub saved_emsg_silent: ::core::ffi::c_int,
    pub next: *mut eslist_T,
}
pub type eslist_T = eslist_elem;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const CSTACK_LEN: C2Rust_Unnamed_1 = 50;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cstack_T {
    pub cs_flags: [::core::ffi::c_int; 50],
    pub cs_pending: [::core::ffi::c_char; 50],
    pub cs_pend: C2Rust_Unnamed_2,
    pub cs_forinfo: [*mut ::core::ffi::c_void; 50],
    pub cs_line: [::core::ffi::c_int; 50],
    pub cs_idx: ::core::ffi::c_int,
    pub cs_looplevel: ::core::ffi::c_int,
    pub cs_trylevel: ::core::ffi::c_int,
    pub cs_emsg_silent_list: *mut eslist_T,
    pub cs_lflags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
    pub csp_rv: [*mut ::core::ffi::c_void; 50],
    pub csp_ex: [*mut ::core::ffi::c_void; 50],
}
pub type C2Rust_Unnamed_3 = ::core::ffi::c_uint;
pub const CSF_SILENT: C2Rust_Unnamed_3 = 16384;
pub const CSF_FINISHED: C2Rust_Unnamed_3 = 8192;
pub const CSF_CAUGHT: C2Rust_Unnamed_3 = 4096;
pub const CSF_THROWN: C2Rust_Unnamed_3 = 2048;
pub const CSF_FINALLY: C2Rust_Unnamed_3 = 512;
pub const CSF_TRY: C2Rust_Unnamed_3 = 256;
pub const CSF_FOR: C2Rust_Unnamed_3 = 16;
pub const CSF_WHILE: C2Rust_Unnamed_3 = 8;
pub const CSF_ELSE: C2Rust_Unnamed_3 = 4;
pub const CSF_ACTIVE: C2Rust_Unnamed_3 = 2;
pub const CSF_TRUE: C2Rust_Unnamed_3 = 1;
pub type C2Rust_Unnamed_4 = ::core::ffi::c_uint;
pub const CSTP_FINISH: C2Rust_Unnamed_4 = 32;
pub const CSTP_RETURN: C2Rust_Unnamed_4 = 24;
pub const CSTP_CONTINUE: C2Rust_Unnamed_4 = 16;
pub const CSTP_BREAK: C2Rust_Unnamed_4 = 8;
pub const CSTP_THROW: C2Rust_Unnamed_4 = 4;
pub const CSTP_INTERRUPT: C2Rust_Unnamed_4 = 2;
pub const CSTP_ERROR: C2Rust_Unnamed_4 = 1;
pub const CSTP_NONE: C2Rust_Unnamed_4 = 0;
pub type C2Rust_Unnamed_5 = ::core::ffi::c_uint;
pub const CSL_HAD_FINA: C2Rust_Unnamed_5 = 8;
pub const CSL_HAD_CONT: C2Rust_Unnamed_5 = 4;
pub const CSL_HAD_ENDLOOP: C2Rust_Unnamed_5 = 2;
pub const CSL_HAD_LOOP: C2Rust_Unnamed_5 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msglist {
    pub next: *mut msglist_T,
    pub msg: *mut ::core::ffi::c_char,
    pub throw_msg: *mut ::core::ffi::c_char,
    pub sfile: *mut ::core::ffi::c_char,
    pub slnum: linenr_T,
    pub multiline: bool,
}
pub type msglist_T = msglist;
pub type except_type_T = ::core::ffi::c_uint;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vim_exception {
    pub type_0: except_type_T,
    pub value: *mut ::core::ffi::c_char,
    pub messages: *mut msglist_T,
    pub throw_name: *mut ::core::ffi::c_char,
    pub throw_lnum: linenr_T,
    pub stacktrace: *mut list_T,
    pub caught: *mut except_T,
}
pub type except_T = vim_exception;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cleanup_stuff {
    pub pending: ::core::ffi::c_int,
    pub exception: *mut except_T,
}
pub type cleanup_T = cleanup_stuff;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct exception_state_S {
    pub estate_current_exception: *mut except_T,
    pub estate_did_throw: bool,
    pub estate_need_rethrow: bool,
    pub estate_trylevel: ::core::ffi::c_int,
    pub estate_did_emsg: ::core::ffi::c_int,
}
pub type exception_state_T = exception_state_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
}
pub type CMD_index = ::core::ffi::c_int;
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
pub type cmdidx_T = CMD_index;
pub type cmd_addr_T = ::core::ffi::c_uint;
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
pub struct exarg {
    pub arg: *mut ::core::ffi::c_char,
    pub args: *mut *mut ::core::ffi::c_char,
    pub arglens: *mut size_t,
    pub argc: size_t,
    pub nextcmd: *mut ::core::ffi::c_char,
    pub cmd: *mut ::core::ffi::c_char,
    pub cmdlinep: *mut *mut ::core::ffi::c_char,
    pub cmdline_tofree: *mut ::core::ffi::c_char,
    pub cmdidx: cmdidx_T,
    pub argt: uint32_t,
    pub skip: ::core::ffi::c_int,
    pub forceit: ::core::ffi::c_int,
    pub addr_count: ::core::ffi::c_int,
    pub line1: linenr_T,
    pub line2: linenr_T,
    pub addr_type: cmd_addr_T,
    pub flags: ::core::ffi::c_int,
    pub do_ecmd_cmd: *mut ::core::ffi::c_char,
    pub do_ecmd_lnum: linenr_T,
    pub append: ::core::ffi::c_int,
    pub usefilter: ::core::ffi::c_int,
    pub amount: ::core::ffi::c_int,
    pub regname: ::core::ffi::c_int,
    pub force_bin: ::core::ffi::c_int,
    pub read_edit: ::core::ffi::c_int,
    pub mkdir_p: ::core::ffi::c_int,
    pub force_ff: ::core::ffi::c_int,
    pub force_enc: ::core::ffi::c_int,
    pub bad_char: ::core::ffi::c_int,
    pub useridx: ::core::ffi::c_int,
    pub errmsg: *mut ::core::ffi::c_char,
    pub ea_getline: LineGetter,
    pub cookie: *mut ::core::ffi::c_void,
    pub cstack: *mut cstack_T,
}
pub type LineGetter = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
        ::core::ffi::c_int,
        bool,
    ) -> *mut ::core::ffi::c_char,
>;
pub type exarg_T = exarg;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct evalarg_T {
    pub eval_flags: ::core::ffi::c_int,
    pub eval_getline: LineGetter,
    pub eval_cookie: *mut ::core::ffi::c_void,
    pub eval_tofree: *mut ::core::ffi::c_char,
}
pub type VimVarIndex = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: C2Rust_Unnamed_6,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
pub type AutoPatCmd = AutoPatCmd_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPatCmd_S {
    pub lastpat: *mut AutoPat,
    pub auidx: size_t,
    pub ausize: size_t,
    pub afile_orig: *mut ::core::ffi::c_char,
    pub fname: *mut ::core::ffi::c_char,
    pub sfname: *mut ::core::ffi::c_char,
    pub tail: *mut ::core::ffi::c_char,
    pub group: ::core::ffi::c_int,
    pub event: event_T,
    pub script_ctx: sctx_T,
    pub arg_bufnr: ::core::ffi::c_int,
    pub data: *mut Object,
    pub next: *mut AutoPatCmd,
}
pub type event_T = auto_event;
pub type auto_event = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPat {
    pub refcount: size_t,
    pub pat: *mut ::core::ffi::c_char,
    pub reg_prog: *mut regprog_T,
    pub group: ::core::ffi::c_int,
    pub patlen: ::core::ffi::c_int,
    pub buflocal_nr: ::core::ffi::c_int,
    pub allow_dirs: ::core::ffi::c_char,
}
pub type etype_T = ::core::ffi::c_uint;
pub const ETYPE_SPELL: etype_T = 9;
pub const ETYPE_INTERNAL: etype_T = 8;
pub const ETYPE_ENV: etype_T = 7;
pub const ETYPE_ARGS: etype_T = 6;
pub const ETYPE_EXCEPT: etype_T = 5;
pub const ETYPE_MODELINE: etype_T = 4;
pub const ETYPE_AUCMD: etype_T = 3;
pub const ETYPE_UFUNC: etype_T = 2;
pub const ETYPE_SCRIPT: etype_T = 1;
pub const ETYPE_TOP: etype_T = 0;
pub type estack_arg_T = ::core::ffi::c_uint;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const ESTACK_NONE: estack_arg_T = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
static e_multiple_else: GlobalCell<[::core::ffi::c_char; 21]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(*b"E583: Multiple :else\0")
});
static e_multiple_finally: GlobalCell<[::core::ffi::c_char; 24]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"E607: Multiple :finally\0")
});
pub const THROW_ON_ERROR: ::core::ffi::c_int = true_0;
unsafe extern "C" fn discard_pending_return(mut p: *mut typval_T) {
    tv_free(p);
}
static cause_abort: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
#[no_mangle]
pub unsafe extern "C" fn aborting() -> bool {
    return did_emsg.get() != 0 && force_abort.get() as ::core::ffi::c_int != 0
        || got_int.get() as ::core::ffi::c_int != 0
        || did_throw.get() as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn update_force_abort() {
    if cause_abort.get() {
        force_abort.set(true_0 != 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn should_abort(mut retcode: ::core::ffi::c_int) -> bool {
    return retcode == FAIL && trylevel.get() != 0 as ::core::ffi::c_int && emsg_silent.get() == 0
        || aborting() as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn aborted_in_try() -> bool {
    return force_abort.get();
}
#[no_mangle]
pub unsafe extern "C" fn cause_errthrow(
    mut mesg: *const ::core::ffi::c_char,
    mut multiline: bool,
    mut concat: bool,
    mut severe: bool,
    mut ignore: *mut bool,
) -> bool {
    let mut elem: *mut msglist_T = ::core::ptr::null_mut::<msglist_T>();
    if suppress_errthrow.get() {
        return false_0 != 0;
    }
    if did_emsg.get() == 0 {
        cause_abort.set(force_abort.get());
        force_abort.set(false_0 != 0);
    }
    if (trylevel.get() == 0 as ::core::ffi::c_int && !cause_abort.get() || emsg_silent.get() != 0)
        && !did_throw.get()
    {
        return false_0 != 0;
    }
    if mesg
        == gettext(&raw const e_interr as *const ::core::ffi::c_char) as *const ::core::ffi::c_char
    {
        *ignore = true_0 != 0;
        return true_0 != 0;
    }
    cause_abort.set(true_0 != 0);
    if did_throw.get() {
        if (*current_exception.get()).type_0 as ::core::ffi::c_uint
            == ET_INTERRUPT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            got_int.set(false_0 != 0);
        }
        discard_current_exception();
    }
    if !(*msg_list.ptr()).is_null() {
        let mut plist: *mut *mut msglist_T = msg_list.get();
        while !(*plist).is_null() {
            if (**plist).next.is_null() && concat as ::core::ffi::c_int != 0 {
                (**plist).msg = xrealloc(
                    (**plist).msg as *mut ::core::ffi::c_void,
                    strlen((**plist).msg)
                        .wrapping_add(strlen(mesg))
                        .wrapping_add(1 as size_t),
                ) as *mut ::core::ffi::c_char;
                (**plist).throw_msg = strcat((**plist).msg, mesg);
                return true_0 != 0;
            }
            plist = &raw mut (**plist).next;
        }
        elem = xmalloc(::core::mem::size_of::<msglist_T>()) as *mut msglist_T;
        (*elem).msg = xstrdup(mesg);
        (*elem).multiline = multiline;
        (*elem).next = ::core::ptr::null_mut::<msglist_T>();
        (*elem).throw_msg = ::core::ptr::null_mut::<::core::ffi::c_char>();
        *plist = elem;
        if plist == msg_list.get() || severe as ::core::ffi::c_int != 0 {
            let mut tmsg: *mut ::core::ffi::c_char = (*elem).msg;
            if strncmp(
                tmsg,
                b"Vim E\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
                && ascii_isdigit(
                    *tmsg.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                && ascii_isdigit(
                    *tmsg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                && ascii_isdigit(
                    *tmsg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                && *tmsg.offset(8 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int
                && *tmsg.offset(9 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ' ' as ::core::ffi::c_int
            {
                (**msg_list.get()).throw_msg = tmsg.offset(4 as ::core::ffi::c_int as isize);
            } else {
                (**msg_list.get()).throw_msg = tmsg;
            }
        }
        (*elem).sfile = estack_sfile(ESTACK_NONE);
        (*elem).slnum = (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
    }
    return true_0 != 0;
}
unsafe extern "C" fn free_msglist(mut l: *mut msglist_T) {
    let mut messages: *mut msglist_T = l;
    while !messages.is_null() {
        let mut next: *mut msglist_T = (*messages).next;
        xfree((*messages).msg as *mut ::core::ffi::c_void);
        xfree((*messages).sfile as *mut ::core::ffi::c_void);
        xfree(messages as *mut ::core::ffi::c_void);
        messages = next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn free_global_msglist() {
    free_msglist(*msg_list.get());
    *msg_list.get() = ::core::ptr::null_mut::<msglist_T>();
}
#[no_mangle]
pub unsafe extern "C" fn do_errthrow(
    mut cstack: *mut cstack_T,
    mut cmdname: *mut ::core::ffi::c_char,
) {
    if cause_abort.get() {
        cause_abort.set(false_0 != 0);
        force_abort.set(true_0 != 0);
    }
    if (*msg_list.ptr()).is_null() || (*msg_list.get()).is_null() {
        return;
    }
    if throw_exception(
        *msg_list.get() as *mut ::core::ffi::c_void,
        ET_ERROR,
        cmdname,
    ) == FAIL
    {
        free_msglist(*msg_list.get());
    } else if !cstack.is_null() {
        do_throw(cstack);
    } else {
        need_rethrow.set(true_0 != 0);
    }
    *msg_list.get() = ::core::ptr::null_mut::<msglist_T>();
}
#[no_mangle]
pub unsafe extern "C" fn do_intthrow(mut cstack: *mut cstack_T) -> bool {
    if !got_int.get() || trylevel.get() == 0 as ::core::ffi::c_int && !did_throw.get() {
        return false_0 != 0;
    }
    if did_throw.get() {
        if (*current_exception.get()).type_0 as ::core::ffi::c_uint
            == ET_INTERRUPT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return false_0 != 0;
        }
        discard_current_exception();
    }
    if throw_exception(
        b"Vim:Interrupt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_void,
        ET_INTERRUPT,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    ) != FAIL
    {
        do_throw(cstack);
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn get_exception_string(
    mut value: *mut ::core::ffi::c_void,
    mut type_0: except_type_T,
    mut cmdname: *mut ::core::ffi::c_char,
    mut should_free: *mut bool,
) -> *mut ::core::ffi::c_char {
    let mut ret: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if type_0 as ::core::ffi::c_uint == ET_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint {
        let mut val: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        *should_free = true_0 != 0;
        let mut mesg: *mut ::core::ffi::c_char = (*(value as *mut msglist_T)).throw_msg;
        if !cmdname.is_null() && *cmdname as ::core::ffi::c_int != NUL {
            let mut cmdlen: size_t = strlen(cmdname);
            ret = xstrnsave(
                b"Vim(\0".as_ptr() as *const ::core::ffi::c_char,
                (4 as size_t)
                    .wrapping_add(cmdlen)
                    .wrapping_add(2 as size_t)
                    .wrapping_add(strlen(mesg)),
            );
            strcpy(ret.offset(4 as ::core::ffi::c_int as isize), cmdname);
            strcpy(
                ret.offset((4 as size_t).wrapping_add(cmdlen) as isize),
                b"):\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            val = ret
                .offset(4 as ::core::ffi::c_int as isize)
                .offset(cmdlen as isize)
                .offset(2 as ::core::ffi::c_int as isize);
        } else {
            ret = xstrnsave(
                b"Vim:\0".as_ptr() as *const ::core::ffi::c_char,
                (4 as size_t).wrapping_add(strlen(mesg)),
            );
            val = ret.offset(4 as ::core::ffi::c_int as isize);
        }
        let mut p: *mut ::core::ffi::c_char = mesg;
        loop {
            if *p as ::core::ffi::c_int == NUL
                || *p as ::core::ffi::c_int == 'E' as ::core::ffi::c_int
                    && ascii_isdigit(
                        *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                    && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                        || ascii_isdigit(
                            *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                            && (*p.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == ':' as ::core::ffi::c_int
                                || ascii_isdigit(*p.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                                    && *p.offset(4 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == ':' as ::core::ffi::c_int))
            {
                if *p as ::core::ffi::c_int == NUL || p == mesg {
                    strcat(val, mesg);
                    break;
                } else if !(*mesg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '"' as ::core::ffi::c_int
                    || p.offset(-(2 as ::core::ffi::c_int as isize))
                        < mesg.offset(1 as ::core::ffi::c_int as isize)
                    || *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '"' as ::core::ffi::c_int
                    || *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != ' ' as ::core::ffi::c_int)
                {
                    strcat(val, p);
                    *p.offset(-2 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
                    snprintf(
                        val.offset(strlen(p) as isize),
                        strlen(b" (%s)\0".as_ptr() as *const ::core::ffi::c_char),
                        b" (%s)\0".as_ptr() as *const ::core::ffi::c_char,
                        mesg.offset(1 as ::core::ffi::c_int as isize),
                    );
                    *p.offset(-2 as ::core::ffi::c_int as isize) = '"' as ::core::ffi::c_char;
                    break;
                }
            }
            p = p.offset(1);
        }
    } else {
        *should_free = false_0 != 0;
        ret = value as *mut ::core::ffi::c_char;
    }
    return ret;
}
unsafe extern "C" fn throw_exception(
    mut value: *mut ::core::ffi::c_void,
    mut type_0: except_type_T,
    mut cmdname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut excp: *mut except_T = ::core::ptr::null_mut::<except_T>();
    let mut should_free: bool = false;
    '_fail: {
        if type_0 as ::core::ffi::c_uint == ET_USER as ::core::ffi::c_int as ::core::ffi::c_uint {
            if strncmp(
                value as *const ::core::ffi::c_char,
                b"Vim\0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
                && (*(value as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == NUL
                    || *(value as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                    || *(value as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == '(' as ::core::ffi::c_int)
            {
                emsg(gettext(
                    b"E608: Cannot :throw exceptions with 'Vim' prefix\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                break '_fail;
            }
        }
        excp = xmalloc(::core::mem::size_of::<except_T>()) as *mut except_T;
        if type_0 as ::core::ffi::c_uint == ET_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint {
            (*excp).messages = value as *mut msglist_T;
        }
        should_free = false;
        (*excp).value = get_exception_string(value, type_0, cmdname, &raw mut should_free);
        if (*excp).value.is_null() && should_free as ::core::ffi::c_int != 0 {
            xfree(excp as *mut ::core::ffi::c_void);
            suppress_errthrow.set(true_0 != 0);
            emsg(gettext(&raw const e_outofmem as *const ::core::ffi::c_char));
        } else {
            (*excp).type_0 = type_0;
            if type_0 as ::core::ffi::c_uint
                == ET_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
                && !(*(value as *mut msglist_T)).sfile.is_null()
            {
                let mut entry: *mut msglist_T = value as *mut msglist_T;
                (*excp).throw_name = (*entry).sfile;
                (*entry).sfile = ::core::ptr::null_mut::<::core::ffi::c_char>();
                (*excp).throw_lnum = (*entry).slnum;
            } else {
                (*excp).throw_name = estack_sfile(ESTACK_NONE);
                if (*excp).throw_name.is_null() {
                    (*excp).throw_name = xstrdup(b"\0".as_ptr() as *const ::core::ffi::c_char);
                }
                (*excp).throw_lnum = (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum;
            }
            (*excp).stacktrace = stacktrace_create();
            tv_list_ref((*excp).stacktrace);
            if p_verbose.get() >= 13 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int
            {
                let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
                if debug_break_level.get() > 0 as ::core::ffi::c_int {
                    msg_silent.set(false_0);
                } else {
                    verbose_enter();
                }
                (*no_wait_return.ptr()) += 1;
                if debug_break_level.get() > 0 as ::core::ffi::c_int
                    || *p_vfile.get() as ::core::ffi::c_int == NUL
                {
                    msg_scroll.set(true_0);
                }
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"Exception thrown: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    (*excp).value,
                );
                msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
                if debug_break_level.get() > 0 as ::core::ffi::c_int
                    || *p_vfile.get() as ::core::ffi::c_int == NUL
                {
                    cmdline_row.set(msg_row.get());
                }
                (*no_wait_return.ptr()) -= 1;
                if debug_break_level.get() > 0 as ::core::ffi::c_int {
                    msg_silent.set(save_msg_silent);
                } else {
                    verbose_leave();
                }
            }
            current_exception.set(excp);
            return OK;
        }
    }
    current_exception.set(::core::ptr::null_mut::<except_T>());
    return FAIL;
}
unsafe extern "C" fn discard_exception(mut excp: *mut except_T, mut was_finished: bool) {
    if current_exception.get() == excp {
        current_exception.set(::core::ptr::null_mut::<except_T>());
    }
    if excp.is_null() {
        internal_error(b"discard_exception()\0".as_ptr() as *const ::core::ffi::c_char);
        return;
    }
    if p_verbose.get() >= 13 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int {
        let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
        let mut saved_IObuff: *mut ::core::ffi::c_char =
            xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
        if debug_break_level.get() > 0 as ::core::ffi::c_int {
            msg_silent.set(false_0);
        } else {
            verbose_enter();
        }
        (*no_wait_return.ptr()) += 1;
        if debug_break_level.get() > 0 as ::core::ffi::c_int
            || *p_vfile.get() as ::core::ffi::c_int == NUL
        {
            msg_scroll.set(true_0);
        }
        smsg(
            0 as ::core::ffi::c_int,
            if was_finished as ::core::ffi::c_int != 0 {
                gettext(b"Exception finished: %s\0".as_ptr() as *const ::core::ffi::c_char)
            } else {
                gettext(b"Exception discarded: %s\0".as_ptr() as *const ::core::ffi::c_char)
            },
            (*excp).value,
        );
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        if debug_break_level.get() > 0 as ::core::ffi::c_int
            || *p_vfile.get() as ::core::ffi::c_int == NUL
        {
            cmdline_row.set(msg_row.get());
        }
        (*no_wait_return.ptr()) -= 1;
        if debug_break_level.get() > 0 as ::core::ffi::c_int {
            msg_silent.set(save_msg_silent);
        } else {
            verbose_leave();
        }
        xstrlcpy(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            saved_IObuff,
            IOSIZE as size_t,
        );
        xfree(saved_IObuff as *mut ::core::ffi::c_void);
    }
    if (*excp).type_0 as ::core::ffi::c_uint
        != ET_INTERRUPT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        xfree((*excp).value as *mut ::core::ffi::c_void);
    }
    if (*excp).type_0 as ::core::ffi::c_uint
        == ET_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        free_msglist((*excp).messages);
    }
    xfree((*excp).throw_name as *mut ::core::ffi::c_void);
    tv_list_unref((*excp).stacktrace);
    xfree(excp as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn discard_current_exception() {
    if !(*current_exception.ptr()).is_null() {
        discard_exception(current_exception.get(), false_0 != 0);
    }
    did_throw.set(false_0 != 0);
    need_rethrow.set(false_0 != 0);
}
unsafe extern "C" fn catch_exception(mut excp: *mut except_T) {
    (*excp).caught = caught_stack.get();
    caught_stack.set(excp);
    set_vim_var_string(VV_EXCEPTION, (*excp).value, -1 as ptrdiff_t);
    set_vim_var_list(VV_STACKTRACE, (*excp).stacktrace);
    if *(*excp).throw_name as ::core::ffi::c_int != NUL {
        let mut IObufflen: size_t = 0;
        if (*excp).throw_lnum != 0 as linenr_T {
            IObufflen = vim_snprintf_safelen(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                gettext(b"%s, line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                (*excp).throw_name,
                (*excp).throw_lnum as int64_t,
            );
        } else {
            IObufflen = vim_snprintf_safelen(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*excp).throw_name,
            );
        }
        set_vim_var_string(
            VV_THROWPOINT,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IObufflen as ptrdiff_t,
        );
    } else {
        set_vim_var_string(
            VV_THROWPOINT,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
    }
    if p_verbose.get() >= 13 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int {
        let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
        if debug_break_level.get() > 0 as ::core::ffi::c_int {
            msg_silent.set(false_0);
        } else {
            verbose_enter();
        }
        (*no_wait_return.ptr()) += 1;
        if debug_break_level.get() > 0 as ::core::ffi::c_int
            || *p_vfile.get() as ::core::ffi::c_int == NUL
        {
            msg_scroll.set(true_0);
        }
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"Exception caught: %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*excp).value,
        );
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        if debug_break_level.get() > 0 as ::core::ffi::c_int
            || *p_vfile.get() as ::core::ffi::c_int == NUL
        {
            cmdline_row.set(msg_row.get());
        }
        (*no_wait_return.ptr()) -= 1;
        if debug_break_level.get() > 0 as ::core::ffi::c_int {
            msg_silent.set(save_msg_silent);
        } else {
            verbose_leave();
        }
    }
}
unsafe extern "C" fn finish_exception(mut excp: *mut except_T) {
    if excp != caught_stack.get() {
        internal_error(b"finish_exception()\0".as_ptr() as *const ::core::ffi::c_char);
    }
    caught_stack.set((*caught_stack.get()).caught);
    if !(*caught_stack.ptr()).is_null() {
        set_vim_var_string(VV_EXCEPTION, (*caught_stack.get()).value, -1 as ptrdiff_t);
        set_vim_var_list(VV_STACKTRACE, (*caught_stack.get()).stacktrace);
        if *(*caught_stack.get()).throw_name as ::core::ffi::c_int != NUL {
            let mut IObufflen: size_t = 0;
            if (*caught_stack.get()).throw_lnum != 0 as linenr_T {
                IObufflen = vim_snprintf_safelen(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    gettext(b"%s, line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                    (*caught_stack.get()).throw_name,
                    (*caught_stack.get()).throw_lnum as int64_t,
                );
            } else {
                IObufflen = vim_snprintf_safelen(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    (*caught_stack.get()).throw_name,
                );
            }
            set_vim_var_string(
                VV_THROWPOINT,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IObufflen as ptrdiff_t,
            );
        } else {
            set_vim_var_string(
                VV_THROWPOINT,
                ::core::ptr::null::<::core::ffi::c_char>(),
                -1 as ptrdiff_t,
            );
        }
    } else {
        set_vim_var_string(
            VV_EXCEPTION,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_string(
            VV_THROWPOINT,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        set_vim_var_list(VV_STACKTRACE, ::core::ptr::null_mut::<list_T>());
    }
    discard_exception(excp, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn exception_state_save(mut estate: *mut exception_state_T) {
    (*estate).estate_current_exception = current_exception.get();
    (*estate).estate_did_throw = did_throw.get();
    (*estate).estate_need_rethrow = need_rethrow.get();
    (*estate).estate_trylevel = trylevel.get();
    (*estate).estate_did_emsg = did_emsg.get();
}
#[no_mangle]
pub unsafe extern "C" fn exception_state_restore(mut estate: *mut exception_state_T) {
    if did_throw.get() {
        handle_did_throw();
    }
    current_exception.set((*estate).estate_current_exception);
    did_throw.set((*estate).estate_did_throw);
    need_rethrow.set((*estate).estate_need_rethrow);
    trylevel.set((*estate).estate_trylevel);
    did_emsg.set((*estate).estate_did_emsg);
}
#[no_mangle]
pub unsafe extern "C" fn exception_state_clear() {
    current_exception.set(::core::ptr::null_mut::<except_T>());
    did_throw.set(false_0 != 0);
    need_rethrow.set(false_0 != 0);
    trylevel.set(0 as ::core::ffi::c_int);
    did_emsg.set(0 as ::core::ffi::c_int);
}
pub const RP_MAKE: ::core::ffi::c_int = 0;
pub const RP_RESUME: ::core::ffi::c_int = 1;
pub const RP_DISCARD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
unsafe extern "C" fn report_pending(
    mut action: ::core::ffi::c_int,
    mut pending: ::core::ffi::c_int,
    mut value: *mut ::core::ffi::c_void,
) {
    let mut mesg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_c2rust_label: {
        if !value.is_null() || pending & CSTP_THROW as ::core::ffi::c_int == 0 {
        } else {
            __assert_fail(
                b"value || !(pending & CSTP_THROW)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ex_eval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                723 as ::core::ffi::c_uint,
                b"void report_pending(int, int, void *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    match action {
        RP_MAKE => {
            mesg = gettext(b"%s made pending\0".as_ptr() as *const ::core::ffi::c_char);
        }
        RP_RESUME => {
            mesg = gettext(b"%s resumed\0".as_ptr() as *const ::core::ffi::c_char);
        }
        _ => {
            mesg = gettext(b"%s discarded\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    match pending {
        0 => return,
        16 => {
            s = b":continue\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        8 => {
            s = b":break\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        32 => {
            s = b":finish\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        24 => {
            s = get_return_cmd(value);
        }
        _ => {
            if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
                vim_snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    mesg,
                    gettext(b"Exception\0".as_ptr() as *const ::core::ffi::c_char),
                );
                mesg = concat_str(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    b": %s\0".as_ptr() as *const ::core::ffi::c_char,
                );
                s = (*(value as *mut except_T)).value;
            } else if pending & CSTP_ERROR as ::core::ffi::c_int != 0
                && pending & CSTP_INTERRUPT as ::core::ffi::c_int != 0
            {
                s = gettext(b"Error and interrupt\0".as_ptr() as *const ::core::ffi::c_char);
            } else if pending & CSTP_ERROR as ::core::ffi::c_int != 0 {
                s = gettext(b"Error\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                s = gettext(b"Interrupt\0".as_ptr() as *const ::core::ffi::c_char);
            }
        }
    }
    let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
    if debug_break_level.get() > 0 as ::core::ffi::c_int {
        msg_silent.set(false_0);
    }
    (*no_wait_return.ptr()) += 1;
    msg_scroll.set(true_0);
    smsg(0 as ::core::ffi::c_int, mesg, s);
    msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    cmdline_row.set(msg_row.get());
    (*no_wait_return.ptr()) -= 1;
    if debug_break_level.get() > 0 as ::core::ffi::c_int {
        msg_silent.set(save_msg_silent);
    }
    if pending == CSTP_RETURN as ::core::ffi::c_int {
        xfree(s as *mut ::core::ffi::c_void);
    } else if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
        xfree(mesg as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn report_make_pending(
    mut pending: ::core::ffi::c_int,
    mut value: *mut ::core::ffi::c_void,
) {
    if p_verbose.get() >= 14 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int {
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_enter();
        }
        report_pending(RP_MAKE, pending, value);
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_leave();
        }
    }
}
unsafe extern "C" fn report_resume_pending(
    mut pending: ::core::ffi::c_int,
    mut value: *mut ::core::ffi::c_void,
) {
    if p_verbose.get() >= 14 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int {
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_enter();
        }
        report_pending(RP_RESUME, pending, value);
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_leave();
        }
    }
}
unsafe extern "C" fn report_discard_pending(
    mut pending: ::core::ffi::c_int,
    mut value: *mut ::core::ffi::c_void,
) {
    if p_verbose.get() >= 14 as OptInt || debug_break_level.get() > 0 as ::core::ffi::c_int {
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_enter();
        }
        report_pending(RP_DISCARD, pending, value);
        if debug_break_level.get() <= 0 as ::core::ffi::c_int {
            verbose_leave();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_eval(mut eap: *mut exarg_T) {
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
    if eval0((*eap).arg, &raw mut tv, eap, &raw mut evalarg) == OK {
        tv_clear(&raw mut tv);
    }
    clear_evalarg(&raw mut evalarg, eap);
}
#[no_mangle]
pub unsafe extern "C" fn ex_if(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    if (*cstack).cs_idx == CSTACK_LEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        (*eap).errmsg =
            gettext(b"E579: :if nesting too deep\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        (*cstack).cs_idx += 1;
        (*cstack).cs_flags[(*cstack).cs_idx as usize] = 0 as ::core::ffi::c_int;
        let mut skip: bool = did_emsg.get() != 0
            || got_int.get() as ::core::ffi::c_int != 0
            || did_throw.get() as ::core::ffi::c_int != 0
            || (*cstack).cs_idx > 0 as ::core::ffi::c_int
                && (*cstack).cs_flags[((*cstack).cs_idx - 1 as ::core::ffi::c_int) as usize]
                    & CSF_ACTIVE as ::core::ffi::c_int
                    == 0;
        let mut error: bool = false;
        let mut result: bool = eval_to_bool((*eap).arg, &raw mut error, eap, skip, false_0 != 0);
        if !skip && !error {
            if result {
                (*cstack).cs_flags[(*cstack).cs_idx as usize] =
                    CSF_ACTIVE as ::core::ffi::c_int | CSF_TRUE as ::core::ffi::c_int;
            }
        } else {
            (*cstack).cs_flags[(*cstack).cs_idx as usize] = CSF_TRUE as ::core::ffi::c_int;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_endif(mut eap: *mut exarg_T) {
    did_endif.set(true_0 != 0);
    if (*(*eap).cstack).cs_idx < 0 as ::core::ffi::c_int
        || (*(*eap).cstack).cs_flags[(*(*eap).cstack).cs_idx as usize]
            & (CSF_WHILE as ::core::ffi::c_int
                | CSF_FOR as ::core::ffi::c_int
                | CSF_TRY as ::core::ffi::c_int)
            != 0
    {
        (*eap).errmsg =
            gettext(b"E580: :endif without :if\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        if (*(*eap).cstack).cs_flags[(*(*eap).cstack).cs_idx as usize]
            & CSF_TRUE as ::core::ffi::c_int
            == 0
            && dbg_check_skipped(eap) as ::core::ffi::c_int != 0
        {
            do_intthrow((*eap).cstack);
        }
        (*(*eap).cstack).cs_idx -= 1;
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_else(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    let mut skip: bool = did_emsg.get() != 0
        || got_int.get() as ::core::ffi::c_int != 0
        || did_throw.get() as ::core::ffi::c_int != 0
        || (*cstack).cs_idx > 0 as ::core::ffi::c_int
            && (*cstack).cs_flags[((*cstack).cs_idx - 1 as ::core::ffi::c_int) as usize]
                & CSF_ACTIVE as ::core::ffi::c_int
                == 0;
    if (*cstack).cs_idx < 0 as ::core::ffi::c_int
        || (*cstack).cs_flags[(*cstack).cs_idx as usize]
            & (CSF_WHILE as ::core::ffi::c_int
                | CSF_FOR as ::core::ffi::c_int
                | CSF_TRY as ::core::ffi::c_int)
            != 0
    {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_else as ::core::ffi::c_int {
            (*eap).errmsg =
                gettext(b"E581: :else without :if\0".as_ptr() as *const ::core::ffi::c_char);
            return;
        }
        (*eap).errmsg =
            gettext(b"E582: :elseif without :if\0".as_ptr() as *const ::core::ffi::c_char);
        skip = true_0 != 0;
    } else if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_ELSE as ::core::ffi::c_int != 0 {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_else as ::core::ffi::c_int {
            (*eap).errmsg =
                gettext((e_multiple_else.ptr() as *const _) as *const ::core::ffi::c_char);
            return;
        }
        (*eap).errmsg =
            gettext(b"E584: :elseif after :else\0".as_ptr() as *const ::core::ffi::c_char);
        skip = true_0 != 0;
    }
    if skip as ::core::ffi::c_int != 0
        || (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRUE as ::core::ffi::c_int != 0
    {
        if (*eap).errmsg.is_null() {
            (*cstack).cs_flags[(*cstack).cs_idx as usize] = CSF_TRUE as ::core::ffi::c_int;
        }
        skip = true_0 != 0;
    } else {
        (*cstack).cs_flags[(*cstack).cs_idx as usize] = CSF_ACTIVE as ::core::ffi::c_int;
    }
    if !skip
        && dbg_check_skipped(eap) as ::core::ffi::c_int != 0
        && got_int.get() as ::core::ffi::c_int != 0
    {
        do_intthrow(cstack);
        skip = true_0 != 0;
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_elseif as ::core::ffi::c_int {
        let mut result: bool = false_0 != 0;
        let mut error: bool = false;
        if skip as ::core::ffi::c_int != 0
            && *(*eap).arg as ::core::ffi::c_int != '"' as ::core::ffi::c_int
            && ends_excmd(*(*eap).arg as ::core::ffi::c_int) != 0
        {
            semsg(
                gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        } else {
            result = eval_to_bool((*eap).arg, &raw mut error, eap, skip, false_0 != 0);
        }
        if !skip && !error {
            if result {
                (*cstack).cs_flags[(*cstack).cs_idx as usize] =
                    CSF_ACTIVE as ::core::ffi::c_int | CSF_TRUE as ::core::ffi::c_int;
            } else {
                (*cstack).cs_flags[(*cstack).cs_idx as usize] = 0 as ::core::ffi::c_int;
            }
        } else if (*eap).errmsg.is_null() {
            (*cstack).cs_flags[(*cstack).cs_idx as usize] = CSF_TRUE as ::core::ffi::c_int;
        }
    } else {
        (*cstack).cs_flags[(*cstack).cs_idx as usize] |= CSF_ELSE as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_while(mut eap: *mut exarg_T) {
    let mut error: bool = false;
    let cstack: *mut cstack_T = (*eap).cstack;
    if (*cstack).cs_idx == CSTACK_LEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        (*eap).errmsg =
            gettext(b"E585: :while/:for nesting too deep\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        let mut result: bool = false;
        if (*cstack).cs_lflags & CSL_HAD_LOOP as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*cstack).cs_idx += 1;
            (*cstack).cs_looplevel += 1;
            (*cstack).cs_line[(*cstack).cs_idx as usize] = -1 as ::core::ffi::c_int;
        }
        (*cstack).cs_flags[(*cstack).cs_idx as usize] =
            if (*eap).cmdidx as ::core::ffi::c_int == CMD_while as ::core::ffi::c_int {
                CSF_WHILE as ::core::ffi::c_int
            } else {
                CSF_FOR as ::core::ffi::c_int
            };
        let mut skip: ::core::ffi::c_int = (did_emsg.get() != 0
            || got_int.get() as ::core::ffi::c_int != 0
            || did_throw.get() as ::core::ffi::c_int != 0
            || (*cstack).cs_idx > 0 as ::core::ffi::c_int
                && (*cstack).cs_flags[((*cstack).cs_idx - 1 as ::core::ffi::c_int) as usize]
                    & CSF_ACTIVE as ::core::ffi::c_int
                    == 0) as ::core::ffi::c_int;
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_while as ::core::ffi::c_int {
            result = eval_to_bool((*eap).arg, &raw mut error, eap, skip != 0, false_0 != 0);
        } else {
            let mut evalarg: evalarg_T = evalarg_T {
                eval_flags: 0,
                eval_getline: None,
                eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            fill_evalarg_from_eap(&raw mut evalarg, eap, skip != 0);
            let mut fi: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
            if (*cstack).cs_lflags & CSL_HAD_LOOP as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                fi = (*cstack).cs_forinfo[(*cstack).cs_idx as usize];
                error = false_0 != 0;
            } else {
                fi = eval_for_line((*eap).arg, &raw mut error, eap, &raw mut evalarg);
                (*cstack).cs_forinfo[(*cstack).cs_idx as usize] = fi;
            }
            if !error && !fi.is_null() && skip == 0 {
                result = next_for_item(fi, (*eap).arg);
            } else {
                result = false_0 != 0;
            }
            if !result {
                free_for_info(fi);
                (*cstack).cs_forinfo[(*cstack).cs_idx as usize] = NULL;
            }
            clear_evalarg(&raw mut evalarg, eap);
        }
        if skip == 0 && !error && result as ::core::ffi::c_int != 0 {
            (*cstack).cs_flags[(*cstack).cs_idx as usize] |=
                CSF_ACTIVE as ::core::ffi::c_int | CSF_TRUE as ::core::ffi::c_int;
            (*cstack).cs_lflags ^= CSL_HAD_LOOP as ::core::ffi::c_int;
        } else {
            (*cstack).cs_lflags &= !(CSL_HAD_LOOP as ::core::ffi::c_int);
            if skip == 0 && !error {
                (*cstack).cs_flags[(*cstack).cs_idx as usize] |= CSF_TRUE as ::core::ffi::c_int;
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_continue(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    if (*cstack).cs_looplevel <= 0 as ::core::ffi::c_int
        || (*cstack).cs_idx < 0 as ::core::ffi::c_int
    {
        (*eap).errmsg = gettext(
            b"E586: :continue without :while or :for\0".as_ptr() as *const ::core::ffi::c_char
        );
    } else {
        let mut idx: ::core::ffi::c_int = cleanup_conditionals(
            cstack,
            CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
            false_0,
        );
        '_c2rust_label: {
            if idx >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"idx >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/ex_eval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1069 as ::core::ffi::c_uint,
                    b"void ex_continue(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if (*cstack).cs_flags[idx as usize]
            & (CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int)
            != 0
        {
            rewind_conditionals(
                cstack,
                idx,
                CSF_TRY as ::core::ffi::c_int,
                &raw mut (*cstack).cs_trylevel,
            );
            (*cstack).cs_lflags |= CSL_HAD_CONT as ::core::ffi::c_int;
        } else {
            (*cstack).cs_pending[idx as usize] =
                CSTP_CONTINUE as ::core::ffi::c_int as ::core::ffi::c_char;
            report_make_pending(CSTP_CONTINUE as ::core::ffi::c_int, NULL);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_break(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    if (*cstack).cs_looplevel <= 0 as ::core::ffi::c_int
        || (*cstack).cs_idx < 0 as ::core::ffi::c_int
    {
        (*eap).errmsg = gettext(
            b"E587: :break without :while or :for\0".as_ptr() as *const ::core::ffi::c_char
        );
    } else {
        let mut idx: ::core::ffi::c_int = cleanup_conditionals(
            cstack,
            CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
            true_0,
        );
        if idx >= 0 as ::core::ffi::c_int
            && (*cstack).cs_flags[idx as usize]
                & (CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int)
                == 0
        {
            (*cstack).cs_pending[idx as usize] =
                CSTP_BREAK as ::core::ffi::c_int as ::core::ffi::c_char;
            report_make_pending(CSTP_BREAK as ::core::ffi::c_int, NULL);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_endwhile(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    let mut err: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut csf: ::core::ffi::c_int = 0;
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_endwhile as ::core::ffi::c_int {
        err = &raw const e_while as *const ::core::ffi::c_char;
        csf = CSF_WHILE as ::core::ffi::c_int;
    } else {
        err = &raw const e_for as *const ::core::ffi::c_char;
        csf = CSF_FOR as ::core::ffi::c_int;
    }
    if (*cstack).cs_looplevel <= 0 as ::core::ffi::c_int
        || (*cstack).cs_idx < 0 as ::core::ffi::c_int
    {
        (*eap).errmsg = gettext(err);
    } else {
        let mut fl: ::core::ffi::c_int = (*cstack).cs_flags[(*cstack).cs_idx as usize];
        if fl & csf == 0 {
            if fl & CSF_WHILE as ::core::ffi::c_int != 0 {
                (*eap).errmsg = gettext(
                    b"E732: Using :endfor with :while\0".as_ptr() as *const ::core::ffi::c_char
                );
            } else if fl & CSF_FOR as ::core::ffi::c_int != 0 {
                (*eap).errmsg = gettext(
                    b"E733: Using :endwhile with :for\0".as_ptr() as *const ::core::ffi::c_char
                );
            }
        }
        if fl & (CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int) == 0 {
            if fl & CSF_TRY as ::core::ffi::c_int == 0 {
                (*eap).errmsg = gettext(&raw const e_endif as *const ::core::ffi::c_char);
            } else if fl & CSF_FINALLY as ::core::ffi::c_int != 0 {
                (*eap).errmsg = gettext(&raw const e_endtry as *const ::core::ffi::c_char);
            }
            let mut idx: ::core::ffi::c_int = 0;
            idx = (*cstack).cs_idx;
            while idx > 0 as ::core::ffi::c_int {
                fl = (*cstack).cs_flags[idx as usize];
                if fl & CSF_TRY as ::core::ffi::c_int != 0
                    && fl & CSF_FINALLY as ::core::ffi::c_int == 0
                {
                    (*eap).errmsg = gettext(err);
                    return;
                }
                if fl & csf != 0 {
                    break;
                }
                idx -= 1;
            }
            cleanup_conditionals(
                cstack,
                CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
                false_0,
            );
            rewind_conditionals(
                cstack,
                idx,
                CSF_TRY as ::core::ffi::c_int,
                &raw mut (*cstack).cs_trylevel,
            );
        } else if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRUE as ::core::ffi::c_int
            != 0
            && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_ACTIVE as ::core::ffi::c_int == 0
            && dbg_check_skipped(eap) as ::core::ffi::c_int != 0
        {
            do_intthrow(cstack);
        }
        (*cstack).cs_lflags |= CSL_HAD_ENDLOOP as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_throw(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *arg as ::core::ffi::c_int != NUL
        && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
        && *arg as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
    {
        value = eval_to_string_skip(arg, eap, (*eap).skip != 0);
    } else {
        emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
        value = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*eap).skip == 0 && !value.is_null() {
        if throw_exception(
            value as *mut ::core::ffi::c_void,
            ET_USER,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ) == FAIL
        {
            xfree(value as *mut ::core::ffi::c_void);
        } else {
            do_throw((*eap).cstack);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn do_throw(mut cstack: *mut cstack_T) {
    let mut inactivate_try: bool = false_0 != 0;
    let mut idx: ::core::ffi::c_int = cleanup_conditionals(
        cstack,
        0 as ::core::ffi::c_int,
        inactivate_try as ::core::ffi::c_int,
    );
    if idx >= 0 as ::core::ffi::c_int {
        if (*cstack).cs_flags[idx as usize] & CSF_CAUGHT as ::core::ffi::c_int == 0 {
            if (*cstack).cs_flags[idx as usize] & CSF_ACTIVE as ::core::ffi::c_int != 0 {
                (*cstack).cs_flags[idx as usize] |= CSF_THROWN as ::core::ffi::c_int;
            } else {
                (*cstack).cs_flags[idx as usize] &= !(CSF_THROWN as ::core::ffi::c_int);
            }
        }
        (*cstack).cs_flags[idx as usize] &= !(CSF_ACTIVE as ::core::ffi::c_int);
        (*cstack).cs_pend.csp_ex[idx as usize] =
            current_exception.get() as *mut ::core::ffi::c_void;
    }
    did_throw.set(true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn ex_try(mut eap: *mut exarg_T) {
    let cstack: *mut cstack_T = (*eap).cstack;
    if (*cstack).cs_idx == CSTACK_LEN as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        (*eap).errmsg =
            gettext(b"E601: :try nesting too deep\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        (*cstack).cs_idx += 1;
        (*cstack).cs_trylevel += 1;
        (*cstack).cs_flags[(*cstack).cs_idx as usize] = CSF_TRY as ::core::ffi::c_int;
        (*cstack).cs_pending[(*cstack).cs_idx as usize] =
            CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
        let mut skip: ::core::ffi::c_int = (did_emsg.get() != 0
            || got_int.get() as ::core::ffi::c_int != 0
            || did_throw.get() as ::core::ffi::c_int != 0
            || (*cstack).cs_idx > 0 as ::core::ffi::c_int
                && (*cstack).cs_flags[((*cstack).cs_idx - 1 as ::core::ffi::c_int) as usize]
                    & CSF_ACTIVE as ::core::ffi::c_int
                    == 0) as ::core::ffi::c_int;
        if skip == 0 {
            (*cstack).cs_flags[(*cstack).cs_idx as usize] |=
                CSF_ACTIVE as ::core::ffi::c_int | CSF_TRUE as ::core::ffi::c_int;
            if emsg_silent.get() != 0 {
                let mut elem: *mut eslist_T =
                    xmalloc(::core::mem::size_of::<eslist_T>()) as *mut eslist_T;
                (*elem).saved_emsg_silent = emsg_silent.get();
                (*elem).next = (*cstack).cs_emsg_silent_list;
                (*cstack).cs_emsg_silent_list = elem;
                (*cstack).cs_flags[(*cstack).cs_idx as usize] |= CSF_SILENT as ::core::ffi::c_int;
                emsg_silent.set(0 as ::core::ffi::c_int);
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_catch(mut eap: *mut exarg_T) {
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut give_up: bool = false_0 != 0;
    let mut skip: bool = false_0 != 0;
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_cpo: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let cstack: *mut cstack_T = (*eap).cstack;
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*cstack).cs_trylevel <= 0 as ::core::ffi::c_int
        || (*cstack).cs_idx < 0 as ::core::ffi::c_int
    {
        (*eap).errmsg =
            gettext(b"E603: :catch without :try\0".as_ptr() as *const ::core::ffi::c_char);
        give_up = true_0 != 0;
    } else {
        if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRY as ::core::ffi::c_int == 0 {
            (*eap).errmsg = get_end_emsg(cstack);
            skip = true_0 != 0;
        }
        idx = (*cstack).cs_idx;
        while idx > 0 as ::core::ffi::c_int {
            if (*cstack).cs_flags[idx as usize] & CSF_TRY as ::core::ffi::c_int != 0 {
                break;
            }
            idx -= 1;
        }
        if (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int != 0 {
            (*eap).errmsg =
                gettext(b"E604: :catch after :finally\0".as_ptr() as *const ::core::ffi::c_char);
            give_up = true_0 != 0;
        } else {
            rewind_conditionals(
                cstack,
                idx,
                CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
                &raw mut (*cstack).cs_looplevel,
            );
        }
    }
    if ends_excmd(*(*eap).arg as ::core::ffi::c_int) != 0 {
        pat = b".*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        end = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*eap).nextcmd = find_nextcmd((*eap).arg);
    } else {
        pat = (*eap).arg.offset(1 as ::core::ffi::c_int as isize);
        end = skip_regexp_err(pat, *(*eap).arg as ::core::ffi::c_int, true_0);
        if end.is_null() {
            give_up = true_0 != 0;
        }
    }
    if !give_up {
        let mut caught: bool = false_0 != 0;
        if !did_throw.get()
            || (*cstack).cs_flags[idx as usize] & CSF_TRUE as ::core::ffi::c_int == 0
        {
            skip = true_0 != 0;
        }
        if !skip
            && (*cstack).cs_flags[idx as usize] & CSF_THROWN as ::core::ffi::c_int != 0
            && (*cstack).cs_flags[idx as usize] & CSF_CAUGHT as ::core::ffi::c_int == 0
        {
            if !end.is_null()
                && *end as ::core::ffi::c_int != NUL
                && ends_excmd(
                    *skipwhite(end.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                ) == 0
            {
                semsg(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    end,
                );
                return;
            }
            if !dbg_check_skipped(eap) || !do_intthrow(cstack) {
                let mut save_char: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
                if !end.is_null() {
                    save_char = *end;
                    *end = NUL as ::core::ffi::c_char;
                }
                save_cpo = p_cpo.get();
                p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
                (*emsg_off.ptr()) += 1;
                regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
                (*emsg_off.ptr()) -= 1;
                regmatch.rm_ic = false_0 != 0;
                if !end.is_null() {
                    *end = save_char;
                }
                p_cpo.set(save_cpo);
                if regmatch.regprog.is_null() {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        pat,
                    );
                } else {
                    let mut prev_got_int: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
                    got_int.set(false_0 != 0);
                    caught = vim_regexec_nl(
                        &raw mut regmatch,
                        (*current_exception.get()).value,
                        0 as colnr_T,
                    );
                    got_int.set(got_int.get() as ::core::ffi::c_int | prev_got_int != 0);
                    vim_regfree(regmatch.regprog);
                }
            }
        }
        if caught {
            (*cstack).cs_flags[idx as usize] |=
                CSF_ACTIVE as ::core::ffi::c_int | CSF_CAUGHT as ::core::ffi::c_int;
            did_throw.set(false_0 != 0);
            got_int.set(did_throw.get());
            did_emsg.set(got_int.get() as ::core::ffi::c_int);
            catch_exception((*cstack).cs_pend.csp_ex[idx as usize] as *mut except_T);
            if (*cstack).cs_pend.csp_ex[(*cstack).cs_idx as usize]
                != current_exception.get() as *mut ::core::ffi::c_void
            {
                internal_error(b"ex_catch()\0".as_ptr() as *const ::core::ffi::c_char);
            }
        } else {
            cleanup_conditionals(cstack, CSF_TRY as ::core::ffi::c_int, true_0);
        }
    }
    if !end.is_null() {
        (*eap).nextcmd = find_nextcmd(end);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_finally(mut eap: *mut exarg_T) {
    let mut idx: ::core::ffi::c_int = 0;
    let mut pending: ::core::ffi::c_int = CSTP_NONE as ::core::ffi::c_int;
    let cstack: *mut cstack_T = (*eap).cstack;
    idx = (*cstack).cs_idx;
    while idx >= 0 as ::core::ffi::c_int {
        if (*cstack).cs_flags[idx as usize] & CSF_TRY as ::core::ffi::c_int != 0 {
            break;
        }
        idx -= 1;
    }
    if (*cstack).cs_trylevel <= 0 as ::core::ffi::c_int || idx < 0 as ::core::ffi::c_int {
        (*eap).errmsg =
            gettext(b"E606: :finally without :try\0".as_ptr() as *const ::core::ffi::c_char);
        return;
    }
    if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRY as ::core::ffi::c_int == 0 {
        (*eap).errmsg = get_end_emsg(cstack);
        pending = CSTP_ERROR as ::core::ffi::c_int;
    }
    if (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int != 0 {
        (*eap).errmsg =
            gettext((e_multiple_finally.ptr() as *const _) as *const ::core::ffi::c_char);
        return;
    }
    rewind_conditionals(
        cstack,
        idx,
        CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
        &raw mut (*cstack).cs_looplevel,
    );
    let mut skip: ::core::ffi::c_int = ((*cstack).cs_flags[(*cstack).cs_idx as usize]
        & CSF_TRUE as ::core::ffi::c_int
        == 0) as ::core::ffi::c_int;
    if skip == 0 {
        if dbg_check_skipped(eap) {
            do_intthrow(cstack);
        }
        cleanup_conditionals(cstack, CSF_TRY as ::core::ffi::c_int, false_0);
        if pending == CSTP_ERROR as ::core::ffi::c_int
            || did_emsg.get() != 0
            || got_int.get() as ::core::ffi::c_int != 0
            || did_throw.get() as ::core::ffi::c_int != 0
        {
            if (*cstack).cs_pending[(*cstack).cs_idx as usize] as ::core::ffi::c_int
                == CSTP_RETURN as ::core::ffi::c_int
            {
                report_discard_pending(
                    CSTP_RETURN as ::core::ffi::c_int,
                    (*cstack).cs_pend.csp_rv[(*cstack).cs_idx as usize],
                );
                discard_pending_return(
                    (*cstack).cs_pend.csp_rv[(*cstack).cs_idx as usize] as *mut typval_T,
                );
            }
            if pending == CSTP_ERROR as ::core::ffi::c_int && did_emsg.get() == 0 {
                pending |= if THROW_ON_ERROR != 0 {
                    CSTP_THROW as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            } else {
                pending |= if did_throw.get() as ::core::ffi::c_int != 0 {
                    CSTP_THROW as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            pending |= if did_emsg.get() != 0 {
                CSTP_ERROR as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
            pending |= if got_int.get() as ::core::ffi::c_int != 0 {
                CSTP_INTERRUPT as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
            '_c2rust_label: {
                if pending >= -127 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                    && pending <= 127 as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"pending >= CHAR_MIN && pending <= CHAR_MAX\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/ex_eval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1535 as ::core::ffi::c_uint,
                        b"void ex_finally(exarg_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            (*cstack).cs_pending[(*cstack).cs_idx as usize] = pending as ::core::ffi::c_char;
            if did_throw.get() as ::core::ffi::c_int != 0
                && (*cstack).cs_pend.csp_ex[(*cstack).cs_idx as usize]
                    != current_exception.get() as *mut ::core::ffi::c_void
            {
                internal_error(b"ex_finally()\0".as_ptr() as *const ::core::ffi::c_char);
            }
        }
        (*cstack).cs_lflags |= CSL_HAD_FINA as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_endtry(mut eap: *mut exarg_T) {
    let mut idx: ::core::ffi::c_int = 0;
    let mut rethrow: bool = false_0 != 0;
    let mut pending: ::core::ffi::c_char = CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
    let mut rettv: *mut ::core::ffi::c_void = NULL;
    let cstack: *mut cstack_T = (*eap).cstack;
    idx = (*cstack).cs_idx;
    while idx >= 0 as ::core::ffi::c_int {
        if (*cstack).cs_flags[idx as usize] & CSF_TRY as ::core::ffi::c_int != 0 {
            break;
        }
        idx -= 1;
    }
    if (*cstack).cs_trylevel <= 0 as ::core::ffi::c_int || idx < 0 as ::core::ffi::c_int {
        (*eap).errmsg =
            gettext(b"E602: :endtry without :try\0".as_ptr() as *const ::core::ffi::c_char);
        return;
    }
    let mut skip: bool = did_emsg.get() != 0
        || got_int.get() as ::core::ffi::c_int != 0
        || did_throw.get() as ::core::ffi::c_int != 0
        || (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRUE as ::core::ffi::c_int == 0;
    if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRY as ::core::ffi::c_int == 0 {
        (*eap).errmsg = get_end_emsg(cstack);
        rewind_conditionals(
            cstack,
            idx,
            CSF_WHILE as ::core::ffi::c_int | CSF_FOR as ::core::ffi::c_int,
            &raw mut (*cstack).cs_looplevel,
        );
        skip = true_0 != 0;
        if did_throw.get() {
            discard_current_exception();
        }
        did_emsg.set(false_0);
    } else {
        idx = (*cstack).cs_idx;
        if did_throw.get() as ::core::ffi::c_int != 0
            && (*cstack).cs_flags[idx as usize] & CSF_TRUE as ::core::ffi::c_int != 0
            && (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int == 0
        {
            rethrow = true_0 != 0;
        }
    }
    if (rethrow as ::core::ffi::c_int != 0
        || !skip
            && (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int == 0
            && (*cstack).cs_pending[idx as usize] == 0)
        && dbg_check_skipped(eap) as ::core::ffi::c_int != 0
    {
        if got_int.get() {
            skip = true_0 != 0;
            do_intthrow(cstack);
            rethrow = false_0 != 0;
            if did_throw.get() as ::core::ffi::c_int != 0
                && (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int == 0
            {
                rethrow = true_0 != 0;
            }
        }
    }
    if !skip {
        pending = (*cstack).cs_pending[idx as usize];
        (*cstack).cs_pending[idx as usize] = CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
        if pending as ::core::ffi::c_int == CSTP_RETURN as ::core::ffi::c_int {
            rettv = (*cstack).cs_pend.csp_rv[idx as usize];
        } else if pending as ::core::ffi::c_int & CSTP_THROW as ::core::ffi::c_int != 0 {
            current_exception.set((*cstack).cs_pend.csp_ex[idx as usize] as *mut except_T);
        }
    }
    cleanup_conditionals(
        cstack,
        CSF_TRY as ::core::ffi::c_int | CSF_SILENT as ::core::ffi::c_int,
        true_0,
    );
    if (*cstack).cs_idx >= 0 as ::core::ffi::c_int
        && (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_TRY as ::core::ffi::c_int != 0
    {
        (*cstack).cs_idx -= 1;
    }
    (*cstack).cs_trylevel -= 1;
    if !skip {
        report_resume_pending(
            pending as ::core::ffi::c_int,
            if pending as ::core::ffi::c_int == CSTP_RETURN as ::core::ffi::c_int {
                rettv
            } else if pending as ::core::ffi::c_int & CSTP_THROW as ::core::ffi::c_int != 0 {
                current_exception.get() as *mut ::core::ffi::c_void
            } else {
                NULL
            },
        );
        match pending as ::core::ffi::c_int {
            0 => {}
            16 => {
                ex_continue(eap);
            }
            8 => {
                ex_break(eap);
            }
            24 => {
                do_return(eap, false_0 != 0, false_0 != 0, rettv);
            }
            32 => {
                do_finish(eap, false_0 != 0);
            }
            _ => {
                if pending as ::core::ffi::c_int & CSTP_ERROR as ::core::ffi::c_int != 0 {
                    did_emsg.set(true_0);
                }
                if pending as ::core::ffi::c_int & CSTP_INTERRUPT as ::core::ffi::c_int != 0 {
                    got_int.set(true_0 != 0);
                }
                if pending as ::core::ffi::c_int & CSTP_THROW as ::core::ffi::c_int != 0 {
                    rethrow = true_0 != 0;
                }
            }
        }
    }
    if rethrow {
        do_throw(cstack);
    }
}
#[no_mangle]
pub unsafe extern "C" fn enter_cleanup(mut csp: *mut cleanup_T) {
    let mut pending: ::core::ffi::c_int = CSTP_NONE as ::core::ffi::c_int;
    if did_emsg.get() != 0
        || got_int.get() as ::core::ffi::c_int != 0
        || did_throw.get() as ::core::ffi::c_int != 0
        || need_rethrow.get() as ::core::ffi::c_int != 0
    {
        (*csp).pending = (if did_emsg.get() != 0 {
            CSTP_ERROR as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | (if got_int.get() as ::core::ffi::c_int != 0 {
            CSTP_INTERRUPT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | (if did_throw.get() as ::core::ffi::c_int != 0 {
            CSTP_THROW as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | (if need_rethrow.get() as ::core::ffi::c_int != 0 {
            CSTP_THROW as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
        if did_throw.get() as ::core::ffi::c_int != 0
            || need_rethrow.get() as ::core::ffi::c_int != 0
        {
            (*csp).exception = current_exception.get();
            current_exception.set(::core::ptr::null_mut::<except_T>());
        } else {
            (*csp).exception = ::core::ptr::null_mut::<except_T>();
            if did_emsg.get() != 0 {
                force_abort.set(
                    force_abort.get() as ::core::ffi::c_int
                        | cause_abort.get() as ::core::ffi::c_int
                        != 0,
                );
                cause_abort.set(false_0 != 0);
            }
        }
        need_rethrow.set(false_0 != 0);
        did_throw.set(need_rethrow.get());
        got_int.set(did_throw.get());
        did_emsg.set(got_int.get() as ::core::ffi::c_int);
        report_make_pending(pending, (*csp).exception as *mut ::core::ffi::c_void);
    } else {
        (*csp).pending = CSTP_NONE as ::core::ffi::c_int;
        (*csp).exception = ::core::ptr::null_mut::<except_T>();
    };
}
#[no_mangle]
pub unsafe extern "C" fn leave_cleanup(mut csp: *mut cleanup_T) {
    let mut pending: ::core::ffi::c_int = (*csp).pending;
    if pending == CSTP_NONE as ::core::ffi::c_int {
        return;
    }
    if aborting() as ::core::ffi::c_int != 0 || need_rethrow.get() as ::core::ffi::c_int != 0 {
        if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
            discard_exception((*csp).exception, false_0 != 0);
        } else {
            report_discard_pending(pending, NULL);
        }
        if !(*msg_list.ptr()).is_null() {
            free_global_msglist();
        }
    } else {
        if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
            current_exception.set((*csp).exception);
        } else if pending & CSTP_ERROR as ::core::ffi::c_int != 0 {
            cause_abort.set(force_abort.get());
            force_abort.set(false_0 != 0);
        }
        if pending & CSTP_ERROR as ::core::ffi::c_int != 0 {
            did_emsg.set(true_0);
        }
        if pending & CSTP_INTERRUPT as ::core::ffi::c_int != 0 {
            got_int.set(true_0 != 0);
        }
        if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
            need_rethrow.set(true_0 != 0);
        }
        report_resume_pending(
            pending,
            if pending & CSTP_THROW as ::core::ffi::c_int != 0 {
                current_exception.get() as *mut ::core::ffi::c_void
            } else {
                NULL
            },
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn cleanup_conditionals(
    mut cstack: *mut cstack_T,
    mut searched_cond: ::core::ffi::c_int,
    mut inclusive: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut idx: ::core::ffi::c_int = 0;
    let mut stop: bool = false_0 != 0;
    idx = (*cstack).cs_idx;
    while idx >= 0 as ::core::ffi::c_int {
        if (*cstack).cs_flags[idx as usize] & CSF_TRY as ::core::ffi::c_int != 0 {
            if did_emsg.get() != 0
                || got_int.get() as ::core::ffi::c_int != 0
                || (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int != 0
            {
                match (*cstack).cs_pending[idx as usize] as ::core::ffi::c_int {
                    0 => {}
                    16 | 8 | 32 => {
                        report_discard_pending(
                            (*cstack).cs_pending[idx as usize] as ::core::ffi::c_int,
                            NULL,
                        );
                        (*cstack).cs_pending[idx as usize] =
                            CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
                    }
                    24 => {
                        report_discard_pending(
                            CSTP_RETURN as ::core::ffi::c_int,
                            (*cstack).cs_pend.csp_rv[idx as usize],
                        );
                        discard_pending_return(
                            (*cstack).cs_pend.csp_rv[idx as usize] as *mut typval_T,
                        );
                        (*cstack).cs_pending[idx as usize] =
                            CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
                    }
                    _ => {
                        if (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int != 0
                        {
                            if (*cstack).cs_pending[idx as usize] as ::core::ffi::c_int
                                & CSTP_THROW as ::core::ffi::c_int
                                != 0
                                && !(*cstack).cs_pend.csp_ex[idx as usize].is_null()
                            {
                                discard_exception(
                                    (*cstack).cs_pend.csp_ex[idx as usize] as *mut except_T,
                                    false_0 != 0,
                                );
                            } else {
                                report_discard_pending(
                                    (*cstack).cs_pending[idx as usize] as ::core::ffi::c_int,
                                    NULL,
                                );
                            }
                            (*cstack).cs_pending[idx as usize] =
                                CSTP_NONE as ::core::ffi::c_int as ::core::ffi::c_char;
                        }
                    }
                }
            }
            if (*cstack).cs_flags[idx as usize] & CSF_FINALLY as ::core::ffi::c_int == 0 {
                if (*cstack).cs_flags[idx as usize] & CSF_ACTIVE as ::core::ffi::c_int != 0
                    && (*cstack).cs_flags[idx as usize] & CSF_CAUGHT as ::core::ffi::c_int != 0
                    && (*cstack).cs_flags[idx as usize] & CSF_FINISHED as ::core::ffi::c_int == 0
                {
                    finish_exception((*cstack).cs_pend.csp_ex[idx as usize] as *mut except_T);
                    (*cstack).cs_flags[idx as usize] |= CSF_FINISHED as ::core::ffi::c_int;
                }
                if (*cstack).cs_flags[idx as usize] & CSF_TRUE as ::core::ffi::c_int != 0 {
                    if searched_cond == 0 as ::core::ffi::c_int && inclusive == 0 {
                        break;
                    }
                    stop = true_0 != 0;
                }
            }
        }
        if (*cstack).cs_flags[idx as usize] & searched_cond != 0 {
            if inclusive == 0 {
                break;
            }
            stop = true_0 != 0;
        }
        (*cstack).cs_flags[idx as usize] &= !(CSF_ACTIVE as ::core::ffi::c_int);
        if stop as ::core::ffi::c_int != 0
            && searched_cond != CSF_TRY as ::core::ffi::c_int | CSF_SILENT as ::core::ffi::c_int
        {
            break;
        }
        if (*cstack).cs_flags[idx as usize] & CSF_TRY as ::core::ffi::c_int != 0
            && (*cstack).cs_flags[idx as usize] & CSF_SILENT as ::core::ffi::c_int != 0
        {
            let mut elem: *mut eslist_T = ::core::ptr::null_mut::<eslist_T>();
            elem = (*cstack).cs_emsg_silent_list;
            (*cstack).cs_emsg_silent_list = (*elem).next;
            emsg_silent.set((*elem).saved_emsg_silent);
            xfree(elem as *mut ::core::ffi::c_void);
            (*cstack).cs_flags[idx as usize] &= !(CSF_SILENT as ::core::ffi::c_int);
        }
        if stop {
            break;
        }
        idx -= 1;
    }
    return idx;
}
unsafe extern "C" fn get_end_emsg(mut cstack: *mut cstack_T) -> *mut ::core::ffi::c_char {
    if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_WHILE as ::core::ffi::c_int != 0 {
        return gettext(&raw const e_endwhile as *const ::core::ffi::c_char);
    }
    if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_FOR as ::core::ffi::c_int != 0 {
        return gettext(&raw const e_endfor as *const ::core::ffi::c_char);
    }
    return gettext(&raw const e_endif as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn rewind_conditionals(
    mut cstack: *mut cstack_T,
    mut idx: ::core::ffi::c_int,
    mut cond_type: ::core::ffi::c_int,
    mut cond_level: *mut ::core::ffi::c_int,
) {
    while (*cstack).cs_idx > idx {
        if (*cstack).cs_flags[(*cstack).cs_idx as usize] & cond_type != 0 {
            *cond_level -= 1;
        }
        if (*cstack).cs_flags[(*cstack).cs_idx as usize] & CSF_FOR as ::core::ffi::c_int != 0 {
            free_for_info((*cstack).cs_forinfo[(*cstack).cs_idx as usize]);
        }
        (*cstack).cs_idx -= 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_endfunction(mut _eap: *mut exarg_T) {
    semsg(
        gettext(&raw const e_str_not_inside_function as *const ::core::ffi::c_char),
        b":endfunction\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn has_loop_cmd(mut p: *mut ::core::ffi::c_char) -> bool {
    loop {
        while *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == '\t' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        let mut len: ::core::ffi::c_int = modifier_len(p);
        if len == 0 as ::core::ffi::c_int {
            break;
        }
        p = p.offset(len as isize);
    }
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'w' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'h' as ::core::ffi::c_int
        || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'f' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'o' as ::core::ffi::c_int
            && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'r' as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
