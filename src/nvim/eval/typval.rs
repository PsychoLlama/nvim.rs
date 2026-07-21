use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
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
    fn strtod(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
    fn abort() -> !;
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strcoll(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xstrndup(str: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn xmemdup(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_intern2: [::core::ffi::c_char; 0];
    static e_invarg: [::core::ffi::c_char; 0];
    static e_invrange: [::core::ffi::c_char; 0];
    static e_blobidx: [::core::ffi::c_char; 0];
    static e_toomanyarg: [::core::ffi::c_char; 0];
    static e_dictkey: [::core::ffi::c_char; 0];
    static e_listreq: [::core::ffi::c_char; 0];
    static e_list_index_out_of_range_nr: [::core::ffi::c_char; 0];
    static e_listarg: [::core::ffi::c_char; 0];
    static e_value_is_locked: [::core::ffi::c_char; 0];
    static e_value_is_locked_str: [::core::ffi::c_char; 0];
    static e_cannot_change_value: [::core::ffi::c_char; 0];
    static e_cannot_change_value_of_str: [::core::ffi::c_char; 0];
    static e_invalid_value_for_blob_nr: [::core::ffi::c_char; 0];
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn vim_str2nr(
        start: *const ::core::ffi::c_char,
        prep: *mut ::core::ffi::c_int,
        len: *mut ::core::ffi::c_int,
        what: ::core::ffi::c_int,
        nptr: *mut varnumber_T,
        unptr: *mut uvarnumber_T,
        maxlen: ::core::ffi::c_int,
        strict: bool,
        overflow: *mut bool,
    );
    fn partial_name(pt: *mut partial_T) -> *mut ::core::ffi::c_char;
    fn partial_unref(pt: *mut partial_T);
    fn func_equal(tv1: *mut typval_T, tv2: *mut typval_T, ic: bool) -> bool;
    fn get_copyID() -> ::core::ffi::c_int;
    fn callback_from_typval(callback: *mut Callback, arg: *const typval_T) -> bool;
    fn callback_call(
        callback: *mut Callback,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        rettv: *mut typval_T,
    ) -> bool;
    fn var2fpos(
        tv: *const typval_T,
        dollar_lnum: bool,
        ret_fnum: *mut ::core::ffi::c_int,
        charcol: bool,
        wp: *mut win_T,
    ) -> *mut pos_T;
    fn set_selfdict(rettv: *mut typval_T, selfdict: *mut dict_T);
    fn var_item_copy(
        conv: *const vimconv_T,
        from: *mut typval_T,
        to: *mut typval_T,
        deep: bool,
        copyID: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    static encode_bool_var_names: [*const ::core::ffi::c_char; 0];
    static encode_special_var_names: [*const ::core::ffi::c_char; 0];
    fn encode_vim_list_to_buf(
        list: *const list_T,
        ret_len: *mut size_t,
        ret_buf: *mut *mut ::core::ffi::c_char,
    ) -> bool;
    fn encode_tv2string(tv: *mut typval_T, len: *mut size_t) -> *mut ::core::ffi::c_char;
    fn encode_tv2echo(tv: *mut typval_T, len: *mut size_t) -> *mut ::core::ffi::c_char;
    fn eexe_mod_op(
        tv1: *mut typval_T,
        tv2: *const typval_T,
        op: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    static gc_first_dict: GlobalCell<*mut dict_T>;
    static gc_first_list: GlobalCell<*mut list_T>;
    static hash_removed: ::core::ffi::c_char;
    fn hash_init(ht: *mut hashtab_T);
    fn hash_clear(ht: *mut hashtab_T);
    fn hash_find(ht: *const hashtab_T, key: *const ::core::ffi::c_char) -> *mut hashitem_T;
    fn hash_find_len(
        ht: *const hashtab_T,
        key: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut hashitem_T;
    fn hash_add(ht: *mut hashtab_T, key: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn hash_remove(ht: *mut hashtab_T, hi: *mut hashitem_T);
    fn hash_lock(ht: *mut hashtab_T);
    fn hash_unlock(ht: *mut hashtab_T);
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn internal_error(where_0: *const ::core::ffi::c_char);
    fn call_func(
        funcname: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
        rettv: *mut typval_T,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        funcexe: *mut funcexe_T,
    ) -> ::core::ffi::c_int;
    fn func_unref(name: *mut ::core::ffi::c_char);
    fn func_ref(name: *mut ::core::ffi::c_char);
    fn get_funccal_local_ht() -> *mut hashtab_T;
    fn get_globvar_dict() -> *mut dict_T;
    fn var_check_ro(
        flags: ::core::ffi::c_int,
        name: *const ::core::ffi::c_char,
        name_len: size_t,
    ) -> bool;
    fn var_check_fixed(
        flags: ::core::ffi::c_int,
        name: *const ::core::ffi::c_char,
        name_len: size_t,
    ) -> bool;
    fn var_wrong_func_name(name: *const ::core::ffi::c_char, new_var: bool) -> bool;
    fn valid_varname(varname: *const ::core::ffi::c_char) -> bool;
    static eval_msgpack_type_lists: GlobalCell<[*const list_T; 8]>;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    fn ga_append_via_ptr(gap: *mut garray_T, item_size: size_t) -> *mut ::core::ffi::c_void;
    static did_emsg: GlobalCell<::core::ffi::c_int>;
    static curwin: GlobalCell<*mut win_T>;
    static got_int: GlobalCell<bool>;
    fn api_free_luaref(ref_0: LuaRef);
    fn api_new_luaref(original_ref: LuaRef) -> LuaRef;
    fn nlua_funcref_str(ref_0: LuaRef, arena: *mut Arena) -> *mut ::core::ffi::c_char;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn string_convert(
        vcp: *const vimconv_T,
        ptr: *mut ::core::ffi::c_char,
        lenp: *mut size_t,
    ) -> *mut ::core::ffi::c_char;
    fn mb_strcmp_ic(
        ic: bool,
        s1: *const ::core::ffi::c_char,
        s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn line_breakcheck();
    fn fast_breakcheck();
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __time_t = ::core::ffi::c_long;
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct alist_T {
    pub al_ga: garray_T,
    pub al_refcount: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
}
pub type linenr_T = int32_t;
pub type colnr_T = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub coladd: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MsgpackRpcRequestHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: ApiDispatchWrapper,
    pub fast: bool,
    pub ret_alloc: bool,
}
pub type ApiDispatchWrapper =
    Option<unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error {
    pub type_0: ErrorType,
    pub msg: *mut ::core::ffi::c_char,
}
pub type ErrorType = ::core::ffi::c_int;
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union EvalFuncData {
    pub float_func: Option<unsafe extern "C" fn(float_T) -> float_T>,
    pub api_handler: *const MsgpackRpcRequestHandler,
    pub null: *mut ::core::ffi::c_void,
}
pub type proftime_T = uint64_t;
pub type OptInt = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct file_buffer {
    pub handle: handle_T,
    pub b_ml: memline_T,
    pub b_next: *mut buf_T,
    pub b_prev: *mut buf_T,
    pub b_nwindows: ::core::ffi::c_int,
    pub b_flags: ::core::ffi::c_int,
    pub b_locked: ::core::ffi::c_int,
    pub b_locked_split: ::core::ffi::c_int,
    pub b_ro_locked: ::core::ffi::c_int,
    pub b_ffname: *mut ::core::ffi::c_char,
    pub b_sfname: *mut ::core::ffi::c_char,
    pub b_fname: *mut ::core::ffi::c_char,
    pub file_id_valid: bool,
    pub file_id: FileID,
    pub b_changed: ::core::ffi::c_int,
    pub b_changed_invalid: bool,
    pub changedtick_di: ChangedtickDictItem,
    pub b_last_changedtick: varnumber_T,
    pub b_last_changedtick_i: varnumber_T,
    pub b_last_changedtick_pum: varnumber_T,
    pub b_saving: bool,
    pub b_mod_set: bool,
    pub b_mod_top: linenr_T,
    pub b_mod_bot: linenr_T,
    pub b_mod_xlines: linenr_T,
    pub b_wininfo: C2Rust_Unnamed_11,
    pub b_mod_tick_syn: disptick_T,
    pub b_mod_tick_decor: disptick_T,
    pub b_mtime: int64_t,
    pub b_mtime_ns: int64_t,
    pub b_mtime_read: int64_t,
    pub b_mtime_read_ns: int64_t,
    pub b_orig_size: uint64_t,
    pub b_orig_mode: ::core::ffi::c_int,
    pub b_last_used: time_t,
    pub b_namedm: [fmark_T; 26],
    pub b_visual: visualinfo_T,
    pub b_visual_mode_eval: ::core::ffi::c_int,
    pub b_last_cursor: fmark_T,
    pub b_last_insert: fmark_T,
    pub b_last_change: fmark_T,
    pub b_changelist: [fmark_T; 100],
    pub b_changelistlen: ::core::ffi::c_int,
    pub b_new_change: bool,
    pub b_chartab: [uint64_t; 4],
    pub b_maphash: [*mut mapblock_T; 256],
    pub b_first_abbr: *mut mapblock_T,
    pub b_ucmds: garray_T,
    pub b_op_start: pos_T,
    pub b_op_start_orig: pos_T,
    pub b_op_end: pos_T,
    pub b_marks_read: bool,
    pub b_modified_was_set: bool,
    pub b_did_filetype: bool,
    pub b_keep_filetype: bool,
    pub b_au_did_filetype: bool,
    pub b_u_oldhead: *mut u_header_T,
    pub b_u_newhead: *mut u_header_T,
    pub b_u_curhead: *mut u_header_T,
    pub b_u_numhead: ::core::ffi::c_int,
    pub b_u_synced: bool,
    pub b_u_seq_last: ::core::ffi::c_int,
    pub b_u_save_nr_last: ::core::ffi::c_int,
    pub b_u_seq_cur: ::core::ffi::c_int,
    pub b_u_time_cur: time_t,
    pub b_u_save_nr_cur: ::core::ffi::c_int,
    pub b_u_line_ptr: *mut ::core::ffi::c_char,
    pub b_u_line_lnum: linenr_T,
    pub b_u_line_colnr: colnr_T,
    pub b_scanned: bool,
    pub b_p_iminsert: OptInt,
    pub b_p_imsearch: OptInt,
    pub b_kmap_state: int16_t,
    pub b_kmap_ga: garray_T,
    pub b_p_initialized: bool,
    pub b_p_script_ctx: [sctx_T; 92],
    pub b_p_ac: ::core::ffi::c_int,
    pub b_p_ai: ::core::ffi::c_int,
    pub b_p_ai_nopaste: ::core::ffi::c_int,
    pub b_p_bkc: *mut ::core::ffi::c_char,
    pub b_bkc_flags: ::core::ffi::c_uint,
    pub b_p_ci: ::core::ffi::c_int,
    pub b_p_bin: ::core::ffi::c_int,
    pub b_p_bomb: ::core::ffi::c_int,
    pub b_p_bh: *mut ::core::ffi::c_char,
    pub b_p_bt: *mut ::core::ffi::c_char,
    pub b_p_busy: OptInt,
    pub b_has_qf_entry: ::core::ffi::c_int,
    pub b_p_bl: ::core::ffi::c_int,
    pub b_p_channel: OptInt,
    pub b_p_cin: ::core::ffi::c_int,
    pub b_p_cino: *mut ::core::ffi::c_char,
    pub b_p_cink: *mut ::core::ffi::c_char,
    pub b_p_cinw: *mut ::core::ffi::c_char,
    pub b_p_cinsd: *mut ::core::ffi::c_char,
    pub b_p_com: *mut ::core::ffi::c_char,
    pub b_p_cms: *mut ::core::ffi::c_char,
    pub b_p_cot: *mut ::core::ffi::c_char,
    pub b_cot_flags: ::core::ffi::c_uint,
    pub b_p_cpt: *mut ::core::ffi::c_char,
    pub b_p_cpt_cb: *mut Callback,
    pub b_p_cpt_count: ::core::ffi::c_int,
    pub b_p_cfu: *mut ::core::ffi::c_char,
    pub b_cfu_cb: Callback,
    pub b_p_ofu: *mut ::core::ffi::c_char,
    pub b_ofu_cb: Callback,
    pub b_p_tfu: *mut ::core::ffi::c_char,
    pub b_tfu_cb: Callback,
    pub b_p_ffu: *mut ::core::ffi::c_char,
    pub b_ffu_cb: Callback,
    pub b_p_eof: ::core::ffi::c_int,
    pub b_p_eol: ::core::ffi::c_int,
    pub b_p_fixeol: ::core::ffi::c_int,
    pub b_p_et: ::core::ffi::c_int,
    pub b_p_et_nobin: ::core::ffi::c_int,
    pub b_p_et_nopaste: ::core::ffi::c_int,
    pub b_p_fenc: *mut ::core::ffi::c_char,
    pub b_p_ff: *mut ::core::ffi::c_char,
    pub b_p_ft: *mut ::core::ffi::c_char,
    pub b_p_fo: *mut ::core::ffi::c_char,
    pub b_p_flp: *mut ::core::ffi::c_char,
    pub b_p_inf: ::core::ffi::c_int,
    pub b_p_isk: *mut ::core::ffi::c_char,
    pub b_p_def: *mut ::core::ffi::c_char,
    pub b_p_inc: *mut ::core::ffi::c_char,
    pub b_p_inex: *mut ::core::ffi::c_char,
    pub b_p_inex_flags: uint32_t,
    pub b_p_inde: *mut ::core::ffi::c_char,
    pub b_p_inde_flags: uint32_t,
    pub b_p_indk: *mut ::core::ffi::c_char,
    pub b_p_fp: *mut ::core::ffi::c_char,
    pub b_p_fex: *mut ::core::ffi::c_char,
    pub b_p_fex_flags: uint32_t,
    pub b_p_fs: ::core::ffi::c_int,
    pub b_p_kp: *mut ::core::ffi::c_char,
    pub b_p_lisp: ::core::ffi::c_int,
    pub b_p_lop: *mut ::core::ffi::c_char,
    pub b_p_menc: *mut ::core::ffi::c_char,
    pub b_p_mps: *mut ::core::ffi::c_char,
    pub b_p_ml: ::core::ffi::c_int,
    pub b_p_ml_nobin: ::core::ffi::c_int,
    pub b_p_ma: ::core::ffi::c_int,
    pub b_p_nf: *mut ::core::ffi::c_char,
    pub b_p_pi: ::core::ffi::c_int,
    pub b_p_qe: *mut ::core::ffi::c_char,
    pub b_p_ro: ::core::ffi::c_int,
    pub b_p_sw: OptInt,
    pub b_p_scbk: OptInt,
    pub b_p_si: ::core::ffi::c_int,
    pub b_p_sts: OptInt,
    pub b_p_sts_nopaste: OptInt,
    pub b_p_sua: *mut ::core::ffi::c_char,
    pub b_p_swf: ::core::ffi::c_int,
    pub b_p_smc: OptInt,
    pub b_p_syn: *mut ::core::ffi::c_char,
    pub b_p_ts: OptInt,
    pub b_p_tw: OptInt,
    pub b_p_tw_nobin: OptInt,
    pub b_p_tw_nopaste: OptInt,
    pub b_p_wm: OptInt,
    pub b_p_wm_nobin: OptInt,
    pub b_p_wm_nopaste: OptInt,
    pub b_p_vsts: *mut ::core::ffi::c_char,
    pub b_p_vsts_array: *mut colnr_T,
    pub b_p_vsts_nopaste: *mut ::core::ffi::c_char,
    pub b_p_vts: *mut ::core::ffi::c_char,
    pub b_p_vts_array: *mut colnr_T,
    pub b_p_keymap: *mut ::core::ffi::c_char,
    pub b_p_gefm: *mut ::core::ffi::c_char,
    pub b_p_gp: *mut ::core::ffi::c_char,
    pub b_p_mp: *mut ::core::ffi::c_char,
    pub b_p_efm: *mut ::core::ffi::c_char,
    pub b_p_ep: *mut ::core::ffi::c_char,
    pub b_p_path: *mut ::core::ffi::c_char,
    pub b_p_ar: ::core::ffi::c_int,
    pub b_p_tags: *mut ::core::ffi::c_char,
    pub b_p_tc: *mut ::core::ffi::c_char,
    pub b_tc_flags: ::core::ffi::c_uint,
    pub b_p_dict: *mut ::core::ffi::c_char,
    pub b_p_dia: *mut ::core::ffi::c_char,
    pub b_p_tsr: *mut ::core::ffi::c_char,
    pub b_p_tsrfu: *mut ::core::ffi::c_char,
    pub b_tsrfu_cb: Callback,
    pub b_p_ul: OptInt,
    pub b_p_udf: ::core::ffi::c_int,
    pub b_p_lw: *mut ::core::ffi::c_char,
    pub b_ind_level: ::core::ffi::c_int,
    pub b_ind_open_imag: ::core::ffi::c_int,
    pub b_ind_no_brace: ::core::ffi::c_int,
    pub b_ind_first_open: ::core::ffi::c_int,
    pub b_ind_open_extra: ::core::ffi::c_int,
    pub b_ind_close_extra: ::core::ffi::c_int,
    pub b_ind_open_left_imag: ::core::ffi::c_int,
    pub b_ind_jump_label: ::core::ffi::c_int,
    pub b_ind_case: ::core::ffi::c_int,
    pub b_ind_case_code: ::core::ffi::c_int,
    pub b_ind_case_break: ::core::ffi::c_int,
    pub b_ind_param: ::core::ffi::c_int,
    pub b_ind_func_type: ::core::ffi::c_int,
    pub b_ind_comment: ::core::ffi::c_int,
    pub b_ind_in_comment: ::core::ffi::c_int,
    pub b_ind_in_comment2: ::core::ffi::c_int,
    pub b_ind_cpp_baseclass: ::core::ffi::c_int,
    pub b_ind_continuation: ::core::ffi::c_int,
    pub b_ind_unclosed: ::core::ffi::c_int,
    pub b_ind_unclosed2: ::core::ffi::c_int,
    pub b_ind_unclosed_noignore: ::core::ffi::c_int,
    pub b_ind_unclosed_wrapped: ::core::ffi::c_int,
    pub b_ind_unclosed_whiteok: ::core::ffi::c_int,
    pub b_ind_matching_paren: ::core::ffi::c_int,
    pub b_ind_paren_prev: ::core::ffi::c_int,
    pub b_ind_maxparen: ::core::ffi::c_int,
    pub b_ind_maxcomment: ::core::ffi::c_int,
    pub b_ind_scopedecl: ::core::ffi::c_int,
    pub b_ind_scopedecl_code: ::core::ffi::c_int,
    pub b_ind_java: ::core::ffi::c_int,
    pub b_ind_js: ::core::ffi::c_int,
    pub b_ind_keep_case_label: ::core::ffi::c_int,
    pub b_ind_hash_comment: ::core::ffi::c_int,
    pub b_ind_cpp_namespace: ::core::ffi::c_int,
    pub b_ind_if_for_while: ::core::ffi::c_int,
    pub b_ind_cpp_extern_c: ::core::ffi::c_int,
    pub b_ind_pragma: ::core::ffi::c_int,
    pub b_no_eol_lnum: linenr_T,
    pub b_start_eof: ::core::ffi::c_int,
    pub b_start_eol: ::core::ffi::c_int,
    pub b_start_ffc: ::core::ffi::c_int,
    pub b_start_fenc: *mut ::core::ffi::c_char,
    pub b_bad_char: ::core::ffi::c_int,
    pub b_start_bomb: ::core::ffi::c_int,
    pub b_bufvar: ScopeDictDictItem,
    pub b_vars: *mut dict_T,
    pub b_may_swap: bool,
    pub b_did_warn: bool,
    pub b_help: bool,
    pub b_spell: bool,
    pub b_prompt_text: *mut ::core::ffi::c_char,
    pub b_prompt_callback: Callback,
    pub b_prompt_interrupt: Callback,
    pub b_prompt_append_new_line: bool,
    pub b_prompt_insert: ::core::ffi::c_int,
    pub b_prompt_start: fmark_T,
    pub b_s: synblock_T,
    pub b_signcols: C2Rust_Unnamed_3,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_1,
    pub update_callbacks: C2Rust_Unnamed_0,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut BufUpdateCallbacks,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BufUpdateCallbacks {
    pub on_lines: LuaRef,
    pub on_bytes: LuaRef,
    pub on_changedtick: LuaRef,
    pub on_detach: LuaRef,
    pub on_reload: LuaRef,
    pub utf_sizes: bool,
    pub preview: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_1 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint32_t_uint32_t {
    pub set: Set_uint32_t,
    pub values: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_uint32_t {
    pub h: MapHash,
    pub keys: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MapHash {
    pub n_buckets: uint32_t,
    pub size: uint32_t,
    pub n_occupied: uint32_t,
    pub upper_bound: uint32_t,
    pub n_keys: uint32_t,
    pub keys_capacity: uint32_t,
    pub hash: *mut uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTree {
    pub root: *mut MTNode,
    pub meta_root: [uint32_t; 5],
    pub n_keys: size_t,
    pub n_nodes: size_t,
    pub id2node: [Map_uint64_t_ptr_t; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint64_t_ptr_t {
    pub set: Set_uint64_t,
    pub values: *mut ptr_t,
}
pub type ptr_t = *mut ::core::ffi::c_void;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_uint64_t {
    pub h: MapHash,
    pub keys: *mut uint64_t,
}
pub type MTNode = mtnode_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mtnode_s {
    pub n: int32_t,
    pub level: int16_t,
    pub p_idx: int16_t,
    pub intersect: Intersection,
    pub parent: *mut MTNode,
    pub key: [MTKey; 19],
    pub s: [mtnode_inner_s; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mtnode_inner_s {
    pub i_ptr: [*mut MTNode; 20],
    pub i_meta: [[uint32_t; 5]; 20],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTKey {
    pub pos: MTPos,
    pub ns: uint32_t,
    pub id: uint32_t,
    pub flags: uint16_t,
    pub decor_data: DecorInlineData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union DecorInlineData {
    pub hl: DecorHighlightInline,
    pub ext: DecorExt,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorExt {
    pub sh_idx: uint32_t,
    pub vt: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorVirtText {
    pub flags: uint8_t,
    pub hl_mode: uint8_t,
    pub priority: DecorPriority,
    pub width: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub pos: VirtTextPos,
    pub data: C2Rust_Unnamed_2,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_2 {
    pub virt_text: VirtText,
    pub virt_lines: VirtLines,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtLines {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut virt_line,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct virt_line {
    pub line: VirtText,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtText {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut VirtTextChunk,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VirtTextChunk {
    pub text: *mut ::core::ffi::c_char,
    pub hl_id: ::core::ffi::c_int,
}
pub type VirtTextPos = ::core::ffi::c_uint;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub type DecorPriority = uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorHighlightInline {
    pub flags: uint16_t,
    pub priority: DecorPriority,
    pub hl_id: ::core::ffi::c_int,
    pub conceal_char: schar_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTPos {
    pub row: int32_t,
    pub col: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Intersection {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
    pub init_array: [uint64_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AdditionalData {
    pub nitems: uint32_t,
    pub nbytes: uint32_t,
    pub data: [::core::ffi::c_char; 0],
}
pub type Terminal = terminal;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub max: ::core::ffi::c_int,
    pub last_max: ::core::ffi::c_int,
    pub count: [::core::ffi::c_int; 9],
    pub autom: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct synblock_T {
    pub b_keywtab: hashtab_T,
    pub b_keywtab_ic: hashtab_T,
    pub b_syn_error: bool,
    pub b_syn_slow: bool,
    pub b_syn_ic: ::core::ffi::c_int,
    pub b_syn_foldlevel: ::core::ffi::c_int,
    pub b_syn_spell: ::core::ffi::c_int,
    pub b_syn_patterns: garray_T,
    pub b_syn_clusters: garray_T,
    pub b_spell_cluster_id: ::core::ffi::c_int,
    pub b_nospell_cluster_id: ::core::ffi::c_int,
    pub b_syn_containedin: ::core::ffi::c_int,
    pub b_syn_sync_flags: ::core::ffi::c_int,
    pub b_syn_sync_id: int16_t,
    pub b_syn_sync_minlines: linenr_T,
    pub b_syn_sync_maxlines: linenr_T,
    pub b_syn_sync_linebreaks: linenr_T,
    pub b_syn_linecont_pat: *mut ::core::ffi::c_char,
    pub b_syn_linecont_prog: *mut regprog_T,
    pub b_syn_linecont_time: syn_time_T,
    pub b_syn_linecont_ic: ::core::ffi::c_int,
    pub b_syn_topgrp: ::core::ffi::c_int,
    pub b_syn_conceal: ::core::ffi::c_int,
    pub b_syn_folditems: ::core::ffi::c_int,
    pub b_sst_array: *mut synstate_T,
    pub b_sst_len: ::core::ffi::c_int,
    pub b_sst_first: *mut synstate_T,
    pub b_sst_firstfree: *mut synstate_T,
    pub b_sst_freecount: ::core::ffi::c_int,
    pub b_sst_check_lnum: linenr_T,
    pub b_sst_lasttick: disptick_T,
    pub b_langp: garray_T,
    pub b_spell_ismw: [bool; 256],
    pub b_spell_ismw_mb: *mut ::core::ffi::c_char,
    pub b_p_spc: *mut ::core::ffi::c_char,
    pub b_cap_prog: *mut regprog_T,
    pub b_p_spf: *mut ::core::ffi::c_char,
    pub b_p_spl: *mut ::core::ffi::c_char,
    pub b_p_spo: *mut ::core::ffi::c_char,
    pub b_p_spo_flags: ::core::ffi::c_uint,
    pub b_cjk: ::core::ffi::c_int,
    pub b_syn_chartab: [uint8_t; 32],
    pub b_syn_isk: *mut ::core::ffi::c_char,
}
pub type regprog_T = regprog;
pub type disptick_T = uint64_t;
pub type synstate_T = syn_state;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_state {
    pub sst_next: *mut synstate_T,
    pub sst_lnum: linenr_T,
    pub sst_union: C2Rust_Unnamed_4,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
    pub sst_stack: [bufstate_T; 7],
    pub sst_ga: garray_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufstate_T {
    pub bs_idx: ::core::ffi::c_int,
    pub bs_flags: ::core::ffi::c_int,
    pub bs_seqnr: ::core::ffi::c_int,
    pub bs_cchar: ::core::ffi::c_int,
    pub bs_extmatch: *mut reg_extmatch_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct reg_extmatch_T {
    pub refcnt: int16_t,
    pub matches: [*mut uint8_t; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct syn_time_T {
    pub total: proftime_T,
    pub slowest: proftime_T,
    pub count: ::core::ffi::c_int,
    pub match_0: ::core::ffi::c_int,
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
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
}
pub type hash_T = size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fmark_T {
    pub mark: pos_T,
    pub fnum: ::core::ffi::c_int,
    pub timestamp: Timestamp,
    pub view: fmarkv_T,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fmarkv_T {
    pub topline_offset: linenr_T,
    pub skipcol: colnr_T,
}
pub type Timestamp = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Callback {
    pub data: C2Rust_Unnamed_5,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_5 {
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
    pub fc_fixvar: [C2Rust_Unnamed_6; 12],
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
pub struct C2Rust_Unnamed_6 {
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
pub type u_header_T = u_header;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct u_header {
    pub uh_next: C2Rust_Unnamed_10,
    pub uh_prev: C2Rust_Unnamed_9,
    pub uh_alt_next: C2Rust_Unnamed_8,
    pub uh_alt_prev: C2Rust_Unnamed_7,
    pub uh_seq: ::core::ffi::c_int,
    pub uh_walk: ::core::ffi::c_int,
    pub uh_entry: *mut u_entry_T,
    pub uh_getbot_entry: *mut u_entry_T,
    pub uh_cursor: pos_T,
    pub uh_cursor_vcol: colnr_T,
    pub uh_flags: ::core::ffi::c_int,
    pub uh_namedm: [fmark_T; 26],
    pub uh_extmark: extmark_undo_vec_t,
    pub uh_visual: visualinfo_T,
    pub uh_time: time_t,
    pub uh_save_nr: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct visualinfo_T {
    pub vi_start: pos_T,
    pub vi_end: pos_T,
    pub vi_mode: ::core::ffi::c_int,
    pub vi_curswant: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct extmark_undo_vec_t {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ExtmarkUndoObject,
}
pub type ExtmarkUndoObject = undo_object;
pub type u_entry_T = u_entry;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct u_entry {
    pub ue_next: *mut u_entry_T,
    pub ue_top: linenr_T,
    pub ue_bot: linenr_T,
    pub ue_lcount: linenr_T,
    pub ue_array: *mut *mut ::core::ffi::c_char,
    pub ue_size: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
}
pub type mapblock_T = mapblock;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mapblock {
    pub m_next: *mut mapblock_T,
    pub m_alt: *mut mapblock_T,
    pub m_keys: *mut ::core::ffi::c_char,
    pub m_str: *mut ::core::ffi::c_char,
    pub m_orig_str: *mut ::core::ffi::c_char,
    pub m_luaref: LuaRef,
    pub m_keylen: ::core::ffi::c_int,
    pub m_mode: ::core::ffi::c_int,
    pub m_simplified: ::core::ffi::c_int,
    pub m_noremap: ::core::ffi::c_int,
    pub m_silent: ::core::ffi::c_char,
    pub m_nowait: ::core::ffi::c_char,
    pub m_expr: ::core::ffi::c_char,
    pub m_script_ctx: sctx_T,
    pub m_desc: *mut ::core::ffi::c_char,
    pub m_replace_keycodes: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_11 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut WinInfo,
}
pub type WinInfo = wininfo_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wininfo_S {
    pub wi_win: *mut win_T,
    pub wi_mark: fmark_T,
    pub wi_optset: bool,
    pub wi_opt: winopt_T,
    pub wi_fold_manual: bool,
    pub wi_folds: garray_T,
    pub wi_changelistidx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winopt_T {
    pub wo_arab: ::core::ffi::c_int,
    pub wo_bri: ::core::ffi::c_int,
    pub wo_briopt: *mut ::core::ffi::c_char,
    pub wo_diff: ::core::ffi::c_int,
    pub wo_fdc: *mut ::core::ffi::c_char,
    pub wo_eiw: *mut ::core::ffi::c_char,
    pub wo_fdc_save: *mut ::core::ffi::c_char,
    pub wo_fen: ::core::ffi::c_int,
    pub wo_fen_save: ::core::ffi::c_int,
    pub wo_fdi: *mut ::core::ffi::c_char,
    pub wo_fdl: OptInt,
    pub wo_fdl_save: OptInt,
    pub wo_fdm: *mut ::core::ffi::c_char,
    pub wo_fdm_save: *mut ::core::ffi::c_char,
    pub wo_fml: OptInt,
    pub wo_fdn: OptInt,
    pub wo_fde: *mut ::core::ffi::c_char,
    pub wo_fdt: *mut ::core::ffi::c_char,
    pub wo_fmr: *mut ::core::ffi::c_char,
    pub wo_lbr: ::core::ffi::c_int,
    pub wo_list: ::core::ffi::c_int,
    pub wo_nu: ::core::ffi::c_int,
    pub wo_rnu: ::core::ffi::c_int,
    pub wo_ve: *mut ::core::ffi::c_char,
    pub wo_ve_flags: ::core::ffi::c_uint,
    pub wo_nuw: OptInt,
    pub wo_wfb: ::core::ffi::c_int,
    pub wo_wfh: ::core::ffi::c_int,
    pub wo_wfw: ::core::ffi::c_int,
    pub wo_pvw: ::core::ffi::c_int,
    pub wo_lhi: OptInt,
    pub wo_rl: ::core::ffi::c_int,
    pub wo_rlc: *mut ::core::ffi::c_char,
    pub wo_scr: OptInt,
    pub wo_sms: ::core::ffi::c_int,
    pub wo_spell: ::core::ffi::c_int,
    pub wo_cuc: ::core::ffi::c_int,
    pub wo_cul: ::core::ffi::c_int,
    pub wo_culopt: *mut ::core::ffi::c_char,
    pub wo_cc: *mut ::core::ffi::c_char,
    pub wo_sbr: *mut ::core::ffi::c_char,
    pub wo_stc: *mut ::core::ffi::c_char,
    pub wo_stl: *mut ::core::ffi::c_char,
    pub wo_wbr: *mut ::core::ffi::c_char,
    pub wo_scb: ::core::ffi::c_int,
    pub wo_diff_saved: ::core::ffi::c_int,
    pub wo_scb_save: ::core::ffi::c_int,
    pub wo_wrap: ::core::ffi::c_int,
    pub wo_wrap_save: ::core::ffi::c_int,
    pub wo_cocu: *mut ::core::ffi::c_char,
    pub wo_cole: OptInt,
    pub wo_crb: ::core::ffi::c_int,
    pub wo_crb_save: ::core::ffi::c_int,
    pub wo_scl: *mut ::core::ffi::c_char,
    pub wo_siso: OptInt,
    pub wo_so: OptInt,
    pub wo_winhl: *mut ::core::ffi::c_char,
    pub wo_lcs: *mut ::core::ffi::c_char,
    pub wo_fcs: *mut ::core::ffi::c_char,
    pub wo_winbl: OptInt,
    pub wo_wrap_flags: uint32_t,
    pub wo_stl_flags: uint32_t,
    pub wo_wbr_flags: uint32_t,
    pub wo_fde_flags: uint32_t,
    pub wo_fdt_flags: uint32_t,
    pub wo_script_ctx: [sctx_T; 51],
}
pub type win_T = window_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_S {
    pub handle: handle_T,
    pub w_buffer: *mut buf_T,
    pub w_s: *mut synblock_T,
    pub w_ns_hl: ::core::ffi::c_int,
    pub w_ns_hl_winhl: ::core::ffi::c_int,
    pub w_ns_hl_active: ::core::ffi::c_int,
    pub w_ns_hl_attr: *mut ::core::ffi::c_int,
    pub w_ns_set: Set_uint32_t,
    pub w_hl_id_normal: ::core::ffi::c_int,
    pub w_hl_attr_normal: ::core::ffi::c_int,
    pub w_hl_attr_normalnc: ::core::ffi::c_int,
    pub w_hl_needs_update: ::core::ffi::c_int,
    pub w_prev: *mut win_T,
    pub w_next: *mut win_T,
    pub w_locked: bool,
    pub w_frame: *mut frame_T,
    pub w_cursor: pos_T,
    pub w_curswant: colnr_T,
    pub w_set_curswant: ::core::ffi::c_int,
    pub w_cursorline: linenr_T,
    pub w_last_cursorline: linenr_T,
    pub w_old_visual_mode: ::core::ffi::c_char,
    pub w_old_cursor_lnum: linenr_T,
    pub w_old_cursor_fcol: colnr_T,
    pub w_old_cursor_lcol: colnr_T,
    pub w_old_visual_lnum: linenr_T,
    pub w_old_visual_col: colnr_T,
    pub w_old_curswant: colnr_T,
    pub w_last_cursor_lnum_rnu: linenr_T,
    pub w_p_lcs_chars: lcs_chars_T,
    pub w_p_fcs_chars: fcs_chars_T,
    pub w_topline: linenr_T,
    pub w_topline_was_set: ::core::ffi::c_char,
    pub w_topfill: ::core::ffi::c_int,
    pub w_old_topfill: ::core::ffi::c_int,
    pub w_botfill: bool,
    pub w_old_botfill: bool,
    pub w_leftcol: colnr_T,
    pub w_skipcol: colnr_T,
    pub w_last_topline: linenr_T,
    pub w_last_topfill: ::core::ffi::c_int,
    pub w_last_leftcol: colnr_T,
    pub w_last_skipcol: colnr_T,
    pub w_last_width: ::core::ffi::c_int,
    pub w_last_height: ::core::ffi::c_int,
    pub w_winrow: ::core::ffi::c_int,
    pub w_height: ::core::ffi::c_int,
    pub w_prev_winrow: ::core::ffi::c_int,
    pub w_prev_height: ::core::ffi::c_int,
    pub w_status_height: ::core::ffi::c_int,
    pub w_winbar_height: ::core::ffi::c_int,
    pub w_wincol: ::core::ffi::c_int,
    pub w_width: ::core::ffi::c_int,
    pub w_hsep_height: ::core::ffi::c_int,
    pub w_vsep_width: ::core::ffi::c_int,
    pub w_save_cursor: pos_save_T,
    pub w_do_win_fix_cursor: bool,
    pub w_winrow_off: ::core::ffi::c_int,
    pub w_wincol_off: ::core::ffi::c_int,
    pub w_view_height: ::core::ffi::c_int,
    pub w_view_width: ::core::ffi::c_int,
    pub w_height_request: ::core::ffi::c_int,
    pub w_width_request: ::core::ffi::c_int,
    pub w_border_adj: [::core::ffi::c_int; 4],
    pub w_height_outer: ::core::ffi::c_int,
    pub w_width_outer: ::core::ffi::c_int,
    pub w_valid: ::core::ffi::c_int,
    pub w_valid_cursor: pos_T,
    pub w_valid_leftcol: colnr_T,
    pub w_valid_skipcol: colnr_T,
    pub w_viewport_invalid: bool,
    pub w_viewport_last_topline: linenr_T,
    pub w_viewport_last_botline: linenr_T,
    pub w_viewport_last_topfill: linenr_T,
    pub w_viewport_last_skipcol: linenr_T,
    pub w_cline_height: ::core::ffi::c_int,
    pub w_cline_folded: bool,
    pub w_cline_row: ::core::ffi::c_int,
    pub w_virtcol: colnr_T,
    pub w_wrow: ::core::ffi::c_int,
    pub w_wcol: ::core::ffi::c_int,
    pub w_botline: linenr_T,
    pub w_empty_rows: ::core::ffi::c_int,
    pub w_filler_rows: ::core::ffi::c_int,
    pub w_lines_valid: ::core::ffi::c_int,
    pub w_lines: *mut wline_T,
    pub w_lines_size: ::core::ffi::c_int,
    pub w_folds: garray_T,
    pub w_fold_manual: bool,
    pub w_foldinvalid: bool,
    pub w_nrwidth: ::core::ffi::c_int,
    pub w_scwidth: ::core::ffi::c_int,
    pub w_minscwidth: ::core::ffi::c_int,
    pub w_maxscwidth: ::core::ffi::c_int,
    pub w_redr_type: ::core::ffi::c_int,
    pub w_upd_rows: ::core::ffi::c_int,
    pub w_redraw_top: linenr_T,
    pub w_redraw_bot: linenr_T,
    pub w_redr_status: bool,
    pub w_redr_border: bool,
    pub w_redr_statuscol: bool,
    pub w_display_tick: disptick_T,
    pub w_stl_cursor: pos_T,
    pub w_stl_virtcol: colnr_T,
    pub w_stl_topline: linenr_T,
    pub w_stl_line_count: linenr_T,
    pub w_stl_topfill: ::core::ffi::c_int,
    pub w_stl_empty: ::core::ffi::c_char,
    pub w_stl_recording: ::core::ffi::c_int,
    pub w_stl_state: ::core::ffi::c_int,
    pub w_stl_visual_mode: ::core::ffi::c_int,
    pub w_stl_visual_pos: pos_T,
    pub w_alt_fnum: ::core::ffi::c_int,
    pub w_alist: *mut alist_T,
    pub w_arg_idx: ::core::ffi::c_int,
    pub w_arg_idx_invalid: ::core::ffi::c_int,
    pub w_localdir: *mut ::core::ffi::c_char,
    pub w_prevdir: *mut ::core::ffi::c_char,
    pub w_onebuf_opt: winopt_T,
    pub w_allbuf_opt: winopt_T,
    pub w_p_cc_cols: *mut ::core::ffi::c_int,
    pub w_p_culopt_flags: uint8_t,
    pub w_briopt_min: ::core::ffi::c_int,
    pub w_briopt_shift: ::core::ffi::c_int,
    pub w_briopt_sbr: bool,
    pub w_briopt_list: ::core::ffi::c_int,
    pub w_briopt_vcol: ::core::ffi::c_int,
    pub w_scbind_pos: ::core::ffi::c_int,
    pub w_winvar: ScopeDictDictItem,
    pub w_vars: *mut dict_T,
    pub w_pcmark: pos_T,
    pub w_prev_pcmark: pos_T,
    pub w_jumplist: [xfmark_T; 100],
    pub w_jumplistlen: ::core::ffi::c_int,
    pub w_jumplistidx: ::core::ffi::c_int,
    pub w_changelistidx: ::core::ffi::c_int,
    pub w_match_head: *mut matchitem_T,
    pub w_next_match_id: ::core::ffi::c_int,
    pub w_tagstack: [taggy_T; 20],
    pub w_tagstackidx: ::core::ffi::c_int,
    pub w_tagstacklen: ::core::ffi::c_int,
    pub w_grid: GridView,
    pub w_grid_alloc: ScreenGrid,
    pub w_pos_changed: bool,
    pub w_floating: bool,
    pub w_float_is_info: bool,
    pub w_config: WinConfig,
    pub w_fraction: ::core::ffi::c_int,
    pub w_prev_fraction_row: ::core::ffi::c_int,
    pub w_nrwidth_line_count: linenr_T,
    pub w_statuscol_line_count: linenr_T,
    pub w_nrwidth_width: ::core::ffi::c_int,
    pub w_llist: *mut qf_info_T,
    pub w_llist_ref: *mut qf_info_T,
    pub w_status_click_defs: *mut StlClickDefinition,
    pub w_status_click_defs_size: size_t,
    pub w_winbar_click_defs: *mut StlClickDefinition,
    pub w_winbar_click_defs_size: size_t,
    pub w_statuscol_click_defs: *mut StlClickDefinition,
    pub w_statuscol_click_defs_size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StlClickDefinition {
    pub type_0: C2Rust_Unnamed_12,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_12 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_12 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_12 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_12 = 0;
pub type qf_info_T = qf_info_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WinConfig {
    pub window: Window,
    pub bufpos: lpos_T,
    pub height: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub row: ::core::ffi::c_double,
    pub col: ::core::ffi::c_double,
    pub anchor: FloatAnchor,
    pub relative: FloatRelative,
    pub external: bool,
    pub focusable: bool,
    pub mouse: bool,
    pub split: WinSplit,
    pub zindex: ::core::ffi::c_int,
    pub style: WinStyle,
    pub border: bool,
    pub shadow: bool,
    pub border_chars: [[::core::ffi::c_char; 32]; 8],
    pub border_hl_ids: [::core::ffi::c_int; 8],
    pub border_attr: [::core::ffi::c_int; 8],
    pub title: bool,
    pub title_pos: AlignTextPos,
    pub title_chunks: VirtText,
    pub title_width: ::core::ffi::c_int,
    pub footer: bool,
    pub footer_pos: AlignTextPos,
    pub footer_chunks: VirtText,
    pub footer_width: ::core::ffi::c_int,
    pub noautocmd: bool,
    pub fixed: bool,
    pub hide: bool,
    pub _cmdline_offset: ::core::ffi::c_int,
}
pub type AlignTextPos = ::core::ffi::c_uint;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub type WinStyle = ::core::ffi::c_uint;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub type WinSplit = ::core::ffi::c_uint;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub type FloatRelative = ::core::ffi::c_uint;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub type FloatAnchor = ::core::ffi::c_int;
pub type Window = handle_T;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScreenGrid {
    pub handle: handle_T,
    pub chars: *mut schar_T,
    pub attrs: *mut sattr_T,
    pub vcols: *mut colnr_T,
    pub line_offset: *mut size_t,
    pub dirty_col: *mut ::core::ffi::c_int,
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub valid: bool,
    pub throttled: bool,
    pub blending: bool,
    pub mouse_enabled: bool,
    pub zindex: ::core::ffi::c_int,
    pub comp_row: ::core::ffi::c_int,
    pub comp_col: ::core::ffi::c_int,
    pub comp_width: ::core::ffi::c_int,
    pub comp_height: ::core::ffi::c_int,
    pub comp_index: size_t,
    pub comp_disabled: bool,
    pub pending_comp_index_update: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GridView {
    pub target: *mut ScreenGrid,
    pub row_offset: ::core::ffi::c_int,
    pub col_offset: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct taggy_T {
    pub tagname: *mut ::core::ffi::c_char,
    pub fmark: fmark_T,
    pub cur_match: ::core::ffi::c_int,
    pub cur_fnum: ::core::ffi::c_int,
    pub user_data: *mut ::core::ffi::c_char,
}
pub type matchitem_T = matchitem;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct matchitem {
    pub mit_next: *mut matchitem_T,
    pub mit_id: ::core::ffi::c_int,
    pub mit_priority: ::core::ffi::c_int,
    pub mit_pattern: *mut ::core::ffi::c_char,
    pub mit_match: regmmatch_T,
    pub mit_pos_array: *mut llpos_T,
    pub mit_pos_count: ::core::ffi::c_int,
    pub mit_pos_cur: ::core::ffi::c_int,
    pub mit_toplnum: linenr_T,
    pub mit_botlnum: linenr_T,
    pub mit_hl: match_T,
    pub mit_hlg_id: ::core::ffi::c_int,
    pub mit_conceal_char: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct match_T {
    pub rm: regmmatch_T,
    pub buf: *mut buf_T,
    pub lnum: linenr_T,
    pub attr: ::core::ffi::c_int,
    pub attr_cur: ::core::ffi::c_int,
    pub first_lnum: linenr_T,
    pub startcol: colnr_T,
    pub endcol: colnr_T,
    pub is_addpos: bool,
    pub has_cursor: bool,
    pub tm: proftime_T,
}
pub type buf_T = file_buffer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmmatch_T {
    pub regprog: *mut regprog_T,
    pub startpos: [lpos_T; 10],
    pub endpos: [lpos_T; 10],
    pub rmm_matchcol: colnr_T,
    pub rmm_ic: ::core::ffi::c_int,
    pub rmm_maxcol: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct llpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub len: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct xfmark_T {
    pub fmark: fmark_T,
    pub fname: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wline_T {
    pub wl_lnum: linenr_T,
    pub wl_size: uint16_t,
    pub wl_valid: bool,
    pub wl_folded: bool,
    pub wl_foldend: linenr_T,
    pub wl_lastlnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_save_T {
    pub w_topline_save: ::core::ffi::c_int,
    pub w_topline_corr: ::core::ffi::c_int,
    pub w_cursor_save: pos_T,
    pub w_cursor_corr: pos_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fcs_chars_T {
    pub stl: schar_T,
    pub stlnc: schar_T,
    pub wbr: schar_T,
    pub horiz: schar_T,
    pub horizup: schar_T,
    pub horizdown: schar_T,
    pub vert: schar_T,
    pub vertleft: schar_T,
    pub vertright: schar_T,
    pub verthoriz: schar_T,
    pub fold: schar_T,
    pub foldopen: schar_T,
    pub foldclosed: schar_T,
    pub foldsep: schar_T,
    pub foldinner: schar_T,
    pub diff: schar_T,
    pub msgsep: schar_T,
    pub eob: schar_T,
    pub lastline: schar_T,
    pub trunc: schar_T,
    pub truncrl: schar_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lcs_chars_T {
    pub eol: schar_T,
    pub ext: schar_T,
    pub prec: schar_T,
    pub nbsp: schar_T,
    pub space: schar_T,
    pub tab1: schar_T,
    pub tab2: schar_T,
    pub tab3: schar_T,
    pub leadtab1: schar_T,
    pub leadtab2: schar_T,
    pub leadtab3: schar_T,
    pub lead: schar_T,
    pub trail: schar_T,
    pub multispace: *mut schar_T,
    pub leadmultispace: *mut schar_T,
    pub conceal: schar_T,
}
pub type frame_T = frame_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct frame_S {
    pub fr_layout: ::core::ffi::c_char,
    pub fr_width: ::core::ffi::c_int,
    pub fr_newwidth: ::core::ffi::c_int,
    pub fr_height: ::core::ffi::c_int,
    pub fr_newheight: ::core::ffi::c_int,
    pub fr_parent: *mut frame_T,
    pub fr_next: *mut frame_T,
    pub fr_prev: *mut frame_T,
    pub fr_child: *mut frame_T,
    pub fr_win: *mut win_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ChangedtickDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 12],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileID {
    pub inode: uint64_t,
    pub device_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct memline_T {
    pub ml_line_count: linenr_T,
    pub ml_mfp: *mut memfile_T,
    pub ml_stack: *mut infoptr_T,
    pub ml_stack_top: ::core::ffi::c_int,
    pub ml_stack_size: ::core::ffi::c_int,
    pub ml_flags: ::core::ffi::c_int,
    pub ml_line_textlen: colnr_T,
    pub ml_line_lnum: linenr_T,
    pub ml_line_ptr: *mut ::core::ffi::c_char,
    pub ml_line_offset: size_t,
    pub ml_line_offset_ff: ::core::ffi::c_int,
    pub ml_locked: *mut bhdr_T,
    pub ml_locked_low: linenr_T,
    pub ml_locked_high: linenr_T,
    pub ml_locked_lineadd: ::core::ffi::c_int,
    pub ml_chunksize: *mut chunksize_T,
    pub ml_numchunks: ::core::ffi::c_int,
    pub ml_usedchunks: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct chunksize_T {
    pub mlcs_numlines: ::core::ffi::c_int,
    pub mlcs_totalsize: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bhdr_T {
    pub bh_bnum: blocknr_T,
    pub bh_data: *mut ::core::ffi::c_void,
    pub bh_page_count: ::core::ffi::c_uint,
    pub bh_flags: ::core::ffi::c_uint,
}
pub type blocknr_T = int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct infoptr_T {
    pub ip_bnum: blocknr_T,
    pub ip_low: linenr_T,
    pub ip_high: linenr_T,
    pub ip_index: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct memfile_T {
    pub mf_fname: *mut ::core::ffi::c_char,
    pub mf_ffname: *mut ::core::ffi::c_char,
    pub mf_fd: ::core::ffi::c_int,
    pub mf_flags: ::core::ffi::c_int,
    pub mf_reopen: bool,
    pub mf_free_first: *mut bhdr_T,
    pub mf_hash: Map_int64_t_ptr_t,
    pub mf_trans: Map_int64_t_int64_t,
    pub mf_blocknr_max: blocknr_T,
    pub mf_blocknr_min: blocknr_T,
    pub mf_neg_count: blocknr_T,
    pub mf_infile_count: blocknr_T,
    pub mf_page_size: ::core::ffi::c_uint,
    pub mf_dirty: mfdirty_T,
}
pub type mfdirty_T = ::core::ffi::c_uint;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int64_t_int64_t {
    pub set: Set_int64_t,
    pub values: *mut int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_int64_t {
    pub h: MapHash,
    pub keys: *mut int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int64_t_ptr_t {
    pub set: Set_int64_t,
    pub values: *mut ptr_t,
}
pub type uvarnumber_T = uint64_t;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const DO_NOT_FREE_CNT: C2Rust_Unnamed_13 = 1073741823;
pub type ListLenSpecials = ::core::ffi::c_int;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DictWatcher {
    pub callback: Callback,
    pub key_pattern: *mut ::core::ffi::c_char,
    pub key_pattern_len: size_t,
    pub node: QUEUE,
    pub busy: bool,
    pub needs_free: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct staticList10_T {
    pub sl_list: list_T,
    pub sl_items: [listitem_T; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const DI_FLAGS_ALLOC: C2Rust_Unnamed_14 = 16;
pub const DI_FLAGS_LOCK: C2Rust_Unnamed_14 = 8;
pub const DI_FLAGS_FIX: C2Rust_Unnamed_14 = 4;
pub const DI_FLAGS_RO_SBX: C2Rust_Unnamed_14 = 2;
pub const DI_FLAGS_RO: C2Rust_Unnamed_14 = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_15 = 65;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const STR2NR_QUOTE: C2Rust_Unnamed_16 = 16;
pub const STR2NR_NO_OCT: C2Rust_Unnamed_16 = 13;
pub const STR2NR_ALL: C2Rust_Unnamed_16 = 15;
pub const STR2NR_FORCE: C2Rust_Unnamed_16 = 128;
pub const STR2NR_OOCT: C2Rust_Unnamed_16 = 8;
pub const STR2NR_HEX: C2Rust_Unnamed_16 = 4;
pub const STR2NR_OCT: C2Rust_Unnamed_16 = 2;
pub const STR2NR_BIN: C2Rust_Unnamed_16 = 1;
pub const STR2NR_DEC: C2Rust_Unnamed_16 = 0;
pub type MessagePackType = ::core::ffi::c_uint;
pub const kMPExt: MessagePackType = 7;
pub const kMPMap: MessagePackType = 6;
pub const kMPArray: MessagePackType = 5;
pub const kMPString: MessagePackType = 4;
pub const kMPFloat: MessagePackType = 3;
pub const kMPInteger: MessagePackType = 2;
pub const kMPBoolean: MessagePackType = 1;
pub const kMPNil: MessagePackType = 0;
pub type iconv_t = *mut ::core::ffi::c_void;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const CONV_ICONV: C2Rust_Unnamed_17 = 5;
pub const CONV_TO_LATIN9: C2Rust_Unnamed_17 = 4;
pub const CONV_TO_LATIN1: C2Rust_Unnamed_17 = 3;
pub const CONV_9_TO_UTF8: C2Rust_Unnamed_17 = 2;
pub const CONV_TO_UTF8: C2Rust_Unnamed_17 = 1;
pub const CONV_NONE: C2Rust_Unnamed_17 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimconv_T {
    pub vc_type: ::core::ffi::c_int,
    pub vc_factor: ::core::ffi::c_int,
    pub vc_fd: iconv_t,
    pub vc_fail: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MPConvStackVal {
    pub type_0: MPConvStackValType,
    pub tv: *mut typval_T,
    pub saved_copyID: ::core::ffi::c_int,
    pub data: C2Rust_Unnamed_18,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_18 {
    pub d: C2Rust_Unnamed_22,
    pub l: C2Rust_Unnamed_21,
    pub p: C2Rust_Unnamed_20,
    pub a: C2Rust_Unnamed_19,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_19 {
    pub arg: *mut typval_T,
    pub argv: *mut typval_T,
    pub todo: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_20 {
    pub stage: MPConvPartialStage,
    pub pt: *mut partial_T,
}
pub type MPConvPartialStage = ::core::ffi::c_uint;
pub const kMPConvPartialEnd: MPConvPartialStage = 2;
pub const kMPConvPartialSelf: MPConvPartialStage = 1;
pub const kMPConvPartialArgs: MPConvPartialStage = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
    pub list: *mut list_T,
    pub li: *mut listitem_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_22 {
    pub dict: *mut dict_T,
    pub dictp: *mut *mut dict_T,
    pub hi: *mut hashitem_T,
    pub todo: size_t,
}
pub type MPConvStackValType = ::core::ffi::c_uint;
pub const kMPConvPartialList: MPConvStackValType = 4;
pub const kMPConvPartial: MPConvStackValType = 3;
pub const kMPConvPairs: MPConvStackValType = 2;
pub const kMPConvList: MPConvStackValType = 1;
pub const kMPConvDict: MPConvStackValType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MPConvStack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MPConvStackVal,
    pub init_array: [MPConvStackVal; 8],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Join {
    pub s: String_0,
    pub tofree: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sortinfo_T {
    pub item_compare_ic: ::core::ffi::c_int,
    pub item_compare_lc: bool,
    pub item_compare_numeric: bool,
    pub item_compare_numbers: bool,
    pub item_compare_float: bool,
    pub item_compare_func: *const ::core::ffi::c_char,
    pub item_compare_partial: *mut partial_T,
    pub item_compare_selfdict: *mut dict_T,
    pub item_compare_func_err: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ListSortItem {
    pub item: *mut listitem_T,
    pub idx: ::core::ffi::c_int,
}
pub type ListSorter = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct funcexe_T {
    pub fe_argv_func: ArgvFunc,
    pub fe_firstline: linenr_T,
    pub fe_lastline: linenr_T,
    pub fe_doesrange: *mut bool,
    pub fe_evaluate: bool,
    pub fe_partial: *mut partial_T,
    pub fe_selfdict: *mut dict_T,
    pub fe_basetv: *mut typval_T,
    pub fe_found_var: bool,
}
pub type ArgvFunc = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut typval_T,
        ::core::ffi::c_int,
        *mut ufunc_T,
    ) -> ::core::ffi::c_int,
>;
pub type DictListType = ::core::ffi::c_uint;
pub const kDict2ListItems: DictListType = 2;
pub const kDict2ListValues: DictListType = 1;
pub const kDict2ListKeys: DictListType = 0;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 58] = unsafe {
    ::core::mem::transmute::<[u8; 58], [::core::ffi::c_char; 58]>(
        *b"void tv_list_set_lock(list_T *const, const VarLockStatus)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INT8_MIN: ::core::ffi::c_int = -128 as ::core::ffi::c_int;
pub const INT8_MAX: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return dest;
}
#[inline(always)]
unsafe extern "C" fn QUEUE_EMPTY(q: *const QUEUE) -> ::core::ffi::c_int {
    return (q == (*q).next as *const QUEUE) as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn QUEUE_INIT(q: *mut QUEUE) {
    (*q).next = q as *mut queue;
    (*q).prev = q as *mut queue;
}
#[inline(always)]
unsafe extern "C" fn QUEUE_INSERT_TAIL(h: *mut QUEUE, q: *mut QUEUE) {
    (*q).next = h as *mut queue;
    (*q).prev = (*h).prev;
    (*(*q).prev).next = q as *mut queue;
    (*h).prev = q as *mut queue;
}
#[inline(always)]
unsafe extern "C" fn QUEUE_REMOVE(q: *mut QUEUE) {
    (*(*q).prev).next = (*q).next;
    (*(*q).next).prev = (*q).prev;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
}
#[inline(always)]
unsafe extern "C" fn tv_list_set_ret(tv: *mut typval_T, l: *mut list_T) {
    (*tv).v_type = VAR_LIST;
    (*tv).vval.v_list = l;
    tv_list_ref(l);
}
#[inline]
unsafe extern "C" fn tv_list_locked(l: *const list_T) -> VarLockStatus {
    if l.is_null() {
        return VAR_FIXED;
    }
    return (*l).lv_lock;
}
#[inline]
unsafe extern "C" fn tv_list_set_lock(l: *mut list_T, lock: VarLockStatus) {
    if l.is_null() {
        '_c2rust_label: {
            if lock as ::core::ffi::c_uint == VAR_FIXED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"lock == VAR_FIXED\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    76 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        return;
    }
    (*l).lv_lock = lock;
}
#[inline]
unsafe extern "C" fn tv_list_set_copyid(l: *mut list_T, copyid: ::core::ffi::c_int) {
    (*l).lv_copyID = copyid;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_copyid(l: *const list_T) -> ::core::ffi::c_int {
    return (*l).lv_copyID;
}
#[inline]
unsafe extern "C" fn tv_list_uidx(
    l: *const list_T,
    mut n: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if n < 0 as ::core::ffi::c_int {
        n += tv_list_len(l);
    }
    if n < 0 as ::core::ffi::c_int || n >= tv_list_len(l) {
        return -1 as ::core::ffi::c_int;
    }
    return n;
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
#[inline]
unsafe extern "C" fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_last;
}
#[inline(always)]
unsafe extern "C" fn tv_dict_set_ret(tv: *mut typval_T, d: *mut dict_T) {
    (*tv).v_type = VAR_DICT;
    (*tv).vval.v_dict = d;
    if !d.is_null() {
        (*d).dv_refcount += 1;
    }
}
#[inline]
unsafe extern "C" fn tv_dict_len(d: *const dict_T) -> ::core::ffi::c_long {
    if d.is_null() {
        return 0 as ::core::ffi::c_long;
    }
    return (*d).dv_hashtab.ht_used as ::core::ffi::c_long;
}
#[inline]
unsafe extern "C" fn tv_dict_is_watched(d: *const dict_T) -> bool {
    return !d.is_null() && QUEUE_EMPTY(&raw const (*d).watchers) == 0;
}
#[inline(always)]
unsafe extern "C" fn tv_blob_set_ret(tv: *mut typval_T, b: *mut blob_T) {
    (*tv).v_type = VAR_BLOB;
    (*tv).vval.v_blob = b;
    if !b.is_null() {
        (*b).bv_refcount += 1;
    }
}
#[inline]
unsafe extern "C" fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    if b.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*b).bv_ga.ga_len;
}
#[inline(always)]
unsafe extern "C" fn tv_blob_get(b: *const blob_T, mut idx: ::core::ffi::c_int) -> uint8_t {
    return *((*b).bv_ga.ga_data as *mut uint8_t).offset(idx as isize);
}
#[inline(always)]
unsafe extern "C" fn tv_blob_set(blob: *mut blob_T, mut idx: ::core::ffi::c_int, mut c: uint8_t) {
    *((*blob).bv_ga.ga_data as *mut uint8_t).offset(idx as isize) = c;
}
#[inline(always)]
unsafe extern "C" fn tv_dict_watcher_node_data(mut q: *mut QUEUE) -> *mut DictWatcher {
    return (q as *mut ::core::ffi::c_char).offset(-(32 as ::core::ffi::c_ulong as isize))
        as *mut DictWatcher;
}
#[inline(always)]
unsafe extern "C" fn tv_is_func(tv: typval_T) -> bool {
    return tv.v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv.v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint;
}
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX.wrapping_sub(1 as ::core::ffi::c_ulong);
static e_variable_nested_too_deep_for_unlock: GlobalCell<[::core::ffi::c_char; 44]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
            *b"E743: Variable nested too deep for (un)lock\0",
        )
    });
static e_using_invalid_value_as_string: GlobalCell<[::core::ffi::c_char; 41]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
            *b"E908: Using an invalid value as a String\0",
        )
    });
static e_string_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
            *b"E1174: String required for argument %d\0",
        )
    });
static e_non_empty_string_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E1175: Non-empty string required for argument %d\0",
        )
    });
static e_dict_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 43]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
            *b"E1206: Dictionary required for argument %d\0",
        )
    });
static e_number_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
            *b"E1210: Number required for argument %d\0",
        )
    });
static e_list_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E1211: List required for argument %d\0",
        )
    });
static e_bool_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E1212: Bool required for argument %d\0",
        )
    });
static e_float_or_number_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 48]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
            *b"E1219: Float or Number required for argument %d\0",
        )
    });
static e_string_or_number_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E1220: String or Number required for argument %d\0",
        )
    });
static e_string_or_list_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 47]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
            *b"E1222: String or List required for argument %d\0",
        )
    });
static e_list_dict_blob_or_string_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 65]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
            *b"E1225: List, Dictionary, Blob or String required for argument %d\0",
        )
    });
static e_list_or_blob_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 45]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
            *b"E1226: List or Blob required for argument %d\0",
        )
    });
static e_blob_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E1238: Blob required for argument %d\0",
        )
    });
static e_string_list_or_blob_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 53]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 53], [::core::ffi::c_char; 53]>(
            *b"E1252: String, List or Blob required for argument %d\0",
        )
    });
static e_string_or_function_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 51]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 51], [::core::ffi::c_char; 51]>(
            *b"E1256: String or function required for argument %d\0",
        )
    });
static e_non_null_dict_required_for_argument_nr: GlobalCell<[::core::ffi::c_char; 52]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 52], [::core::ffi::c_char; 52]>(
            *b"E1297: Non-NULL Dictionary required for argument %d\0",
        )
    });
#[no_mangle]
pub static tv_in_free_unref_items: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const DICT_MAXNEST: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
#[no_mangle]
pub static tv_empty_string: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"\0".as_ptr() as *const ::core::ffi::c_char);
unsafe extern "C" fn tv_list_item_alloc() -> *mut listitem_T {
    return xmalloc(::core::mem::size_of::<listitem_T>()) as *mut listitem_T;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_item_remove(
    l: *mut list_T,
    item: *mut listitem_T,
) -> *mut listitem_T {
    let next_item: *mut listitem_T = (*item).li_next;
    tv_list_drop_items(l, item, item);
    tv_clear(&raw mut (*item).li_tv);
    xfree(item as *mut ::core::ffi::c_void);
    return next_item;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_watch_add(l: *mut list_T, lw: *mut listwatch_T) {
    (*lw).lw_next = (*l).lv_watch;
    (*l).lv_watch = lw;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_watch_remove(l: *mut list_T, lwrem: *mut listwatch_T) {
    let mut lwp: *mut *mut listwatch_T = &raw mut (*l).lv_watch;
    let mut lw: *mut listwatch_T = (*l).lv_watch;
    while !lw.is_null() {
        if lw == lwrem {
            *lwp = (*lw).lw_next;
            break;
        } else {
            lwp = &raw mut (*lw).lw_next;
            lw = (*lw).lw_next;
        }
    }
}
unsafe extern "C" fn tv_list_watch_fix(l: *mut list_T, item: *const listitem_T) {
    let mut lw: *mut listwatch_T = (*l).lv_watch;
    while !lw.is_null() {
        if (*lw).lw_item == item as *mut listitem_T {
            (*lw).lw_item = (*item).li_next;
        }
        lw = (*lw).lw_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_alloc(_len: ptrdiff_t) -> *mut list_T {
    let list: *mut list_T = xcalloc(1 as size_t, ::core::mem::size_of::<list_T>()) as *mut list_T;
    if !(*gc_first_list.ptr()).is_null() {
        (*gc_first_list.get()).lv_used_prev = list;
    }
    (*list).lv_used_prev = ::core::ptr::null_mut::<list_T>();
    (*list).lv_used_next = gc_first_list.get();
    gc_first_list.set(list);
    (*list).lua_table_ref = LUA_NOREF as LuaRef;
    return list;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_init_static10(sl: *mut staticList10_T) {
    let l: *mut list_T = &raw mut (*sl).sl_list;
    memset(
        sl as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<staticList10_T>(),
    );
    (*l).lv_first =
        (&raw mut (*sl).sl_items as *mut listitem_T).offset(0 as ::core::ffi::c_int as isize);
    (*l).lv_last = (&raw mut (*sl).sl_items as *mut listitem_T)
        .offset(SL_SIZE.wrapping_sub(1 as usize) as isize);
    (*l).lv_refcount = DO_NOT_FREE_CNT as ::core::ffi::c_int;
    tv_list_set_lock(l, VAR_FIXED);
    (*sl).sl_list.lv_len = 10 as ::core::ffi::c_int;
    (*sl).sl_items[0 as ::core::ffi::c_int as usize].li_prev =
        ::core::ptr::null_mut::<listitem_T>();
    (*sl).sl_items[0 as ::core::ffi::c_int as usize].li_next =
        (&raw mut (*sl).sl_items as *mut listitem_T).offset(1 as ::core::ffi::c_int as isize);
    (*sl).sl_items[SL_SIZE.wrapping_sub(1 as usize) as usize].li_prev = (&raw mut (*sl).sl_items
        as *mut listitem_T)
        .offset(SL_SIZE.wrapping_sub(2 as usize) as isize);
    (*sl).sl_items[SL_SIZE.wrapping_sub(1 as usize) as usize].li_next =
        ::core::ptr::null_mut::<listitem_T>();
    let mut i: size_t = 1 as size_t;
    while i < SL_SIZE.wrapping_sub(1 as usize) {
        let li: *mut listitem_T = (&raw mut (*sl).sl_items as *mut listitem_T).offset(i as isize);
        (*li).li_prev = li.offset(-(1 as ::core::ffi::c_int as isize));
        (*li).li_next = li.offset(1 as ::core::ffi::c_int as isize);
        i = i.wrapping_add(1);
    }
}
pub const SL_SIZE: usize = ::core::mem::size_of::<[listitem_T; 10]>()
    .wrapping_div(::core::mem::size_of::<listitem_T>())
    .wrapping_div(
        (::core::mem::size_of::<[listitem_T; 10]>()
            .wrapping_rem(::core::mem::size_of::<listitem_T>())
            == 0) as ::core::ffi::c_int as usize,
    );
#[no_mangle]
pub unsafe extern "C" fn tv_list_init_static(l: *mut list_T) {
    memset(
        l as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<list_T>(),
    );
    (*l).lv_refcount = DO_NOT_FREE_CNT as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_free_contents(l: *mut list_T) {
    let mut item: *mut listitem_T = (*l).lv_first;
    while !item.is_null() {
        (*l).lv_first = (*item).li_next;
        tv_clear(&raw mut (*item).li_tv);
        xfree(item as *mut ::core::ffi::c_void);
        item = (*l).lv_first;
    }
    (*l).lv_len = 0 as ::core::ffi::c_int;
    (*l).lv_idx_item = ::core::ptr::null_mut::<listitem_T>();
    (*l).lv_last = ::core::ptr::null_mut::<listitem_T>();
    '_c2rust_label: {
        if (*l).lv_watch.is_null() {
        } else {
            __assert_fail(
                b"l->lv_watch == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                282 as ::core::ffi::c_uint,
                b"void tv_list_free_contents(list_T *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_free_list(l: *mut list_T) {
    if (*l).lv_used_prev.is_null() {
        gc_first_list.set((*l).lv_used_next);
    } else {
        (*(*l).lv_used_prev).lv_used_next = (*l).lv_used_next;
    }
    if !(*l).lv_used_next.is_null() {
        (*(*l).lv_used_next).lv_used_prev = (*l).lv_used_prev;
    }
    if (*l).lua_table_ref != LUA_NOREF {
        api_free_luaref((*l).lua_table_ref);
        (*l).lua_table_ref = LUA_NOREF as LuaRef;
    }
    xfree(l as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_free(l: *mut list_T) {
    if tv_in_free_unref_items.get() {
        return;
    }
    tv_list_free_contents(l);
    tv_list_free_list(l);
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_unref(l: *mut list_T) {
    if !l.is_null() && {
        (*l).lv_refcount -= 1;
        (*l).lv_refcount <= 0 as ::core::ffi::c_int
    } {
        tv_list_free(l);
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_drop_items(
    l: *mut list_T,
    item: *mut listitem_T,
    item2: *mut listitem_T,
) {
    let mut ip: *mut listitem_T = item;
    while ip != (*item2).li_next {
        (*l).lv_len -= 1;
        tv_list_watch_fix(l, ip);
        ip = (*ip).li_next;
    }
    if (*item2).li_next.is_null() {
        (*l).lv_last = (*item).li_prev;
    } else {
        (*(*item2).li_next).li_prev = (*item).li_prev;
    }
    if (*item).li_prev.is_null() {
        (*l).lv_first = (*item2).li_next;
    } else {
        (*(*item).li_prev).li_next = (*item2).li_next;
    }
    (*l).lv_idx_item = ::core::ptr::null_mut::<listitem_T>();
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_remove_items(
    l: *mut list_T,
    item: *mut listitem_T,
    item2: *mut listitem_T,
) {
    tv_list_drop_items(l, item, item2);
    let mut li: *mut listitem_T = item;
    loop {
        tv_clear(&raw mut (*li).li_tv);
        let nli: *mut listitem_T = (*li).li_next;
        xfree(li as *mut ::core::ffi::c_void);
        if li == item2 {
            break;
        }
        li = nli;
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_move_items(
    l: *mut list_T,
    item: *mut listitem_T,
    item2: *mut listitem_T,
    tgt_l: *mut list_T,
    cnt: ::core::ffi::c_int,
) {
    tv_list_drop_items(l, item, item2);
    (*item).li_prev = (*tgt_l).lv_last;
    (*item2).li_next = ::core::ptr::null_mut::<listitem_T>();
    if (*tgt_l).lv_last.is_null() {
        (*tgt_l).lv_first = item;
    } else {
        (*(*tgt_l).lv_last).li_next = item;
    }
    (*tgt_l).lv_last = item2;
    (*tgt_l).lv_len += cnt;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_insert(
    l: *mut list_T,
    ni: *mut listitem_T,
    item: *mut listitem_T,
) {
    if item.is_null() {
        tv_list_append(l, ni);
    } else {
        (*ni).li_prev = (*item).li_prev;
        (*ni).li_next = item;
        if (*item).li_prev.is_null() {
            (*l).lv_first = ni;
            (*l).lv_idx += 1;
        } else {
            (*(*item).li_prev).li_next = ni;
            (*l).lv_idx_item = ::core::ptr::null_mut::<listitem_T>();
        }
        (*item).li_prev = ni;
        (*l).lv_len += 1;
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_insert_tv(
    l: *mut list_T,
    tv: *mut typval_T,
    item: *mut listitem_T,
) {
    let ni: *mut listitem_T = tv_list_item_alloc();
    tv_copy(tv, &raw mut (*ni).li_tv);
    tv_list_insert(l, ni, item);
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_append(l: *mut list_T, item: *mut listitem_T) {
    if (*l).lv_last.is_null() {
        (*l).lv_first = item;
        (*l).lv_last = item;
        (*item).li_prev = ::core::ptr::null_mut::<listitem_T>();
    } else {
        (*(*l).lv_last).li_next = item;
        (*item).li_prev = (*l).lv_last;
        (*l).lv_last = item;
    }
    (*l).lv_len += 1;
    (*item).li_next = ::core::ptr::null_mut::<listitem_T>();
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_append_tv(l: *mut list_T, tv: *mut typval_T) {
    let li: *mut listitem_T = tv_list_item_alloc();
    tv_copy(tv, &raw mut (*li).li_tv);
    tv_list_append(l, li);
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_append_owned_tv(
    l: *mut list_T,
    mut tv: typval_T,
) -> *mut typval_T {
    let li: *mut listitem_T = tv_list_item_alloc();
    (*li).li_tv = tv;
    tv_list_append(l, li);
    return &raw mut (*li).li_tv;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_append_list(l: *mut list_T, itemlist: *mut list_T) {
    tv_list_append_owned_tv(
        l,
        typval_T {
            v_type: VAR_LIST,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_list: itemlist },
        },
    );
    tv_list_ref(itemlist);
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_append_dict(l: *mut list_T, dict: *mut dict_T) {
    tv_list_append_owned_tv(
        l,
        typval_T {
            v_type: VAR_DICT,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_dict: dict },
        },
    );
    if !dict.is_null() {
        (*dict).dv_refcount += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_append_string(
    l: *mut list_T,
    str: *const ::core::ffi::c_char,
    len: ssize_t,
) {
    tv_list_append_owned_tv(
        l,
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: (if str.is_null() {
                    NULL_0
                } else if len >= 0 as ssize_t {
                    xmemdupz(str as *const ::core::ffi::c_void, len as size_t)
                } else {
                    xstrdup(str) as *mut ::core::ffi::c_void
                }) as *mut ::core::ffi::c_char,
            },
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_append_allocated_string(
    l: *mut list_T,
    str: *mut ::core::ffi::c_char,
) {
    tv_list_append_owned_tv(
        l,
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_string: str },
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_append_number(l: *mut list_T, n: varnumber_T) {
    tv_list_append_owned_tv(
        l,
        typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: n },
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_copy(
    conv: *const vimconv_T,
    orig: *mut list_T,
    deep: bool,
    copyID: ::core::ffi::c_int,
) -> *mut list_T {
    if orig.is_null() {
        return ::core::ptr::null_mut::<list_T>();
    }
    let mut copy: *mut list_T = tv_list_alloc(tv_list_len(orig) as ptrdiff_t);
    tv_list_ref(copy);
    if copyID != 0 as ::core::ffi::c_int {
        (*orig).lv_copyID = copyID;
        (*orig).lv_copylist = copy;
    }
    let l_: *mut list_T = orig;
    's_99: {
        if !l_.is_null() {
            let mut item: *mut listitem_T = (*l_).lv_first;
            loop {
                if item.is_null() {
                    break 's_99;
                }
                if got_int.get() {
                    break 's_99;
                }
                let ni: *mut listitem_T = tv_list_item_alloc();
                if deep {
                    if var_item_copy(
                        conv,
                        &raw mut (*item).li_tv,
                        &raw mut (*ni).li_tv,
                        deep,
                        copyID,
                    ) == 0 as ::core::ffi::c_int
                    {
                        xfree(ni as *mut ::core::ffi::c_void);
                        break;
                    }
                } else {
                    tv_copy(&raw mut (*item).li_tv, &raw mut (*ni).li_tv);
                }
                tv_list_append(copy, ni);
                item = (*item).li_next;
            }
            tv_list_unref(copy);
            return ::core::ptr::null_mut::<list_T>();
        }
    }
    return copy;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_check_range_index_one(
    l: *mut list_T,
    n1: *mut ::core::ffi::c_int,
    quiet: bool,
) -> *mut listitem_T {
    let mut li: *mut listitem_T = tv_list_find_index(l, n1);
    if !li.is_null() {
        return li;
    }
    if !quiet {
        semsg(
            gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
            *n1 as int64_t,
        );
    }
    return ::core::ptr::null_mut::<listitem_T>();
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_check_range_index_two(
    l: *mut list_T,
    n1: *mut ::core::ffi::c_int,
    li1: *const listitem_T,
    n2: *mut ::core::ffi::c_int,
    quiet: bool,
) -> ::core::ffi::c_int {
    if *n2 < 0 as ::core::ffi::c_int {
        let mut ni: *mut listitem_T = tv_list_find(l, *n2);
        if ni.is_null() {
            if !quiet {
                semsg(
                    gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
                    *n2 as int64_t,
                );
            }
            return FAIL;
        }
        *n2 = tv_list_idx_of_item(l, ni);
    }
    if *n1 < 0 as ::core::ffi::c_int {
        *n1 = tv_list_idx_of_item(l, li1);
    }
    if *n2 < *n1 {
        if !quiet {
            semsg(
                gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
                *n2 as int64_t,
            );
        }
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_assign_range(
    dest: *mut list_T,
    src: *mut list_T,
    idx1_arg: ::core::ffi::c_int,
    idx2: ::core::ffi::c_int,
    empty_idx2: bool,
    op: *const ::core::ffi::c_char,
    varname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut idx1: ::core::ffi::c_int = idx1_arg;
    let first_li: *mut listitem_T = tv_list_find_index(dest, &raw mut idx1);
    let mut src_li: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    let mut idx: ::core::ffi::c_int = idx1;
    let mut dest_li: *mut listitem_T = first_li;
    src_li = tv_list_first(src);
    while !src_li.is_null() && !dest_li.is_null() {
        if value_check_lock((*dest_li).li_tv.v_lock, varname, TV_CSTRING as size_t) {
            return FAIL;
        }
        src_li = (*src_li).li_next;
        if src_li.is_null() || !empty_idx2 && idx2 == idx {
            break;
        }
        dest_li = (*dest_li).li_next;
        idx += 1;
    }
    idx = idx1;
    dest_li = first_li;
    src_li = tv_list_first(src);
    while !src_li.is_null() {
        '_c2rust_label: {
            if !dest_li.is_null() {
            } else {
                __assert_fail(
                    b"dest_li != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    710 as ::core::ffi::c_uint,
                    b"int tv_list_assign_range(list_T *const, list_T *const, const int, const int, const _Bool, const char *const, const char *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if !op.is_null() && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
            eexe_mod_op(&raw mut (*dest_li).li_tv, &raw mut (*src_li).li_tv, op);
        } else {
            tv_clear(&raw mut (*dest_li).li_tv);
            tv_copy(&raw mut (*src_li).li_tv, &raw mut (*dest_li).li_tv);
        }
        src_li = (*src_li).li_next;
        if src_li.is_null() || !empty_idx2 && idx2 == idx {
            break;
        }
        if (*dest_li).li_next.is_null() {
            tv_list_append_number(dest, 0 as varnumber_T);
            dest_li = tv_list_last(dest);
        } else {
            dest_li = (*dest_li).li_next;
        }
        idx += 1;
    }
    if !src_li.is_null() {
        emsg(gettext(
            b"E710: List value has more items than target\0".as_ptr() as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if if empty_idx2 as ::core::ffi::c_int != 0 {
        (!dest_li.is_null() && !(*dest_li).li_next.is_null()) as ::core::ffi::c_int
    } else {
        (idx != idx2) as ::core::ffi::c_int
    } != 0
    {
        emsg(gettext(
            b"E711: List value has not enough items\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_flatten(
    mut list: *mut list_T,
    mut first: *mut listitem_T,
    mut maxitems: int64_t,
    mut maxdepth: int64_t,
) {
    let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if maxdepth == 0 as int64_t {
        return;
    }
    if first.is_null() {
        item = (*list).lv_first;
    } else {
        item = first;
    }
    while !item.is_null() && (done as int64_t) < maxitems {
        let mut next: *mut listitem_T = (*item).li_next;
        fast_breakcheck();
        if got_int.get() {
            return;
        }
        if (*item).li_tv.v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut itemlist: *mut list_T = (*item).li_tv.vval.v_list;
            tv_list_drop_items(list, item, item);
            tv_list_extend(list, itemlist, next);
            if maxdepth > 0 as int64_t {
                tv_list_flatten(
                    list,
                    if (*item).li_prev.is_null() {
                        (*list).lv_first
                    } else {
                        (*(*item).li_prev).li_next
                    },
                    (*itemlist).lv_len as int64_t,
                    maxdepth - 1 as int64_t,
                );
            }
            tv_clear(&raw mut (*item).li_tv);
            xfree(item as *mut ::core::ffi::c_void);
        }
        done += 1;
        item = next;
    }
}
unsafe extern "C" fn tv_blob2items(mut argvars: *mut typval_T, mut rettv: *mut typval_T) {
    let mut blob: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_blob;
    tv_list_alloc_ret(rettv, tv_blob_len(blob) as ptrdiff_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < tv_blob_len(blob) {
        let mut l2: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
        tv_list_append_list((*rettv).vval.v_list, l2);
        tv_list_append_number(l2, i as varnumber_T);
        tv_list_append_number(l2, tv_blob_get(blob, i) as varnumber_T);
        i += 1;
    }
}
unsafe extern "C" fn tv_dict2items(mut argvars: *mut typval_T, mut rettv: *mut typval_T) {
    tv_dict2list(argvars, rettv, kDict2ListItems);
}
unsafe extern "C" fn tv_list2items(mut argvars: *mut typval_T, mut rettv: *mut typval_T) {
    let mut l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    tv_list_alloc_ret(rettv, tv_list_len(l) as ptrdiff_t);
    if l.is_null() {
        return;
    }
    let mut idx: varnumber_T = 0 as varnumber_T;
    let l_: *mut list_T = l;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut l2: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
            tv_list_append_list((*rettv).vval.v_list, l2);
            tv_list_append_number(l2, idx);
            tv_list_append_tv(l2, &raw mut (*li).li_tv);
            idx += 1;
            li = (*li).li_next;
        }
    }
}
unsafe extern "C" fn tv_string2items(mut argvars: *mut typval_T, mut rettv: *mut typval_T) {
    let mut p: *const ::core::ffi::c_char = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_string;
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if p.is_null() {
        return;
    }
    let mut idx: varnumber_T = 0 as varnumber_T;
    while *p as ::core::ffi::c_int != NUL {
        let mut len: ::core::ffi::c_int = utfc_ptr2len(p);
        if len == 0 as ::core::ffi::c_int {
            break;
        }
        let mut l2: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
        tv_list_append_list((*rettv).vval.v_list, l2);
        tv_list_append_number(l2, idx);
        tv_list_append_string(l2, p, len as ssize_t);
        p = p.offset(len as isize);
        idx += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_extend(l1: *mut list_T, l2: *mut list_T, bef: *mut listitem_T) {
    let mut todo: ::core::ffi::c_int = tv_list_len(l2);
    let befbef: *mut listitem_T = if bef.is_null() {
        ::core::ptr::null_mut::<listitem_T>()
    } else {
        (*bef).li_prev
    };
    let saved_next: *mut listitem_T = if befbef.is_null() {
        ::core::ptr::null_mut::<listitem_T>()
    } else {
        (*befbef).li_next
    };
    let mut item: *mut listitem_T = tv_list_first(l2);
    while !item.is_null() && {
        let c2rust_fresh8 = todo;
        todo = todo - 1;
        c2rust_fresh8 != 0
    } {
        tv_list_insert_tv(l1, &raw mut (*item).li_tv, bef);
        item = if item == befbef {
            saved_next
        } else {
            (*item).li_next
        };
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_concat(
    l1: *mut list_T,
    l2: *mut list_T,
    tv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    (*tv).v_type = VAR_LIST;
    (*tv).v_lock = VAR_UNLOCKED;
    if l1.is_null() && l2.is_null() {
        l = ::core::ptr::null_mut::<list_T>();
    } else if l1.is_null() {
        l = tv_list_copy(
            ::core::ptr::null::<vimconv_T>(),
            l2,
            false_0 != 0,
            0 as ::core::ffi::c_int,
        );
    } else {
        l = tv_list_copy(
            ::core::ptr::null::<vimconv_T>(),
            l1,
            false_0 != 0,
            0 as ::core::ffi::c_int,
        );
        if !l.is_null() && !l2.is_null() {
            tv_list_extend(l, l2, ::core::ptr::null_mut::<listitem_T>());
        }
    }
    if l.is_null() && !(l1.is_null() && l2.is_null()) {
        return FAIL;
    }
    (*tv).vval.v_list = l;
    return OK;
}
unsafe extern "C" fn tv_list_slice(
    mut ol: *mut list_T,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
) -> *mut list_T {
    let mut l: *mut list_T = tv_list_alloc(n2 as ptrdiff_t - n1 as ptrdiff_t + 1 as ptrdiff_t);
    let mut item: *mut listitem_T = tv_list_find(ol, n1 as ::core::ffi::c_int);
    while n1 <= n2 {
        tv_list_append_tv(l, &raw mut (*item).li_tv);
        item = (*item).li_next;
        n1 += 1;
    }
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_slice_or_index(
    mut _list: *mut list_T,
    mut range: bool,
    mut n1_arg: varnumber_T,
    mut n2_arg: varnumber_T,
    mut exclusive: bool,
    mut rettv: *mut typval_T,
    mut verbose: bool,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = tv_list_len((*rettv).vval.v_list);
    let mut n1: varnumber_T = n1_arg;
    let mut n2: varnumber_T = n2_arg;
    if n1 < 0 as varnumber_T {
        n1 = len as varnumber_T + n1;
    }
    if n1 < 0 as varnumber_T || n1 >= len as varnumber_T {
        if !range {
            if verbose {
                semsg(
                    gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
                    n1_arg,
                );
            }
            return FAIL;
        }
        n1 = len as varnumber_T;
    }
    if range {
        if n2 < 0 as varnumber_T {
            n2 = len as varnumber_T + n2;
        } else if n2 >= len as varnumber_T {
            n2 = (len
                - (if exclusive as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                })) as varnumber_T;
        }
        if exclusive {
            n2 -= 1;
        }
        if n2 < 0 as varnumber_T || (n2 + 1 as varnumber_T) < n1 {
            n2 = -1 as varnumber_T;
        }
        let mut l: *mut list_T = tv_list_slice((*rettv).vval.v_list, n1, n2);
        tv_clear(rettv);
        tv_list_set_ret(rettv, l);
    } else {
        let mut var1: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv_copy(
            &raw mut (*(tv_list_find
                as unsafe extern "C" fn(*mut list_T, ::core::ffi::c_int) -> *mut listitem_T)(
                (*rettv).vval.v_list,
                n1 as ::core::ffi::c_int,
            ))
            .li_tv,
            &raw mut var1,
        );
        tv_clear(rettv);
        *rettv = var1;
    }
    return OK;
}
unsafe extern "C" fn list_join_inner(
    gap: *mut garray_T,
    l: *mut list_T,
    sep: *const ::core::ffi::c_char,
    join_gap: *mut garray_T,
) -> ::core::ffi::c_int {
    let mut sumlen: size_t = 0 as size_t;
    let mut first: bool = true_0 != 0;
    let l_: *mut list_T = l;
    if !l_.is_null() {
        let mut item: *mut listitem_T = (*l_).lv_first;
        while !item.is_null() {
            if got_int.get() {
                break;
            }
            let mut s: String_0 = String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            };
            s.data = encode_tv2echo(&raw mut (*item).li_tv, &raw mut s.size);
            if s.data.is_null() {
                return 0 as ::core::ffi::c_int;
            }
            sumlen = sumlen.wrapping_add(s.size);
            let p: *mut Join =
                ga_append_via_ptr(join_gap, ::core::mem::size_of::<Join>()) as *mut Join;
            (*p).s = s;
            (*p).tofree = s.data;
            line_breakcheck();
            item = (*item).li_next;
        }
    }
    let mut seplen: size_t = strlen(sep);
    if (*join_gap).ga_len >= 2 as ::core::ffi::c_int {
        sumlen = sumlen.wrapping_add(
            seplen.wrapping_mul(((*join_gap).ga_len - 1 as ::core::ffi::c_int) as size_t),
        );
    }
    ga_grow(gap, sumlen as ::core::ffi::c_int + 2 as ::core::ffi::c_int);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*join_gap).ga_len && !got_int.get() {
        if first {
            first = false_0 != 0;
        } else {
            ga_concat_len(gap, sep, seplen);
        }
        let p_0: *const Join = ((*join_gap).ga_data as *const Join).offset(i as isize);
        if !(*p_0).s.data.is_null() {
            ga_concat_len(gap, (*p_0).s.data, (*p_0).s.size);
        }
        line_breakcheck();
        i += 1;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_join(
    gap: *mut garray_T,
    l: *mut list_T,
    sep: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if tv_list_len(l) == 0 {
        return OK;
    }
    let mut join_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut retval: ::core::ffi::c_int = 0;
    ga_init(
        &raw mut join_ga,
        ::core::mem::size_of::<Join>() as ::core::ffi::c_int,
        tv_list_len(l),
    );
    retval = list_join_inner(gap, l, sep, &raw mut join_ga);
    let mut _gap: *mut garray_T = &raw mut join_ga;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut Join = ((*_gap).ga_data as *mut Join).offset(i as isize);
            xfree((*_item).tofree as *mut ::core::ffi::c_void);
            i += 1;
        }
    }
    ga_clear(_gap);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn f_join(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    }
    let sep: *const ::core::ffi::c_char = if (*argvars.offset(1 as ::core::ffi::c_int as isize))
        .v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        b" \0".as_ptr() as *const ::core::ffi::c_char
    } else {
        tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize))
    };
    (*rettv).v_type = VAR_STRING;
    if !sep.is_null() {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
            80 as ::core::ffi::c_int,
        );
        tv_list_join(
            &raw mut ga,
            (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list,
            sep,
        );
        ga_append(&raw mut ga, NUL as uint8_t);
        (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
    } else {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_list2str(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if l.is_null() {
        return;
    }
    ga_init(
        &raw mut ga,
        1 as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let mut buf: [::core::ffi::c_char; 22] = [0; 22];
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let n: varnumber_T = tv_get_number(&raw const (*li).li_tv);
            let buflen: size_t = utf_char2bytes(
                n as ::core::ffi::c_int,
                &raw mut buf as *mut ::core::ffi::c_char,
            ) as size_t;
            buf[buflen as usize] = '\0' as ::core::ffi::c_char;
            ga_concat_len(
                &raw mut ga,
                &raw mut buf as *mut ::core::ffi::c_char,
                buflen,
            );
            li = (*li).li_next;
        }
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_remove(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
) {
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut error: bool = false_0 != 0;
    l = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if value_check_lock(tv_list_locked(l), arg_errmsg, TV_TRANSLATE as size_t) {
        return;
    }
    let mut idx: int64_t = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut error,
    );
    let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    if !error {
        item = tv_list_find(l, idx as ::core::ffi::c_int);
        if item.is_null() {
            semsg(
                gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
                idx,
            );
        } else if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_list_drop_items(l, item, item);
            *rettv = (*item).li_tv;
            xfree(item as *mut ::core::ffi::c_void);
        } else {
            let mut item2: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
            let mut end: int64_t = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            );
            if !error {
                item2 = tv_list_find(l, end as ::core::ffi::c_int);
                if item2.is_null() {
                    semsg(
                        gettext(
                            &raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char,
                        ),
                        end,
                    );
                } else {
                    let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut li: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
                    li = item;
                    while !li.is_null() {
                        cnt += 1;
                        if li == item2 {
                            break;
                        }
                        li = (*li).li_next;
                    }
                    if li.is_null() {
                        emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
                    } else {
                        tv_list_move_items(
                            l,
                            item,
                            item2,
                            tv_list_alloc_ret(rettv, cnt as ptrdiff_t),
                            cnt,
                        );
                    }
                }
            }
        }
    }
}
static sortinfo: GlobalCell<*mut sortinfo_T> =
    GlobalCell::new(::core::ptr::null_mut::<sortinfo_T>());
pub const ITEM_COMPARE_FAIL: ::core::ffi::c_int = 999 as ::core::ffi::c_int;
unsafe extern "C" fn item_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
    mut keep_zero: bool,
) -> ::core::ffi::c_int {
    let mut tofree1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tofree2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let si1: *mut ListSortItem = s1 as *mut ListSortItem;
    let si2: *mut ListSortItem = s2 as *mut ListSortItem;
    let tv1: *mut typval_T = &raw mut (*(*si1).item).li_tv;
    let tv2: *mut typval_T = &raw mut (*(*si2).item).li_tv;
    let mut res: ::core::ffi::c_int = 0;
    if (*sortinfo.get()).item_compare_numbers {
        let v1: varnumber_T = tv_get_number(tv1);
        let v2: varnumber_T = tv_get_number(tv2);
        res = if v1 == v2 {
            0 as ::core::ffi::c_int
        } else if v1 > v2 {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    } else if (*sortinfo.get()).item_compare_float {
        let v1_0: float_T = tv_get_float(tv1);
        let v2_0: float_T = tv_get_float(tv2);
        res = if v1_0 == v2_0 {
            0 as ::core::ffi::c_int
        } else if v1_0 > v2_0 {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    } else {
        tofree1 = ::core::ptr::null_mut::<::core::ffi::c_char>();
        tofree2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p1 = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*tv1).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*tv2).v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*sortinfo.get()).item_compare_numeric as ::core::ffi::c_int != 0
            {
                p1 = b"'\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                p1 = (*tv1).vval.v_string;
            }
        } else {
            p1 = encode_tv2string(tv1, ::core::ptr::null_mut::<size_t>());
            tofree1 = p1;
        }
        if (*tv2).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*tv1).v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*sortinfo.get()).item_compare_numeric as ::core::ffi::c_int != 0
            {
                p2 = b"'\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            } else {
                p2 = (*tv2).vval.v_string;
            }
        } else {
            p2 = encode_tv2string(tv2, ::core::ptr::null_mut::<size_t>());
            tofree2 = p2;
        }
        if p1.is_null() {
            p1 = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        if p2.is_null() {
            p2 = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        if !(*sortinfo.get()).item_compare_numeric {
            if (*sortinfo.get()).item_compare_lc {
                res = strcoll(p1, p2);
            } else {
                res = if (*sortinfo.get()).item_compare_ic != 0 {
                    strcasecmp(p1, p2)
                } else {
                    strcmp(p1, p2)
                };
            }
        } else {
            let mut n1: ::core::ffi::c_double = strtod(p1, &raw mut p1);
            let mut n2: ::core::ffi::c_double = strtod(p2, &raw mut p2);
            res = if n1 == n2 {
                0 as ::core::ffi::c_int
            } else if n1 > n2 {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        }
        xfree(tofree1 as *mut ::core::ffi::c_void);
        xfree(tofree2 as *mut ::core::ffi::c_void);
    }
    if res == 0 as ::core::ffi::c_int && !keep_zero {
        res = if (*si1).idx > (*si2).idx {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    return res;
}
unsafe extern "C" fn item_compare_keeping_zero(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    return item_compare(s1, s2, true_0 != 0);
}
unsafe extern "C" fn item_compare_not_keeping_zero(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    return item_compare(s1, s2, false_0 != 0);
}
unsafe extern "C" fn item_compare2(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
    mut keep_zero: bool,
) -> ::core::ffi::c_int {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut argv: [typval_T; 3] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 3];
    let mut func_name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut partial: *mut partial_T = (*sortinfo.get()).item_compare_partial;
    if (*sortinfo.get()).item_compare_func_err {
        return 0 as ::core::ffi::c_int;
    }
    let mut si1: *mut ListSortItem = s1 as *mut ListSortItem;
    let mut si2: *mut ListSortItem = s2 as *mut ListSortItem;
    if partial.is_null() {
        func_name = (*sortinfo.get()).item_compare_func;
    } else {
        func_name = partial_name(partial);
    }
    tv_copy(
        &raw mut (*(*si1).item).li_tv,
        (&raw mut argv as *mut typval_T).offset(0 as ::core::ffi::c_int as isize),
    );
    tv_copy(
        &raw mut (*(*si2).item).li_tv,
        (&raw mut argv as *mut typval_T).offset(1 as ::core::ffi::c_int as isize),
    );
    rettv.v_type = VAR_UNKNOWN;
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_evaluate = true_0 != 0;
    funcexe.fe_partial = partial;
    funcexe.fe_selfdict = (*sortinfo.get()).item_compare_selfdict;
    let mut res: ::core::ffi::c_int = call_func(
        func_name,
        -1 as ::core::ffi::c_int,
        &raw mut rettv,
        2 as ::core::ffi::c_int,
        &raw mut argv as *mut typval_T,
        &raw mut funcexe,
    );
    tv_clear((&raw mut argv as *mut typval_T).offset(0 as ::core::ffi::c_int as isize));
    tv_clear((&raw mut argv as *mut typval_T).offset(1 as ::core::ffi::c_int as isize));
    if res == FAIL {
        res = ITEM_COMPARE_FAIL;
        (*sortinfo.get()).item_compare_func_err = true_0 != 0;
    } else {
        let mut n: varnumber_T = tv_get_number_chk(
            &raw mut rettv,
            &raw mut (*sortinfo.get()).item_compare_func_err,
        );
        res = if n > 0 as varnumber_T {
            1 as ::core::ffi::c_int
        } else if n < 0 as varnumber_T {
            -1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
    if (*sortinfo.get()).item_compare_func_err {
        res = ITEM_COMPARE_FAIL;
    }
    tv_clear(&raw mut rettv);
    if res == 0 as ::core::ffi::c_int && !keep_zero {
        res = if (*si1).idx > (*si2).idx {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    return res;
}
unsafe extern "C" fn item_compare2_keeping_zero(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    return item_compare2(s1, s2, true_0 != 0);
}
unsafe extern "C" fn item_compare2_not_keeping_zero(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    return item_compare2(s1, s2, false_0 != 0);
}
unsafe extern "C" fn do_sort(mut l: *mut list_T, mut info: *mut sortinfo_T) {
    let len: ::core::ffi::c_int = tv_list_len(l);
    let mut ptrs: *mut ListSortItem = xmalloc(
        (len as ::core::ffi::c_uint as usize).wrapping_mul(::core::mem::size_of::<ListSortItem>()),
    ) as *mut ListSortItem;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *mut list_T = l;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            (*ptrs.offset(i as isize)).item = li;
            (*ptrs.offset(i as isize)).idx = i;
            i += 1;
            li = (*li).li_next;
        }
    }
    (*info).item_compare_func_err = false_0 != 0;
    let mut item_compare_func: ListSorter =
        if (*info).item_compare_func.is_null() && (*info).item_compare_partial.is_null() {
            Some(
                item_compare_not_keeping_zero
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            )
        } else {
            Some(
                item_compare2_not_keeping_zero
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            )
        };
    qsort(
        ptrs as *mut ::core::ffi::c_void,
        len as size_t,
        ::core::mem::size_of::<ListSortItem>(),
        item_compare_func as __compar_fn_t,
    );
    if !(*info).item_compare_func_err {
        (*l).lv_first = ::core::ptr::null_mut::<listitem_T>();
        (*l).lv_last = ::core::ptr::null_mut::<listitem_T>();
        (*l).lv_idx_item = ::core::ptr::null_mut::<listitem_T>();
        (*l).lv_len = 0 as ::core::ffi::c_int;
        i = 0 as ::core::ffi::c_int;
        while i < len {
            tv_list_append(l, (*ptrs.offset(i as isize)).item);
            i += 1;
        }
    }
    if (*info).item_compare_func_err {
        emsg(gettext(
            b"E702: Sort compare function failed\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
    xfree(ptrs as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn do_uniq(mut l: *mut list_T, mut info: *mut sortinfo_T) {
    let len: ::core::ffi::c_int = tv_list_len(l);
    let mut ptrs: *mut ListSortItem = xmalloc(
        (len as ::core::ffi::c_uint as usize).wrapping_mul(::core::mem::size_of::<ListSortItem>()),
    ) as *mut ListSortItem;
    (*info).item_compare_func_err = false_0 != 0;
    let mut item_compare_func: ListSorter =
        if (*info).item_compare_func.is_null() && (*info).item_compare_partial.is_null() {
            Some(
                item_compare_keeping_zero
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            )
        } else {
            Some(
                item_compare2_keeping_zero
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            )
        };
    let mut li: *mut listitem_T = (*tv_list_first(l)).li_next;
    while !li.is_null() {
        let prev_li: *mut listitem_T = (*li).li_prev;
        if item_compare_func.expect("non-null function pointer")(
            &raw const prev_li as *const ::core::ffi::c_void,
            &raw mut li as *const ::core::ffi::c_void,
        ) == 0 as ::core::ffi::c_int
        {
            li = tv_list_item_remove(l, li);
        } else {
            li = (*li).li_next;
        }
        if !(*info).item_compare_func_err {
            continue;
        }
        emsg(gettext(
            b"E882: Uniq compare function failed\0".as_ptr() as *const ::core::ffi::c_char
        ));
        break;
    }
    xfree(ptrs as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn parse_sort_uniq_args(
    mut argvars: *mut typval_T,
    mut info: *mut sortinfo_T,
) -> ::core::ffi::c_int {
    (*info).item_compare_ic = false_0;
    (*info).item_compare_lc = false_0 != 0;
    (*info).item_compare_numeric = false_0 != 0;
    (*info).item_compare_numbers = false_0 != 0;
    (*info).item_compare_float = false_0 != 0;
    (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
    (*info).item_compare_partial = ::core::ptr::null_mut::<partial_T>();
    (*info).item_compare_selfdict = ::core::ptr::null_mut::<dict_T>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return OK;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*info).item_compare_func = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_string;
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*info).item_compare_partial = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_partial;
    } else {
        let mut error: bool = false_0 != 0;
        let mut nr: ::core::ffi::c_int = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if error {
            return FAIL;
        }
        if nr == 1 as ::core::ffi::c_int {
            (*info).item_compare_ic = true_0;
        } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*info).item_compare_func =
                tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
        } else if nr != 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return FAIL;
        }
        if !(*info).item_compare_func.is_null() {
            if *(*info).item_compare_func as ::core::ffi::c_int == NUL {
                (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
            } else if strcmp(
                (*info).item_compare_func,
                b"n\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                (*info).item_compare_numeric = true_0 != 0;
            } else if strcmp(
                (*info).item_compare_func,
                b"N\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                (*info).item_compare_numbers = true_0 != 0;
            } else if strcmp(
                (*info).item_compare_func,
                b"f\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                (*info).item_compare_float = true_0 != 0;
            } else if strcmp(
                (*info).item_compare_func,
                b"i\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                (*info).item_compare_ic = true_0;
            } else if strcmp(
                (*info).item_compare_func,
                b"l\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                (*info).item_compare_lc = true_0 != 0;
            }
        }
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_check_for_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return FAIL;
        }
        (*info).item_compare_selfdict = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
    }
    return OK;
}
unsafe extern "C" fn do_sort_uniq(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut sort: bool,
) {
    let mut len: ::core::ffi::c_int = 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            if sort as ::core::ffi::c_int != 0 {
                b"sort()\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"uniq()\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        return;
    }
    let mut info: sortinfo_T = sortinfo_T {
        item_compare_ic: 0,
        item_compare_lc: false,
        item_compare_numeric: false,
        item_compare_numbers: false,
        item_compare_float: false,
        item_compare_func: ::core::ptr::null::<::core::ffi::c_char>(),
        item_compare_partial: ::core::ptr::null_mut::<partial_T>(),
        item_compare_selfdict: ::core::ptr::null_mut::<dict_T>(),
        item_compare_func_err: false,
    };
    let mut old_sortinfo: *mut sortinfo_T = sortinfo.get();
    sortinfo.set(&raw mut info);
    let arg_errmsg: *const ::core::ffi::c_char = if sort as ::core::ffi::c_int != 0 {
        b"sort() argument\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"uniq() argument\0".as_ptr() as *const ::core::ffi::c_char
    };
    let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if !value_check_lock(tv_list_locked(l), arg_errmsg, TV_TRANSLATE as size_t) {
        tv_list_set_ret(rettv, l);
        len = tv_list_len(l);
        if len > 1 as ::core::ffi::c_int {
            if parse_sort_uniq_args(argvars, &raw mut info) != FAIL {
                if sort {
                    do_sort(l, &raw mut info);
                } else {
                    do_uniq(l, &raw mut info);
                }
            }
        }
    }
    sortinfo.set(old_sortinfo);
}
#[no_mangle]
pub unsafe extern "C" fn f_sort(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    do_sort_uniq(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_uniq(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    do_sort_uniq(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_equal(l1: *mut list_T, l2: *mut list_T, ic: bool) -> bool {
    if l1 == l2 {
        return true_0 != 0;
    }
    if tv_list_len(l1) != tv_list_len(l2) {
        return false_0 != 0;
    }
    if tv_list_len(l1) == 0 as ::core::ffi::c_int {
        return true_0 != 0;
    }
    if l1.is_null() || l2.is_null() {
        return false_0 != 0;
    }
    let mut item1: *mut listitem_T = tv_list_first(l1);
    let mut item2: *mut listitem_T = tv_list_first(l2);
    while !item1.is_null() && !item2.is_null() {
        if !tv_equal(&raw mut (*item1).li_tv, &raw mut (*item2).li_tv, ic) {
            return false_0 != 0;
        }
        item1 = (*item1).li_next;
        item2 = (*item2).li_next;
    }
    '_c2rust_label: {
        if item1.is_null() && item2.is_null() {
        } else {
            __assert_fail(
                b"item1 == NULL && item2 == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1568 as ::core::ffi::c_uint,
                b"_Bool tv_list_equal(list_T *const, list_T *const, const _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_reverse(l: *mut list_T) {
    if tv_list_len(l) <= 1 as ::core::ffi::c_int {
        return;
    }
    let mut tmp: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    tmp = (*l).lv_first;
    (*l).lv_first = (*l).lv_last;
    (*l).lv_last = tmp;
    let mut li: *mut listitem_T = (*l).lv_first;
    while !li.is_null() {
        tmp = (*li).li_next;
        (*li).li_next = (*li).li_prev;
        (*li).li_prev = tmp;
        li = (*li).li_next;
    }
    (*l).lv_idx = (*l).lv_len - (*l).lv_idx - 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_find(
    l: *mut list_T,
    mut n: ::core::ffi::c_int,
) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    n = tv_list_uidx(l, n);
    if n == -1 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    let mut idx: ::core::ffi::c_int = 0;
    let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    if !(*l).lv_idx_item.is_null() {
        if n < (*l).lv_idx / 2 as ::core::ffi::c_int {
            item = (*l).lv_first;
            idx = 0 as ::core::ffi::c_int;
        } else if n > ((*l).lv_idx + (*l).lv_len) / 2 as ::core::ffi::c_int {
            item = (*l).lv_last;
            idx = (*l).lv_len - 1 as ::core::ffi::c_int;
        } else {
            item = (*l).lv_idx_item;
            idx = (*l).lv_idx;
        }
    } else if n < (*l).lv_len / 2 as ::core::ffi::c_int {
        item = (*l).lv_first;
        idx = 0 as ::core::ffi::c_int;
    } else {
        item = (*l).lv_last;
        idx = (*l).lv_len - 1 as ::core::ffi::c_int;
    }
    while n > idx {
        item = (*item).li_next;
        idx += 1;
    }
    while n < idx {
        item = (*item).li_prev;
        idx -= 1;
    }
    '_c2rust_label: {
        if idx == n {
        } else {
            __assert_fail(
                b"idx == n\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1661 as ::core::ffi::c_uint,
                b"listitem_T *tv_list_find(list_T *const, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*l).lv_idx = idx;
    (*l).lv_idx_item = item;
    return item;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_find_nr(
    l: *mut list_T,
    n: ::core::ffi::c_int,
    ret_error: *mut bool,
) -> varnumber_T {
    let li: *const listitem_T = tv_list_find(l, n);
    if li.is_null() {
        if !ret_error.is_null() {
            *ret_error = true_0 != 0;
        }
        return -1 as varnumber_T;
    }
    return tv_get_number_chk(&raw const (*li).li_tv, ret_error);
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_find_str(
    l: *mut list_T,
    n: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let li: *const listitem_T = tv_list_find(l, n);
    if li.is_null() {
        semsg(
            gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
            n as int64_t,
        );
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return tv_get_string(&raw const (*li).li_tv);
}
unsafe extern "C" fn tv_list_find_index(
    l: *mut list_T,
    idx: *mut ::core::ffi::c_int,
) -> *mut listitem_T {
    let mut li: *mut listitem_T = tv_list_find(l, *idx);
    if !li.is_null() {
        return li;
    }
    if *idx < 0 as ::core::ffi::c_int {
        *idx = 0 as ::core::ffi::c_int;
        li = tv_list_find(l, *idx);
    }
    return li;
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_idx_of_item(
    l: *const list_T,
    item: *const listitem_T,
) -> ::core::ffi::c_int {
    if l.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if li == item {
                return idx;
            }
            idx += 1;
            li = (*li).li_next;
        }
    }
    return -1 as ::core::ffi::c_int;
}
unsafe extern "C" fn tv_dict_watcher_free(mut watcher: *mut DictWatcher) {
    callback_free(&raw mut (*watcher).callback);
    xfree((*watcher).key_pattern as *mut ::core::ffi::c_void);
    xfree(watcher as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_watcher_add(
    dict: *mut dict_T,
    key_pattern: *const ::core::ffi::c_char,
    key_pattern_len: size_t,
    mut callback: Callback,
) {
    if dict.is_null() {
        return;
    }
    let watcher: *mut DictWatcher =
        xmalloc(::core::mem::size_of::<DictWatcher>()) as *mut DictWatcher;
    (*watcher).key_pattern = xmemdupz(key_pattern as *const ::core::ffi::c_void, key_pattern_len)
        as *mut ::core::ffi::c_char;
    (*watcher).key_pattern_len = key_pattern_len;
    (*watcher).callback = callback;
    (*watcher).busy = false_0 != 0;
    (*watcher).needs_free = false_0 != 0;
    QUEUE_INSERT_TAIL(&raw mut (*dict).watchers, &raw mut (*watcher).node);
}
#[no_mangle]
pub unsafe extern "C" fn tv_callback_equal(
    mut cb1: *const Callback,
    mut cb2: *const Callback,
) -> bool {
    if (*cb1).type_0 as ::core::ffi::c_uint != (*cb2).type_0 as ::core::ffi::c_uint {
        return false_0 != 0;
    }
    match (*cb1).type_0 as ::core::ffi::c_uint {
        1 => {
            return strcmp((*cb1).data.funcref, (*cb2).data.funcref) == 0 as ::core::ffi::c_int;
        }
        2 => return (*cb1).data.partial == (*cb2).data.partial,
        3 => return (*cb1).data.luaref == (*cb2).data.luaref,
        0 => return true_0 != 0,
        _ => {}
    }
    abort();
}
#[no_mangle]
pub unsafe extern "C" fn callback_free(mut callback: *mut Callback) {
    match (*callback).type_0 as ::core::ffi::c_uint {
        1 => {
            func_unref((*callback).data.funcref);
            xfree((*callback).data.funcref as *mut ::core::ffi::c_void);
        }
        2 => {
            partial_unref((*callback).data.partial);
        }
        3 => {
            if (*callback).data.luaref != LUA_NOREF {
                api_free_luaref((*callback).data.luaref);
                (*callback).data.luaref = LUA_NOREF as LuaRef;
            }
        }
        0 | _ => {}
    }
    (*callback).type_0 = kCallbackNone;
    (*callback).data.funcref = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn callback_put(mut cb: *mut Callback, mut tv: *mut typval_T) {
    match (*cb).type_0 as ::core::ffi::c_uint {
        2 => {
            (*tv).v_type = VAR_PARTIAL;
            (*tv).vval.v_partial = (*cb).data.partial;
            (*(*cb).data.partial).pt_refcount += 1;
        }
        1 => {
            (*tv).v_type = VAR_FUNC;
            (*tv).vval.v_string = xstrdup((*cb).data.funcref);
            func_ref((*cb).data.funcref);
        }
        3 | _ => {
            (*tv).v_type = VAR_SPECIAL;
            (*tv).vval.v_special = kSpecialVarNull;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn callback_copy(mut dest: *mut Callback, mut src: *mut Callback) {
    (*dest).type_0 = (*src).type_0;
    match (*src).type_0 as ::core::ffi::c_uint {
        2 => {
            (*dest).data.partial = (*src).data.partial;
            (*(*dest).data.partial).pt_refcount += 1;
        }
        1 => {
            (*dest).data.funcref = xstrdup((*src).data.funcref);
            func_ref((*src).data.funcref);
        }
        3 => {
            (*dest).data.luaref = api_new_luaref((*src).data.luaref);
        }
        _ => {
            (*dest).data.funcref = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn callback_to_string(
    mut cb: *mut Callback,
    mut arena: *mut Arena,
) -> *mut ::core::ffi::c_char {
    if (*cb).type_0 as ::core::ffi::c_uint
        == kCallbackLua as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return nlua_funcref_str((*cb).data.luaref, arena);
    }
    let msglen: size_t = 100 as size_t;
    let mut msg: *mut ::core::ffi::c_char = xmallocz(msglen) as *mut ::core::ffi::c_char;
    match (*cb).type_0 as ::core::ffi::c_uint {
        1 => {
            snprintf(
                msg,
                msglen,
                b"<vim function: %s>\0".as_ptr() as *const ::core::ffi::c_char,
                (*cb).data.funcref,
            );
        }
        2 => {
            snprintf(
                msg,
                msglen,
                b"<vim partial: %s>\0".as_ptr() as *const ::core::ffi::c_char,
                (*(*cb).data.partial).pt_name,
            );
        }
        _ => {
            *msg = NUL as ::core::ffi::c_char;
        }
    }
    return msg;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_watcher_remove(
    dict: *mut dict_T,
    key_pattern: *const ::core::ffi::c_char,
    key_pattern_len: size_t,
    mut callback: Callback,
) -> bool {
    if dict.is_null() {
        return false_0 != 0;
    }
    let mut w: *mut QUEUE = ::core::ptr::null_mut::<QUEUE>();
    let mut watcher: *mut DictWatcher = ::core::ptr::null_mut::<DictWatcher>();
    let mut matched: bool = false_0 != 0;
    let mut queue_is_busy: bool = false_0 != 0;
    w = (*dict).watchers.next as *mut QUEUE;
    while w != &raw mut (*dict).watchers {
        let mut next: *mut QUEUE = (*w).next as *mut QUEUE;
        watcher = tv_dict_watcher_node_data(w);
        if (*watcher).busy {
            queue_is_busy = true;
        }
        if tv_callback_equal(&raw mut (*watcher).callback, &raw mut callback) as ::core::ffi::c_int
            != 0
            && (*watcher).key_pattern_len == key_pattern_len
            && memcmp(
                (*watcher).key_pattern as *const ::core::ffi::c_void,
                key_pattern as *const ::core::ffi::c_void,
                key_pattern_len,
            ) == 0 as ::core::ffi::c_int
        {
            matched = true;
            break;
        } else {
            w = next;
        }
    }
    if !matched {
        return false_0 != 0;
    }
    if queue_is_busy {
        (*watcher).needs_free = true_0 != 0;
    } else {
        QUEUE_REMOVE(w);
        tv_dict_watcher_free(watcher);
    }
    return true_0 != 0;
}
unsafe extern "C" fn tv_dict_watcher_matches(
    mut watcher: *mut DictWatcher,
    key: *const ::core::ffi::c_char,
) -> bool {
    let len: size_t = (*watcher).key_pattern_len;
    if len != 0
        && *(*watcher)
            .key_pattern
            .offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            == '*' as ::core::ffi::c_int
    {
        return strncmp(key, (*watcher).key_pattern, len.wrapping_sub(1 as size_t))
            == 0 as ::core::ffi::c_int;
    }
    return strcmp(key, (*watcher).key_pattern) == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_watcher_notify(
    dict: *mut dict_T,
    key: *const ::core::ffi::c_char,
    newtv: *mut typval_T,
    oldtv: *mut typval_T,
) {
    let mut argv: [typval_T; 3] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 3];
    argv[0 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
    argv[0 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
    argv[0 as ::core::ffi::c_int as usize].vval.v_dict = dict;
    argv[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
    argv[1 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
    argv[1 as ::core::ffi::c_int as usize].vval.v_string = xstrdup(key);
    argv[2 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
    argv[2 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
    argv[2 as ::core::ffi::c_int as usize].vval.v_dict = tv_dict_alloc();
    (*argv[2 as ::core::ffi::c_int as usize].vval.v_dict).dv_refcount += 1;
    if !newtv.is_null() {
        let v: *mut dictitem_T = tv_dict_item_alloc_len(
            b"new\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        );
        tv_copy(newtv, &raw mut (*v).di_tv);
        tv_dict_add(argv[2 as ::core::ffi::c_int as usize].vval.v_dict, v);
    }
    if !oldtv.is_null()
        && (*oldtv).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let v_0: *mut dictitem_T = tv_dict_item_alloc_len(
            b"old\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        );
        tv_copy(oldtv, &raw mut (*v_0).di_tv);
        tv_dict_add(argv[2 as ::core::ffi::c_int as usize].vval.v_dict, v_0);
    }
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut any_needs_free: bool = false_0 != 0;
    (*dict).dv_refcount += 1;
    let mut w: *mut QUEUE = ::core::ptr::null_mut::<QUEUE>();
    w = (*dict).watchers.next as *mut QUEUE;
    while w != &raw mut (*dict).watchers {
        let mut next: *mut QUEUE = (*w).next as *mut QUEUE;
        let mut watcher: *mut DictWatcher = tv_dict_watcher_node_data(w);
        if !(*watcher).busy && tv_dict_watcher_matches(watcher, key) as ::core::ffi::c_int != 0 {
            rettv = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            (*watcher).busy = true;
            callback_call(
                &raw mut (*watcher).callback,
                3 as ::core::ffi::c_int,
                &raw mut argv as *mut typval_T,
                &raw mut rettv,
            );
            (*watcher).busy = false;
            tv_clear(&raw mut rettv);
            if (*watcher).needs_free {
                any_needs_free = true;
            }
        }
        w = next;
    }
    if any_needs_free {
        w = (*dict).watchers.next as *mut QUEUE;
        while w != &raw mut (*dict).watchers {
            let mut next_0: *mut QUEUE = (*w).next as *mut QUEUE;
            let mut watcher_0: *mut DictWatcher = tv_dict_watcher_node_data(w);
            if (*watcher_0).needs_free {
                QUEUE_REMOVE(w);
                tv_dict_watcher_free(watcher_0);
            }
            w = next_0;
        }
    }
    tv_dict_unref(dict);
    let mut i: size_t = 1 as size_t;
    while i < ::core::mem::size_of::<[typval_T; 3]>()
        .wrapping_div(::core::mem::size_of::<typval_T>())
        .wrapping_div(
            (::core::mem::size_of::<[typval_T; 3]>()
                .wrapping_rem(::core::mem::size_of::<typval_T>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        tv_clear((&raw mut argv as *mut typval_T).offset(i as isize));
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_item_alloc_len(
    key: *const ::core::ffi::c_char,
    key_len: size_t,
) -> *mut dictitem_T {
    let di: *mut dictitem_T = xmalloc(
        if ::core::mem::size_of::<dictitem_T>()
            > (17 as size_t)
                .wrapping_add(key_len)
                .wrapping_add(1 as size_t)
        {
            ::core::mem::size_of::<dictitem_T>()
        } else {
            (17 as size_t)
                .wrapping_add(key_len)
                .wrapping_add(1 as size_t)
        },
    ) as *mut dictitem_T;
    memcpy(
        &raw mut (*di).di_key as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        key as *const ::core::ffi::c_void,
        key_len,
    );
    *(&raw mut (*di).di_key as *mut ::core::ffi::c_char).offset(key_len as isize) =
        NUL as ::core::ffi::c_char;
    (*di).di_flags = DI_FLAGS_ALLOC as ::core::ffi::c_int as uint8_t;
    (*di).di_tv.v_lock = VAR_UNLOCKED;
    (*di).di_tv.v_type = VAR_UNKNOWN;
    return di;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_item_alloc(key: *const ::core::ffi::c_char) -> *mut dictitem_T {
    return tv_dict_item_alloc_len(key, strlen(key));
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_item_free(item: *mut dictitem_T) {
    tv_clear(&raw mut (*item).di_tv);
    if (*item).di_flags as ::core::ffi::c_int & DI_FLAGS_ALLOC as ::core::ffi::c_int != 0 {
        xfree(item as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_item_copy(di: *mut dictitem_T) -> *mut dictitem_T {
    let new_di: *mut dictitem_T =
        tv_dict_item_alloc(&raw mut (*di).di_key as *mut ::core::ffi::c_char);
    tv_copy(&raw mut (*di).di_tv, &raw mut (*new_di).di_tv);
    return new_di;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_item_remove(dict: *mut dict_T, item: *mut dictitem_T) {
    let hi: *mut hashitem_T = hash_find(
        &raw mut (*dict).dv_hashtab,
        &raw mut (*item).di_key as *mut ::core::ffi::c_char,
    );
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
    {
        semsg(
            gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
            b"tv_dict_item_remove()\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        hash_remove(&raw mut (*dict).dv_hashtab, hi);
    }
    tv_dict_item_free(item);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_alloc() -> *mut dict_T {
    let d: *mut dict_T = xcalloc(1 as size_t, ::core::mem::size_of::<dict_T>()) as *mut dict_T;
    if !(*gc_first_dict.ptr()).is_null() {
        (*gc_first_dict.get()).dv_used_prev = d;
    }
    (*d).dv_used_next = gc_first_dict.get();
    (*d).dv_used_prev = ::core::ptr::null_mut::<dict_T>();
    gc_first_dict.set(d);
    hash_init(&raw mut (*d).dv_hashtab);
    (*d).dv_lock = VAR_UNLOCKED;
    (*d).dv_scope = VAR_NO_SCOPE;
    (*d).dv_refcount = 0 as ::core::ffi::c_int;
    (*d).dv_copyID = 0 as ::core::ffi::c_int;
    QUEUE_INIT(&raw mut (*d).watchers);
    (*d).lua_table_ref = LUA_NOREF as LuaRef;
    return d;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_free_contents(d: *mut dict_T) {
    hash_lock(&raw mut (*d).dv_hashtab);
    '_c2rust_label: {
        if (*d).dv_hashtab.ht_locked > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"d->dv_hashtab.ht_locked > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2163 as ::core::ffi::c_uint,
                b"void tv_dict_free_contents(dict_T *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let hiht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
    let mut hitodo_: size_t = (*hiht_).ht_used;
    let mut hi: *mut hashitem_T = (*hiht_).ht_array;
    while hitodo_ != 0 {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            hitodo_ = hitodo_.wrapping_sub(1);
            let di: *mut dictitem_T =
                (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
            hash_remove(&raw mut (*d).dv_hashtab, hi);
            tv_dict_item_free(di);
        }
        hi = hi.offset(1);
    }
    while QUEUE_EMPTY(&raw mut (*d).watchers) == 0 {
        let mut w: *mut QUEUE = (*d).watchers.next as *mut QUEUE;
        QUEUE_REMOVE(w);
        let mut watcher: *mut DictWatcher = tv_dict_watcher_node_data(w);
        tv_dict_watcher_free(watcher);
    }
    hash_clear(&raw mut (*d).dv_hashtab);
    (*d).dv_hashtab.ht_locked -= 1;
    hash_init(&raw mut (*d).dv_hashtab);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_free_dict(d: *mut dict_T) {
    if (*d).dv_used_prev.is_null() {
        gc_first_dict.set((*d).dv_used_next);
    } else {
        (*(*d).dv_used_prev).dv_used_next = (*d).dv_used_next;
    }
    if !(*d).dv_used_next.is_null() {
        (*(*d).dv_used_next).dv_used_prev = (*d).dv_used_prev;
    }
    if (*d).lua_table_ref != LUA_NOREF {
        api_free_luaref((*d).lua_table_ref);
        (*d).lua_table_ref = LUA_NOREF as LuaRef;
    }
    xfree(d as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_free(d: *mut dict_T) {
    if tv_in_free_unref_items.get() {
        return;
    }
    tv_dict_free_contents(d);
    tv_dict_free_dict(d);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_unref(d: *mut dict_T) {
    if !d.is_null() && {
        (*d).dv_refcount -= 1;
        (*d).dv_refcount <= 0 as ::core::ffi::c_int
    } {
        tv_dict_free(d);
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_find(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    len: ptrdiff_t,
) -> *mut dictitem_T {
    if d.is_null() {
        return ::core::ptr::null_mut::<dictitem_T>();
    }
    let hi: *mut hashitem_T = if len < 0 as ptrdiff_t {
        hash_find(&raw const (*d).dv_hashtab, key)
    } else {
        hash_find_len(&raw const (*d).dv_hashtab, key, len as size_t)
    };
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
    {
        return ::core::ptr::null_mut::<dictitem_T>();
    }
    return (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_has_key(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
) -> bool {
    return !tv_dict_find(d, key, -1 as ptrdiff_t).is_null();
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_get_tv(
    mut d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let di: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
    if di.is_null() {
        return FAIL;
    }
    tv_copy(&raw mut (*di).di_tv, rettv);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_get_number(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
) -> varnumber_T {
    return tv_dict_get_number_def(d, key, 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_get_number_def(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    def: ::core::ffi::c_int,
) -> varnumber_T {
    let di: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
    if di.is_null() {
        return def as varnumber_T;
    }
    return tv_get_number(&raw mut (*di).di_tv);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_get_bool(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    def: ::core::ffi::c_int,
) -> varnumber_T {
    let di: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
    if di.is_null() {
        return def as varnumber_T;
    }
    return tv_get_bool(&raw mut (*di).di_tv);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_to_env(mut denv: *mut dict_T) -> *mut *mut ::core::ffi::c_char {
    let mut env_size: size_t = tv_dict_len(denv) as size_t;
    let mut i: size_t = 0 as size_t;
    let mut env: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    env = xmalloc(
        env_size
            .wrapping_add(1 as size_t)
            .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
    ) as *mut *mut ::core::ffi::c_char;
    let varhi_ht_: *mut hashtab_T = &raw mut (*denv).dv_hashtab;
    let mut varhi_todo_: size_t = (*varhi_ht_).ht_used;
    let mut varhi_: *mut hashitem_T = (*varhi_ht_).ht_array;
    while varhi_todo_ != 0 {
        if !((*varhi_).hi_key.is_null()
            || (*varhi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            varhi_todo_ = varhi_todo_.wrapping_sub(1);
            let var: *mut dictitem_T = (*varhi_)
                .hi_key
                .offset(-(17 as ::core::ffi::c_ulong as isize))
                as *mut dictitem_T;
            let mut str: *const ::core::ffi::c_char = tv_get_string(&raw mut (*var).di_tv);
            '_c2rust_label: {
                if !str.is_null() {
                } else {
                    __assert_fail(
                        b"str\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2339 as ::core::ffi::c_uint,
                        b"char **tv_dict_to_env(dict_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut len: size_t = strlen(&raw mut (*var).di_key as *mut ::core::ffi::c_char)
                .wrapping_add(strlen(str))
                .wrapping_add(strlen(b"=\0".as_ptr() as *const ::core::ffi::c_char))
                .wrapping_add(1 as size_t);
            *env.offset(i as isize) = xmalloc(len) as *mut ::core::ffi::c_char;
            snprintf(
                *env.offset(i as isize),
                len,
                b"%s=%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut (*var).di_key as *mut ::core::ffi::c_char,
                str,
            );
            i = i.wrapping_add(1);
        }
        varhi_ = varhi_.offset(1);
    }
    *env.offset(env_size as isize) = ::core::ptr::null_mut::<::core::ffi::c_char>();
    return env;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_get_string(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    save: bool,
) -> *mut ::core::ffi::c_char {
    static numbuf: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new([0; 65]);
    let s: *const ::core::ffi::c_char =
        tv_dict_get_string_buf(d, key, numbuf.ptr() as *mut ::core::ffi::c_char);
    if save as ::core::ffi::c_int != 0 && !s.is_null() {
        return xstrdup(s);
    }
    return s as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_get_string_buf(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    numbuf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let di: *const dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
    if di.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return tv_get_string_buf(&raw const (*di).di_tv, numbuf);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_get_string_buf_chk(
    d: *const dict_T,
    key: *const ::core::ffi::c_char,
    key_len: ptrdiff_t,
    numbuf: *mut ::core::ffi::c_char,
    def: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let di: *const dictitem_T = tv_dict_find(d, key, key_len);
    if di.is_null() {
        return def;
    }
    return tv_get_string_buf_chk(&raw const (*di).di_tv, numbuf);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_get_callback(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: ptrdiff_t,
    result: *mut Callback,
) -> bool {
    (*result).type_0 = kCallbackNone;
    let di: *mut dictitem_T = tv_dict_find(d, key, key_len);
    if di.is_null() {
        return true_0 != 0;
    }
    if !tv_is_func((*di).di_tv)
        && (*di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(
            b"E6000: Argument is not a function or function name\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return false_0 != 0;
    }
    let mut tv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    tv_copy(&raw mut (*di).di_tv, &raw mut tv);
    set_selfdict(&raw mut tv, d);
    let res: bool = callback_from_typval(result, &raw mut tv);
    tv_clear(&raw mut tv);
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_wrong_func_name(
    mut d: *mut dict_T,
    mut tv: *mut typval_T,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return ((d == get_globvar_dict() || &raw mut (*d).dv_hashtab == get_funccal_local_ht())
        && tv_is_func(*tv) as ::core::ffi::c_int != 0
        && var_wrong_func_name(name, true_0 != 0) as ::core::ffi::c_int != 0)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int {
    if tv_dict_wrong_func_name(
        d,
        &raw mut (*item).di_tv,
        &raw mut (*item).di_key as *mut ::core::ffi::c_char,
    ) != 0
    {
        return FAIL;
    }
    return hash_add(
        &raw mut (*d).dv_hashtab,
        &raw mut (*item).di_key as *mut ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add_list(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    list: *mut list_T,
) -> ::core::ffi::c_int {
    let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
    (*item).di_tv.v_type = VAR_LIST;
    (*item).di_tv.vval.v_list = list;
    tv_list_ref(list);
    if tv_dict_add(d, item) == FAIL {
        tv_dict_item_free(item);
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add_tv(
    mut d: *mut dict_T,
    mut key: *const ::core::ffi::c_char,
    key_len: size_t,
    mut tv: *mut typval_T,
) -> ::core::ffi::c_int {
    let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
    tv_copy(tv, &raw mut (*item).di_tv);
    if tv_dict_add(d, item) == FAIL {
        tv_dict_item_free(item);
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add_dict(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    dict: *mut dict_T,
) -> ::core::ffi::c_int {
    let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
    (*item).di_tv.v_type = VAR_DICT;
    (*item).di_tv.vval.v_dict = dict;
    (*dict).dv_refcount += 1;
    if tv_dict_add(d, item) == FAIL {
        tv_dict_item_free(item);
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add_nr(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    nr: varnumber_T,
) -> ::core::ffi::c_int {
    let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
    (*item).di_tv.v_type = VAR_NUMBER;
    (*item).di_tv.vval.v_number = nr;
    if tv_dict_add(d, item) == FAIL {
        tv_dict_item_free(item);
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add_float(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    nr: float_T,
) -> ::core::ffi::c_int {
    let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
    (*item).di_tv.v_type = VAR_FLOAT;
    (*item).di_tv.vval.v_float = nr;
    if tv_dict_add(d, item) == FAIL {
        tv_dict_item_free(item);
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add_bool(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    mut val: BoolVarValue,
) -> ::core::ffi::c_int {
    let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
    (*item).di_tv.v_type = VAR_BOOL;
    (*item).di_tv.vval.v_bool = val;
    if tv_dict_add(d, item) == FAIL {
        tv_dict_item_free(item);
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add_str(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return tv_dict_add_str_len(d, key, key_len, val, -1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add_str_len(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !val.is_null() {
        s = if len < 0 as ::core::ffi::c_int {
            xstrdup(val)
        } else {
            xstrndup(val, len as size_t)
        };
    }
    return tv_dict_add_allocated_str(d, key, key_len, s);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add_allocated_str(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
    (*item).di_tv.v_type = VAR_STRING;
    (*item).di_tv.vval.v_string = val;
    if tv_dict_add(d, item) == FAIL {
        tv_dict_item_free(item);
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_add_func(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    fp: *mut ufunc_T,
) -> ::core::ffi::c_int {
    let item: *mut dictitem_T = tv_dict_item_alloc_len(key, key_len);
    (*item).di_tv.v_type = VAR_FUNC;
    (*item).di_tv.vval.v_string = xmemdupz(
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        (*fp).uf_namelen,
    ) as *mut ::core::ffi::c_char;
    if tv_dict_add(d, item) == FAIL {
        tv_dict_item_free(item);
        return FAIL;
    }
    func_ref((*item).di_tv.vval.v_string);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_clear(d: *mut dict_T) {
    hash_lock(&raw mut (*d).dv_hashtab);
    '_c2rust_label: {
        if (*d).dv_hashtab.ht_locked > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"d->dv_hashtab.ht_locked > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2698 as ::core::ffi::c_uint,
                b"void tv_dict_clear(dict_T *const)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let hiht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
    let mut hitodo_: size_t = (*hiht_).ht_used;
    let mut hi: *mut hashitem_T = (*hiht_).ht_array;
    while hitodo_ != 0 {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            hitodo_ = hitodo_.wrapping_sub(1);
            tv_dict_item_free(
                (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T
            );
            hash_remove(&raw mut (*d).dv_hashtab, hi);
        }
        hi = hi.offset(1);
    }
    hash_unlock(&raw mut (*d).dv_hashtab);
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_extend(
    d1: *mut dict_T,
    d2: *mut dict_T,
    action: *const ::core::ffi::c_char,
) {
    let watched: bool = tv_dict_is_watched(d1);
    let arg_errmsg: *const ::core::ffi::c_char =
        gettext(b"extend() argument\0".as_ptr() as *const ::core::ffi::c_char);
    let arg_errmsg_len: size_t = strlen(arg_errmsg);
    if *action as ::core::ffi::c_int == 'm' as ::core::ffi::c_int {
        hash_lock(&raw mut (*d2).dv_hashtab);
    }
    let hi2ht_: *mut hashtab_T = &raw mut (*d2).dv_hashtab;
    let mut hi2todo_: size_t = (*hi2ht_).ht_used;
    let mut hi2: *mut hashitem_T = (*hi2ht_).ht_array;
    while hi2todo_ != 0 {
        if !((*hi2).hi_key.is_null()
            || (*hi2).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            hi2todo_ = hi2todo_.wrapping_sub(1);
            let di2: *mut dictitem_T =
                (*hi2).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
            let di1: *mut dictitem_T = tv_dict_find(
                d1,
                &raw mut (*di2).di_key as *mut ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if (*d1).dv_scope as ::core::ffi::c_uint
                != VAR_NO_SCOPE as ::core::ffi::c_int as ::core::ffi::c_uint
                && !valid_varname(&raw mut (*di2).di_key as *mut ::core::ffi::c_char)
            {
                break;
            }
            if di1.is_null() {
                if *action as ::core::ffi::c_int == 'm' as ::core::ffi::c_int {
                    let new_di: *mut dictitem_T = di2;
                    if tv_dict_add(d1, new_di) == 1 as ::core::ffi::c_int {
                        hash_remove(&raw mut (*d2).dv_hashtab, hi2);
                        tv_dict_watcher_notify(
                            d1,
                            &raw mut (*new_di).di_key as *mut ::core::ffi::c_char,
                            &raw mut (*new_di).di_tv,
                            ::core::ptr::null_mut::<typval_T>(),
                        );
                    }
                } else {
                    let new_di_0: *mut dictitem_T = tv_dict_item_copy(di2);
                    if tv_dict_add(d1, new_di_0) == 0 as ::core::ffi::c_int {
                        tv_dict_item_free(new_di_0);
                    } else if watched {
                        tv_dict_watcher_notify(
                            d1,
                            &raw mut (*new_di_0).di_key as *mut ::core::ffi::c_char,
                            &raw mut (*new_di_0).di_tv,
                            ::core::ptr::null_mut::<typval_T>(),
                        );
                    }
                }
            } else if *action as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                semsg(
                    gettext(
                        b"E737: Key already exists: %s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    &raw mut (*di2).di_key as *mut ::core::ffi::c_char,
                );
                break;
            } else if *action as ::core::ffi::c_int == 'f' as ::core::ffi::c_int && di2 != di1 {
                let mut oldtv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                if value_check_lock((*di1).di_tv.v_lock, arg_errmsg, arg_errmsg_len)
                    as ::core::ffi::c_int
                    != 0
                    || var_check_ro(
                        (*di1).di_flags as ::core::ffi::c_int,
                        arg_errmsg,
                        arg_errmsg_len,
                    ) as ::core::ffi::c_int
                        != 0
                {
                    break;
                }
                if tv_dict_wrong_func_name(
                    d1,
                    &raw mut (*di2).di_tv,
                    &raw mut (*di2).di_key as *mut ::core::ffi::c_char,
                ) != 0
                {
                    break;
                }
                if watched {
                    tv_copy(&raw mut (*di1).di_tv, &raw mut oldtv);
                }
                tv_clear(&raw mut (*di1).di_tv);
                tv_copy(&raw mut (*di2).di_tv, &raw mut (*di1).di_tv);
                if watched {
                    tv_dict_watcher_notify(
                        d1,
                        &raw mut (*di1).di_key as *mut ::core::ffi::c_char,
                        &raw mut (*di1).di_tv,
                        &raw mut oldtv,
                    );
                    tv_clear(&raw mut oldtv);
                }
            }
        }
        hi2 = hi2.offset(1);
    }
    if *action as ::core::ffi::c_int == 'm' as ::core::ffi::c_int {
        hash_unlock(&raw mut (*d2).dv_hashtab);
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_equal(d1: *mut dict_T, d2: *mut dict_T, ic: bool) -> bool {
    if d1 == d2 {
        return true_0 != 0;
    }
    if tv_dict_len(d1) != tv_dict_len(d2) {
        return false_0 != 0;
    }
    if tv_dict_len(d1) == 0 as ::core::ffi::c_long {
        return true_0 != 0;
    }
    if d1.is_null() || d2.is_null() {
        return false_0 != 0;
    }
    let di1hi_ht_: *mut hashtab_T = &raw mut (*d1).dv_hashtab;
    let mut di1hi_todo_: size_t = (*di1hi_ht_).ht_used;
    let mut di1hi_: *mut hashitem_T = (*di1hi_ht_).ht_array;
    while di1hi_todo_ != 0 {
        if !((*di1hi_).hi_key.is_null()
            || (*di1hi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            di1hi_todo_ = di1hi_todo_.wrapping_sub(1);
            let di1: *mut dictitem_T = (*di1hi_)
                .hi_key
                .offset(-(17 as ::core::ffi::c_ulong as isize))
                as *mut dictitem_T;
            let di2: *mut dictitem_T = tv_dict_find(
                d2,
                &raw mut (*di1).di_key as *mut ::core::ffi::c_char,
                -1 as ptrdiff_t,
            );
            if di2.is_null() {
                return false;
            }
            if !tv_equal(&raw mut (*di1).di_tv, &raw mut (*di2).di_tv, ic) {
                return false;
            }
        }
        di1hi_ = di1hi_.offset(1);
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_copy(
    conv: *const vimconv_T,
    orig: *mut dict_T,
    deep: bool,
    copyID: ::core::ffi::c_int,
) -> *mut dict_T {
    if orig.is_null() {
        return ::core::ptr::null_mut::<dict_T>();
    }
    let mut copy: *mut dict_T = tv_dict_alloc();
    if copyID != 0 as ::core::ffi::c_int {
        (*orig).dv_copyID = copyID;
        (*orig).dv_copydict = copy;
    }
    let dihi_ht_: *mut hashtab_T = &raw mut (*orig).dv_hashtab;
    let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
    let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
    while dihi_todo_ != 0 {
        if !((*dihi_).hi_key.is_null()
            || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            dihi_todo_ = dihi_todo_.wrapping_sub(1);
            let di: *mut dictitem_T = (*dihi_)
                .hi_key
                .offset(-(17 as ::core::ffi::c_ulong as isize))
                as *mut dictitem_T;
            if got_int.get() {
                break;
            }
            let mut new_di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
            if conv.is_null() || (*conv).vc_type == CONV_NONE as ::core::ffi::c_int {
                new_di = tv_dict_item_alloc(&raw mut (*di).di_key as *mut ::core::ffi::c_char);
            } else {
                let mut len: size_t = strlen(&raw mut (*di).di_key as *mut ::core::ffi::c_char);
                let key: *mut ::core::ffi::c_char = string_convert(
                    conv,
                    &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                    &raw mut len,
                );
                if key.is_null() {
                    new_di = tv_dict_item_alloc_len(
                        &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                        len,
                    );
                } else {
                    new_di = tv_dict_item_alloc_len(key, len);
                    xfree(key as *mut ::core::ffi::c_void);
                }
            }
            if deep {
                if var_item_copy(
                    conv,
                    &raw mut (*di).di_tv,
                    &raw mut (*new_di).di_tv,
                    deep,
                    copyID,
                ) == 0 as ::core::ffi::c_int
                {
                    xfree(new_di as *mut ::core::ffi::c_void);
                    break;
                }
            } else {
                tv_copy(&raw mut (*di).di_tv, &raw mut (*new_di).di_tv);
            }
            if tv_dict_add(copy, new_di) == 0 as ::core::ffi::c_int {
                tv_dict_item_free(new_di);
                break;
            }
        }
        dihi_ = dihi_.offset(1);
    }
    (*copy).dv_refcount += 1;
    if got_int.get() {
        tv_dict_unref(copy);
        copy = ::core::ptr::null_mut::<dict_T>();
    }
    return copy;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_set_keys_readonly(dict: *mut dict_T) {
    let dihi_ht_: *mut hashtab_T = &raw mut (*dict).dv_hashtab;
    let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
    let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
    while dihi_todo_ != 0 {
        if !((*dihi_).hi_key.is_null()
            || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            dihi_todo_ = dihi_todo_.wrapping_sub(1);
            let di: *mut dictitem_T = (*dihi_)
                .hi_key
                .offset(-(17 as ::core::ffi::c_ulong as isize))
                as *mut dictitem_T;
            (*di).di_flags = ((*di).di_flags as ::core::ffi::c_int
                | (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int))
                as uint8_t;
        }
        dihi_ = dihi_.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_alloc() -> *mut blob_T {
    let blob: *mut blob_T = xcalloc(1 as size_t, ::core::mem::size_of::<blob_T>()) as *mut blob_T;
    ga_init(
        &raw mut (*blob).bv_ga,
        1 as ::core::ffi::c_int,
        100 as ::core::ffi::c_int,
    );
    return blob;
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_free(b: *mut blob_T) {
    ga_clear(&raw mut (*b).bv_ga);
    xfree(b as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_unref(b: *mut blob_T) {
    if !b.is_null() && {
        (*b).bv_refcount -= 1;
        (*b).bv_refcount <= 0 as ::core::ffi::c_int
    } {
        tv_blob_free(b);
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_equal(b1: *const blob_T, b2: *const blob_T) -> bool {
    let len1: ::core::ffi::c_int = tv_blob_len(b1);
    let len2: ::core::ffi::c_int = tv_blob_len(b2);
    if len1 == 0 as ::core::ffi::c_int && len2 == 0 as ::core::ffi::c_int {
        return true_0 != 0;
    }
    if b1 == b2 {
        return true_0 != 0;
    }
    if len1 != len2 {
        return false_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*b1).bv_ga.ga_len {
        if tv_blob_get(b1, i) as ::core::ffi::c_int != tv_blob_get(b2, i) as ::core::ffi::c_int {
            return false_0 != 0;
        }
        i += 1;
    }
    return true_0 != 0;
}
unsafe extern "C" fn tv_blob_slice(
    mut _blob: *const blob_T,
    mut len: ::core::ffi::c_int,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
    mut exclusive: bool,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    if n1 < 0 as varnumber_T {
        n1 = len as varnumber_T + n1;
        if n1 < 0 as varnumber_T {
            n1 = 0 as varnumber_T;
        }
    }
    if n2 < 0 as varnumber_T {
        n2 = len as varnumber_T + n2;
    } else if n2 >= len as varnumber_T {
        n2 = (len
            - (if exclusive as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            })) as varnumber_T;
    }
    if exclusive {
        n2 -= 1;
    }
    if n1 >= len as varnumber_T || n2 < 0 as varnumber_T || n1 > n2 {
        tv_clear(rettv);
        (*rettv).v_type = VAR_BLOB;
        (*rettv).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
    } else {
        let new_blob: *mut blob_T = tv_blob_alloc();
        ga_grow(
            &raw mut (*new_blob).bv_ga,
            (n2 - n1 + 1 as varnumber_T) as ::core::ffi::c_int,
        );
        (*new_blob).bv_ga.ga_len = (n2 - n1 + 1 as varnumber_T) as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = n1 as ::core::ffi::c_int;
        while i <= n2 as ::core::ffi::c_int {
            tv_blob_set(
                new_blob,
                i - n1 as ::core::ffi::c_int,
                tv_blob_get((*rettv).vval.v_blob, i),
            );
            i += 1;
        }
        tv_clear(rettv);
        tv_blob_set_ret(rettv, new_blob);
    }
    return OK;
}
unsafe extern "C" fn tv_blob_index(
    mut _blob: *const blob_T,
    mut len: ::core::ffi::c_int,
    mut idx: varnumber_T,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    if idx < 0 as varnumber_T {
        idx = len as varnumber_T + idx;
    }
    if idx < len as varnumber_T && idx >= 0 as varnumber_T {
        let v: ::core::ffi::c_int =
            tv_blob_get((*rettv).vval.v_blob, idx as ::core::ffi::c_int) as ::core::ffi::c_int;
        tv_clear(rettv);
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = v as varnumber_T;
    } else {
        semsg(
            gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
            idx,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_slice_or_index(
    mut blob: *const blob_T,
    mut is_range: bool,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
    mut exclusive: bool,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = tv_blob_len((*rettv).vval.v_blob);
    if is_range {
        return tv_blob_slice(blob, len, n1, n2, exclusive, rettv);
    } else {
        return tv_blob_index(blob, len, n1, rettv);
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_check_index(
    mut bloblen: ::core::ffi::c_int,
    mut n1: varnumber_T,
    mut quiet: bool,
) -> ::core::ffi::c_int {
    if n1 < 0 as varnumber_T || n1 > bloblen as varnumber_T {
        if !quiet {
            semsg(
                gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                n1,
            );
        }
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_check_range(
    mut bloblen: ::core::ffi::c_int,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
    mut quiet: bool,
) -> ::core::ffi::c_int {
    if n2 < 0 as varnumber_T || n2 >= bloblen as varnumber_T || n2 < n1 {
        if !quiet {
            semsg(
                gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                n2,
            );
        }
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_set_range(
    mut dest: *mut blob_T,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
    mut src: *mut typval_T,
) -> ::core::ffi::c_int {
    if n2 - n1 + 1 as varnumber_T != tv_blob_len((*src).vval.v_blob) as varnumber_T {
        emsg(gettext(
            b"E972: Blob value does not have the right number of bytes\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    let mut il: ::core::ffi::c_int = n1 as ::core::ffi::c_int;
    let mut ir: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while il <= n2 as ::core::ffi::c_int {
        let c2rust_fresh9 = ir;
        ir = ir + 1;
        tv_blob_set(dest, il, tv_blob_get((*src).vval.v_blob, c2rust_fresh9));
        il += 1;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_set_append(
    mut blob: *mut blob_T,
    mut idx: ::core::ffi::c_int,
    mut byte: uint8_t,
) {
    let mut gap: *mut garray_T = &raw mut (*blob).bv_ga;
    if idx <= (*gap).ga_len {
        if idx == (*gap).ga_len {
            ga_grow(gap, 1 as ::core::ffi::c_int);
            (*gap).ga_len += 1;
        }
        tv_blob_set(blob, idx, byte);
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_remove(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
) {
    let b: *mut blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_blob;
    if !b.is_null()
        && value_check_lock((*b).bv_lock, arg_errmsg, TV_TRANSLATE as size_t) as ::core::ffi::c_int
            != 0
    {
        return;
    }
    let mut error: bool = false_0 != 0;
    let mut idx: int64_t = tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut error,
    );
    if !error {
        let len: ::core::ffi::c_int = tv_blob_len(b);
        if idx < 0 as int64_t {
            idx = len as int64_t + idx;
        }
        if idx < 0 as int64_t || idx >= len as int64_t {
            semsg(
                gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                idx,
            );
            return;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let p: *mut uint8_t = (*b).bv_ga.ga_data as *mut uint8_t;
            (*rettv).vval.v_number = *p.offset(idx as isize) as varnumber_T;
            memmove(
                p.offset(idx as isize) as *mut ::core::ffi::c_void,
                p.offset(idx as isize)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *const ::core::ffi::c_void,
                (len as int64_t - idx - 1 as int64_t) as size_t,
            );
            (*b).bv_ga.ga_len -= 1;
        } else {
            let mut end: int64_t = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            );
            if error {
                return;
            }
            if end < 0 as int64_t {
                end = len as int64_t + end;
            }
            if end >= len as int64_t || idx > end {
                semsg(
                    gettext(&raw const e_blobidx as *const ::core::ffi::c_char),
                    end,
                );
                return;
            }
            let blob: *mut blob_T = tv_blob_alloc();
            (*blob).bv_ga.ga_len = (end - idx + 1 as int64_t) as ::core::ffi::c_int;
            ga_grow(
                &raw mut (*blob).bv_ga,
                (end - idx + 1 as int64_t) as ::core::ffi::c_int,
            );
            let p_0: *mut uint8_t = (*b).bv_ga.ga_data as *mut uint8_t;
            memmove(
                (*blob).bv_ga.ga_data,
                p_0.offset(idx as isize) as *const ::core::ffi::c_void,
                (end - idx + 1 as int64_t) as size_t,
            );
            tv_blob_set_ret(rettv, blob);
            if len as int64_t - end - 1 as int64_t > 0 as int64_t {
                memmove(
                    p_0.offset(idx as isize) as *mut ::core::ffi::c_void,
                    p_0.offset(end as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    (len as int64_t - end - 1 as int64_t) as size_t,
                );
            }
            (*b).bv_ga.ga_len -= (end - idx + 1 as int64_t) as ::core::ffi::c_int;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_blob2list(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if tv_check_for_blob_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let blob: *mut blob_T = (*argvars).vval.v_blob;
    let l: *mut list_T = (*rettv).vval.v_list;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < tv_blob_len(blob) {
        tv_list_append_number(l, tv_blob_get(blob, i) as varnumber_T);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_list2blob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut blob: *mut blob_T = tv_blob_alloc_ret(rettv);
    if tv_check_for_list_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let l: *mut list_T = (*argvars).vval.v_list;
    if l.is_null() {
        return;
    }
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            let mut error: bool = false;
            let mut n: varnumber_T = tv_get_number_chk(&raw const (*li).li_tv, &raw mut error);
            if error as ::core::ffi::c_int != 0 || n < 0 as varnumber_T || n > 255 as varnumber_T {
                if !error {
                    semsg(
                        gettext(
                            &raw const e_invalid_value_for_blob_nr as *const ::core::ffi::c_char,
                        ),
                        n as ::core::ffi::c_int,
                    );
                }
                ga_clear(&raw mut (*blob).bv_ga);
                return;
            }
            ga_append(&raw mut (*blob).bv_ga, n as uint8_t);
            li = (*li).li_next;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T {
    let l: *mut list_T = tv_list_alloc(len);
    tv_list_set_ret(ret_tv, l);
    (*ret_tv).v_lock = VAR_UNLOCKED;
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_alloc_lock(mut lock: VarLockStatus) -> *mut dict_T {
    let d: *mut dict_T = tv_dict_alloc();
    (*d).dv_lock = lock;
    return d;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_alloc_ret(ret_tv: *mut typval_T) {
    let d: *mut dict_T = tv_dict_alloc_lock(VAR_UNLOCKED);
    tv_dict_set_ret(ret_tv, d);
}
unsafe extern "C" fn tv_dict2list(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    what: DictListType,
) {
    if tv_check_for_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
        return;
    }
    let mut d: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    tv_list_alloc_ret(rettv, tv_dict_len(d) as ptrdiff_t);
    if d.is_null() {
        return;
    }
    let dihi_ht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
    let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
    let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
    while dihi_todo_ != 0 {
        if !((*dihi_).hi_key.is_null()
            || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            dihi_todo_ = dihi_todo_.wrapping_sub(1);
            let di: *mut dictitem_T = (*dihi_)
                .hi_key
                .offset(-(17 as ::core::ffi::c_ulong as isize))
                as *mut dictitem_T;
            let mut tv_item: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            match what as ::core::ffi::c_uint {
                0 => {
                    tv_item.v_type = VAR_STRING;
                    tv_item.vval.v_string =
                        xstrdup(&raw mut (*di).di_key as *mut ::core::ffi::c_char);
                }
                1 => {
                    tv_copy(&raw mut (*di).di_tv, &raw mut tv_item);
                }
                2 => {
                    let sub_l: *mut list_T = tv_list_alloc(2 as ptrdiff_t);
                    tv_item.v_type = VAR_LIST;
                    tv_item.vval.v_list = sub_l;
                    tv_list_ref(sub_l);
                    tv_list_append_string(
                        sub_l,
                        &raw mut (*di).di_key as *mut ::core::ffi::c_char,
                        -1 as ssize_t,
                    );
                    tv_list_append_tv(sub_l, &raw mut (*di).di_tv);
                }
                _ => {}
            }
            tv_list_append_owned_tv((*rettv).vval.v_list, tv_item);
        }
        dihi_ = dihi_.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_items(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_string2items(argvars, rettv);
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_list2items(argvars, rettv);
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_blob2items(argvars, rettv);
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_dict2items(argvars, rettv);
    } else {
        semsg(
            gettext(
                (e_list_dict_blob_or_string_required_for_argument_nr.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            1 as ::core::ffi::c_int,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_keys(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict2list(argvars, rettv, kDict2ListKeys);
}
#[no_mangle]
pub unsafe extern "C" fn f_values(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict2list(argvars, rettv, kDict2ListValues);
}
#[no_mangle]
pub unsafe extern "C" fn f_has_key(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict
        .is_null()
    {
        return;
    }
    (*rettv).vval.v_number = !tv_dict_find(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict,
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
        -1 as ptrdiff_t,
    )
    .is_null() as ::core::ffi::c_int as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn tv_dict_remove(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
) {
    let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_toomanyarg as *const ::core::ffi::c_char),
            b"remove()\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        d = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if !d.is_null() && !value_check_lock((*d).dv_lock, arg_errmsg, TV_TRANSLATE as size_t) {
            let mut key: *const ::core::ffi::c_char =
                tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
            if !key.is_null() {
                let mut di: *mut dictitem_T = tv_dict_find(d, key, -1 as ptrdiff_t);
                if di.is_null() {
                    semsg(
                        gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                        key,
                    );
                } else if !var_check_fixed(
                    (*di).di_flags as ::core::ffi::c_int,
                    arg_errmsg,
                    TV_TRANSLATE as size_t,
                ) && !var_check_ro(
                    (*di).di_flags as ::core::ffi::c_int,
                    arg_errmsg,
                    TV_TRANSLATE as size_t,
                ) {
                    *rettv = (*di).di_tv;
                    (*di).di_tv = typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    tv_dict_item_remove(d, di);
                    if tv_dict_is_watched(d) {
                        tv_dict_watcher_notify(d, key, ::core::ptr::null_mut::<typval_T>(), rettv);
                    }
                }
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_alloc_ret(ret_tv: *mut typval_T) -> *mut blob_T {
    let b: *mut blob_T = tv_blob_alloc();
    tv_blob_set_ret(ret_tv, b);
    return b;
}
#[no_mangle]
pub unsafe extern "C" fn tv_blob_copy(from: *mut blob_T, to: *mut typval_T) {
    (*to).v_type = VAR_BLOB;
    (*to).v_lock = VAR_UNLOCKED;
    if from.is_null() {
        (*to).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
    } else {
        tv_blob_alloc_ret(to);
        let mut len: ::core::ffi::c_int = (*from).bv_ga.ga_len;
        if len > 0 as ::core::ffi::c_int {
            (*(*to).vval.v_blob).bv_ga.ga_data = xmemdup((*from).bv_ga.ga_data, len as size_t);
        }
        (*(*to).vval.v_blob).bv_ga.ga_len = len;
        (*(*to).vval.v_blob).bv_ga.ga_maxlen = len;
    };
}
pub const TYPVAL_ENCODE_ALLOW_SPECIALS: ::core::ffi::c_int = false_0;
#[inline(always)]
unsafe extern "C" fn _nothing_conv_func_start(
    tv: *mut typval_T,
    fun: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    (*tv).v_lock = VAR_UNLOCKED;
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let pt_: *mut partial_T = (*tv).vval.v_partial;
        if !pt_.is_null() && (*pt_).pt_refcount > 1 as ::core::ffi::c_int {
            (*pt_).pt_refcount -= 1;
            (*tv).vval.v_partial = ::core::ptr::null_mut::<partial_T>();
            return OK;
        }
    } else {
        func_unref(fun);
        if fun != tv_empty_string.get() as *mut ::core::ffi::c_char {
            xfree(fun as *mut ::core::ffi::c_void);
        }
        (*tv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return NOTDONE;
}
#[inline(always)]
unsafe extern "C" fn _nothing_conv_func_end(tv: *mut typval_T, copyID: ::core::ffi::c_int) {
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let pt: *mut partial_T = (*tv).vval.v_partial;
        if pt.is_null() {
            return;
        }
        '_c2rust_label: {
            if (*pt).pt_dict.is_null() || (*(*pt).pt_dict).dv_copyID == copyID {
            } else {
                __assert_fail(
                    b"pt->pt_dict == NULL || pt->pt_dict->dv_copyID == copyID\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3488 as ::core::ffi::c_uint,
                    b"void _nothing_conv_func_end(typval_T *const, const int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        (*pt).pt_dict = ::core::ptr::null_mut::<dict_T>();
        (*pt).pt_argc = 0 as ::core::ffi::c_int;
        '_c2rust_label_0: {
            if (*pt).pt_refcount <= 1 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"pt->pt_refcount <= 1\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3492 as ::core::ffi::c_uint,
                    b"void _nothing_conv_func_end(typval_T *const, const int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        partial_unref(pt);
        (*tv).vval.v_partial = ::core::ptr::null_mut::<partial_T>();
        '_c2rust_label_1: {
            if (*tv).v_lock as ::core::ffi::c_uint
                == VAR_UNLOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"tv->v_lock == VAR_UNLOCKED\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3495 as ::core::ffi::c_uint,
                    b"void _nothing_conv_func_end(typval_T *const, const int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
    }
}
#[inline(always)]
unsafe extern "C" fn _nothing_conv_empty_dict(tv: *mut typval_T, dictp: *mut *mut dict_T) {
    tv_dict_unref(*dictp);
    *dictp = ::core::ptr::null_mut::<dict_T>();
    if !tv.is_null() {
        (*tv).v_lock = VAR_UNLOCKED;
    }
}
#[inline(always)]
unsafe extern "C" fn _nothing_conv_real_list_after_start(
    tv: *mut typval_T,
    mpsv: *mut MPConvStackVal,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if !tv.is_null() {
        } else {
            __assert_fail(
                b"tv != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3526 as ::core::ffi::c_uint,
                b"int _nothing_conv_real_list_after_start(typval_T *const, MPConvStackVal *const)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    (*tv).v_lock = VAR_UNLOCKED;
    if (*(*tv).vval.v_list).lv_refcount > 1 as ::core::ffi::c_int {
        (*(*tv).vval.v_list).lv_refcount -= 1;
        (*tv).vval.v_list = ::core::ptr::null_mut::<list_T>();
        (*mpsv).data.l.li = ::core::ptr::null_mut::<listitem_T>();
        return OK;
    }
    return NOTDONE;
}
#[inline(always)]
unsafe extern "C" fn _nothing_conv_list_end(tv: *mut typval_T) {
    if tv.is_null() {
        return;
    }
    '_c2rust_label: {
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"tv->v_type == VAR_LIST\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3553 as ::core::ffi::c_uint,
                b"void _nothing_conv_list_end(typval_T *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let list: *mut list_T = (*tv).vval.v_list;
    tv_list_unref(list);
    (*tv).vval.v_list = ::core::ptr::null_mut::<list_T>();
}
#[inline(always)]
unsafe extern "C" fn _nothing_conv_real_dict_after_start(
    tv: *mut typval_T,
    dictp: *mut *mut dict_T,
    nodictvar: *const ::core::ffi::c_void,
    mpsv: *mut MPConvStackVal,
) -> ::core::ffi::c_int {
    if !tv.is_null() {
        (*tv).v_lock = VAR_UNLOCKED;
    }
    if dictp as *const ::core::ffi::c_void != nodictvar
        && (**dictp).dv_refcount > 1 as ::core::ffi::c_int
    {
        (**dictp).dv_refcount -= 1;
        *dictp = ::core::ptr::null_mut::<dict_T>();
        (*mpsv).data.d.todo = 0 as size_t;
        return OK;
    }
    return NOTDONE;
}
#[inline(always)]
unsafe extern "C" fn _nothing_conv_dict_end(
    _tv: *mut typval_T,
    dictp: *mut *mut dict_T,
    nodictvar: *const ::core::ffi::c_void,
) {
    if dictp as *const ::core::ffi::c_void != nodictvar {
        tv_dict_unref(*dictp);
        *dictp = ::core::ptr::null_mut::<dict_T>();
    }
}
#[no_mangle]
pub unsafe extern "C" fn tv_clear(tv: *mut typval_T) {
    if tv.is_null()
        || (*tv).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    let evn_ret: ::core::ffi::c_int = encode_vim_to_nothing(
        ::core::ptr::null::<::core::ffi::c_void>(),
        tv,
        b"tv_clear() argument\0".as_ptr() as *const ::core::ffi::c_char,
    );
    '_c2rust_label: {
        if evn_ret == 1 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"evn_ret == OK\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3663 as ::core::ffi::c_uint,
                b"void tv_clear(typval_T *const)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv_free(mut tv: *mut typval_T) {
    if tv.is_null() {
        return;
    }
    's_68: {
        match (*tv).v_type as ::core::ffi::c_uint {
            9 => {
                partial_unref((*tv).vval.v_partial);
                break 's_68;
            }
            3 => {
                func_unref((*tv).vval.v_string);
            }
            2 => {}
            10 => {
                tv_blob_unref((*tv).vval.v_blob);
                break 's_68;
            }
            4 => {
                tv_list_unref((*tv).vval.v_list);
                break 's_68;
            }
            5 => {
                tv_dict_unref((*tv).vval.v_dict);
                break 's_68;
            }
            7 | 8 | 1 | 6 | 0 | _ => {
                break 's_68;
            }
        }
        xfree((*tv).vval.v_string as *mut ::core::ffi::c_void);
    }
    xfree(tv as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tv_copy(from: *const typval_T, to: *mut typval_T) {
    (*to).v_type = (*from).v_type;
    (*to).v_lock = VAR_UNLOCKED;
    memmove(
        &raw mut (*to).vval as *mut ::core::ffi::c_void,
        &raw const (*from).vval as *const ::core::ffi::c_void,
        ::core::mem::size_of::<typval_vval_union>(),
    );
    match (*from).v_type as ::core::ffi::c_uint {
        2 | 3 => {
            if !(*from).vval.v_string.is_null() {
                (*to).vval.v_string = xstrdup((*from).vval.v_string);
                if (*from).v_type as ::core::ffi::c_uint
                    == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    func_ref((*to).vval.v_string);
                }
            }
        }
        9 => {
            if !(*to).vval.v_partial.is_null() {
                (*(*to).vval.v_partial).pt_refcount += 1;
            }
        }
        10 => {
            if !(*from).vval.v_blob.is_null() {
                (*(*to).vval.v_blob).bv_refcount += 1;
            }
        }
        4 => {
            tv_list_ref((*to).vval.v_list);
        }
        5 => {
            if !(*from).vval.v_dict.is_null() {
                (*(*to).vval.v_dict).dv_refcount += 1;
            }
        }
        0 => {
            semsg(
                gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                b"tv_copy(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        1 | 6 | 7 | 8 | _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv_item_lock(
    tv: *mut typval_T,
    deep: ::core::ffi::c_int,
    lock: bool,
    check_refcount: bool,
) {
    static recurse: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if recurse.get() >= DICT_MAXNEST {
        emsg(gettext(
            (e_variable_nested_too_deep_for_unlock.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return;
    }
    if deep == 0 as ::core::ffi::c_int {
        return;
    }
    (*recurse.ptr()) += 1;
    (*tv).v_lock = [
        (if lock as ::core::ffi::c_int != 0 {
            VAR_LOCKED as ::core::ffi::c_int
        } else {
            VAR_UNLOCKED as ::core::ffi::c_int
        }) as VarLockStatus,
        (if lock as ::core::ffi::c_int != 0 {
            VAR_LOCKED as ::core::ffi::c_int
        } else {
            VAR_UNLOCKED as ::core::ffi::c_int
        }) as VarLockStatus,
        VAR_FIXED,
    ][(*tv).v_lock as usize];
    match (*tv).v_type as ::core::ffi::c_uint {
        10 => {
            let b: *mut blob_T = (*tv).vval.v_blob;
            if !b.is_null()
                && !(check_refcount as ::core::ffi::c_int != 0
                    && (*b).bv_refcount > 1 as ::core::ffi::c_int)
            {
                (*b).bv_lock = [
                    (if lock as ::core::ffi::c_int != 0 {
                        VAR_LOCKED as ::core::ffi::c_int
                    } else {
                        VAR_UNLOCKED as ::core::ffi::c_int
                    }) as VarLockStatus,
                    (if lock as ::core::ffi::c_int != 0 {
                        VAR_LOCKED as ::core::ffi::c_int
                    } else {
                        VAR_UNLOCKED as ::core::ffi::c_int
                    }) as VarLockStatus,
                    VAR_FIXED,
                ][(*b).bv_lock as usize];
            }
        }
        4 => {
            let l: *mut list_T = (*tv).vval.v_list;
            if !l.is_null()
                && !(check_refcount as ::core::ffi::c_int != 0
                    && (*l).lv_refcount > 1 as ::core::ffi::c_int)
            {
                (*l).lv_lock = [
                    (if lock as ::core::ffi::c_int != 0 {
                        VAR_LOCKED as ::core::ffi::c_int
                    } else {
                        VAR_UNLOCKED as ::core::ffi::c_int
                    }) as VarLockStatus,
                    (if lock as ::core::ffi::c_int != 0 {
                        VAR_LOCKED as ::core::ffi::c_int
                    } else {
                        VAR_UNLOCKED as ::core::ffi::c_int
                    }) as VarLockStatus,
                    VAR_FIXED,
                ][(*l).lv_lock as usize];
                if deep < 0 as ::core::ffi::c_int || deep > 1 as ::core::ffi::c_int {
                    let l_: *mut list_T = l;
                    if !l_.is_null() {
                        let mut li: *mut listitem_T = (*l_).lv_first;
                        while !li.is_null() {
                            tv_item_lock(
                                &raw mut (*li).li_tv,
                                deep - 1 as ::core::ffi::c_int,
                                lock,
                                check_refcount,
                            );
                            li = (*li).li_next;
                        }
                    }
                }
            }
        }
        5 => {
            let d: *mut dict_T = (*tv).vval.v_dict;
            if !d.is_null()
                && !(check_refcount as ::core::ffi::c_int != 0
                    && (*d).dv_refcount > 1 as ::core::ffi::c_int)
            {
                (*d).dv_lock = [
                    (if lock as ::core::ffi::c_int != 0 {
                        VAR_LOCKED as ::core::ffi::c_int
                    } else {
                        VAR_UNLOCKED as ::core::ffi::c_int
                    }) as VarLockStatus,
                    (if lock as ::core::ffi::c_int != 0 {
                        VAR_LOCKED as ::core::ffi::c_int
                    } else {
                        VAR_UNLOCKED as ::core::ffi::c_int
                    }) as VarLockStatus,
                    VAR_FIXED,
                ][(*d).dv_lock as usize];
                if deep < 0 as ::core::ffi::c_int || deep > 1 as ::core::ffi::c_int {
                    let dihi_ht_: *mut hashtab_T = &raw mut (*d).dv_hashtab;
                    let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
                    let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
                    while dihi_todo_ != 0 {
                        if !((*dihi_).hi_key.is_null()
                            || (*dihi_).hi_key
                                == &raw const hash_removed as *mut ::core::ffi::c_char)
                        {
                            dihi_todo_ = dihi_todo_.wrapping_sub(1);
                            let di: *mut dictitem_T = (*dihi_)
                                .hi_key
                                .offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T;
                            tv_item_lock(
                                &raw mut (*di).di_tv,
                                deep - 1 as ::core::ffi::c_int,
                                lock,
                                check_refcount,
                            );
                        }
                        dihi_ = dihi_.offset(1);
                    }
                }
            }
        }
        0 => {
            abort();
        }
        1 | 6 | 2 | 3 | 9 | 7 | 8 | _ => {}
    }
    (*recurse.ptr()) -= 1;
}
#[no_mangle]
pub unsafe extern "C" fn tv_islocked(tv: *const typval_T) -> bool {
    return (*tv).v_lock as ::core::ffi::c_uint
        == VAR_LOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            && tv_list_locked((*tv).vval.v_list) as ::core::ffi::c_uint
                == VAR_LOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*tv).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*tv).vval.v_dict.is_null()
            && (*(*tv).vval.v_dict).dv_lock as ::core::ffi::c_uint
                == VAR_LOCKED as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_lock(
    mut tv: *const typval_T,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    let mut lock: VarLockStatus = VAR_UNLOCKED;
    match (*tv).v_type as ::core::ffi::c_uint {
        10 => {
            if !(*tv).vval.v_blob.is_null() {
                lock = (*(*tv).vval.v_blob).bv_lock;
            }
        }
        4 => {
            if !(*tv).vval.v_list.is_null() {
                lock = (*(*tv).vval.v_list).lv_lock;
            }
        }
        5 => {
            if !(*tv).vval.v_dict.is_null() {
                lock = (*(*tv).vval.v_dict).dv_lock;
            }
        }
        _ => {}
    }
    return value_check_lock((*tv).v_lock, name, name_len) as ::core::ffi::c_int != 0
        || lock as ::core::ffi::c_uint
            != VAR_UNLOCKED as ::core::ffi::c_int as ::core::ffi::c_uint
            && value_check_lock(lock, name, name_len) as ::core::ffi::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn value_check_lock(
    mut lock: VarLockStatus,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    let mut error_message: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    match lock as ::core::ffi::c_uint {
        0 => return false_0 != 0,
        1 => {
            error_message = if name.is_null() {
                &raw const e_value_is_locked as *const ::core::ffi::c_char
            } else {
                &raw const e_value_is_locked_str as *const ::core::ffi::c_char
            };
        }
        2 => {
            error_message = if name.is_null() {
                &raw const e_cannot_change_value as *const ::core::ffi::c_char
            } else {
                &raw const e_cannot_change_value_of_str as *const ::core::ffi::c_char
            };
        }
        _ => {}
    }
    '_c2rust_label: {
        if !error_message.is_null() {
        } else {
            __assert_fail(
                b"error_message != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3926 as ::core::ffi::c_uint,
                b"_Bool value_check_lock(VarLockStatus, const char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if name.is_null() {
        emsg(gettext(error_message));
    } else {
        if name_len == TV_TRANSLATE as size_t {
            name = gettext(name);
            name_len = strlen(name);
        } else if name_len == TV_CSTRING as size_t {
            name_len = strlen(name);
        }
        semsg(gettext(error_message), name_len as ::core::ffi::c_int, name);
    }
    return true_0 != 0;
}
static tv_equal_recurse_limit: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
#[no_mangle]
pub unsafe extern "C" fn tv_equal(tv1: *mut typval_T, tv2: *mut typval_T, ic: bool) -> bool {
    static recursive_cnt: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if !(tv_is_func(*tv1) as ::core::ffi::c_int != 0 && tv_is_func(*tv2) as ::core::ffi::c_int != 0)
        && (*tv1).v_type as ::core::ffi::c_uint != (*tv2).v_type as ::core::ffi::c_uint
    {
        return false_0 != 0;
    }
    if recursive_cnt.get() == 0 as ::core::ffi::c_int {
        tv_equal_recurse_limit.set(1000 as ::core::ffi::c_int);
    }
    if recursive_cnt.get() >= tv_equal_recurse_limit.get() {
        (*tv_equal_recurse_limit.ptr()) -= 1;
        return true_0 != 0;
    }
    match (*tv1).v_type as ::core::ffi::c_uint {
        4 => {
            (*recursive_cnt.ptr()) += 1;
            let r: bool = tv_list_equal((*tv1).vval.v_list, (*tv2).vval.v_list, ic);
            (*recursive_cnt.ptr()) -= 1;
            return r;
        }
        5 => {
            (*recursive_cnt.ptr()) += 1;
            let r_0: bool = tv_dict_equal((*tv1).vval.v_dict, (*tv2).vval.v_dict, ic);
            (*recursive_cnt.ptr()) -= 1;
            return r_0;
        }
        9 | 3 => {
            if (*tv1).v_type as ::core::ffi::c_uint
                == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*tv1).vval.v_partial.is_null()
                || (*tv2).v_type as ::core::ffi::c_uint
                    == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*tv2).vval.v_partial.is_null()
            {
                return false_0 != 0;
            }
            (*recursive_cnt.ptr()) += 1;
            let r_1: bool = func_equal(tv1, tv2, ic);
            (*recursive_cnt.ptr()) -= 1;
            return r_1;
        }
        10 => return tv_blob_equal((*tv1).vval.v_blob, (*tv2).vval.v_blob),
        1 => return (*tv1).vval.v_number == (*tv2).vval.v_number,
        6 => return (*tv1).vval.v_float == (*tv2).vval.v_float,
        2 => {
            let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
            let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
            let mut s1: *const ::core::ffi::c_char =
                tv_get_string_buf(tv1, &raw mut buf1 as *mut ::core::ffi::c_char);
            let mut s2: *const ::core::ffi::c_char =
                tv_get_string_buf(tv2, &raw mut buf2 as *mut ::core::ffi::c_char);
            return mb_strcmp_ic(ic, s1, s2) == 0 as ::core::ffi::c_int;
        }
        7 => {
            return (*tv1).vval.v_bool as ::core::ffi::c_uint
                == (*tv2).vval.v_bool as ::core::ffi::c_uint;
        }
        8 => {
            return (*tv1).vval.v_special as ::core::ffi::c_uint
                == (*tv2).vval.v_special as ::core::ffi::c_uint;
        }
        0 => return false_0 != 0,
        _ => {}
    }
    abort();
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_str_or_nr(tv: *const typval_T) -> bool {
    match (*tv).v_type as ::core::ffi::c_uint {
        1 | 2 => return true_0 != 0,
        6 => {
            emsg(gettext(
                b"E805: Expected a Number or a String, Float found\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return false_0 != 0;
        }
        9 | 3 => {
            emsg(gettext(
                b"E703: Expected a Number or a String, Funcref found\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return false_0 != 0;
        }
        4 => {
            emsg(gettext(
                b"E745: Expected a Number or a String, List found\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return false_0 != 0;
        }
        5 => {
            emsg(gettext(
                b"E728: Expected a Number or a String, Dictionary found\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return false_0 != 0;
        }
        10 => {
            emsg(gettext(
                b"E974: Expected a Number or a String, Blob found\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return false_0 != 0;
        }
        7 => {
            emsg(gettext(
                b"E5299: Expected a Number or a String, Boolean found\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return false_0 != 0;
        }
        8 => {
            emsg(gettext(
                b"E5300: Expected a Number or a String\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return false_0 != 0;
        }
        0 => {
            semsg(
                gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                b"tv_check_str_or_nr(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return false_0 != 0;
        }
        _ => {}
    }
    abort();
}
static num_errors: GlobalCell<[*const ::core::ffi::c_char; 11]> = GlobalCell::new([
    b"E685: using an invalid value as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"E703: Using a Funcref as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"E745: Using a List as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"E728: Using a Dictionary as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"E805: Using a Float as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"E703: Using a Funcref as a Number\0".as_ptr() as *const ::core::ffi::c_char,
    b"E974: Using a Blob as a Number\0".as_ptr() as *const ::core::ffi::c_char,
]);
#[no_mangle]
pub unsafe extern "C" fn tv_check_num(tv: *const typval_T) -> bool {
    match (*tv).v_type as ::core::ffi::c_uint {
        1 | 7 | 8 | 2 => return true_0 != 0,
        3 | 9 | 4 | 5 | 6 | 10 | 0 => {
            emsg(gettext((*num_errors.ptr())[(*tv).v_type as usize]));
            return false_0 != 0;
        }
        _ => {}
    }
    abort();
}
static str_errors: GlobalCell<[*const ::core::ffi::c_char; 11]> = GlobalCell::new([
    (e_using_invalid_value_as_string.as_raw() as *const _) as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"E729: Using a Funcref as a String\0".as_ptr() as *const ::core::ffi::c_char,
    b"E730: Using a List as a String\0".as_ptr() as *const ::core::ffi::c_char,
    b"E731: Using a Dictionary as a String\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"E729: Using a Funcref as a String\0".as_ptr() as *const ::core::ffi::c_char,
    b"E976: Using a Blob as a String\0".as_ptr() as *const ::core::ffi::c_char,
]);
#[no_mangle]
pub unsafe extern "C" fn tv_check_str(tv: *const typval_T) -> bool {
    match (*tv).v_type as ::core::ffi::c_uint {
        1 | 7 | 8 | 2 | 6 => return true_0 != 0,
        9 | 3 | 4 | 5 | 10 | 0 => {
            emsg(gettext((*str_errors.ptr())[(*tv).v_type as usize]));
            return false_0 != 0;
        }
        _ => {}
    }
    abort();
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_number(tv: *const typval_T) -> varnumber_T {
    let mut error: bool = false_0 != 0;
    return tv_get_number_chk(tv, &raw mut error);
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_number_chk(
    tv: *const typval_T,
    ret_error: *mut bool,
) -> varnumber_T {
    match (*tv).v_type as ::core::ffi::c_uint {
        3 | 9 | 4 | 5 | 10 | 6 => {
            emsg(gettext((*num_errors.ptr())[(*tv).v_type as usize]));
        }
        1 => return (*tv).vval.v_number,
        2 => {
            let mut n: varnumber_T = 0 as varnumber_T;
            if !(*tv).vval.v_string.is_null() {
                vim_str2nr(
                    (*tv).vval.v_string,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    STR2NR_ALL as ::core::ffi::c_int,
                    &raw mut n,
                    ::core::ptr::null_mut::<uvarnumber_T>(),
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                    ::core::ptr::null_mut::<bool>(),
                );
            }
            return n;
        }
        7 => {
            return (if (*tv).vval.v_bool as ::core::ffi::c_uint
                == kBoolVarTrue as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as varnumber_T;
        }
        8 => return 0 as varnumber_T,
        0 => {
            semsg(
                gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                b"tv_get_number(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        _ => {}
    }
    if !ret_error.is_null() {
        *ret_error = true_0 != 0;
    }
    return (if ret_error.is_null() {
        -1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_bool(tv: *const typval_T) -> varnumber_T {
    return tv_get_number_chk(tv, ::core::ptr::null_mut::<bool>());
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_bool_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T {
    return tv_get_number_chk(tv, ret_error);
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_lnum(tv: *const typval_T) -> linenr_T {
    let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
    let mut lnum: linenr_T = tv_get_number_chk(tv, ::core::ptr::null_mut::<bool>()) as linenr_T;
    if lnum <= 0 as linenr_T
        && did_emsg_before == did_emsg.get()
        && (*tv).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut fnum: ::core::ffi::c_int = 0;
        let fp: *mut pos_T = var2fpos(tv, true_0 != 0, &raw mut fnum, false_0 != 0, curwin.get());
        if !fp.is_null() {
            lnum = (*fp).lnum;
        }
    }
    return lnum;
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_lnum_buf(tv: *const typval_T, buf: *const buf_T) -> linenr_T {
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*tv).vval.v_string.is_null()
        && *(*tv).vval.v_string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '$' as ::core::ffi::c_int
        && *(*tv).vval.v_string.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == NUL
        && !buf.is_null()
    {
        return (*buf).b_ml.ml_line_count;
    }
    return tv_get_number_chk(tv, ::core::ptr::null_mut::<bool>()) as linenr_T;
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_float(tv: *const typval_T) -> float_T {
    match (*tv).v_type as ::core::ffi::c_uint {
        1 => return (*tv).vval.v_number as float_T,
        6 => return (*tv).vval.v_float,
        9 | 3 => {
            emsg(gettext(
                b"E891: Using a Funcref as a Float\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        2 => {
            emsg(gettext(
                b"E892: Using a String as a Float\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        4 => {
            emsg(gettext(
                b"E893: Using a List as a Float\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        5 => {
            emsg(gettext(
                b"E894: Using a Dictionary as a Float\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        7 => {
            emsg(gettext(
                b"E362: Using a boolean value as a Float\0".as_ptr() as *const ::core::ffi::c_char,
            ));
        }
        8 => {
            emsg(gettext(
                b"E907: Using a special value as a Float\0".as_ptr() as *const ::core::ffi::c_char,
            ));
        }
        10 => {
            emsg(gettext(
                b"E975: Using a Blob as a Float\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        0 => {
            semsg(
                gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                b"tv_get_float(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int as float_T;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_string_required_for_argument_nr.ptr() as *const _) as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_nonempty_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if tv_check_for_string_arg(args, idx) == FAIL {
        return FAIL;
    }
    if (*args.offset(idx as isize)).vval.v_string.is_null()
        || *(*args.offset(idx as isize)).vval.v_string as ::core::ffi::c_int == NUL
    {
        semsg(
            gettext(
                (e_non_empty_string_required_for_argument_nr.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_opt_string_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv_check_for_string_arg(args, idx) != FAIL
    {
        OK
    } else {
        FAIL
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_number_required_for_argument_nr.ptr() as *const _) as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_opt_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv_check_for_number_arg(args, idx) != FAIL
    {
        OK
    } else {
        FAIL
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_float_or_nr_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_float_or_number_required_for_argument_nr.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_bool_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
        && !((*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            && ((*args.offset(idx as isize)).vval.v_number == 0 as varnumber_T
                || (*args.offset(idx as isize)).vval.v_number == 1 as varnumber_T))
    {
        semsg(
            gettext(
                (e_bool_required_for_argument_nr.ptr() as *const _) as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_opt_bool_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return OK;
    }
    return tv_check_for_bool_arg(args, idx);
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_blob_required_for_argument_nr.ptr() as *const _) as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_list_required_for_argument_nr.ptr() as *const _) as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_dict_required_for_argument_nr.ptr() as *const _) as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_nonnull_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if tv_check_for_dict_arg(args, idx) == FAIL {
        return FAIL;
    }
    if (*args.offset(idx as isize)).vval.v_dict.is_null() {
        semsg(
            gettext(
                (e_non_null_dict_required_for_argument_nr.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_opt_dict_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv_check_for_dict_arg(args, idx) != FAIL
    {
        OK
    } else {
        FAIL
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_string_or_number_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_string_or_number_required_for_argument_nr.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_buffer_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return tv_check_for_string_or_number_arg(args, idx);
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_lnum_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return tv_check_for_string_or_number_arg(args, idx);
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_string_or_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_string_or_list_required_for_argument_nr.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_string_or_list_or_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_string_list_or_blob_required_for_argument_nr.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_opt_string_or_list_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv_check_for_string_or_list_arg(args, idx) != FAIL
    {
        OK
    } else {
        FAIL
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_string_or_func_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_string_or_function_required_for_argument_nr.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_check_for_list_or_blob_arg(
    args: *const typval_T,
    idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*args.offset(idx as isize)).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(
                (e_list_or_blob_required_for_argument_nr.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ),
            idx + 1 as ::core::ffi::c_int,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_string_buf_chk(
    tv: *const typval_T,
    buf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    match (*tv).v_type as ::core::ffi::c_uint {
        1 => {
            snprintf(
                buf,
                NUMBUFLEN as ::core::ffi::c_int as size_t,
                b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                (*tv).vval.v_number,
            );
            return buf;
        }
        6 => {
            vim_snprintf(
                buf,
                NUMBUFLEN as ::core::ffi::c_int as size_t,
                b"%g\0".as_ptr() as *const ::core::ffi::c_char,
                (*tv).vval.v_float,
            );
            return buf;
        }
        2 => {
            if !(*tv).vval.v_string.is_null() {
                return (*tv).vval.v_string;
            }
            return b"\0".as_ptr() as *const ::core::ffi::c_char;
        }
        7 => {
            strcpy(
                buf,
                *(&raw const encode_bool_var_names as *const *const ::core::ffi::c_char)
                    .offset((*tv).vval.v_bool as isize) as *mut ::core::ffi::c_char,
            );
            return buf;
        }
        8 => {
            strcpy(
                buf,
                *(&raw const encode_special_var_names as *const *const ::core::ffi::c_char)
                    .offset((*tv).vval.v_special as isize)
                    as *mut ::core::ffi::c_char,
            );
            return buf;
        }
        9 | 3 | 4 | 5 | 10 | 0 => {
            emsg(gettext((*str_errors.ptr())[(*tv).v_type as usize]));
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        _ => {}
    }
    abort();
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_string_chk(tv: *const typval_T) -> *const ::core::ffi::c_char {
    static mybuf: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new([0; 65]);
    return tv_get_string_buf_chk(tv, mybuf.ptr() as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char {
    static mybuf: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new([0; 65]);
    return tv_get_string_buf(tv as *mut typval_T, mybuf.ptr() as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn tv_get_string_buf(
    tv: *const typval_T,
    buf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let res: *const ::core::ffi::c_char = tv_get_string_buf_chk(tv, buf);
    return if !res.is_null() {
        res
    } else {
        b"\0".as_ptr() as *const ::core::ffi::c_char
    };
}
#[no_mangle]
pub unsafe extern "C" fn tv2bool(tv: *const typval_T) -> bool {
    match (*tv).v_type as ::core::ffi::c_uint {
        1 => return (*tv).vval.v_number != 0 as varnumber_T,
        6 => return (*tv).vval.v_float != 0.0f64,
        9 => return !(*tv).vval.v_partial.is_null(),
        3 | 2 => {
            return !(*tv).vval.v_string.is_null()
                && *(*tv).vval.v_string as ::core::ffi::c_int != NUL;
        }
        4 => {
            return !(*tv).vval.v_list.is_null()
                && (*(*tv).vval.v_list).lv_len > 0 as ::core::ffi::c_int;
        }
        5 => {
            return !(*tv).vval.v_dict.is_null()
                && (*(*tv).vval.v_dict).dv_hashtab.ht_used > 0 as size_t;
        }
        7 => {
            return (*tv).vval.v_bool as ::core::ffi::c_uint
                == kBoolVarTrue as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        8 => {
            return (*tv).vval.v_special as ::core::ffi::c_uint
                != kSpecialVarNull as ::core::ffi::c_int as ::core::ffi::c_uint;
        }
        10 => {
            return !(*tv).vval.v_blob.is_null()
                && (*(*tv).vval.v_blob).bv_ga.ga_len > 0 as ::core::ffi::c_int;
        }
        0 | _ => {}
    }
    return false_0 != 0;
}
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0 as linenr_T,
    fe_lastline: 0 as linenr_T,
    fe_doesrange: ::core::ptr::null_mut::<bool>(),
    fe_evaluate: false_0 != 0,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false_0 != 0,
};
#[no_mangle]
pub static _typval_encode_nothing_nodict_var: GlobalCell<*const dict_T> =
    GlobalCell::new(::core::ptr::null::<dict_T>());
#[inline(always)]
unsafe extern "C" fn _typval_encode_nothing_check_self_reference(
    _ignored: *const ::core::ffi::c_void,
    _val: *mut ::core::ffi::c_void,
    val_copyID: *mut ::core::ffi::c_int,
    _mpstack: *const MPConvStack,
    copyID: ::core::ffi::c_int,
    _conv_type: MPConvStackValType,
    _objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if *val_copyID == copyID {
        return OK;
    }
    *val_copyID = copyID;
    return NOTDONE;
}
unsafe extern "C" fn _typval_encode_nothing_convert_one_value(
    ignored: *const ::core::ffi::c_void,
    mpstack: *mut MPConvStack,
    _cur_mpsv: *mut MPConvStackVal,
    tv: *mut typval_T,
    copyID: ::core::ffi::c_int,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    '_typval_encode_stop_converting_one_item: {
        match (*tv).v_type as ::core::ffi::c_uint {
            2 => {
                xfree((*tv).vval.v_string as *mut ::core::ffi::c_void);
                (*tv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
                (*tv).v_lock = VAR_UNLOCKED;
            }
            1 => {
                (*tv).vval.v_number = 0 as varnumber_T;
                (*tv).v_lock = VAR_UNLOCKED;
            }
            6 => {
                (*tv).vval.v_float = 0 as ::core::ffi::c_int as float_T;
                (*tv).v_lock = VAR_UNLOCKED;
            }
            10 => {
                tv_blob_unref((*tv).vval.v_blob);
                (*tv).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
                (*tv).v_lock = VAR_UNLOCKED;
            }
            3 => {
                if _nothing_conv_func_start(tv, (*tv).vval.v_string) != NOTDONE {
                    return OK;
                }
                _nothing_conv_func_end(tv, copyID);
            }
            9 => {
                let pt: *mut partial_T = (*tv).vval.v_partial;
                let fun: *mut ::core::ffi::c_char = if pt.is_null() {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    partial_name(pt)
                };
                let _prefix: *const ::core::ffi::c_char = if !fun.is_null()
                    && !pt.is_null()
                    && (*pt).pt_name.is_null()
                    && (*fun.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                        >= 'A' as ::core::ffi::c_uint
                        && *fun.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                            <= 'Z' as ::core::ffi::c_uint)
                {
                    b"g:\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                };
                if _nothing_conv_func_start(tv, fun) != NOTDONE {
                    return OK;
                }
                if (*mpstack).size == (*mpstack).capacity {
                    (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                            .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                            .wrapping_div(
                                (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        (*mpstack).capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[MPConvStackVal; 8]>()
                            .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                            .wrapping_div(
                                (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    (*mpstack).items = (if (*mpstack).capacity
                        == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                            .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                            .wrapping_div(
                                (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                    .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if (*mpstack).items == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                        {
                            (*mpstack).items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                    as *mut ::core::ffi::c_void,
                                (*mpstack).items as *mut ::core::ffi::c_void,
                                (*mpstack)
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                            )
                        }
                    } else {
                        if (*mpstack).items == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                        {
                            memcpy(
                                xmalloc(
                                    (*mpstack)
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                ),
                                (*mpstack).items as *const ::core::ffi::c_void,
                                (*mpstack)
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                            )
                        } else {
                            xrealloc(
                                (*mpstack).items as *mut ::core::ffi::c_void,
                                (*mpstack)
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                            )
                        }
                    }) as *mut MPConvStackVal;
                } else {
                };
                let c2rust_fresh3 = (*mpstack).size;
                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                *(*mpstack).items.offset(c2rust_fresh3 as isize) = MPConvStackVal {
                    type_0: kMPConvPartial,
                    tv: tv,
                    saved_copyID: copyID - 1 as ::core::ffi::c_int,
                    data: C2Rust_Unnamed_18 {
                        p: C2Rust_Unnamed_20 {
                            stage: kMPConvPartialArgs,
                            pt: (*tv).vval.v_partial,
                        },
                    },
                };
            }
            4 => {
                if (*tv).vval.v_list.is_null()
                    || tv_list_len((*tv).vval.v_list) == 0 as ::core::ffi::c_int
                {
                    tv_list_unref((*tv).vval.v_list);
                    (*tv).vval.v_list = ::core::ptr::null_mut::<list_T>();
                    (*tv).v_lock = VAR_UNLOCKED;
                } else {
                    let saved_copyID: ::core::ffi::c_int = tv_list_copyid((*tv).vval.v_list);
                    let te_csr_ret: ::core::ffi::c_int =
                        _typval_encode_nothing_check_self_reference(
                            ignored,
                            (*tv).vval.v_list as *mut ::core::ffi::c_void,
                            &raw mut (*(*tv).vval.v_list).lv_copyID,
                            mpstack,
                            copyID,
                            kMPConvList,
                            objname,
                        );
                    if te_csr_ret != NOTDONE {
                        return te_csr_ret;
                    }
                    '_c2rust_label: {
                        if saved_copyID != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/typval.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                383 as ::core::ffi::c_uint,
                                b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if (*mpstack).size == (*mpstack).capacity {
                        (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*mpstack).capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*mpstack).items = (if (*mpstack).capacity
                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                (*mpstack).items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                        as *mut ::core::ffi::c_void,
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        } else {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                memcpy(
                                    xmalloc(
                                        (*mpstack)
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    ),
                                    (*mpstack).items as *const ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            } else {
                                xrealloc(
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        }) as *mut MPConvStackVal;
                    } else {
                    };
                    let c2rust_fresh4 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh4 as isize) = MPConvStackVal {
                        type_0: kMPConvList,
                        tv: tv,
                        saved_copyID: saved_copyID,
                        data: C2Rust_Unnamed_18 {
                            l: C2Rust_Unnamed_21 {
                                list: (*tv).vval.v_list,
                                li: tv_list_first((*tv).vval.v_list),
                            },
                        },
                    };
                    let _ = _nothing_conv_real_list_after_start(
                        tv,
                        (*mpstack).items.offset(
                            (*mpstack)
                                .size
                                .wrapping_sub(0 as size_t)
                                .wrapping_sub(1 as size_t) as isize,
                        ),
                    ) != NOTDONE;
                }
            }
            7 => match (*tv).vval.v_bool as ::core::ffi::c_uint {
                1 | 0 => {
                    (*tv).vval.v_bool = kBoolVarFalse;
                    (*tv).v_lock = VAR_UNLOCKED;
                }
                _ => {}
            },
            8 => match (*tv).vval.v_special as ::core::ffi::c_uint {
                0 => {
                    (*tv).vval.v_special = kSpecialVarNull;
                    (*tv).v_lock = VAR_UNLOCKED;
                }
                _ => {}
            },
            5 => {
                if (*tv).vval.v_dict.is_null()
                    || (*(*tv).vval.v_dict).dv_hashtab.ht_used == 0 as size_t
                {
                    '_c2rust_label_0: {
                        if &raw mut (*tv).vval.v_dict as *mut ::core::ffi::c_void
                            != (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                as *mut ::core::ffi::c_void
                        {
                        } else {
                            __assert_fail(
                                b"(void *)&(tv->vval.v_dict) != (void *)&TYPVAL_ENCODE_NODICT_VAR\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/eval/typval.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                416 as ::core::ffi::c_uint,
                                b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    _nothing_conv_empty_dict(tv, &raw mut (*tv).vval.v_dict);
                } else {
                    let mut type_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    let mut val_di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
                    's_771: {
                        if TYPVAL_ENCODE_ALLOW_SPECIALS != 0
                            && (*(*tv).vval.v_dict).dv_hashtab.ht_used == 2 as size_t
                            && {
                                type_di = tv_dict_find(
                                    (*tv).vval.v_dict,
                                    b"_TYPE\0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                                        .wrapping_sub(1 as usize)
                                        as ptrdiff_t,
                                );
                                !type_di.is_null()
                            }
                            && (*type_di).di_tv.v_type as ::core::ffi::c_uint
                                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                            && {
                                val_di = tv_dict_find(
                                    (*tv).vval.v_dict,
                                    b"_VAL\0".as_ptr() as *const ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                        .wrapping_sub(1 as usize)
                                        as ptrdiff_t,
                                );
                                !val_di.is_null()
                            }
                        {
                            let mut i: size_t = 0;
                            i = 0 as size_t;
                            while i < ::core::mem::size_of::<[*const list_T; 8]>()
                                .wrapping_div(::core::mem::size_of::<*const list_T>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[*const list_T; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<*const list_T>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                )
                            {
                                if (*type_di).di_tv.vval.v_list
                                    == (*eval_msgpack_type_lists.ptr())[i as usize] as *mut list_T
                                {
                                    break;
                                }
                                i = i.wrapping_add(1);
                            }
                            if i != ::core::mem::size_of::<[*const list_T; 8]>()
                                .wrapping_div(::core::mem::size_of::<*const list_T>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[*const list_T; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<*const list_T>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                )
                            {
                                match i as MessagePackType as ::core::ffi::c_uint {
                                    0 => {
                                        (*tv).vval.v_special = kSpecialVarNull;
                                        (*tv).v_lock = VAR_UNLOCKED;
                                        break '_typval_encode_stop_converting_one_item;
                                    }
                                    1 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_NUMBER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*tv).vval.v_bool = kBoolVarFalse;
                                            (*tv).v_lock = VAR_UNLOCKED;
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    2 => {
                                        let mut val_list: *const list_T =
                                            ::core::ptr::null::<list_T>();
                                        let mut sign: varnumber_T = 0;
                                        let mut highest_bits: varnumber_T = 0;
                                        let mut high_bits: varnumber_T = 0;
                                        let mut low_bits: varnumber_T = 0;
                                        if !((*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            != VAR_LIST as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || {
                                                val_list = (*val_di).di_tv.vval.v_list;
                                                tv_list_len(val_list) != 4 as ::core::ffi::c_int
                                            })
                                        {
                                            let sign_li: *const listitem_T =
                                                tv_list_first(val_list);
                                            if !((*sign_li).li_tv.v_type as ::core::ffi::c_uint
                                                != VAR_NUMBER as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                                || {
                                                    sign = (*sign_li).li_tv.vval.v_number;
                                                    sign == 0 as varnumber_T
                                                })
                                            {
                                                let highest_bits_li: *const listitem_T =
                                                    (*sign_li).li_next;
                                                if !((*highest_bits_li).li_tv.v_type
                                                    as ::core::ffi::c_uint
                                                    != VAR_NUMBER as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                    || {
                                                        highest_bits =
                                                            (*highest_bits_li).li_tv.vval.v_number;
                                                        highest_bits < 0 as varnumber_T
                                                    })
                                                {
                                                    let high_bits_li: *const listitem_T =
                                                        (*highest_bits_li).li_next;
                                                    if !((*high_bits_li).li_tv.v_type
                                                        as ::core::ffi::c_uint
                                                        != VAR_NUMBER as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                        || {
                                                            high_bits =
                                                                (*high_bits_li).li_tv.vval.v_number;
                                                            high_bits < 0 as varnumber_T
                                                        })
                                                    {
                                                        let low_bits_li: *const listitem_T =
                                                            tv_list_last(val_list);
                                                        if !((*low_bits_li).li_tv.v_type
                                                            as ::core::ffi::c_uint
                                                            != VAR_NUMBER as ::core::ffi::c_int
                                                                as ::core::ffi::c_uint
                                                            || {
                                                                low_bits = (*low_bits_li)
                                                                    .li_tv
                                                                    .vval
                                                                    .v_number;
                                                                low_bits < 0 as varnumber_T
                                                            })
                                                        {
                                                            let number: uint64_t = (highest_bits
                                                                as uint64_t)
                                                                << 62 as ::core::ffi::c_int
                                                                | (high_bits as uint64_t)
                                                                    << 31 as ::core::ffi::c_int
                                                                | low_bits as uint64_t;
                                                            if sign <= 0 as varnumber_T {
                                                                let _ = number.wrapping_neg();
                                                                (*tv).vval.v_number =
                                                                    0 as varnumber_T;
                                                                (*tv).v_lock = VAR_UNLOCKED;
                                                            }
                                                            break '_typval_encode_stop_converting_one_item;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    3 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_FLOAT as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        {
                                            (*tv).vval.v_float = 0 as ::core::ffi::c_int as float_T;
                                            (*tv).v_lock = VAR_UNLOCKED;
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    4 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let mut len: size_t = 0;
                                            let mut buf: *mut ::core::ffi::c_char =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            if encode_vim_list_to_buf(
                                                (*val_di).di_tv.vval.v_list,
                                                &raw mut len,
                                                &raw mut buf,
                                            ) {
                                                xfree(buf as *mut ::core::ffi::c_void);
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    5 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let saved_copyID_0: ::core::ffi::c_int =
                                                tv_list_copyid((*val_di).di_tv.vval.v_list);
                                            let te_csr_ret_0: ::core::ffi::c_int =
                                                _typval_encode_nothing_check_self_reference(
                                                    ignored,
                                                    (*val_di).di_tv.vval.v_list
                                                        as *mut ::core::ffi::c_void,
                                                    &raw mut (*(*val_di).di_tv.vval.v_list)
                                                        .lv_copyID,
                                                    mpstack,
                                                    copyID,
                                                    kMPConvList,
                                                    objname,
                                                );
                                            if te_csr_ret_0 != NOTDONE {
                                                return te_csr_ret_0;
                                            }
                                            '_c2rust_label_1: {
                                                if saved_copyID_0 != copyID
                                                    && saved_copyID_0
                                                        != copyID - 1 as ::core::ffi::c_int
                                                {
                                                } else {
                                                    __assert_fail(
                                                        b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        b"src/nvim/eval/typval.rs\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                        532 as ::core::ffi::c_uint,
                                                        b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                            .as_ptr() as *const ::core::ffi::c_char,
                                                    );
                                                }
                                            };
                                            if (*mpstack).size == (*mpstack).capacity {
                                                (*mpstack).capacity = if (*mpstack).capacity
                                                    << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    (*mpstack).capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                                (*mpstack).items = (if (*mpstack).capacity
                                                    == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        ) {
                                                    if (*mpstack).items
                                                        == &raw mut (*mpstack).init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        (*mpstack).items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut (*mpstack).init_array
                                                                as *mut MPConvStackVal
                                                                as *mut ::core::ffi::c_void,
                                                            (*mpstack).items
                                                                as *mut ::core::ffi::c_void,
                                                            (*mpstack).size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if (*mpstack).items
                                                        == &raw mut (*mpstack).init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        memcpy(
                                                            xmalloc(
                                                                (*mpstack).capacity.wrapping_mul(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ),
                                                            ),
                                                            (*mpstack).items
                                                                as *const ::core::ffi::c_void,
                                                            (*mpstack).size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            (*mpstack).items
                                                                as *mut ::core::ffi::c_void,
                                                            (*mpstack).capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut MPConvStackVal;
                                            } else {
                                            };
                                            let c2rust_fresh5 = (*mpstack).size;
                                            (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                            *(*mpstack).items.offset(c2rust_fresh5 as isize) =
                                                MPConvStackVal {
                                                    type_0: kMPConvList,
                                                    tv: tv,
                                                    saved_copyID: saved_copyID_0,
                                                    data: C2Rust_Unnamed_18 {
                                                        l: C2Rust_Unnamed_21 {
                                                            list: (*val_di).di_tv.vval.v_list,
                                                            li: tv_list_first(
                                                                (*val_di).di_tv.vval.v_list,
                                                            ),
                                                        },
                                                    },
                                                };
                                            break '_typval_encode_stop_converting_one_item;
                                        }
                                    }
                                    6 => {
                                        if (*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                                        {
                                            let val_list_0: *mut list_T =
                                                (*val_di).di_tv.vval.v_list;
                                            if val_list_0.is_null()
                                                || tv_list_len(val_list_0)
                                                    == 0 as ::core::ffi::c_int
                                            {
                                                '_c2rust_label_2: {
                                                    if (_typval_encode_nothing_nodict_var.ptr()
                                                        as *const _)
                                                        as *mut ::core::ffi::c_void
                                                        != (_typval_encode_nothing_nodict_var.ptr()
                                                            as *const _)
                                                            as *mut ::core::ffi::c_void
                                                    {
                                                    } else {
                                                        __assert_fail(
                                                            b"(void *)&(_typval_encode_nothing_nodict_var) != (void *)&TYPVAL_ENCODE_NODICT_VAR\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            b"src/nvim/eval/typval.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            552 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                                _nothing_conv_empty_dict(
                                                    tv,
                                                    (_typval_encode_nothing_nodict_var.ptr()
                                                        as *const _)
                                                        as *mut *mut dict_T,
                                                );
                                                break '_typval_encode_stop_converting_one_item;
                                            } else {
                                                let l_: *const list_T = val_list_0;
                                                's_689: {
                                                    if !l_.is_null() {
                                                        let mut li: *const listitem_T =
                                                            (*l_).lv_first;
                                                        loop {
                                                            if li.is_null() {
                                                                break 's_689;
                                                            }
                                                            if (*li).li_tv.v_type
                                                                as ::core::ffi::c_uint
                                                                != VAR_LIST as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                                || tv_list_len(
                                                                    (*li).li_tv.vval.v_list,
                                                                ) != 2 as ::core::ffi::c_int
                                                            {
                                                                break 's_771;
                                                            }
                                                            li = (*li).li_next;
                                                        }
                                                    }
                                                }
                                                let saved_copyID_1: ::core::ffi::c_int =
                                                    tv_list_copyid((*val_di).di_tv.vval.v_list);
                                                let te_csr_ret_1: ::core::ffi::c_int =
                                                    _typval_encode_nothing_check_self_reference(
                                                        ignored,
                                                        val_list_0 as *mut ::core::ffi::c_void,
                                                        &raw mut (*val_list_0).lv_copyID,
                                                        mpstack,
                                                        copyID,
                                                        kMPConvPairs,
                                                        objname,
                                                    );
                                                if te_csr_ret_1 != NOTDONE {
                                                    return te_csr_ret_1;
                                                }
                                                '_c2rust_label_3: {
                                                    if saved_copyID_1 != copyID
                                                        && saved_copyID_1
                                                            != copyID - 1 as ::core::ffi::c_int
                                                    {
                                                    } else {
                                                        __assert_fail(
                                                            b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            b"src/nvim/eval/typval.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            566 as ::core::ffi::c_uint,
                                                            b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                                if (*mpstack).size == (*mpstack).capacity {
                                                    (*mpstack).capacity =
                                                        if (*mpstack).capacity
                                                            << 1 as ::core::ffi::c_int
                                                            > ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            )
                                                        {
                                                            (*mpstack).capacity
                                                                << 1 as ::core::ffi::c_int
                                                        } else {
                                                            ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as size_t,
                                                            )
                                                        };
                                                    (*mpstack).items =
                                                        (if (*mpstack).capacity
                                                            == ::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_div(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            ))
                                                            .wrapping_div(
                                                                (::core::mem::size_of::<
                                                                    [MPConvStackVal; 8],
                                                                >(
                                                                )
                                                                .wrapping_rem(
                                                                    ::core::mem::size_of::<
                                                                        MPConvStackVal,
                                                                    >(
                                                                    ),
                                                                ) == 0)
                                                                    as ::core::ffi::c_int
                                                                    as usize,
                                                            )
                                                        {
                                                            if (*mpstack).items
                                                                == &raw mut (*mpstack).init_array
                                                                    as *mut MPConvStackVal
                                                            {
                                                                (*mpstack).items
                                                                    as *mut ::core::ffi::c_void
                                                            } else {
                                                                _memcpy_free(
                                                                    &raw mut (*mpstack).init_array
                                                                        as *mut MPConvStackVal
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack).items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack).size.wrapping_mul(
                                                                        ::core::mem::size_of::<
                                                                            MPConvStackVal,
                                                                        >(
                                                                        ),
                                                                    ),
                                                                )
                                                            }
                                                        } else {
                                                            if (*mpstack).items
                                                                == &raw mut (*mpstack).init_array
                                                                    as *mut MPConvStackVal
                                                            {
                                                                memcpy(
                                                                xmalloc(
                                                                    (*mpstack)
                                                                        .capacity
                                                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                                                ),
                                                                (*mpstack).items as *const ::core::ffi::c_void,
                                                                (*mpstack)
                                                                    .size
                                                                    .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                                            )
                                                            } else {
                                                                xrealloc(
                                                                    (*mpstack).items
                                                                        as *mut ::core::ffi::c_void,
                                                                    (*mpstack)
                                                                        .capacity
                                                                        .wrapping_mul(
                                                                            ::core::mem::size_of::<
                                                                                MPConvStackVal,
                                                                            >(
                                                                            ),
                                                                        ),
                                                                )
                                                            }
                                                        })
                                                            as *mut MPConvStackVal;
                                                } else {
                                                };
                                                let c2rust_fresh6 = (*mpstack).size;
                                                (*mpstack).size = (*mpstack).size.wrapping_add(1);
                                                *(*mpstack).items.offset(c2rust_fresh6 as isize) =
                                                    MPConvStackVal {
                                                        type_0: kMPConvPairs,
                                                        tv: tv,
                                                        saved_copyID: saved_copyID_1,
                                                        data: C2Rust_Unnamed_18 {
                                                            l: C2Rust_Unnamed_21 {
                                                                list: val_list_0,
                                                                li: tv_list_first(val_list_0),
                                                            },
                                                        },
                                                    };
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    7 => {
                                        let mut val_list_1: *const list_T =
                                            ::core::ptr::null::<list_T>();
                                        let mut type_0: varnumber_T = 0;
                                        if !((*val_di).di_tv.v_type as ::core::ffi::c_uint
                                            != VAR_LIST as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || {
                                                val_list_1 = (*val_di).di_tv.vval.v_list;
                                                tv_list_len(val_list_1) != 2 as ::core::ffi::c_int
                                            }
                                            || (*tv_list_first(val_list_1)).li_tv.v_type
                                                as ::core::ffi::c_uint
                                                != VAR_NUMBER as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || {
                                                type_0 = (*tv_list_first(val_list_1))
                                                    .li_tv
                                                    .vval
                                                    .v_number;
                                                type_0 > INT8_MAX as varnumber_T
                                            }
                                            || type_0 < INT8_MIN as varnumber_T
                                            || (*tv_list_last(val_list_1)).li_tv.v_type
                                                as ::core::ffi::c_uint
                                                != VAR_LIST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                        {
                                            let mut len_0: size_t = 0;
                                            let mut buf_0: *mut ::core::ffi::c_char =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            if encode_vim_list_to_buf(
                                                (*tv_list_last(val_list_1)).li_tv.vval.v_list,
                                                &raw mut len_0,
                                                &raw mut buf_0,
                                            ) {
                                                xfree(buf_0 as *mut ::core::ffi::c_void);
                                                break '_typval_encode_stop_converting_one_item;
                                            }
                                        }
                                    }
                                    _ => {
                                        break '_typval_encode_stop_converting_one_item;
                                    }
                                }
                            }
                        }
                    }
                    let saved_copyID_2: ::core::ffi::c_int = (*(*tv).vval.v_dict).dv_copyID;
                    let te_csr_ret_2: ::core::ffi::c_int =
                        _typval_encode_nothing_check_self_reference(
                            ignored,
                            (*tv).vval.v_dict as *mut ::core::ffi::c_void,
                            &raw mut (*(*tv).vval.v_dict).dv_copyID,
                            mpstack,
                            copyID,
                            kMPConvDict,
                            objname,
                        );
                    if te_csr_ret_2 != NOTDONE {
                        return te_csr_ret_2;
                    }
                    '_c2rust_label_4: {
                        if saved_copyID_2 != copyID {
                        } else {
                            __assert_fail(
                                b"saved_copyID != copyID\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/eval/typval.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                614 as ::core::ffi::c_uint,
                                b"int _typval_encode_nothing_convert_one_value(const void *const, MPConvStack *const, MPConvStackVal *const, typval_T *const, const int, const char *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if (*mpstack).size == (*mpstack).capacity {
                        (*mpstack).capacity = if (*mpstack).capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*mpstack).capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*mpstack).items = (if (*mpstack).capacity
                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                        .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                (*mpstack).items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*mpstack).init_array as *mut MPConvStackVal
                                        as *mut ::core::ffi::c_void,
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        } else {
                            if (*mpstack).items
                                == &raw mut (*mpstack).init_array as *mut MPConvStackVal
                            {
                                memcpy(
                                    xmalloc(
                                        (*mpstack)
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                    ),
                                    (*mpstack).items as *const ::core::ffi::c_void,
                                    (*mpstack)
                                        .size
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            } else {
                                xrealloc(
                                    (*mpstack).items as *mut ::core::ffi::c_void,
                                    (*mpstack)
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<MPConvStackVal>()),
                                )
                            }
                        }) as *mut MPConvStackVal;
                    } else {
                    };
                    let c2rust_fresh7 = (*mpstack).size;
                    (*mpstack).size = (*mpstack).size.wrapping_add(1);
                    *(*mpstack).items.offset(c2rust_fresh7 as isize) = MPConvStackVal {
                        type_0: kMPConvDict,
                        tv: tv,
                        saved_copyID: saved_copyID_2,
                        data: C2Rust_Unnamed_18 {
                            d: C2Rust_Unnamed_22 {
                                dict: (*tv).vval.v_dict,
                                dictp: &raw mut (*tv).vval.v_dict,
                                hi: (*(*tv).vval.v_dict).dv_hashtab.ht_array,
                                todo: (*(*tv).vval.v_dict).dv_hashtab.ht_used,
                            },
                        },
                    };
                    let _ = _nothing_conv_real_dict_after_start(
                        tv,
                        &raw mut (*tv).vval.v_dict,
                        (_typval_encode_nothing_nodict_var.ptr() as *const _)
                            as *mut ::core::ffi::c_void,
                        (*mpstack).items.offset(
                            (*mpstack)
                                .size
                                .wrapping_sub(0 as size_t)
                                .wrapping_sub(1 as size_t) as isize,
                        ),
                    ) != NOTDONE;
                }
            }
            0 => {
                internal_error(b"_typval_encode_nothing_convert_one_value()\0".as_ptr()
                    as *const ::core::ffi::c_char);
                return FAIL;
            }
            _ => {}
        }
    }
    return OK;
}
unsafe extern "C" fn encode_vim_to_nothing(
    ignored: *const ::core::ffi::c_void,
    top_tv: *mut typval_T,
    objname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let copyID: ::core::ffi::c_int = get_copyID();
    let mut mpstack: MPConvStack = MPConvStack {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<MPConvStackVal>(),
        init_array: [MPConvStackVal {
            type_0: kMPConvDict,
            tv: ::core::ptr::null_mut::<typval_T>(),
            saved_copyID: 0,
            data: C2Rust_Unnamed_18 {
                d: C2Rust_Unnamed_22 {
                    dict: ::core::ptr::null_mut::<dict_T>(),
                    dictp: ::core::ptr::null_mut::<*mut dict_T>(),
                    hi: ::core::ptr::null_mut::<hashitem_T>(),
                    todo: 0,
                },
            },
        }; 8],
    };
    mpstack.capacity = ::core::mem::size_of::<[MPConvStackVal; 8]>()
        .wrapping_div(::core::mem::size_of::<MPConvStackVal>())
        .wrapping_div(
            (::core::mem::size_of::<[MPConvStackVal; 8]>()
                .wrapping_rem(::core::mem::size_of::<MPConvStackVal>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    mpstack.size = 0 as size_t;
    mpstack.items = &raw mut mpstack.init_array as *mut MPConvStackVal;
    '_encode_vim_to__error_ret: {
        if _typval_encode_nothing_convert_one_value(
            ignored,
            &raw mut mpstack,
            ::core::ptr::null_mut::<MPConvStackVal>(),
            top_tv,
            copyID,
            objname,
        ) != FAIL
        {
            while mpstack.size != 0 {
                let mut cur_mpsv: *mut MPConvStackVal = mpstack.items.offset(
                    mpstack
                        .size
                        .wrapping_sub(0 as size_t)
                        .wrapping_sub(1 as size_t) as isize,
                );
                let mut tv: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
                match (*cur_mpsv).type_0 as ::core::ffi::c_uint {
                    0 => {
                        if (*cur_mpsv).data.d.todo == 0 {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            (*(*cur_mpsv).data.d.dict).dv_copyID = (*cur_mpsv).saved_copyID;
                            _nothing_conv_dict_end(
                                (*cur_mpsv).tv,
                                (*cur_mpsv).data.d.dictp,
                                (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                    as *mut ::core::ffi::c_void,
                            );
                            continue;
                        } else {
                            let _ = (*cur_mpsv).data.d.todo
                                != (*(*cur_mpsv).data.d.dict).dv_hashtab.ht_used;
                            while (*(*cur_mpsv).data.d.hi).hi_key.is_null()
                                || (*(*cur_mpsv).data.d.hi).hi_key
                                    == &raw const hash_removed as *mut ::core::ffi::c_char
                            {
                                (*cur_mpsv).data.d.hi = (*cur_mpsv).data.d.hi.offset(1);
                            }
                            let di: *mut dictitem_T = (*(*cur_mpsv).data.d.hi)
                                .hi_key
                                .offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T;
                            (*cur_mpsv).data.d.todo = (*cur_mpsv).data.d.todo.wrapping_sub(1);
                            (*cur_mpsv).data.d.hi = (*cur_mpsv).data.d.hi.offset(1);
                            tv = &raw mut (*di).di_tv;
                        }
                    }
                    1 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            _nothing_conv_list_end((*cur_mpsv).tv);
                            continue;
                        } else {
                            let _ = (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list);
                            tv = &raw mut (*(*cur_mpsv).data.l.li).li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    2 => {
                        if (*cur_mpsv).data.l.li.is_null() {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            tv_list_set_copyid((*cur_mpsv).data.l.list, (*cur_mpsv).saved_copyID);
                            _nothing_conv_dict_end(
                                (*cur_mpsv).tv,
                                (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                    as *mut *mut dict_T,
                                (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                    as *mut ::core::ffi::c_void,
                            );
                            continue;
                        } else {
                            let _ = (*cur_mpsv).data.l.li != tv_list_first((*cur_mpsv).data.l.list);
                            let kv_pair: *const list_T = (*(*cur_mpsv).data.l.li).li_tv.vval.v_list;
                            if _typval_encode_nothing_convert_one_value(
                                ignored,
                                &raw mut mpstack,
                                cur_mpsv,
                                &raw mut (*(tv_list_first
                                    as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                    kv_pair,
                                ))
                                .li_tv,
                                copyID,
                                objname,
                            ) == FAIL
                            {
                                break '_encode_vim_to__error_ret;
                            }
                            tv = &raw mut (*(tv_list_last
                                as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                                kv_pair,
                            ))
                            .li_tv;
                            (*cur_mpsv).data.l.li = (*(*cur_mpsv).data.l.li).li_next;
                        }
                    }
                    3 => {
                        let pt: *mut partial_T = (*cur_mpsv).data.p.pt;
                        tv = (*cur_mpsv).tv;
                        match (*cur_mpsv).data.p.stage as ::core::ffi::c_uint {
                            0 => {
                                (*cur_mpsv).data.p.stage = kMPConvPartialSelf;
                                if !pt.is_null() && (*pt).pt_argc > 0 as ::core::ffi::c_int {
                                    if mpstack.size == mpstack.capacity {
                                        mpstack.capacity = if mpstack.capacity
                                            << 1 as ::core::ffi::c_int
                                            > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            mpstack.capacity << 1 as ::core::ffi::c_int
                                        } else {
                                            ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as size_t,
                                                )
                                        };
                                        mpstack.items = (if mpstack.capacity
                                            == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                .wrapping_div(
                                                    ::core::mem::size_of::<MPConvStackVal>(),
                                                )
                                                .wrapping_div(
                                                    (::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_rem(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        == 0)
                                                        as ::core::ffi::c_int
                                                        as usize,
                                                ) {
                                            if mpstack.items
                                                == &raw mut mpstack.init_array
                                                    as *mut MPConvStackVal
                                            {
                                                mpstack.items as *mut ::core::ffi::c_void
                                            } else {
                                                _memcpy_free(
                                                    &raw mut mpstack.init_array
                                                        as *mut MPConvStackVal
                                                        as *mut ::core::ffi::c_void,
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            }
                                        } else {
                                            if mpstack.items
                                                == &raw mut mpstack.init_array
                                                    as *mut MPConvStackVal
                                            {
                                                memcpy(
                                                    xmalloc(mpstack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    )),
                                                    mpstack.items as *const ::core::ffi::c_void,
                                                    mpstack.size.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            } else {
                                                xrealloc(
                                                    mpstack.items as *mut ::core::ffi::c_void,
                                                    mpstack.capacity.wrapping_mul(
                                                        ::core::mem::size_of::<MPConvStackVal>(),
                                                    ),
                                                )
                                            }
                                        })
                                            as *mut MPConvStackVal;
                                    } else {
                                    };
                                    let c2rust_fresh0 = mpstack.size;
                                    mpstack.size = mpstack.size.wrapping_add(1);
                                    *mpstack.items.offset(c2rust_fresh0 as isize) =
                                        MPConvStackVal {
                                            type_0: kMPConvPartialList,
                                            tv: ::core::ptr::null_mut::<typval_T>(),
                                            saved_copyID: copyID - 1 as ::core::ffi::c_int,
                                            data: C2Rust_Unnamed_18 {
                                                a: C2Rust_Unnamed_19 {
                                                    arg: (*pt).pt_argv,
                                                    argv: (*pt).pt_argv,
                                                    todo: (*pt).pt_argc as size_t,
                                                },
                                            },
                                        };
                                }
                                continue;
                            }
                            1 => {
                                (*cur_mpsv).data.p.stage = kMPConvPartialEnd;
                                let dict: *mut dict_T = if pt.is_null() {
                                    ::core::ptr::null_mut::<dict_T>()
                                } else {
                                    (*pt).pt_dict
                                };
                                if dict.is_null() {
                                    continue;
                                }
                                if (*dict).dv_hashtab.ht_used == 0 as size_t {
                                    '_c2rust_label: {
                                        if &raw mut (*pt).pt_dict as *mut ::core::ffi::c_void
                                            != (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                                as *mut ::core::ffi::c_void
                                        {
                                        } else {
                                            __assert_fail(
                                                b"(void *)&(pt->pt_dict) != (void *)&TYPVAL_ENCODE_NODICT_VAR\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/eval/typval.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                772 as ::core::ffi::c_uint,
                                                b"int encode_vim_to_nothing(const void *const, typval_T *const, const char *const)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    _nothing_conv_empty_dict(
                                        ::core::ptr::null_mut::<typval_T>(),
                                        &raw mut (*pt).pt_dict,
                                    );
                                    continue;
                                } else {
                                    let saved_copyID: ::core::ffi::c_int = (*dict).dv_copyID;
                                    let te_csr_ret: ::core::ffi::c_int =
                                        _typval_encode_nothing_check_self_reference(
                                            ignored,
                                            dict as *mut ::core::ffi::c_void,
                                            &raw mut (*dict).dv_copyID,
                                            &raw mut mpstack,
                                            copyID,
                                            kMPConvDict,
                                            objname,
                                        );
                                    if te_csr_ret != NOTDONE {
                                        if te_csr_ret == FAIL {
                                            break '_encode_vim_to__error_ret;
                                        } else {
                                            continue;
                                        }
                                    } else {
                                        '_c2rust_label_0: {
                                            if saved_copyID != copyID
                                                && saved_copyID != copyID - 1 as ::core::ffi::c_int
                                            {
                                            } else {
                                                __assert_fail(
                                                    b"saved_copyID != copyID && saved_copyID != copyID - 1\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    b"src/nvim/eval/typval.rs\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                    789 as ::core::ffi::c_uint,
                                                    b"int encode_vim_to_nothing(const void *const, typval_T *const, const char *const)\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                );
                                            }
                                        };
                                        if mpstack.size == mpstack.capacity {
                                            mpstack.capacity =
                                                if mpstack.capacity << 1 as ::core::ffi::c_int
                                                    > ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                {
                                                    mpstack.capacity << 1 as ::core::ffi::c_int
                                                } else {
                                                    ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as size_t,
                                                        )
                                                };
                                            mpstack.items =
                                                (if mpstack.capacity
                                                    == ::core::mem::size_of::<[MPConvStackVal; 8]>()
                                                        .wrapping_div(::core::mem::size_of::<
                                                            MPConvStackVal,
                                                        >(
                                                        ))
                                                        .wrapping_div(
                                                            (::core::mem::size_of::<
                                                                [MPConvStackVal; 8],
                                                            >(
                                                            )
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                MPConvStackVal,
                                                            >(
                                                            )) == 0)
                                                                as ::core::ffi::c_int
                                                                as usize,
                                                        )
                                                {
                                                    if mpstack.items
                                                        == &raw mut mpstack.init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        mpstack.items as *mut ::core::ffi::c_void
                                                    } else {
                                                        _memcpy_free(
                                                            &raw mut mpstack.init_array
                                                                as *mut MPConvStackVal
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.items
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                } else {
                                                    if mpstack.items
                                                        == &raw mut mpstack.init_array
                                                            as *mut MPConvStackVal
                                                    {
                                                        memcpy(
                                                            xmalloc(mpstack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            )),
                                                            mpstack.items
                                                                as *const ::core::ffi::c_void,
                                                            mpstack.size.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    } else {
                                                        xrealloc(
                                                            mpstack.items
                                                                as *mut ::core::ffi::c_void,
                                                            mpstack.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<
                                                                    MPConvStackVal,
                                                                >(
                                                                ),
                                                            ),
                                                        )
                                                    }
                                                })
                                                    as *mut MPConvStackVal;
                                        } else {
                                        };
                                        let c2rust_fresh1 = mpstack.size;
                                        mpstack.size = mpstack.size.wrapping_add(1);
                                        *mpstack.items.offset(c2rust_fresh1 as isize) =
                                            MPConvStackVal {
                                                type_0: kMPConvDict,
                                                tv: ::core::ptr::null_mut::<typval_T>(),
                                                saved_copyID: saved_copyID,
                                                data: C2Rust_Unnamed_18 {
                                                    d: C2Rust_Unnamed_22 {
                                                        dict: dict,
                                                        dictp: &raw mut (*pt).pt_dict,
                                                        hi: (*dict).dv_hashtab.ht_array,
                                                        todo: (*dict).dv_hashtab.ht_used,
                                                    },
                                                },
                                            };
                                        if _nothing_conv_real_dict_after_start(
                                            ::core::ptr::null_mut::<typval_T>(),
                                            &raw mut (*pt).pt_dict,
                                            (_typval_encode_nothing_nodict_var.ptr() as *const _)
                                                as *mut ::core::ffi::c_void,
                                            mpstack.items.offset(
                                                mpstack
                                                    .size
                                                    .wrapping_sub(0 as size_t)
                                                    .wrapping_sub(1 as size_t)
                                                    as isize,
                                            ),
                                        ) != NOTDONE
                                        {
                                            continue;
                                        } else {
                                            continue;
                                        }
                                    }
                                }
                            }
                            2 => {
                                _nothing_conv_func_end(tv, copyID);
                                mpstack.size = mpstack.size.wrapping_sub(1);
                                continue;
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                    4 => {
                        if (*cur_mpsv).data.a.todo == 0 {
                            mpstack.size = mpstack.size.wrapping_sub(1);
                            _nothing_conv_list_end(::core::ptr::null_mut::<typval_T>());
                            continue;
                        } else {
                            let _ = (*cur_mpsv).data.a.argv != (*cur_mpsv).data.a.arg;
                            let c2rust_fresh2 = (*cur_mpsv).data.a.arg;
                            (*cur_mpsv).data.a.arg = (*cur_mpsv).data.a.arg.offset(1);
                            tv = c2rust_fresh2;
                            (*cur_mpsv).data.a.todo = (*cur_mpsv).data.a.todo.wrapping_sub(1);
                        }
                    }
                    _ => {}
                }
                '_c2rust_label_1: {
                    if !tv.is_null() {
                    } else {
                        __assert_fail(
                            b"tv != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/typval.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            829 as ::core::ffi::c_uint,
                            b"int encode_vim_to_nothing(const void *const, typval_T *const, const char *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if _typval_encode_nothing_convert_one_value(
                    ignored,
                    &raw mut mpstack,
                    cur_mpsv,
                    tv,
                    copyID,
                    objname,
                ) == FAIL
                {
                    break '_encode_vim_to__error_ret;
                }
            }
            if mpstack.items != &raw mut mpstack.init_array as *mut MPConvStackVal {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut mpstack.items as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                let _ = *ptr_;
            }
            return OK;
        }
    }
    if mpstack.items != &raw mut mpstack.init_array as *mut MPConvStackVal {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut mpstack.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
    }
    return FAIL;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
