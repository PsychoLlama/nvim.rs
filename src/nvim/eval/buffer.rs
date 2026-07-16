extern "C" {
    pub type terminal;
    pub type regprog;
    pub type qf_info_S;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strnequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char, n: size_t) -> bool;
    fn aucmd_prepbuf(aco: *mut aco_save_T, buf: *mut buf_T);
    fn aucmd_restbuf(aco: *mut aco_save_T);
    fn block_autocmds();
    fn unblock_autocmds();
    fn buf_ensure_loaded(buf: *mut buf_T) -> bool;
    fn set_bufref(bufref: *mut bufref_T, buf: *mut buf_T);
    fn bufref_valid(bufref: *mut bufref_T) -> bool;
    fn buflist_new(
        ffname_arg: *mut ::core::ffi::c_char,
        sfname_arg: *mut ::core::ffi::c_char,
        lnum: linenr_T,
        flags: ::core::ffi::c_int,
    ) -> *mut buf_T;
    fn buflist_findname_exp(fname: *mut ::core::ffi::c_char) -> *mut buf_T;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn buflist_findlnum(buf: *mut buf_T) -> linenr_T;
    fn buflist_add(
        fname: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn bt_prompt(buf: *mut buf_T) -> bool;
    fn bt_nofilename(buf: *const buf_T) -> bool;
    fn inserted_bytes(
        lnum: linenr_T,
        start_col: colnr_T,
        old_col: ::core::ffi::c_int,
        new_col: ::core::ffi::c_int,
    );
    fn appended_lines_mark(lnum: linenr_T, count: ::core::ffi::c_int);
    fn deleted_lines_mark(lnum: linenr_T, count: ::core::ffi::c_int);
    fn changed_lines(
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        lnume: linenr_T,
        xtra: linenr_T,
        do_buf_event: bool,
    );
    fn check_cursor_col(win: *mut win_T);
    fn buf_prompt_text(buf: *const buf_T) -> *mut ::core::ffi::c_char;
    fn callback_from_typval(callback: *mut Callback, arg: *const typval_T) -> bool;
    fn typval_tostring(arg: *mut typval_T, quotes: bool) -> *mut ::core::ffi::c_char;
    fn tv_get_buf(tv: *mut typval_T, curtab_only: ::core::ffi::c_int) -> *mut buf_T;
    fn tv_get_buf_from_arg(tv: *mut typval_T) -> *mut buf_T;
    fn get_buf_arg(arg: *mut typval_T) -> *mut buf_T;
    fn tv_list_item_remove(l: *mut list_T, item: *mut listitem_T) -> *mut listitem_T;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_dict(l: *mut list_T, dict: *mut dict_T);
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn tv_list_append_number(l: *mut list_T, n: varnumber_T);
    fn callback_free(callback: *mut Callback);
    fn tv_dict_alloc() -> *mut dict_T;
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_add_list(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        list: *mut list_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_dict(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        dict: *mut dict_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_nr(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        nr: varnumber_T,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_str(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        val: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_clear(tv: *mut typval_T);
    fn tv_check_str_or_nr(tv: *const typval_T) -> bool;
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_get_lnum(tv: *const typval_T) -> linenr_T;
    fn tv_get_lnum_buf(tv: *const typval_T, buf: *const buf_T) -> linenr_T;
    fn tv_get_string_chk(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn win_has_winnr(wp: *mut win_T, tp: *mut tabpage_T) -> bool;
    fn check_secure() -> bool;
    fn extmark_splice_cols(
        buf: *mut buf_T,
        start_row: ::core::ffi::c_int,
        start_col: colnr_T,
        old_col: colnr_T,
        new_col: colnr_T,
        undo: ExtmarkOp,
    );
    static mut emsg_off: ::core::ffi::c_int;
    static mut did_emsg: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut firstbuf: *mut buf_T;
    static mut curbuf: *mut buf_T;
    static mut VIsual_active: bool;
    static mut u_sync_once: ::core::ffi::c_int;
    static mut swap_exists_action: ::core::ffi::c_int;
    static mut cmdwin_buf: *mut buf_T;
    fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn ml_append(
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        len: colnr_T,
        newfile: bool,
    ) -> ::core::ffi::c_int;
    fn ml_replace(lnum: linenr_T, line: *mut ::core::ffi::c_char, copy: bool)
        -> ::core::ffi::c_int;
    fn ml_replace_buf(
        buf: *mut buf_T,
        lnum: linenr_T,
        line: *mut ::core::ffi::c_char,
        copy: bool,
        noalloc: bool,
    ) -> ::core::ffi::c_int;
    fn ml_delete_flags(lnum: linenr_T, flags: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn update_topline(wp: *mut win_T);
    fn path_with_url(fname: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn concat_str(
        str1: *const ::core::ffi::c_char,
        str2: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn buf_has_signs(buf: *const buf_T) -> bool;
    fn get_buffer_signs(buf: *mut buf_T) -> *mut list_T;
    fn u_save(top: linenr_T, bot: linenr_T) -> ::core::ffi::c_int;
    fn u_savesub(lnum: linenr_T) -> ::core::ffi::c_int;
    fn u_sync(force: bool);
    fn u_clearallandblockfree(buf: *mut buf_T);
    fn bufIsChanged(buf: *mut buf_T) -> bool;
}
pub type size_t = usize;
pub type __time_t = ::core::ffi::c_long;
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
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
    pub b_wininfo: C2Rust_Unnamed_12,
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
    pub uh_next: C2Rust_Unnamed_11,
    pub uh_prev: C2Rust_Unnamed_10,
    pub uh_alt_next: C2Rust_Unnamed_9,
    pub uh_alt_prev: C2Rust_Unnamed_8,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct undo_object {
    pub type_0: UndoObjectType,
    pub data: C2Rust_Unnamed_7,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub splice: ExtmarkSplice,
    pub move_0: ExtmarkMove,
    pub savepos: ExtmarkSavePos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkSavePos {
    pub mark: uint64_t,
    pub old_row: ::core::ffi::c_int,
    pub old_col: colnr_T,
    pub invalidated: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkMove {
    pub start_row: ::core::ffi::c_int,
    pub start_col: ::core::ffi::c_int,
    pub extent_row: ::core::ffi::c_int,
    pub extent_col: ::core::ffi::c_int,
    pub new_row: ::core::ffi::c_int,
    pub new_col: ::core::ffi::c_int,
    pub start_byte: bcount_t,
    pub extent_byte: bcount_t,
    pub new_byte: bcount_t,
}
pub type bcount_t = ptrdiff_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkSplice {
    pub start_row: ::core::ffi::c_int,
    pub start_col: colnr_T,
    pub old_row: ::core::ffi::c_int,
    pub old_col: colnr_T,
    pub new_row: ::core::ffi::c_int,
    pub new_col: colnr_T,
    pub start_byte: bcount_t,
    pub old_byte: bcount_t,
    pub new_byte: bcount_t,
}
pub type UndoObjectType = ::core::ffi::c_uint;
pub const kExtmarkClear: UndoObjectType = 4;
pub const kExtmarkSavePos: UndoObjectType = 3;
pub const kExtmarkUpdate: UndoObjectType = 2;
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_11 {
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
pub struct C2Rust_Unnamed_12 {
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
    pub type_0: C2Rust_Unnamed_13,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_13 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_13 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_13 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_13 = 0;
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
pub type ExtmarkOp = ::core::ffi::c_uint;
pub const kExtmarkUndoNoRedo: ExtmarkOp = 3;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufref_T {
    pub br_buf: *mut buf_T,
    pub br_fnum: ::core::ffi::c_int,
    pub br_buf_free_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct diffblock_S {
    pub df_next: *mut diff_T,
    pub df_lnum: [linenr_T; 8],
    pub df_count: [linenr_T; 8],
    pub is_linematched: bool,
    pub has_changes: bool,
    pub df_changes: garray_T,
}
pub type diff_T = diffblock_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tabpage_S {
    pub handle: handle_T,
    pub tp_next: *mut tabpage_T,
    pub tp_topframe: *mut frame_T,
    pub tp_curwin: *mut win_T,
    pub tp_prevwin: *mut win_T,
    pub tp_firstwin: *mut win_T,
    pub tp_lastwin: *mut win_T,
    pub tp_old_Rows_avail: int64_t,
    pub tp_old_Columns: int64_t,
    pub tp_ch_used: OptInt,
    pub tp_did_tabclosedpre: bool,
    pub tp_first_diff: *mut diff_T,
    pub tp_diffbuf: [*mut buf_T; 8],
    pub tp_diff_invalid: ::core::ffi::c_int,
    pub tp_diff_update: ::core::ffi::c_int,
    pub tp_snapshot: [*mut frame_T; 3],
    pub tp_winvar: ScopeDictDictItem,
    pub tp_vars: *mut dict_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub tp_prevdir: *mut ::core::ffi::c_char,
}
pub type tabpage_T = tabpage_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aco_save_T {
    pub use_aucmd_win_idx: ::core::ffi::c_int,
    pub save_curwin_handle: handle_T,
    pub new_curwin_handle: handle_T,
    pub save_prevwin_handle: handle_T,
    pub new_curbuf: bufref_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub globaldir: *mut ::core::ffi::c_char,
    pub save_VIsual_active: bool,
    pub save_prompt_insert: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cob_T {
    pub cob_curwin_save: *mut win_T,
    pub cob_aco: aco_save_T,
    pub cob_using_aco: ::core::ffi::c_int,
    pub cob_save_VIsual_active: ::core::ffi::c_int,
}
pub const ML_DEL_MESSAGE: C2Rust_Unnamed_14 = 1;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
#[no_mangle]
pub unsafe extern "C" fn find_buffer(mut avar: *mut typval_T) -> *mut buf_T {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if (*avar).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = buflist_findnr((*avar).vval.v_number as ::core::ffi::c_int);
    } else if (*avar).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*avar).vval.v_string.is_null()
    {
        buf = buflist_findname_exp((*avar).vval.v_string);
        if buf.is_null() {
            let mut bp: *mut buf_T = firstbuf;
            while !bp.is_null() {
                if !(*bp).b_fname.is_null()
                    && (path_with_url((*bp).b_fname) != 0
                        || bt_nofilename(bp) as ::core::ffi::c_int != 0)
                    && strcmp((*bp).b_fname, (*avar).vval.v_string) == 0 as ::core::ffi::c_int
                {
                    buf = bp;
                    break;
                } else {
                    bp = (*bp).b_next;
                }
            }
        }
    }
    return buf;
}
unsafe extern "C" fn find_win_for_curbuf() {
    let mut i: size_t = 0 as size_t;
    while i < (*curbuf).b_wininfo.size {
        let mut wip: *mut WinInfo = *(*curbuf).b_wininfo.items.offset(i as isize);
        if !(*wip).wi_win.is_null() && (*(*wip).wi_win).w_buffer == curbuf {
            curwin = (*wip).wi_win;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
}
unsafe extern "C" fn change_other_buffer_prepare(mut cob: *mut cob_T, mut buf: *mut buf_T) {
    memset(
        cob as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cob_T>(),
    );
    (*cob).cob_save_VIsual_active = VIsual_active as ::core::ffi::c_int;
    VIsual_active = false_0 != 0;
    (*cob).cob_curwin_save = curwin;
    curbuf = buf;
    find_win_for_curbuf();
    if (*curwin).w_buffer != buf {
        curbuf = (*curwin).w_buffer;
        aucmd_prepbuf(&raw mut (*cob).cob_aco, buf);
        (*cob).cob_using_aco = true_0;
    }
}
unsafe extern "C" fn change_other_buffer_restore(mut cob: *mut cob_T) {
    if (*cob).cob_using_aco != 0 {
        aucmd_restbuf(&raw mut (*cob).cob_aco);
    } else {
        curwin = (*cob).cob_curwin_save;
        curbuf = (*curwin).w_buffer;
    }
    VIsual_active = (*cob).cob_save_VIsual_active != 0;
}
unsafe extern "C" fn set_buffer_lines(
    mut buf: *mut buf_T,
    mut lnum_arg: linenr_T,
    mut append: bool,
    mut lines: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    let mut lnum: linenr_T = lnum_arg
        + (if append as ::core::ffi::c_int != 0 {
            1 as linenr_T
        } else {
            0 as linenr_T
        });
    let mut added: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let is_curbuf: bool = buf == curbuf;
    if buf.is_null() || !is_curbuf && (*buf).b_ml.ml_mfp.is_null() || lnum < 1 as linenr_T {
        (*rettv).vval.v_number = 1 as varnumber_T;
        return;
    }
    let mut cob: cob_T = cob_T {
        cob_curwin_save: ::core::ptr::null_mut::<win_T>(),
        cob_aco: aco_save_T {
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
        },
        cob_using_aco: 0,
        cob_save_VIsual_active: 0,
    };
    if !is_curbuf {
        change_other_buffer_prepare(&raw mut cob, buf);
    }
    let mut append_lnum: linenr_T = 0;
    if append {
        append_lnum = lnum - 1 as linenr_T;
    } else {
        append_lnum = (*curbuf).b_ml.ml_line_count;
    }
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut li: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_cleanup: {
        if (*lines).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            l = (*lines).vval.v_list;
            if l.is_null() || tv_list_len(l) == 0 as ::core::ffi::c_int {
                break '_cleanup;
            } else {
                li = tv_list_first(l);
            }
        } else {
            line = typval_tostring(lines, false_0 != 0);
        }
        loop {
            if (*lines).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if li.is_null() {
                    break;
                }
                xfree(line as *mut ::core::ffi::c_void);
                line = typval_tostring(&raw mut (*li).li_tv, false_0 != 0);
                li = (*li).li_next;
            }
            (*rettv).vval.v_number = 1 as varnumber_T;
            if line.is_null() || lnum > (*curbuf).b_ml.ml_line_count + 1 as linenr_T {
                break;
            }
            if u_sync_once == 2 as ::core::ffi::c_int {
                u_sync_once = 1 as ::core::ffi::c_int;
                u_sync(true_0 != 0);
            }
            if !append && lnum <= (*curbuf).b_ml.ml_line_count {
                let mut old_len: ::core::ffi::c_int = strlen(ml_get(lnum)) as ::core::ffi::c_int;
                if u_savesub(lnum) == OK && ml_replace(lnum, line, true_0 != 0) == OK {
                    inserted_bytes(
                        lnum,
                        0 as colnr_T,
                        old_len,
                        strlen(line) as ::core::ffi::c_int,
                    );
                    if is_curbuf as ::core::ffi::c_int != 0 && lnum == (*curwin).w_cursor.lnum {
                        check_cursor_col(curwin);
                    }
                    (*rettv).vval.v_number = 0 as varnumber_T;
                }
            } else if added > 0 as ::core::ffi::c_int || u_save(lnum - 1 as linenr_T, lnum) == OK {
                added += 1;
                if ml_append(lnum - 1 as linenr_T, line, 0 as colnr_T, false_0 != 0) == OK {
                    (*rettv).vval.v_number = 0 as varnumber_T;
                }
            }
            if l.is_null() {
                break;
            }
            lnum += 1;
        }
        xfree(line as *mut ::core::ffi::c_void);
        if added > 0 as ::core::ffi::c_int {
            appended_lines_mark(append_lnum, added);
            let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
            while !tp.is_null() {
                let mut wp: *mut win_T = if tp == curtab {
                    firstwin
                } else {
                    (*tp).tp_firstwin
                };
                while !wp.is_null() {
                    if (*wp).w_buffer == buf
                        && ((*wp).w_buffer != curbuf || wp == curwin)
                        && (*wp).w_cursor.lnum > append_lnum
                    {
                        (*wp).w_cursor.lnum += added as linenr_T;
                    }
                    wp = (*wp).w_next;
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
            check_cursor_col(curwin);
            update_topline(curwin);
        }
    }
    if !is_curbuf {
        change_other_buffer_restore(&raw mut cob);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_append(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg;
    let lnum: linenr_T = tv_get_lnum(argvars.offset(0 as ::core::ffi::c_int as isize));
    if did_emsg == did_emsg_before {
        set_buffer_lines(
            curbuf,
            lnum,
            true_0 != 0,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    }
}
unsafe extern "C" fn buf_set_append_line(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut append: bool,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg;
    let buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        (*rettv).vval.v_number = 1 as varnumber_T;
    } else {
        let lnum: linenr_T = tv_get_lnum_buf(argvars.offset(1 as ::core::ffi::c_int as isize), buf);
        if did_emsg == did_emsg_before {
            set_buffer_lines(
                buf,
                lnum,
                append,
                argvars.offset(2 as ::core::ffi::c_int as isize),
                rettv,
            );
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_appendbufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    buf_set_append_line(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_prompt_appendbuf(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg;
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 1 as varnumber_T;
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    if buf.is_null() || !bt_prompt(buf) {
        return;
    }
    let mut lnum: linenr_T = if 0 as linenr_T > (*buf).b_prompt_start.mark.lnum - 1 as linenr_T {
        0 as linenr_T
    } else {
        (*buf).b_prompt_start.mark.lnum - 1 as linenr_T
    };
    let mut lines: *mut typval_T = argvars.offset(1 as ::core::ffi::c_int as isize);
    let mut did_concat: bool = false_0 != 0;
    if !(*buf).b_prompt_append_new_line {
        let mut text: *const ::core::ffi::c_char = if lnum > 0 as linenr_T {
            ml_get_buf(buf, lnum) as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        };
        if (*lines).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut l: *mut list_T = (*lines).vval.v_list;
            if !l.is_null() && tv_list_len(l) > 0 as ::core::ffi::c_int {
                let mut li: *mut listitem_T = tv_list_first(l);
                let mut str: *const ::core::ffi::c_char = tv_get_string(&raw mut (*li).li_tv);
                let mut new_str: *mut ::core::ffi::c_char = concat_str(text, str);
                tv_clear(&raw mut (*li).li_tv);
                (*li).li_tv.v_type = VAR_STRING;
                (*li).li_tv.vval.v_string = new_str;
                did_concat = true_0 != 0;
            }
        } else if (*lines).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut str_0: *const ::core::ffi::c_char = tv_get_string(lines);
            let mut new_str_0: *mut ::core::ffi::c_char = concat_str(text, str_0);
            tv_clear(lines);
            (*lines).v_type = VAR_STRING;
            (*lines).vval.v_string = new_str_0;
        }
    }
    if did_emsg == did_emsg_before {
        if did_concat as ::core::ffi::c_int != 0
            && tv_list_len((*lines).vval.v_list) > 1 as ::core::ffi::c_int
        {
            let mut l_0: *mut list_T = (*lines).vval.v_list;
            let mut li_0: *mut listitem_T = tv_list_first(l_0);
            set_buffer_lines(buf, lnum, false_0 != 0, &raw mut (*li_0).li_tv, rettv);
            if (*rettv).vval.v_number == 0 as varnumber_T {
                tv_list_item_remove(l_0, li_0);
                set_buffer_lines(buf, lnum, true_0 != 0, lines, rettv);
            }
        } else {
            set_buffer_lines(buf, lnum, (*buf).b_prompt_append_new_line, lines, rettv);
        }
    }
    if (*rettv).vval.v_number == 0 as varnumber_T {
        (*buf).b_prompt_append_new_line = false_0 != 0;
        if (*lines).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut l_1: *mut list_T = (*lines).vval.v_list;
            if !l_1.is_null() && tv_list_len(l_1) > 0 as ::core::ffi::c_int {
                let mut li_1: *mut listitem_T = tv_list_last(l_1);
                let mut str_1: *const ::core::ffi::c_char = tv_get_string(&raw mut (*li_1).li_tv);
                let mut len: size_t = strlen(str_1);
                if len > 0 as size_t
                    && *str_1.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        == '\n' as ::core::ffi::c_int
                {
                    (*buf).b_prompt_append_new_line = true_0 != 0;
                }
            }
        } else if (*lines).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut str_2: *const ::core::ffi::c_char = tv_get_string(lines);
            let mut len_0: size_t = strlen(str_2);
            if len_0 > 0 as size_t
                && *str_2.offset(len_0.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    == '\n' as ::core::ffi::c_int
            {
                (*buf).b_prompt_append_new_line = true_0 != 0;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_bufadd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut name: *mut ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_char;
    (*rettv).vval.v_number = buflist_add(
        if *name as ::core::ffi::c_int == NUL {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            name
        },
        0 as ::core::ffi::c_int,
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_bufexists(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = !find_buffer(argvars.offset(0 as ::core::ffi::c_int as isize))
        .is_null() as ::core::ffi::c_int as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_buflisted(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    buf = find_buffer(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number =
        (!buf.is_null() && (*buf).b_p_bl != 0) as ::core::ffi::c_int as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_bufload(
    mut argvars: *mut typval_T,
    mut unused: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut buf: *mut buf_T = get_buf_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    if !buf.is_null() {
        if swap_exists_action != SEA_READONLY {
            swap_exists_action = SEA_NONE;
        }
        buf_ensure_loaded(buf);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_bufloaded(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    buf = find_buffer(argvars.offset(0 as ::core::ffi::c_int as isize));
    (*rettv).vval.v_number =
        (!buf.is_null() && !(*buf).b_ml.ml_mfp.is_null()) as ::core::ffi::c_int as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_bufname(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut buf: *const buf_T = ::core::ptr::null::<buf_T>();
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = curbuf;
    } else {
        buf = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    }
    if !buf.is_null() && !(*buf).b_fname.is_null() {
        (*rettv).vval.v_string = xstrdup((*buf).b_fname);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_bufnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut buf: *const buf_T = ::core::ptr::null::<buf_T>();
    let mut error: bool = false_0 != 0;
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = curbuf;
    } else {
        if !tv_check_str_or_nr(argvars.offset(0 as ::core::ffi::c_int as isize)) {
            return;
        }
        emsg_off += 1;
        buf = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
        emsg_off -= 1;
    }
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if buf.is_null()
        && (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) != 0 as varnumber_T
        && !error
        && {
            name = tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
            !name.is_null()
        }
    {
        buf = buflist_new(
            name as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            1 as linenr_T,
            0 as ::core::ffi::c_int,
        );
    }
    if !buf.is_null() {
        (*rettv).vval.v_number = (*buf).handle as varnumber_T;
    }
}
unsafe extern "C" fn buf_win_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut get_nr: bool,
) {
    let buf: *const buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    if buf.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
        return;
    }
    let mut winnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut winid: ::core::ffi::c_int = 0;
    let mut found_buf: bool = false_0 != 0;
    let mut wp: *mut win_T = if curtab == curtab {
        firstwin
    } else {
        (*curtab).tp_firstwin
    };
    while !wp.is_null() {
        winnr += win_has_winnr(wp, curtab) as ::core::ffi::c_int;
        if (*wp).w_buffer == buf as *mut buf_T
            && (!get_nr || win_has_winnr(wp, curtab) as ::core::ffi::c_int != 0)
        {
            found_buf = true_0 != 0;
            winid = (*wp).handle as ::core::ffi::c_int;
            break;
        } else {
            wp = (*wp).w_next;
        }
    }
    (*rettv).vval.v_number = (if found_buf as ::core::ffi::c_int != 0 {
        if get_nr as ::core::ffi::c_int != 0 {
            winnr
        } else {
            winid
        }
    } else {
        -1 as ::core::ffi::c_int
    }) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_bufwinid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    buf_win_common(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_bufwinnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    buf_win_common(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_deletebufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg;
    (*rettv).vval.v_number = 1 as varnumber_T;
    let buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        return;
    }
    let mut last: linenr_T = 0;
    let first: linenr_T = tv_get_lnum_buf(argvars.offset(1 as ::core::ffi::c_int as isize), buf);
    if did_emsg > did_emsg_before {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        last = tv_get_lnum_buf(argvars.offset(2 as ::core::ffi::c_int as isize), buf);
    } else {
        last = first;
    }
    if (*buf).b_ml.ml_mfp.is_null()
        || first < 1 as linenr_T
        || first > (*buf).b_ml.ml_line_count
        || last < first
    {
        return;
    }
    let is_curbuf: bool = buf == curbuf;
    let mut cob: cob_T = cob_T {
        cob_curwin_save: ::core::ptr::null_mut::<win_T>(),
        cob_aco: aco_save_T {
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
        },
        cob_using_aco: 0,
        cob_save_VIsual_active: 0,
    };
    if !is_curbuf {
        change_other_buffer_prepare(&raw mut cob, buf);
    }
    if last > (*curbuf).b_ml.ml_line_count {
        last = (*curbuf).b_ml.ml_line_count;
    }
    let count: ::core::ffi::c_int =
        last as ::core::ffi::c_int - first as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if u_sync_once == 2 as ::core::ffi::c_int {
        u_sync_once = 1 as ::core::ffi::c_int;
        u_sync(true_0 != 0);
    }
    if u_save(first - 1 as linenr_T, last + 1 as linenr_T) != FAIL {
        let mut lnum: linenr_T = first;
        while lnum <= last {
            ml_delete_flags(first, ML_DEL_MESSAGE as ::core::ffi::c_int);
            lnum += 1;
        }
        let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab {
                firstwin
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == buf {
                    if (*wp).w_cursor.lnum > last {
                        (*wp).w_cursor.lnum -= count as linenr_T;
                    } else if (*wp).w_cursor.lnum > first {
                        (*wp).w_cursor.lnum = first;
                    }
                    if (*wp).w_cursor.lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
                        (*wp).w_cursor.lnum = (*(*wp).w_buffer).b_ml.ml_line_count;
                    }
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        check_cursor_col(curwin);
        deleted_lines_mark(first, count);
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
    if !is_curbuf {
        change_other_buffer_restore(&raw mut cob);
    }
}
unsafe extern "C" fn get_buffer_info(mut buf: *mut buf_T) -> *mut dict_T {
    let dict: *mut dict_T = tv_dict_alloc();
    tv_dict_add_nr(
        dict,
        b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*buf).handle as varnumber_T,
    );
    tv_dict_add_str(
        dict,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        if !(*buf).b_ffname.is_null() {
            (*buf).b_ffname as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    tv_dict_add_nr(
        dict,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (if buf == curbuf {
            (*curwin).w_cursor.lnum
        } else {
            buflist_findlnum(buf)
        }) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"linecount\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (*buf).b_ml.ml_line_count as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        !(*buf).b_ml.ml_mfp.is_null() as ::core::ffi::c_int as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"listed\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*buf).b_p_bl as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"changed\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        bufIsChanged(buf) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        buf_get_changedtick(buf),
    );
    tv_dict_add_nr(
        dict,
        b"hidden\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (!(*buf).b_ml.ml_mfp.is_null() && (*buf).b_nwindows == 0 as ::core::ffi::c_int)
            as ::core::ffi::c_int as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"command\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (buf == cmdwin_buf) as ::core::ffi::c_int as varnumber_T,
    );
    tv_dict_add_dict(
        dict,
        b"variables\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (*buf).b_vars,
    );
    let windows: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab {
            firstwin
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                tv_list_append_number(windows, (*wp).handle as varnumber_T);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    tv_dict_add_list(
        dict,
        b"windows\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        windows,
    );
    if buf_has_signs(buf) {
        tv_dict_add_list(
            dict,
            b"signs\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            get_buffer_signs(buf),
        );
    }
    tv_dict_add_nr(
        dict,
        b"lastused\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*buf).b_last_used as varnumber_T,
    );
    return dict;
}
#[no_mangle]
pub unsafe extern "C" fn f_getbufinfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut argbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut filtered: bool = false_0 != 0;
    let mut sel_buflisted: bool = false_0 != 0;
    let mut sel_bufloaded: bool = false_0 != 0;
    let mut sel_bufmodified: bool = false_0 != 0;
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut sel_d: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if !sel_d.is_null() {
            let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
            filtered = true_0 != 0;
            di = tv_dict_find(
                sel_d,
                b"buflisted\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if !di.is_null() && tv_get_number(&raw mut (*di).di_tv) != 0 {
                sel_buflisted = true_0 != 0;
            }
            di = tv_dict_find(
                sel_d,
                b"bufloaded\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if !di.is_null() && tv_get_number(&raw mut (*di).di_tv) != 0 {
                sel_bufloaded = true_0 != 0;
            }
            di = tv_dict_find(
                sel_d,
                b"bufmodified\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if !di.is_null() && tv_get_number(&raw mut (*di).di_tv) != 0 {
                sel_bufmodified = true_0 != 0;
            }
        }
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        argbuf = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
        if argbuf.is_null() {
            return;
        }
    }
    let mut buf: *mut buf_T = firstbuf;
    while !buf.is_null() {
        if !(!argbuf.is_null() && argbuf != buf) {
            if !(filtered as ::core::ffi::c_int != 0
                && (sel_bufloaded as ::core::ffi::c_int != 0 && (*buf).b_ml.ml_mfp.is_null()
                    || sel_buflisted as ::core::ffi::c_int != 0 && (*buf).b_p_bl == 0
                    || sel_bufmodified as ::core::ffi::c_int != 0 && (*buf).b_changed == 0))
            {
                let d: *mut dict_T = get_buffer_info(buf);
                tv_list_append_dict((*rettv).vval.v_list, d);
                if !argbuf.is_null() {
                    return;
                }
            }
        }
        buf = (*buf).b_next;
    }
}
unsafe extern "C" fn get_buffer_lines(
    mut buf: *mut buf_T,
    mut start: linenr_T,
    mut end: linenr_T,
    mut retlist: bool,
    mut rettv: *mut typval_T,
) {
    (*rettv).v_type = (if retlist as ::core::ffi::c_int != 0 {
        VAR_LIST as ::core::ffi::c_int
    } else {
        VAR_STRING as ::core::ffi::c_int
    }) as VarType;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() || start < 0 as linenr_T || end < start {
        if retlist {
            tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
        }
        return;
    }
    if retlist {
        if start < 1 as linenr_T {
            start = 1 as ::core::ffi::c_int as linenr_T;
        }
        if end > (*buf).b_ml.ml_line_count {
            end = (*buf).b_ml.ml_line_count;
        }
        tv_list_alloc_ret(rettv, (end - start + 1 as linenr_T) as ptrdiff_t);
        while start <= end {
            tv_list_append_string(
                (*rettv).vval.v_list,
                ml_get_buf(buf, start),
                ml_get_buf_len(buf, start) as ssize_t,
            );
            start += 1;
        }
    } else {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = if start >= 1 as linenr_T && start <= (*buf).b_ml.ml_line_count {
            xstrnsave(ml_get_buf(buf, start), ml_get_buf_len(buf, start) as size_t)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
    };
}
unsafe extern "C" fn getbufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut retlist: bool,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg;
    let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
    let lnum: linenr_T = tv_get_lnum_buf(argvars.offset(1 as ::core::ffi::c_int as isize), buf);
    if did_emsg > did_emsg_before {
        return;
    }
    let end: linenr_T = if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type
        as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lnum
    } else {
        tv_get_lnum_buf(argvars.offset(2 as ::core::ffi::c_int as isize), buf)
    };
    get_buffer_lines(buf, lnum, end, retlist, rettv);
}
#[no_mangle]
pub unsafe extern "C" fn f_getbufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    getbufline(argvars, rettv, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_getbufoneline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    getbufline(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_getline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut end: linenr_T = 0;
    let mut retlist: bool = false;
    let lnum: linenr_T = tv_get_lnum(argvars);
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        end = lnum;
        retlist = false_0 != 0;
    } else {
        end = tv_get_lnum(argvars.offset(1 as ::core::ffi::c_int as isize));
        retlist = true_0 != 0;
    }
    get_buffer_lines(curbuf, lnum, end, retlist, rettv);
}
#[no_mangle]
pub unsafe extern "C" fn f_setbufline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    buf_set_append_line(argvars, rettv, false_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_setline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let did_emsg_before: ::core::ffi::c_int = did_emsg;
    let mut lnum: linenr_T = tv_get_lnum(argvars.offset(0 as ::core::ffi::c_int as isize));
    if did_emsg == did_emsg_before {
        set_buffer_lines(
            curbuf,
            lnum,
            false_0 != 0,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn switch_buffer(mut save_curbuf: *mut bufref_T, mut buf: *mut buf_T) {
    block_autocmds();
    set_bufref(save_curbuf, curbuf);
    (*curbuf).b_nwindows -= 1;
    curbuf = buf;
    (*curwin).w_buffer = buf;
    (*curbuf).b_nwindows += 1;
}
#[no_mangle]
pub unsafe extern "C" fn restore_buffer(mut save_curbuf: *mut bufref_T) {
    unblock_autocmds();
    if bufref_valid(save_curbuf) {
        (*curbuf).b_nwindows -= 1;
        (*curwin).w_buffer = (*save_curbuf).br_buf;
        curbuf = (*save_curbuf).br_buf;
        (*curbuf).b_nwindows += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_prompt_setcallback(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut prompt_callback: Callback = Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if check_secure() {
        return;
    }
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        return;
    }
    if !callback_from_typval(
        &raw mut prompt_callback,
        argvars.offset(1 as ::core::ffi::c_int as isize),
    ) {
        return;
    }
    callback_free(&raw mut (*buf).b_prompt_callback);
    (*buf).b_prompt_callback = prompt_callback;
}
#[no_mangle]
pub unsafe extern "C" fn f_prompt_setinterrupt(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    let mut interrupt_callback: Callback = Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if check_secure() {
        return;
    }
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        return;
    }
    if !callback_from_typval(
        &raw mut interrupt_callback,
        argvars.offset(1 as ::core::ffi::c_int as isize),
    ) {
        return;
    }
    callback_free(&raw mut (*buf).b_prompt_interrupt);
    (*buf).b_prompt_interrupt = interrupt_callback;
}
#[no_mangle]
pub unsafe extern "C" fn f_prompt_setprompt(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    if check_secure() {
        return;
    }
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() {
        return;
    }
    let mut new_prompt: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut new_prompt_len: ::core::ffi::c_int = strlen(new_prompt) as ::core::ffi::c_int;
    if bt_prompt(buf) as ::core::ffi::c_int != 0 && !(*buf).b_ml.ml_mfp.is_null() {
        if (*buf).b_prompt_start.mark.lnum < 1 as linenr_T
            || (*buf).b_prompt_start.mark.lnum > (*curbuf).b_ml.ml_line_count
        {
            (*buf).b_prompt_start.mark.lnum = if 1 as linenr_T
                > (if (*buf).b_prompt_start.mark.lnum < (*buf).b_ml.ml_line_count {
                    (*buf).b_prompt_start.mark.lnum
                } else {
                    (*buf).b_ml.ml_line_count
                }) {
                1 as linenr_T
            } else if (*buf).b_prompt_start.mark.lnum < (*buf).b_ml.ml_line_count {
                (*buf).b_prompt_start.mark.lnum
            } else {
                (*buf).b_ml.ml_line_count
            };
            (*curbuf).b_prompt_append_new_line = true_0 != 0;
        }
        let mut prompt_lno: linenr_T = (*buf).b_prompt_start.mark.lnum;
        let mut old_prompt: *mut ::core::ffi::c_char = buf_prompt_text(buf);
        let mut old_line: *mut ::core::ffi::c_char = ml_get_buf(buf, prompt_lno);
        let mut old_line_len: colnr_T = ml_get_buf_len(buf, prompt_lno);
        let mut old_prompt_len: ::core::ffi::c_int = strlen(old_prompt) as ::core::ffi::c_int;
        let mut cursor_col: colnr_T = (*curwin).w_cursor.col;
        if (*buf).b_prompt_start.mark.col < old_prompt_len
            || (*buf).b_prompt_start.mark.col > old_line_len
            || !strnequal(
                old_prompt,
                old_line
                    .offset((*buf).b_prompt_start.mark.col as isize)
                    .offset(-(old_prompt_len as isize)),
                old_prompt_len as size_t,
            )
        {
            ml_replace_buf(
                buf,
                prompt_lno,
                new_prompt as *mut ::core::ffi::c_char,
                true_0 != 0,
                false_0 != 0,
            );
            extmark_splice_cols(
                buf,
                prompt_lno as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                0 as colnr_T,
                old_line_len,
                new_prompt_len as colnr_T,
                kExtmarkNoUndo,
            );
            cursor_col = new_prompt_len as colnr_T;
        } else {
            let mut new_line: *mut ::core::ffi::c_char = concat_str(
                new_prompt,
                old_line.offset((*buf).b_prompt_start.mark.col as isize),
            );
            if ml_replace_buf(buf, prompt_lno, new_line, false_0 != 0, false_0 != 0) != OK {
                xfree(new_line as *mut ::core::ffi::c_void);
            }
            extmark_splice_cols(
                buf,
                prompt_lno as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                0 as colnr_T,
                (*buf).b_prompt_start.mark.col,
                new_prompt_len as colnr_T,
                kExtmarkNoUndo,
            );
            cursor_col +=
                (new_prompt_len as colnr_T - (*buf).b_prompt_start.mark.col) as ::core::ffi::c_int;
        }
        if (*curwin).w_buffer == buf && (*curwin).w_cursor.lnum == prompt_lno {
            (*curwin).w_cursor.col = cursor_col;
            check_cursor_col(curwin);
        }
        changed_lines(
            buf,
            prompt_lno,
            0 as colnr_T,
            prompt_lno + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
        u_clearallandblockfree(buf);
    }
    xfree((*buf).b_prompt_text as *mut ::core::ffi::c_void);
    (*buf).b_prompt_text = xstrdup(new_prompt);
    (*buf).b_prompt_start.mark.col = new_prompt_len;
}
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
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
pub const SEA_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEA_READONLY: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
