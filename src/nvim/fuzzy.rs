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
    fn ceil(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    fn floor(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut p_ws: ::core::ffi::c_int;
    fn callback_call(
        callback: *mut Callback,
        argcount_in: ::core::ffi::c_int,
        argvars_in: *mut typval_T,
        rettv: *mut typval_T,
    ) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_tv(l: *mut list_T, tv: *mut typval_T);
    fn tv_list_append_list(l: *mut list_T, itemlist: *mut list_T);
    fn tv_list_append_number(l: *mut list_T, n: varnumber_T);
    fn tv_list_find(l: *mut list_T, n: ::core::ffi::c_int) -> *mut listitem_T;
    fn callback_free(callback: *mut Callback);
    fn tv_dict_unref(d: *mut dict_T);
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_has_key(d: *const dict_T, key: *const ::core::ffi::c_char) -> bool;
    fn tv_dict_get_string(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        save: bool,
    ) -> *mut ::core::ffi::c_char;
    fn tv_dict_get_callback(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: ptrdiff_t,
        result: *mut Callback,
    ) -> bool;
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_clear(tv: *mut typval_T);
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_nonnull_dict_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    static mut curbuf: *mut buf_T;
    fn ctrl_x_mode_whole_line() -> bool;
    fn find_word_start(ptr: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn find_word_end(ptr: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn find_line_end(ptr: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_toupper(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn mb_islower(a: ::core::ffi::c_int) -> bool;
    fn mb_tolower(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn mb_isupper(a: ::core::ffi::c_int) -> bool;
    fn mb_charlen(str: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_iswordc(c: ::core::ffi::c_int) -> bool;
    fn vim_iswordp(p: *const ::core::ffi::c_char) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invargval: [::core::ffi::c_char; 0];
    static e_invargNval: [::core::ffi::c_char; 0];
    static e_listarg: [::core::ffi::c_char; 0];
}
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
pub type time_t = __time_t;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub type ptrdiff_t = isize;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub type disptick_T = uint64_t;
pub type linenr_T = int32_t;
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
pub type colnr_T = ::core::ffi::c_int;
pub type Timestamp = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub coladd: colnr_T,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lpos_T {
    pub lnum: linenr_T,
    pub col: colnr_T,
}
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
pub struct alist_T {
    pub al_ga: garray_T,
    pub al_refcount: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
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
pub type ListLenSpecials = ::core::ffi::c_int;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictitem_T {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 0],
}
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const BACKWARD_FILE: C2Rust_Unnamed_13 = -3;
pub const FORWARD_FILE: C2Rust_Unnamed_13 = 3;
pub const BACKWARD: C2Rust_Unnamed_13 = -1;
pub const FORWARD: C2Rust_Unnamed_13 = 1;
pub const kDirectionNotSet: C2Rust_Unnamed_13 = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const FUZZY_MATCH_MAX_LEN: C2Rust_Unnamed_14 = 1024;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_int;
pub const FUZZY_SCORE_NONE: C2Rust_Unnamed_15 = -2147483648;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fuzmatch_str_T {
    pub idx: ::core::ffi::c_int,
    pub str: *mut ::core::ffi::c_char,
    pub score: ::core::ffi::c_int,
}
pub type score_t = ::core::ffi::c_double;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct match_struct {
    pub needle_len: ::core::ffi::c_int,
    pub haystack_len: ::core::ffi::c_int,
    pub lower_needle: [::core::ffi::c_int; 1024],
    pub lower_haystack: [::core::ffi::c_int; 1024],
    pub match_bonus: [score_t; 1024],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct fuzzyItem_T {
    pub idx: ::core::ffi::c_int,
    pub item: *mut listitem_T,
    pub score: ::core::ffi::c_int,
    pub lmatchpos: *mut list_T,
    pub pat: *mut ::core::ffi::c_char,
    pub itemstr: *mut ::core::ffi::c_char,
    pub itemstr_allocated: bool,
    pub startpos: ::core::ffi::c_int,
}
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
pub const SCORE_SCALE: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn fuzzy_match(
    str: *mut ::core::ffi::c_char,
    pat_arg: *const ::core::ffi::c_char,
    matchseq: bool,
    outScore: *mut ::core::ffi::c_int,
    matches: *mut uint32_t,
    maxMatches: ::core::ffi::c_int,
) -> bool {
    let mut complete: bool = false_0 != 0;
    let mut numMatches: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    *outScore = 0 as ::core::ffi::c_int;
    let save_pat: *mut ::core::ffi::c_char = xstrdup(pat_arg);
    let mut pat: *mut ::core::ffi::c_char = save_pat;
    let mut p: *mut ::core::ffi::c_char = pat;
    loop {
        if matchseq {
            complete = true_0 != 0;
        } else {
            p = skipwhite(p);
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            pat = p;
            while *p as ::core::ffi::c_int != NUL && !ascii_iswhite(utf_ptr2char(p)) {
                p = p.offset(utfc_ptr2len(p) as isize);
            }
            if *p as ::core::ffi::c_int == NUL {
                complete = true_0 != 0;
            }
            *p = NUL as ::core::ffi::c_char;
        }
        let mut score: ::core::ffi::c_int = FUZZY_SCORE_NONE as ::core::ffi::c_int;
        if has_match(pat, str) != 0 {
            let mut fzy_score: score_t =
                match_positions(pat, str, matches.offset(numMatches as isize));
            score = if fzy_score == -::core::f32::INFINITY as score_t {
                INT_MIN + 1 as ::core::ffi::c_int
            } else if fzy_score == ::core::f32::INFINITY as score_t {
                INT_MAX
            } else if fzy_score < 0 as ::core::ffi::c_int as score_t {
                ceil(
                    fzy_score as ::core::ffi::c_double * SCORE_SCALE as ::core::ffi::c_double
                        - 0.5f64,
                ) as ::core::ffi::c_int
            } else {
                floor(
                    fzy_score as ::core::ffi::c_double * SCORE_SCALE as ::core::ffi::c_double
                        + 0.5f64,
                ) as ::core::ffi::c_int
            };
        }
        if score == FUZZY_SCORE_NONE as ::core::ffi::c_int {
            numMatches = 0 as ::core::ffi::c_int;
            *outScore = FUZZY_SCORE_NONE as ::core::ffi::c_int;
            break;
        } else {
            if score > 0 as ::core::ffi::c_int && *outScore > INT_MAX - score {
                *outScore = INT_MAX;
            } else if score < 0 as ::core::ffi::c_int
                && *outScore < INT_MIN + 1 as ::core::ffi::c_int - score
            {
                *outScore = INT_MIN + 1 as ::core::ffi::c_int;
            } else {
                *outScore += score;
            }
            numMatches += mb_charlen(pat);
            if complete as ::core::ffi::c_int != 0 || numMatches >= maxMatches {
                break;
            }
            p = p.offset(1);
        }
    }
    xfree(save_pat as *mut ::core::ffi::c_void);
    return numMatches != 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn fuzzy_match_item_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let v1: ::core::ffi::c_int = (*(s1 as *const fuzzyItem_T)).score;
    let v2: ::core::ffi::c_int = (*(s2 as *const fuzzyItem_T)).score;
    if v1 == v2 {
        let pat: *const ::core::ffi::c_char = (*(s1 as *const fuzzyItem_T)).pat;
        let patlen: size_t = strlen(pat);
        let mut startpos: ::core::ffi::c_int = (*(s1 as *const fuzzyItem_T)).startpos;
        let exact_match1: bool = startpos >= 0 as ::core::ffi::c_int
            && strncmp(
                pat,
                (*(s1 as *mut fuzzyItem_T))
                    .itemstr
                    .offset(startpos as isize),
                patlen,
            ) == 0 as ::core::ffi::c_int;
        startpos = (*(s2 as *const fuzzyItem_T)).startpos;
        let exact_match2: bool = startpos >= 0 as ::core::ffi::c_int
            && strncmp(
                pat,
                (*(s2 as *mut fuzzyItem_T))
                    .itemstr
                    .offset(startpos as isize),
                patlen,
            ) == 0 as ::core::ffi::c_int;
        if exact_match1 as ::core::ffi::c_int == exact_match2 as ::core::ffi::c_int {
            let idx1: ::core::ffi::c_int = (*(s1 as *const fuzzyItem_T)).idx;
            let idx2: ::core::ffi::c_int = (*(s2 as *const fuzzyItem_T)).idx;
            return if idx1 == idx2 {
                0 as ::core::ffi::c_int
            } else if idx1 > idx2 {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        } else if exact_match2 {
            return 1 as ::core::ffi::c_int;
        }
        return -1 as ::core::ffi::c_int;
    } else {
        return if v1 > v2 {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    };
}
unsafe extern "C" fn fuzzy_match_in_list(
    l: *mut list_T,
    str: *mut ::core::ffi::c_char,
    matchseq: bool,
    key: *const ::core::ffi::c_char,
    item_cb: *mut Callback,
    retmatchpos: bool,
    fmatchlist: *mut list_T,
    max_matches: ::core::ffi::c_int,
) {
    let mut len: ::core::ffi::c_int = tv_list_len(l);
    if len == 0 as ::core::ffi::c_int {
        return;
    }
    if max_matches > 0 as ::core::ffi::c_int && len > max_matches {
        len = max_matches;
    }
    let items: *mut fuzzyItem_T =
        xcalloc(len as size_t, ::core::mem::size_of::<fuzzyItem_T>()) as *mut fuzzyItem_T;
    let mut match_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut matches: [uint32_t; 1024] = [0; 1024];
    let l_: *mut list_T = l;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if max_matches > 0 as ::core::ffi::c_int && match_count >= max_matches {
                break;
            }
            let mut itemstr: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut itemstr_allocate: bool = false;
            let mut rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            rettv.v_type = VAR_UNKNOWN;
            let tv: *const typval_T = &raw mut (*li).li_tv;
            if (*tv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                itemstr = (*tv).vval.v_string;
            } else if (*tv).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                && (!key.is_null()
                    || (*item_cb).type_0 as ::core::ffi::c_uint
                        != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                if !key.is_null() {
                    itemstr = tv_dict_get_string((*tv).vval.v_dict, key, false);
                } else {
                    let mut argv: [typval_T; 2] = [typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    }; 2];
                    (*(*tv).vval.v_dict).dv_refcount += 1;
                    argv[0 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
                    argv[0 as ::core::ffi::c_int as usize].vval.v_dict = (*tv).vval.v_dict;
                    argv[1 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
                    if callback_call(
                        item_cb,
                        1 as ::core::ffi::c_int,
                        &raw mut argv as *mut typval_T,
                        &raw mut rettv,
                    ) {
                        if rettv.v_type as ::core::ffi::c_uint
                            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            itemstr = rettv.vval.v_string;
                            itemstr_allocate = true;
                        }
                    }
                    tv_dict_unref((*tv).vval.v_dict);
                }
            }
            let mut score: ::core::ffi::c_int = 0;
            if !itemstr.is_null()
                && fuzzy_match(
                    itemstr,
                    str,
                    matchseq,
                    &raw mut score,
                    &raw mut matches as *mut uint32_t,
                    FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0
            {
                let mut itemstr_copy: *mut ::core::ffi::c_char =
                    if itemstr_allocate as ::core::ffi::c_int != 0 {
                        xstrdup(itemstr)
                    } else {
                        itemstr
                    };
                let mut match_positions_0: *mut list_T = ::core::ptr::null_mut::<list_T>();
                if retmatchpos {
                    match_positions_0 =
                        tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
                    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut p: *const ::core::ffi::c_char = str;
                    while *p as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
                        && j < FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int
                    {
                        if !ascii_iswhite(utf_ptr2char(p)) || matchseq as ::core::ffi::c_int != 0 {
                            tv_list_append_number(
                                match_positions_0,
                                matches[j as usize] as varnumber_T,
                            );
                            j += 1;
                        }
                        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
                    }
                }
                (*items.offset(match_count as isize)).idx = match_count;
                (*items.offset(match_count as isize)).item = li;
                (*items.offset(match_count as isize)).score = score;
                (*items.offset(match_count as isize)).pat = str;
                (*items.offset(match_count as isize)).startpos =
                    matches[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int;
                (*items.offset(match_count as isize)).itemstr = itemstr_copy;
                (*items.offset(match_count as isize)).itemstr_allocated = itemstr_allocate;
                (*items.offset(match_count as isize)).lmatchpos = match_positions_0;
                match_count += 1;
            }
            tv_clear(&raw mut rettv);
            li = (*li).li_next;
        }
    }
    if match_count > 0 as ::core::ffi::c_int {
        qsort(
            items as *mut ::core::ffi::c_void,
            match_count as size_t,
            ::core::mem::size_of::<fuzzyItem_T>(),
            Some(
                fuzzy_match_item_compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        let mut retlist: *mut list_T = ::core::ptr::null_mut::<list_T>();
        if retmatchpos {
            let li_0: *const listitem_T = tv_list_find(fmatchlist, 0 as ::core::ffi::c_int);
            '_c2rust_label: {
                if !li_0.is_null() && !(*li_0).li_tv.vval.v_list.is_null() {
                } else {
                    __assert_fail(
                        b"li != NULL && TV_LIST_ITEM_TV(li)->vval.v_list != NULL\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/fuzzy.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        293 as ::core::ffi::c_uint,
                        b"void fuzzy_match_in_list(list_T *const, char *const, const _Bool, const char *const, Callback *const, const _Bool, list_T *const, const int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            retlist = (*li_0).li_tv.vval.v_list;
        } else {
            retlist = fmatchlist;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < match_count {
            tv_list_append_tv(retlist, &raw mut (*(*items.offset(i as isize)).item).li_tv);
            i += 1;
        }
        if retmatchpos {
            let mut li_1: *const listitem_T = tv_list_find(fmatchlist, -2 as ::core::ffi::c_int);
            '_c2rust_label_0: {
                if !li_1.is_null() && !(*li_1).li_tv.vval.v_list.is_null() {
                } else {
                    __assert_fail(
                        b"li != NULL && TV_LIST_ITEM_TV(li)->vval.v_list != NULL\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/fuzzy.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        307 as ::core::ffi::c_uint,
                        b"void fuzzy_match_in_list(list_T *const, char *const, const _Bool, const char *const, Callback *const, const _Bool, list_T *const, const int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            retlist = (*li_1).li_tv.vval.v_list;
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < match_count {
                '_c2rust_label_1: {
                    if !(*items.offset(i_0 as isize)).lmatchpos.is_null() {
                    } else {
                        __assert_fail(
                            b"items[i].lmatchpos != NULL\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/fuzzy.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            311 as ::core::ffi::c_uint,
                            b"void fuzzy_match_in_list(list_T *const, char *const, const _Bool, const char *const, Callback *const, const _Bool, list_T *const, const int)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                tv_list_append_list(retlist, (*items.offset(i_0 as isize)).lmatchpos);
                (*items.offset(i_0 as isize)).lmatchpos = ::core::ptr::null_mut::<list_T>();
                i_0 += 1;
            }
            li_1 = tv_list_find(fmatchlist, -1 as ::core::ffi::c_int);
            '_c2rust_label_2: {
                if !li_1.is_null() && !(*li_1).li_tv.vval.v_list.is_null() {
                } else {
                    __assert_fail(
                        b"li != NULL && TV_LIST_ITEM_TV(li)->vval.v_list != NULL\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/fuzzy.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        318 as ::core::ffi::c_uint,
                        b"void fuzzy_match_in_list(list_T *const, char *const, const _Bool, const char *const, Callback *const, const _Bool, list_T *const, const int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            retlist = (*li_1).li_tv.vval.v_list;
            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_1 < match_count {
                tv_list_append_number(retlist, (*items.offset(i_1 as isize)).score as varnumber_T);
                i_1 += 1;
            }
        }
    }
    let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_2 < match_count {
        if (*items.offset(i_2 as isize)).itemstr_allocated {
            xfree((*items.offset(i_2 as isize)).itemstr as *mut ::core::ffi::c_void);
        }
        '_c2rust_label_3: {
            if (*items.offset(i_2 as isize)).lmatchpos.is_null() {
            } else {
                __assert_fail(
                    b"items[i].lmatchpos == NULL\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/fuzzy.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    330 as ::core::ffi::c_uint,
                    b"void fuzzy_match_in_list(list_T *const, char *const, const _Bool, const char *const, Callback *const, const _Bool, list_T *const, const int)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        i_2 += 1;
    }
    xfree(items as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn do_fuzzymatch(
    argvars: *const typval_T,
    rettv: *mut typval_T,
    retmatchpos: bool,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list
            .is_null()
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            if retmatchpos as ::core::ffi::c_int != 0 {
                b"matchfuzzypos()\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"matchfuzzy()\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_string
            .is_null()
    {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
        );
        return;
    }
    let mut cb: Callback = Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut matchseq: bool = false_0 != 0;
    let mut max_matches: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_check_for_nonnull_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let d: *mut dict_T = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        let mut di: *const dictitem_T = ::core::ptr::null::<dictitem_T>();
        di = tv_dict_find(
            d,
            b"key\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            if (*di).di_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*di).di_tv.vval.v_string.is_null()
                || *(*di).di_tv.vval.v_string as ::core::ffi::c_int == NUL
            {
                semsg(
                    gettext(&raw const e_invargNval as *const ::core::ffi::c_char),
                    b"key\0".as_ptr() as *const ::core::ffi::c_char,
                    tv_get_string(&raw const (*di).di_tv),
                );
                return;
            }
            key = tv_get_string(&raw const (*di).di_tv);
        } else if !tv_dict_get_callback(
            d,
            b"text_cb\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
            &raw mut cb,
        ) {
            semsg(
                gettext(&raw const e_invargval as *const ::core::ffi::c_char),
                b"text_cb\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        di = tv_dict_find(
            d,
            b"limit\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            if (*di).di_tv.v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                semsg(
                    gettext(&raw const e_invargval as *const ::core::ffi::c_char),
                    b"limit\0".as_ptr() as *const ::core::ffi::c_char,
                );
                return;
            }
            max_matches = tv_get_number_chk(&raw const (*di).di_tv, ::core::ptr::null_mut::<bool>())
                as ::core::ffi::c_int;
        }
        if tv_dict_has_key(d, b"matchseq\0".as_ptr() as *const ::core::ffi::c_char) {
            matchseq = true_0 != 0;
        }
    }
    tv_list_alloc_ret(
        rettv,
        (if retmatchpos as ::core::ffi::c_int != 0 {
            3 as ::core::ffi::c_int
        } else {
            kListLenUnknown as ::core::ffi::c_int
        }) as ptrdiff_t,
    );
    if retmatchpos {
        tv_list_append_list(
            (*rettv).vval.v_list,
            tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t),
        );
        tv_list_append_list(
            (*rettv).vval.v_list,
            tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t),
        );
        tv_list_append_list(
            (*rettv).vval.v_list,
            tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t),
        );
    }
    fuzzy_match_in_list(
        (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list,
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char,
        matchseq,
        key,
        &raw mut cb,
        retmatchpos,
        (*rettv).vval.v_list,
        max_matches,
    );
    callback_free(&raw mut cb);
}
#[no_mangle]
pub unsafe extern "C" fn f_matchfuzzy(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    do_fuzzymatch(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_matchfuzzypos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    do_fuzzymatch(argvars, rettv, true_0 != 0);
}
unsafe extern "C" fn fuzzy_match_str_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let v1: ::core::ffi::c_int = (*(s1 as *mut fuzmatch_str_T)).score;
    let v2: ::core::ffi::c_int = (*(s2 as *mut fuzmatch_str_T)).score;
    let idx1: ::core::ffi::c_int = (*(s1 as *mut fuzmatch_str_T)).idx;
    let idx2: ::core::ffi::c_int = (*(s2 as *mut fuzmatch_str_T)).idx;
    if v1 == v2 {
        return if idx1 == idx2 {
            0 as ::core::ffi::c_int
        } else if idx1 > idx2 {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    } else {
        return if v1 > v2 {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    };
}
unsafe extern "C" fn fuzzy_match_str_sort(fm: *mut fuzmatch_str_T, sz: ::core::ffi::c_int) {
    qsort(
        fm as *mut ::core::ffi::c_void,
        sz as size_t,
        ::core::mem::size_of::<fuzmatch_str_T>(),
        Some(
            fuzzy_match_str_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
}
unsafe extern "C" fn fuzzy_match_func_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let v1: ::core::ffi::c_int = (*(s1 as *mut fuzmatch_str_T)).score;
    let v2: ::core::ffi::c_int = (*(s2 as *mut fuzmatch_str_T)).score;
    let idx1: ::core::ffi::c_int = (*(s1 as *mut fuzmatch_str_T)).idx;
    let idx2: ::core::ffi::c_int = (*(s2 as *mut fuzmatch_str_T)).idx;
    let str1: *const ::core::ffi::c_char = (*(s1 as *mut fuzmatch_str_T)).str;
    let str2: *const ::core::ffi::c_char = (*(s2 as *mut fuzmatch_str_T)).str;
    if *str1 as ::core::ffi::c_int != '<' as ::core::ffi::c_int
        && *str2 as ::core::ffi::c_int == '<' as ::core::ffi::c_int
    {
        return -1 as ::core::ffi::c_int;
    }
    if *str1 as ::core::ffi::c_int == '<' as ::core::ffi::c_int
        && *str2 as ::core::ffi::c_int != '<' as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    if v1 == v2 {
        return if idx1 == idx2 {
            0 as ::core::ffi::c_int
        } else if idx1 > idx2 {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
    }
    return if v1 > v2 {
        -1 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    };
}
unsafe extern "C" fn fuzzy_match_func_sort(fm: *mut fuzmatch_str_T, sz: ::core::ffi::c_int) {
    qsort(
        fm as *mut ::core::ffi::c_void,
        sz as size_t,
        ::core::mem::size_of::<fuzmatch_str_T>(),
        Some(
            fuzzy_match_func_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn fuzzy_match_str(
    str: *mut ::core::ffi::c_char,
    pat: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if str.is_null() || pat.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut score: ::core::ffi::c_int = FUZZY_SCORE_NONE as ::core::ffi::c_int;
    let mut matchpos: [uint32_t; 1024] = [0; 1024];
    fuzzy_match(
        str,
        pat,
        true_0 != 0,
        &raw mut score,
        &raw mut matchpos as *mut uint32_t,
        ::core::mem::size_of::<[uint32_t; 1024]>()
            .wrapping_div(::core::mem::size_of::<uint32_t>())
            .wrapping_div(
                (::core::mem::size_of::<[uint32_t; 1024]>()
                    .wrapping_rem(::core::mem::size_of::<uint32_t>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int,
    );
    return score;
}
#[no_mangle]
pub unsafe extern "C" fn fuzzy_match_str_with_pos(
    str: *mut ::core::ffi::c_char,
    pat: *const ::core::ffi::c_char,
) -> *mut garray_T {
    if str.is_null() || pat.is_null() {
        return ::core::ptr::null_mut::<garray_T>();
    }
    let mut match_positions_0: *mut garray_T =
        xmalloc(::core::mem::size_of::<garray_T>()) as *mut garray_T;
    ga_init(
        match_positions_0,
        ::core::mem::size_of::<uint32_t>() as ::core::ffi::c_int,
        10 as ::core::ffi::c_int,
    );
    let mut score: ::core::ffi::c_int = FUZZY_SCORE_NONE as ::core::ffi::c_int;
    let mut matches: [uint32_t; 1024] = [0; 1024];
    if !fuzzy_match(
        str,
        pat,
        false_0 != 0,
        &raw mut score,
        &raw mut matches as *mut uint32_t,
        FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int,
    ) || score == FUZZY_SCORE_NONE as ::core::ffi::c_int
    {
        ga_clear(match_positions_0);
        xfree(match_positions_0 as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<garray_T>();
    }
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *const ::core::ffi::c_char = pat;
    while *p as ::core::ffi::c_int != NUL {
        if !ascii_iswhite(utf_ptr2char(p)) {
            ga_grow(match_positions_0, 1 as ::core::ffi::c_int);
            *((*match_positions_0).ga_data as *mut uint32_t)
                .offset((*match_positions_0).ga_len as isize) = matches[j as usize];
            (*match_positions_0).ga_len += 1;
            j += 1;
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    return match_positions_0;
}
#[no_mangle]
pub unsafe extern "C" fn fuzzy_match_str_in_line(
    mut ptr: *mut *mut ::core::ffi::c_char,
    mut pat: *mut ::core::ffi::c_char,
    mut len: *mut ::core::ffi::c_int,
    mut current_pos: *mut pos_T,
    mut score: *mut ::core::ffi::c_int,
) -> bool {
    let mut str: *mut ::core::ffi::c_char = *ptr;
    let mut strBegin: *mut ::core::ffi::c_char = str;
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut found: bool = false_0 != 0;
    if str.is_null() || pat.is_null() {
        return found;
    }
    let mut line_end: *mut ::core::ffi::c_char = find_line_end(str);
    while str < line_end {
        start = find_word_start(str);
        if *start as ::core::ffi::c_int == NUL {
            break;
        }
        end = find_word_end(start);
        let mut save_end: ::core::ffi::c_char = *end;
        *end = NUL as ::core::ffi::c_char;
        *score = fuzzy_match_str(start, pat);
        *end = save_end;
        if *score != FUZZY_SCORE_NONE as ::core::ffi::c_int {
            *len = end.offset_from(start) as ::core::ffi::c_int;
            found = true_0 != 0;
            *ptr = start;
            if !current_pos.is_null() {
                (*current_pos).col += end.offset_from(strBegin) as ::core::ffi::c_int;
            }
            break;
        } else {
            str = end;
            while *str as ::core::ffi::c_int != NUL && !vim_iswordp(str) {
                str = str.offset(utfc_ptr2len(str) as isize);
            }
        }
    }
    if !found {
        *ptr = line_end;
    }
    return found;
}
#[no_mangle]
pub unsafe extern "C" fn search_for_fuzzy_match(
    mut buf: *mut buf_T,
    mut pos: *mut pos_T,
    mut pattern: *mut ::core::ffi::c_char,
    mut dir: ::core::ffi::c_int,
    mut start_pos: *mut pos_T,
    mut len: *mut ::core::ffi::c_int,
    mut ptr: *mut *mut ::core::ffi::c_char,
    mut score: *mut ::core::ffi::c_int,
) -> bool {
    let mut current_pos: pos_T = *pos;
    let mut circly_end: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut found_new_match: bool = false_0 != 0;
    let mut looped_around: bool = false_0 != 0;
    let mut whole_line: bool = ctrl_x_mode_whole_line();
    if buf == curbuf {
        circly_end = *start_pos;
    } else {
        circly_end.lnum = (*buf).b_ml.ml_line_count;
        circly_end.col = 0 as ::core::ffi::c_int as colnr_T;
        circly_end.coladd = 0 as ::core::ffi::c_int as colnr_T;
    }
    if whole_line as ::core::ffi::c_int != 0 && (*start_pos).lnum != (*pos).lnum {
        current_pos.lnum = (current_pos.lnum as ::core::ffi::c_int + dir) as linenr_T;
    }
    while !(looped_around as ::core::ffi::c_int != 0
        && (if whole_line as ::core::ffi::c_int != 0 {
            (current_pos.lnum == circly_end.lnum) as ::core::ffi::c_int
        } else {
            equalpos(current_pos, circly_end) as ::core::ffi::c_int
        }) != 0)
    {
        if current_pos.lnum >= 1 as linenr_T && current_pos.lnum <= (*buf).b_ml.ml_line_count {
            *ptr = ml_get_buf(buf, current_pos.lnum);
            if !whole_line {
                *ptr = (*ptr).offset(current_pos.col as isize);
            }
            if !(*ptr).is_null() && **ptr as ::core::ffi::c_int != NUL {
                if !whole_line {
                    found_new_match =
                        fuzzy_match_str_in_line(ptr, pattern, len, &raw mut current_pos, score);
                    if found_new_match {
                        *pos = current_pos;
                        break;
                    } else if looped_around as ::core::ffi::c_int != 0
                        && current_pos.lnum == circly_end.lnum
                    {
                        break;
                    }
                } else if fuzzy_match_str(*ptr, pattern) != FUZZY_SCORE_NONE as ::core::ffi::c_int {
                    found_new_match = true_0 != 0;
                    *pos = current_pos;
                    *len = ml_get_buf_len(buf, current_pos.lnum) as ::core::ffi::c_int;
                    break;
                }
            }
        }
        if dir == FORWARD as ::core::ffi::c_int {
            current_pos.lnum += 1;
            if current_pos.lnum > (*buf).b_ml.ml_line_count {
                if p_ws == 0 {
                    break;
                }
                current_pos.lnum = 1 as ::core::ffi::c_int as linenr_T;
                looped_around = true_0 != 0;
            }
        } else {
            current_pos.lnum -= 1;
            if current_pos.lnum < 1 as linenr_T {
                if p_ws == 0 {
                    break;
                }
                current_pos.lnum = (*buf).b_ml.ml_line_count;
                looped_around = true_0 != 0;
            }
        }
        current_pos.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    return found_new_match;
}
#[no_mangle]
pub unsafe extern "C" fn fuzmatch_str_free(
    fuzmatch: *mut fuzmatch_str_T,
    mut count: ::core::ffi::c_int,
) {
    if fuzmatch.is_null() {
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count {
        xfree((*fuzmatch.offset(count as isize)).str as *mut ::core::ffi::c_void);
        i += 1;
    }
    xfree(fuzmatch as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn fuzzymatches_to_strmatches(
    fuzmatch: *mut fuzmatch_str_T,
    matches: *mut *mut *mut ::core::ffi::c_char,
    count: ::core::ffi::c_int,
    funcsort: bool,
) {
    if count > 0 as ::core::ffi::c_int {
        *matches = xmalloc(
            (count as size_t).wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
        ) as *mut *mut ::core::ffi::c_char;
        if funcsort {
            fuzzy_match_func_sort(fuzmatch, count);
        } else {
            fuzzy_match_str_sort(fuzmatch, count);
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < count {
            *(*matches).offset(i as isize) = (*fuzmatch.offset(i as isize)).str;
            i += 1;
        }
    }
    xfree(fuzmatch as *mut ::core::ffi::c_void);
}
pub const SCORE_GAP_LEADING: ::core::ffi::c_double = -0.005f64;
pub const SCORE_GAP_TRAILING: ::core::ffi::c_double = -0.005f64;
pub const SCORE_GAP_INNER: ::core::ffi::c_double = -0.01f64;
pub const SCORE_MATCH_CONSECUTIVE: ::core::ffi::c_double = 1.0f64;
pub const SCORE_MATCH_SLASH: ::core::ffi::c_double = 0.9f64;
pub const SCORE_MATCH_WORD: ::core::ffi::c_double = 0.8f64;
pub const SCORE_MATCH_CAPITAL: ::core::ffi::c_double = 0.7f64;
pub const SCORE_MATCH_DOT: ::core::ffi::c_double = 0.6f64;
unsafe extern "C" fn has_match(
    needle: *const ::core::ffi::c_char,
    haystack: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if needle.is_null() || haystack.is_null() || *needle == 0 {
        return FAIL;
    }
    let mut n_ptr: *const ::core::ffi::c_char = needle;
    let mut h_ptr: *const ::core::ffi::c_char = haystack;
    while *n_ptr != 0 {
        let n_char: ::core::ffi::c_int = utf_ptr2char(n_ptr);
        let mut found: bool = false_0 != 0;
        while *h_ptr != 0 {
            let h_char: ::core::ffi::c_int = utf_ptr2char(h_ptr);
            if n_char == h_char || mb_toupper(n_char) == h_char {
                found = true_0 != 0;
                h_ptr = h_ptr.offset(utfc_ptr2len(h_ptr) as isize);
                break;
            } else {
                h_ptr = h_ptr.offset(utfc_ptr2len(h_ptr) as isize);
            }
        }
        if !found {
            return FAIL;
        }
        n_ptr = n_ptr.offset(utfc_ptr2len(n_ptr) as isize);
    }
    return OK;
}
unsafe extern "C" fn compute_bonus_codepoint(
    mut last_c: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> score_t {
    if c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(c) as ::core::ffi::c_int != 0
        || vim_iswordc(c) as ::core::ffi::c_int != 0
    {
        if last_c == '/' as ::core::ffi::c_int {
            return SCORE_MATCH_SLASH;
        }
        if last_c == '-' as ::core::ffi::c_int
            || last_c == '_' as ::core::ffi::c_int
            || last_c == ' ' as ::core::ffi::c_int
        {
            return SCORE_MATCH_WORD;
        }
        if last_c == '.' as ::core::ffi::c_int {
            return SCORE_MATCH_DOT;
        }
        if mb_isupper(c) as ::core::ffi::c_int != 0 && mb_islower(last_c) as ::core::ffi::c_int != 0
        {
            return SCORE_MATCH_CAPITAL;
        }
    }
    return 0 as ::core::ffi::c_int as score_t;
}
unsafe extern "C" fn setup_match_struct(
    match_0: *mut match_struct,
    needle: *const ::core::ffi::c_char,
    haystack: *const ::core::ffi::c_char,
) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *const ::core::ffi::c_char = needle;
    while *p as ::core::ffi::c_int != NUL && i < FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int {
        let c: ::core::ffi::c_int = utf_ptr2char(p);
        let c2rust_fresh1 = i;
        i = i + 1;
        (*match_0).lower_needle[c2rust_fresh1 as usize] = mb_tolower(c);
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    (*match_0).needle_len = i;
    i = 0 as ::core::ffi::c_int;
    p = haystack;
    let mut prev_c: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != NUL && i < FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int {
        let c_0: ::core::ffi::c_int = utf_ptr2char(p);
        (*match_0).lower_haystack[i as usize] = mb_tolower(c_0);
        (*match_0).match_bonus[i as usize] = compute_bonus_codepoint(prev_c, c_0);
        prev_c = c_0;
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        i += 1;
    }
    (*match_0).haystack_len = i;
}
#[inline]
unsafe extern "C" fn match_row(
    mut match_0: *const match_struct,
    mut row: ::core::ffi::c_int,
    mut curr_D: *mut score_t,
    mut curr_M: *mut score_t,
    mut last_D: *const score_t,
    mut last_M: *const score_t,
) {
    let mut n: ::core::ffi::c_int = (*match_0).needle_len;
    let mut m: ::core::ffi::c_int = (*match_0).haystack_len;
    let mut i: ::core::ffi::c_int = row;
    let mut lower_needle: *const ::core::ffi::c_int =
        &raw const (*match_0).lower_needle as *const ::core::ffi::c_int;
    let mut lower_haystack: *const ::core::ffi::c_int =
        &raw const (*match_0).lower_haystack as *const ::core::ffi::c_int;
    let mut match_bonus: *const score_t = &raw const (*match_0).match_bonus as *const score_t;
    let mut prev_score: score_t = -::core::f32::INFINITY as score_t;
    let mut gap_score: score_t = if i == n - 1 as ::core::ffi::c_int {
        SCORE_GAP_TRAILING
    } else {
        SCORE_GAP_INNER
    };
    let mut prev_M: score_t = -::core::f32::INFINITY as score_t;
    let mut prev_D: score_t = -::core::f32::INFINITY as score_t;
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while j < m {
        if *lower_needle.offset(i as isize) == *lower_haystack.offset(j as isize) {
            let mut score: score_t = -::core::f32::INFINITY as score_t;
            if i == 0 {
                score = j as score_t * SCORE_GAP_LEADING + *match_bonus.offset(j as isize);
            } else if j != 0 {
                score = (if prev_M + *match_bonus.offset(j as isize)
                    > prev_D as ::core::ffi::c_double + 1.0f64
                {
                    prev_M as ::core::ffi::c_double
                        + *match_bonus.offset(j as isize) as ::core::ffi::c_double
                } else {
                    prev_D as ::core::ffi::c_double + 1.0f64
                }) as score_t;
            }
            prev_D = *last_D.offset(j as isize);
            prev_M = *last_M.offset(j as isize);
            *curr_D.offset(j as isize) = score;
            prev_score = if score > prev_score + gap_score {
                score
            } else {
                prev_score + gap_score
            };
            *curr_M.offset(j as isize) = prev_score;
        } else {
            prev_D = *last_D.offset(j as isize);
            prev_M = *last_M.offset(j as isize);
            *curr_D.offset(j as isize) = -::core::f32::INFINITY as score_t;
            prev_score = prev_score + gap_score;
            *curr_M.offset(j as isize) = prev_score;
        }
        j += 1;
    }
}
unsafe extern "C" fn match_positions(
    needle: *const ::core::ffi::c_char,
    haystack: *const ::core::ffi::c_char,
    positions: *mut uint32_t,
) -> score_t {
    if needle.is_null() || haystack.is_null() || *needle == 0 {
        return -::core::f32::INFINITY as score_t;
    }
    let mut match_0: match_struct = match_struct {
        needle_len: 0,
        haystack_len: 0,
        lower_needle: [0; 1024],
        lower_haystack: [0; 1024],
        match_bonus: [0.; 1024],
    };
    setup_match_struct(&raw mut match_0, needle, haystack);
    let mut n: ::core::ffi::c_int = match_0.needle_len;
    let mut m: ::core::ffi::c_int = match_0.haystack_len;
    if m > FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int || n > m {
        return -::core::f32::INFINITY as score_t;
    } else if n == m {
        if !positions.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < n {
                *positions.offset(i as isize) = i as uint32_t;
                i += 1;
            }
        }
        return ::core::f32::INFINITY as score_t;
    }
    if n as size_t
        > (SIZE_MAX as usize)
            .wrapping_div(::core::mem::size_of::<score_t>())
            .wrapping_div(FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int as usize)
            .wrapping_div(2 as usize)
    {
        return -::core::f32::INFINITY as score_t;
    }
    let mut block: *mut score_t = xmalloc(
        ::core::mem::size_of::<score_t>()
            .wrapping_mul(FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int as size_t)
            .wrapping_mul(n as size_t)
            .wrapping_mul(2 as size_t),
    ) as *mut score_t;
    let mut D: *mut [score_t; 1024] = block as *mut [score_t; 1024];
    let mut M: *mut [score_t; 1024] = block.offset(
        (FUZZY_MATCH_MAX_LEN as ::core::ffi::c_int as size_t).wrapping_mul(n as size_t) as isize,
    ) as *mut [score_t; 1024];
    match_row(
        &raw mut match_0,
        0 as ::core::ffi::c_int,
        &raw mut *D.offset(0 as ::core::ffi::c_int as isize) as *mut score_t,
        &raw mut *M.offset(0 as ::core::ffi::c_int as isize) as *mut score_t,
        &raw mut *D.offset(0 as ::core::ffi::c_int as isize) as *mut score_t,
        &raw mut *M.offset(0 as ::core::ffi::c_int as isize) as *mut score_t,
    );
    let mut i_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i_0 < n {
        match_row(
            &raw mut match_0,
            i_0,
            &raw mut *D.offset(i_0 as isize) as *mut score_t,
            &raw mut *M.offset(i_0 as isize) as *mut score_t,
            &raw mut *D.offset((i_0 - 1 as ::core::ffi::c_int) as isize) as *mut score_t,
            &raw mut *M.offset((i_0 - 1 as ::core::ffi::c_int) as isize) as *mut score_t,
        );
        i_0 += 1;
    }
    if !positions.is_null() {
        let mut match_required: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i_1: ::core::ffi::c_int = n - 1 as ::core::ffi::c_int;
        let mut j: ::core::ffi::c_int = m - 1 as ::core::ffi::c_int;
        while i_1 >= 0 as ::core::ffi::c_int {
            while j >= 0 as ::core::ffi::c_int {
                if (*D.offset(i_1 as isize))[j as usize] != -::core::f32::INFINITY as score_t
                    && (match_required != 0
                        || (*D.offset(i_1 as isize))[j as usize]
                            == (*M.offset(i_1 as isize))[j as usize])
                {
                    match_required = (i_1 != 0
                        && j != 0
                        && (*M.offset(i_1 as isize))[j as usize]
                            == (*D.offset((i_1 - 1 as ::core::ffi::c_int) as isize))
                                [(j - 1 as ::core::ffi::c_int) as usize]
                                as ::core::ffi::c_double
                                + SCORE_MATCH_CONSECUTIVE)
                        as ::core::ffi::c_int;
                    let c2rust_fresh0 = j;
                    j = j - 1;
                    *positions.offset(i_1 as isize) = c2rust_fresh0 as uint32_t;
                    break;
                } else {
                    j -= 1;
                }
            }
            i_1 -= 1;
        }
    }
    let mut result: score_t =
        (*M.offset((n - 1 as ::core::ffi::c_int) as isize))[(m - 1 as ::core::ffi::c_int) as usize];
    xfree(block as *mut ::core::ffi::c_void);
    return result;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
