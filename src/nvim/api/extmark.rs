use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type terminal;
    pub type regprog;
    pub type qf_info_S;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn mh_get_ptr_t(set: *mut Set_ptr_t, key: ptr_t) -> uint32_t;
    fn mh_put_ptr_t(set: *mut Set_ptr_t, key: ptr_t, new: *mut MHPutStatus) -> uint32_t;
    fn mh_delete_uint32_t(set: *mut Set_uint32_t, key: *mut uint32_t) -> uint32_t;
    fn mh_put_uint32_t(set: *mut Set_uint32_t, key: uint32_t, new: *mut MHPutStatus) -> uint32_t;
    fn mh_get_uint32_t(set: *mut Set_uint32_t, key: uint32_t) -> uint32_t;
    fn mh_get_String(set: *mut Set_String, key: String_0) -> uint32_t;
    fn map_put_ref_String_int(
        map: *mut Map_String_int,
        key: String_0,
        key_alloc: *mut *mut String_0,
        new_item: *mut bool,
    ) -> *mut ::core::ffi::c_int;
    static namespace_ids: GlobalCell<Map_String_int>;
    static namespace_localscope: GlobalCell<Set_uint32_t>;
    static next_namespace_id: GlobalCell<handle_T>;
    static set_extmark_table: GlobalCell<[KeySetLink; 36]>;
    fn find_buffer_by_handle(buffer: Buffer, err: *mut Error) -> *mut buf_T;
    fn find_window_by_handle(window: Window, err: *mut Error) -> *mut win_T;
    fn string_to_cstr(str: String_0) -> *mut ::core::ffi::c_char;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn copy_string(str: String_0, arena: *mut Arena) -> String_0;
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn object_to_hl_id(
        obj: Object,
        what: *const ::core::ffi::c_char,
        err: *mut Error,
    ) -> ::core::ffi::c_int;
    fn api_typename(t: ObjectType) -> *mut ::core::ffi::c_char;
    fn api_err_invalid(
        err: *mut Error,
        name: *const ::core::ffi::c_char,
        val_s: *const ::core::ffi::c_char,
        val_n: int64_t,
        quote_val: bool,
    );
    fn api_err_exp(
        err: *mut Error,
        name: *const ::core::ffi::c_char,
        expected: *const ::core::ffi::c_char,
        actual: *const ::core::ffi::c_char,
    );
    static decor_state: GlobalCell<DecorState>;
    fn decor_put_sh(item: DecorSignHighlight) -> uint32_t;
    fn decor_put_vt(vt: DecorVirtText, next: *mut DecorVirtText) -> *mut DecorVirtText;
    fn decor_sh_from_inline(item: DecorHighlightInline) -> DecorSignHighlight;
    fn decor_free(decor: DecorInline);
    fn clear_virttext(text: *mut VirtText);
    fn clear_virtlines(lines: *mut VirtLines);
    fn decor_range_add_virt(
        state: *mut DecorState,
        start_row: ::core::ffi::c_int,
        start_col: ::core::ffi::c_int,
        end_row: ::core::ffi::c_int,
        end_col: ::core::ffi::c_int,
        vt: *mut DecorVirtText,
        owned: bool,
    );
    fn decor_range_add_sh(
        state: *mut DecorState,
        start_row: ::core::ffi::c_int,
        start_col: ::core::ffi::c_int,
        end_row: ::core::ffi::c_int,
        end_col: ::core::ffi::c_int,
        sh: *mut DecorSignHighlight,
        owned: bool,
        ns: uint32_t,
        mark_id: uint32_t,
        subpriority: DecorPriority,
    );
    fn decor_to_dict_legacy(dict: *mut Dict, decor: DecorInline, hl_name: bool, arena: *mut Arena);
    fn hl_group_name(hl_id: ::core::ffi::c_int, hl_name: bool) -> Object;
    fn get_decor_provider(ns_id: NS, force: bool) -> *mut DecorProvider;
    fn decor_provider_clear(p: *mut DecorProvider);
    fn extmark_set(
        buf: *mut buf_T,
        ns_id: uint32_t,
        idp: *mut uint32_t,
        row: ::core::ffi::c_int,
        col: colnr_T,
        end_row: ::core::ffi::c_int,
        end_col: colnr_T,
        decor: DecorInline,
        decor_flags: uint16_t,
        right_gravity: bool,
        end_right_gravity: bool,
        no_undo: bool,
        invalidate: bool,
        err: *mut Error,
    );
    fn extmark_del_id(buf: *mut buf_T, ns_id: uint32_t, id: uint32_t) -> bool;
    fn extmark_clear(
        buf: *mut buf_T,
        ns_id: uint32_t,
        l_row: ::core::ffi::c_int,
        l_col: colnr_T,
        u_row: ::core::ffi::c_int,
        u_col: colnr_T,
    ) -> bool;
    fn extmark_get(
        buf: *mut buf_T,
        ns_id: uint32_t,
        l_row: ::core::ffi::c_int,
        l_col: colnr_T,
        u_row: ::core::ffi::c_int,
        u_col: colnr_T,
        amount: int64_t,
        type_filter: ExtmarkType,
        overlap: bool,
    ) -> ExtmarkInfoArray;
    fn extmark_from_id(buf: *mut buf_T, ns_id: uint32_t, id: uint32_t) -> MTPair;
    fn redraw_all_later(type_0: ::core::ffi::c_int);
    static firstwin: GlobalCell<*mut win_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    fn schar_high(sc: schar_T) -> bool;
    fn mt_inspect(b: *mut MarkTree, keys: bool, dot: bool) -> String_0;
    fn mb_string2cells(str: *const ::core::ffi::c_char) -> size_t;
    fn utfc_ptr2schar(p: *const ::core::ffi::c_char, firstc: *mut ::core::ffi::c_int) -> schar_T;
    fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T;
    fn changed_window_setting(wp: *mut win_T);
    fn init_sign_text(
        sp: *mut sign_T,
        sign_text: *mut schar_T,
        text: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn transstr(s: *const ::core::ffi::c_char, untab: bool) -> *mut ::core::ffi::c_char;
    fn vim_isprintc(c: ::core::ffi::c_int) -> bool;
}
pub type ptrdiff_t = isize;
pub type size_t = usize;
pub type __time_t = ::core::ffi::c_long;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type time_t = __time_t;
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
pub type NS = handle_T;
pub type proftime_T = uint64_t;
pub type TriState = ::core::ffi::c_int;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
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
pub type Buffer = handle_T;
pub type OptionalKeys = uint64_t;
pub type HLGroupID = Integer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeySetLink {
    pub str: *mut ::core::ffi::c_char,
    pub ptr_off: size_t,
    pub type_0: ::core::ffi::c_int,
    pub opt_index: ::core::ffi::c_int,
    pub is_hlgroup: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_set_decoration_provider {
    pub is_set__set_decoration_provider_: OptionalKeys,
    pub on_start: LuaRef,
    pub on_buf: LuaRef,
    pub on_win: LuaRef,
    pub on_line: LuaRef,
    pub on_range: LuaRef,
    pub on_end: LuaRef,
    pub _on_hl_def: LuaRef,
    pub _on_spell_nav: LuaRef,
    pub _on_conceal_line: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_set_extmark {
    pub is_set__set_extmark_: OptionalKeys,
    pub id: Integer,
    pub end_line: Integer,
    pub end_row: Integer,
    pub end_col: Integer,
    pub hl_group: Object,
    pub virt_text: Array,
    pub virt_text_pos: String_0,
    pub virt_text_win_col: Integer,
    pub virt_text_hide: Boolean,
    pub virt_text_repeat_linebreak: Boolean,
    pub hl_eol: Boolean,
    pub hl_mode: String_0,
    pub invalidate: Boolean,
    pub ephemeral: Boolean,
    pub priority: Integer,
    pub right_gravity: Boolean,
    pub end_right_gravity: Boolean,
    pub virt_lines: Array,
    pub virt_lines_above: Boolean,
    pub virt_lines_leftcol: Boolean,
    pub virt_lines_overflow: String_0,
    pub strict: Boolean,
    pub sign_text: String_0,
    pub sign_hl_group: HLGroupID,
    pub number_hl_group: HLGroupID,
    pub line_hl_group: HLGroupID,
    pub cursorline_hl_group: HLGroupID,
    pub conceal: String_0,
    pub conceal_lines: String_0,
    pub spell: Boolean,
    pub ui_watched: Boolean,
    pub undo_restore: Boolean,
    pub url: String_0,
    pub scoped: Boolean,
    pub _subpriority: Integer,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_extmark {
    pub is_set__get_extmark_: OptionalKeys,
    pub details: Boolean,
    pub hl_name: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_extmarks {
    pub is_set__get_extmarks_: OptionalKeys,
    pub limit: Integer,
    pub details: Boolean,
    pub hl_name: Boolean,
    pub overlap: Boolean,
    pub type_0: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_ns_opts {
    pub is_set__ns_opts_: OptionalKeys,
    pub wins: Array,
}
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const MAXLNUM: C2Rust_Unnamed_14 = 2147483647;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_15 = 2147483647;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const kVLScroll: C2Rust_Unnamed_16 = 2;
pub const kVLLeftcol: C2Rust_Unnamed_16 = 1;
pub type DecorPriorityInternal = uint32_t;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kHlModeBlend: C2Rust_Unnamed_17 = 3;
pub const kHlModeCombine: C2Rust_Unnamed_17 = 2;
pub const kHlModeReplace: C2Rust_Unnamed_17 = 1;
pub const kHlModeUnknown: C2Rust_Unnamed_17 = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kSHConcealLines: C2Rust_Unnamed_18 = 128;
pub const kSHConceal: C2Rust_Unnamed_18 = 64;
pub const kSHSpellOff: C2Rust_Unnamed_18 = 32;
pub const kSHSpellOn: C2Rust_Unnamed_18 = 16;
pub const kSHUIWatchedOverlay: C2Rust_Unnamed_18 = 8;
pub const kSHUIWatched: C2Rust_Unnamed_18 = 4;
pub const kSHHlEol: C2Rust_Unnamed_18 = 2;
pub const kSHIsSign: C2Rust_Unnamed_18 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorSignHighlight {
    pub flags: uint16_t,
    pub priority: DecorPriority,
    pub hl_id: ::core::ffi::c_int,
    pub text: [schar_T; 2],
    pub sign_name: *mut ::core::ffi::c_char,
    pub sign_add_id: ::core::ffi::c_int,
    pub number_hl_id: ::core::ffi::c_int,
    pub line_hl_id: ::core::ffi::c_int,
    pub cursorline_hl_id: ::core::ffi::c_int,
    pub next: uint32_t,
    pub url: *const ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kVTRepeatLinebreak: C2Rust_Unnamed_19 = 8;
pub const kVTLinesAbove: C2Rust_Unnamed_19 = 4;
pub const kVTHide: C2Rust_Unnamed_19 = 2;
pub const kVTIsLines: C2Rust_Unnamed_19 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorInline {
    pub ext: bool,
    pub data: DecorInlineData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorProvider {
    pub ns_id: NS,
    pub state: C2Rust_Unnamed_20,
    pub win_skip_row: ::core::ffi::c_int,
    pub win_skip_col: ::core::ffi::c_int,
    pub redraw_start: LuaRef,
    pub redraw_buf: LuaRef,
    pub redraw_win: LuaRef,
    pub redraw_line: LuaRef,
    pub redraw_range: LuaRef,
    pub redraw_end: LuaRef,
    pub hl_def: LuaRef,
    pub spell_nav: LuaRef,
    pub conceal_line: LuaRef,
    pub hl_valid: ::core::ffi::c_int,
    pub hl_cached: bool,
    pub error_count: uint8_t,
}
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const kDecorProviderDisabled: C2Rust_Unnamed_20 = 4;
pub const kDecorProviderRedrawDisabled: C2Rust_Unnamed_20 = 3;
pub const kDecorProviderWinDisabled: C2Rust_Unnamed_20 = 2;
pub const kDecorProviderActive: C2Rust_Unnamed_20 = 1;
pub type MHPutStatus = ::core::ffi::c_uint;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_ptr_t {
    pub h: MapHash,
    pub keys: *mut ptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_String {
    pub h: MapHash,
    pub keys: *mut String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_String_int {
    pub set: Set_String,
    pub values: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MarkTreeIter {
    pub pos: MTPos,
    pub lvl: ::core::ffi::c_int,
    pub x: *mut MTNode,
    pub i: ::core::ffi::c_int,
    pub s: [C2Rust_Unnamed_21; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
    pub oldcol: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTPair {
    pub start: MTKey,
    pub end_pos: MTPos,
    pub end_right_gravity: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sign_T {
    pub sn_name: *mut ::core::ffi::c_char,
    pub sn_icon: *mut ::core::ffi::c_char,
    pub sn_text: [schar_T; 2],
    pub sn_line_hl: ::core::ffi::c_int,
    pub sn_text_hl: ::core::ffi::c_int,
    pub sn_cul_hl: ::core::ffi::c_int,
    pub sn_num_hl: ::core::ffi::c_int,
    pub sn_priority: ::core::ffi::c_int,
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
pub struct ExtmarkInfoArray {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTPair,
}
pub type ExtmarkType = ::core::ffi::c_uint;
pub const kExtmarkHighlight: ExtmarkType = 32;
pub const kExtmarkVirtLines: ExtmarkType = 16;
pub const kExtmarkVirtText: ExtmarkType = 8;
pub const kExtmarkSignHL: ExtmarkType = 4;
pub const kExtmarkSign: ExtmarkType = 2;
pub const kExtmarkNone: ExtmarkType = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorState {
    pub itr: [MarkTreeIter; 1],
    pub slots: C2Rust_Unnamed_23,
    pub ranges_i: C2Rust_Unnamed_22,
    pub current_end: ::core::ffi::c_int,
    pub future_begin: ::core::ffi::c_int,
    pub free_slot_i: ::core::ffi::c_int,
    pub new_range_ordering: ::core::ffi::c_int,
    pub win: *mut win_T,
    pub top_row: ::core::ffi::c_int,
    pub row: ::core::ffi::c_int,
    pub col_last: ::core::ffi::c_int,
    pub current: ::core::ffi::c_int,
    pub eol_col: ::core::ffi::c_int,
    pub conceal: ::core::ffi::c_int,
    pub conceal_char: schar_T,
    pub conceal_attr: ::core::ffi::c_int,
    pub spell: TriState,
    pub running_decor_provider: bool,
    pub itr_valid: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_22 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_23 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut DecorRangeSlot,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union DecorRangeSlot {
    pub range: DecorRange,
    pub next_free_i: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorRange {
    pub start_row: ::core::ffi::c_int,
    pub start_col: ::core::ffi::c_int,
    pub end_row: ::core::ffi::c_int,
    pub end_col: ::core::ffi::c_int,
    pub ordering: ::core::ffi::c_int,
    pub priority_internal: DecorPriorityInternal,
    pub owned: bool,
    pub kind: DecorRangeKind,
    pub data: C2Rust_Unnamed_24,
    pub attr_id: ::core::ffi::c_int,
    pub draw_col: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_24 {
    pub sh: DecorSignHighlight,
    pub vt: *mut DecorVirtText,
    pub ui: C2Rust_Unnamed_25,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_25 {
    pub ns_id: uint32_t,
    pub mark_id: uint32_t,
    pub pos: VirtTextPos,
}
pub type DecorRangeKind = uint8_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_26 {
    pub name: *const ::core::ffi::c_char,
    pub source: *mut LuaRef,
    pub dest: *mut LuaRef,
}
pub const UPD_NOT_VALID: C2Rust_Unnamed_27 = 40;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_27 = 50;
pub const UPD_SOME_VALID: C2Rust_Unnamed_27 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_27 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_27 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_27 = 20;
pub const UPD_VALID: C2Rust_Unnamed_27 = 10;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 87] = unsafe {
    ::core::mem::transmute::<
        [u8; 87],
        [::core::ffi::c_char; 87],
    >(
        *b"void nvim_set_decoration_provider(Integer, KeyDict_set_decoration_provider *, Error *)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    conceal_char: 0 as schar_T,
};
pub const DECOR_SIGN_HIGHLIGHT_INIT: DecorSignHighlight = DecorSignHighlight {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    text: [0 as schar_T, 0 as schar_T],
    sign_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    sign_add_id: 0 as ::core::ffi::c_int,
    number_hl_id: 0 as ::core::ffi::c_int,
    line_hl_id: 0 as ::core::ffi::c_int,
    cursorline_hl_id: 0 as ::core::ffi::c_int,
    next: DECOR_ID_INVALID as uint32_t,
    url: ::core::ptr::null::<::core::ffi::c_char>(),
};
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false_0 != 0,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_ptr_t(mut set: *mut Set_ptr_t, mut key: ptr_t) -> bool {
    return mh_get_ptr_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn set_put_ptr_t(
    mut set: *mut Set_ptr_t,
    mut key: ptr_t,
    mut key_alloc: *mut *mut ptr_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_ptr_t(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn set_has_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> bool {
    return mh_get_uint32_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn set_del_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> uint32_t {
    mh_delete_uint32_t(set, &raw mut key);
    return key;
}
#[inline]
unsafe extern "C" fn set_put_uint32_t(
    mut set: *mut Set_uint32_t,
    mut key: uint32_t,
    mut key_alloc: *mut *mut uint32_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_uint32_t(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn map_put_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut value: ::core::ffi::c_int,
) {
    let mut val: *mut ::core::ffi::c_int = map_put_ref_String_int(
        map,
        key,
        ::core::ptr::null_mut::<*mut String_0>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_String(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_int.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[no_mangle]
pub unsafe extern "C" fn api_extmark_free_all_mem() {
    let mut name: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*namespace_ids.ptr()).set.h.n_keys {
        name = *(*namespace_ids.ptr()).set.keys.offset(__i as isize);
        xfree(name.data as *mut ::core::ffi::c_void);
        __i = __i.wrapping_add(1);
    }
    xfree((*namespace_ids.ptr()).set.keys as *mut ::core::ffi::c_void);
    xfree((*namespace_ids.ptr()).set.h.hash as *mut ::core::ffi::c_void);
    (*namespace_ids.ptr()).set = Set_String {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<String_0>(),
    };
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*namespace_ids.ptr()).values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    *ptr_;
    xfree((*namespace_localscope.ptr()).keys as *mut ::core::ffi::c_void);
    xfree((*namespace_localscope.ptr()).h.hash as *mut ::core::ffi::c_void);
    namespace_localscope.set(Set_uint32_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<uint32_t>(),
    });
}
#[no_mangle]
pub unsafe extern "C" fn nvim_create_namespace(mut name: String_0) -> Integer {
    let mut id: handle_T = map_get_String_int(namespace_ids.ptr(), name);
    if id > 0 as ::core::ffi::c_int {
        return id as Integer;
    }
    let c2rust_fresh0 = next_namespace_id.get();
    next_namespace_id.set(next_namespace_id.get() + 1);
    id = c2rust_fresh0;
    if name.size > 0 as size_t {
        let mut name_alloc: String_0 = copy_string(name, ::core::ptr::null_mut::<Arena>());
        map_put_String_int(namespace_ids.ptr(), name_alloc, id as ::core::ffi::c_int);
    }
    return id as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_namespaces(mut arena: *mut Arena) -> Dict {
    let mut retval: Dict = arena_dict(arena, (*namespace_ids.ptr()).set.h.size as size_t);
    let mut name: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut id: handle_T = 0;
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*namespace_ids.ptr()).set.h.n_keys {
        name = *(*namespace_ids.ptr()).set.keys.offset(__i as isize);
        id = *(*namespace_ids.ptr()).values.offset(__i as isize) as handle_T;
        let c2rust_fresh1 = retval.size;
        retval.size = retval.size.wrapping_add(1);
        *retval.items.offset(c2rust_fresh1 as isize) = key_value_pair {
            key: cstr_as_string(name.data),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: id as Integer,
                },
            },
        };
        __i = __i.wrapping_add(1);
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn describe_ns(
    mut ns_id: NS,
    mut unknown: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut name: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut id: handle_T = 0;
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*namespace_ids.ptr()).set.h.n_keys {
        name = *(*namespace_ids.ptr()).set.keys.offset(__i as isize);
        id = *(*namespace_ids.ptr()).values.offset(__i as isize) as handle_T;
        if id == ns_id && name.size != 0 {
            return name.data;
        }
        __i = __i.wrapping_add(1);
    }
    return unknown;
}
#[no_mangle]
pub unsafe extern "C" fn ns_initialized(mut ns: uint32_t) -> bool {
    if ns < 1 as uint32_t {
        return false_0 != 0;
    }
    return ns < next_namespace_id.get() as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn virt_text_to_array(
    mut vt: VirtText,
    mut hl_name: bool,
    mut arena: *mut Arena,
) -> Array {
    let mut chunks: Array = arena_array(arena, vt.size);
    let mut i: size_t = 0 as size_t;
    while i < vt.size {
        let mut j: size_t = i;
        while j < vt.size {
            if !(*vt.items.offset(j as isize)).text.is_null() {
                break;
            }
            j = j.wrapping_add(1);
        }
        let mut hl_array: Array = arena_array(
            arena,
            if i < j {
                j.wrapping_sub(i).wrapping_add(1 as size_t)
            } else {
                0 as size_t
            },
        );
        while i < j {
            let mut hl_id: ::core::ffi::c_int = (*vt.items.offset(i as isize)).hl_id;
            if hl_id >= 0 as ::core::ffi::c_int {
                let c2rust_fresh2 = hl_array.size;
                hl_array.size = hl_array.size.wrapping_add(1);
                *hl_array.items.offset(c2rust_fresh2 as isize) = hl_group_name(hl_id, hl_name);
            }
            i = i.wrapping_add(1);
        }
        let mut text: *mut ::core::ffi::c_char = (*vt.items.offset(i as isize)).text;
        let mut hl_id_0: ::core::ffi::c_int = (*vt.items.offset(i as isize)).hl_id;
        let mut chunk: Array = arena_array(arena, 2 as size_t);
        let c2rust_fresh3 = chunk.size;
        chunk.size = chunk.size.wrapping_add(1);
        *chunk.items.offset(c2rust_fresh3 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(text),
            },
        };
        if hl_array.size > 0 as size_t {
            if hl_id_0 >= 0 as ::core::ffi::c_int {
                let c2rust_fresh4 = hl_array.size;
                hl_array.size = hl_array.size.wrapping_add(1);
                *hl_array.items.offset(c2rust_fresh4 as isize) = hl_group_name(hl_id_0, hl_name);
            }
            let c2rust_fresh5 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh5 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: hl_array },
            };
        } else if hl_id_0 >= 0 as ::core::ffi::c_int {
            let c2rust_fresh6 = chunk.size;
            chunk.size = chunk.size.wrapping_add(1);
            *chunk.items.offset(c2rust_fresh6 as isize) = hl_group_name(hl_id_0, hl_name);
        }
        let c2rust_fresh7 = chunks.size;
        chunks.size = chunks.size.wrapping_add(1);
        *chunks.items.offset(c2rust_fresh7 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: chunk },
        };
        i = i.wrapping_add(1);
    }
    return chunks;
}
unsafe extern "C" fn extmark_to_array(
    mut extmark: MTPair,
    mut id: bool,
    mut add_dict: bool,
    mut hl_name: bool,
    mut arena: *mut Arena,
) -> Array {
    let mut start: MTKey = extmark.start;
    let mut rv: Array = arena_array(arena, 4 as size_t);
    if id {
        let c2rust_fresh8 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh8 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: start.id as Integer,
            },
        };
    }
    let c2rust_fresh9 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh9 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: start.pos.row as Integer,
        },
    };
    let c2rust_fresh10 = rv.size;
    rv.size = rv.size.wrapping_add(1);
    *rv.items.offset(c2rust_fresh10 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: start.pos.col as Integer,
        },
    };
    if add_dict {
        let mut dict: Dict = arena_dict(
            arena,
            ::core::mem::size_of::<[KeySetLink; 36]>()
                .wrapping_div(::core::mem::size_of::<KeySetLink>())
                .wrapping_div(
                    (::core::mem::size_of::<[KeySetLink; 36]>()
                        .wrapping_rem(::core::mem::size_of::<KeySetLink>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
        );
        let c2rust_fresh11 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh11 as isize) = key_value_pair {
            key: cstr_as_string(b"ns_id\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: start.ns as Integer,
                },
            },
        };
        let c2rust_fresh12 = dict.size;
        dict.size = dict.size.wrapping_add(1);
        *dict.items.offset(c2rust_fresh12 as isize) = key_value_pair {
            key: cstr_as_string(b"right_gravity\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: mt_right(start),
                },
            },
        };
        if mt_paired(start) {
            let c2rust_fresh13 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh13 as isize) = key_value_pair {
                key: cstr_as_string(b"end_row\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: extmark.end_pos.row as Integer,
                    },
                },
            };
            let c2rust_fresh14 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh14 as isize) = key_value_pair {
                key: cstr_as_string(b"end_col\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: extmark.end_pos.col as Integer,
                    },
                },
            };
            let c2rust_fresh15 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh15 as isize) = key_value_pair {
                key: cstr_as_string(b"end_right_gravity\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed {
                        boolean: extmark.end_right_gravity,
                    },
                },
            };
        }
        if mt_no_undo(start) {
            let c2rust_fresh16 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh16 as isize) = key_value_pair {
                key: cstr_as_string(b"undo_restore\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: false },
                },
            };
        }
        if mt_invalidate(start) {
            let c2rust_fresh17 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh17 as isize) = key_value_pair {
                key: cstr_as_string(b"invalidate\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        if mt_invalid(start) {
            let c2rust_fresh18 = dict.size;
            dict.size = dict.size.wrapping_add(1);
            *dict.items.offset(c2rust_fresh18 as isize) = key_value_pair {
                key: cstr_as_string(b"invalid\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        decor_to_dict_legacy(&raw mut dict, mt_decor(start), hl_name, arena);
        let c2rust_fresh19 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh19 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: dict },
        };
    }
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_extmark_by_id(
    mut buf: Buffer,
    mut ns_id: Integer,
    mut id: Integer,
    mut opts: *mut KeyDict_get_extmark,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return rv;
    }
    if !ns_initialized(ns_id as uint32_t) {
        api_err_invalid(
            err,
            b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return rv;
    }
    let mut details: bool = (*opts).details as bool;
    let mut hl_name: bool = if (*opts).is_set__get_extmark_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_extmark__hl_name
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).hl_name as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut extmark: MTPair = extmark_from_id(b, ns_id as uint32_t, id as uint32_t);
    if extmark.start.pos.row < 0 as int32_t {
        return rv;
    }
    return extmark_to_array(extmark, false_0 != 0, details, hl_name, arena);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_extmarks(
    mut buf: Buffer,
    mut ns_id: Integer,
    mut start: Object,
    mut end: Object,
    mut opts: *mut KeyDict_get_extmarks,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return rv;
    }
    if !(ns_id == -1 as Integer || ns_initialized(ns_id as uint32_t) as ::core::ffi::c_int != 0) {
        api_err_invalid(
            err,
            b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return rv;
    }
    let mut details: bool = (*opts).details as bool;
    let mut hl_name: bool = if (*opts).is_set__get_extmarks_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_extmarks__hl_name
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).hl_name as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut type_0: ExtmarkType = kExtmarkNone;
    if (*opts).is_set__get_extmarks_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_extmarks__type
        != 0 as ::core::ffi::c_ulonglong
    {
        if strequal(
            (*opts).type_0.data,
            b"sign\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            type_0 = kExtmarkSign;
        } else if strequal(
            (*opts).type_0.data,
            b"virt_text\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            type_0 = kExtmarkVirtText;
        } else if strequal(
            (*opts).type_0.data,
            b"virt_lines\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            type_0 = kExtmarkVirtLines;
        } else if strequal(
            (*opts).type_0.data,
            b"highlight\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            type_0 = kExtmarkHighlight;
        } else if true {
            api_err_exp(
                err,
                b"type\0".as_ptr() as *const ::core::ffi::c_char,
                b"sign, virt_text, virt_lines or highlight\0".as_ptr()
                    as *const ::core::ffi::c_char,
                (*opts).type_0.data,
            );
            return rv;
        }
    }
    let mut limit: Integer = if (*opts).is_set__get_extmarks_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_extmarks__limit
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).limit
    } else {
        -1 as Integer
    };
    if limit == 0 as Integer {
        return rv;
    } else if limit < 0 as Integer {
        limit = INT64_MAX as Integer;
    }
    let mut l_row: ::core::ffi::c_int = 0;
    let mut l_col: colnr_T = 0;
    if !extmark_get_index_from_obj(b, ns_id, start, &raw mut l_row, &raw mut l_col, err) {
        return rv;
    }
    let mut u_row: ::core::ffi::c_int = 0;
    let mut u_col: colnr_T = 0;
    if !extmark_get_index_from_obj(b, ns_id, end, &raw mut u_row, &raw mut u_col, err) {
        return rv;
    }
    let mut rv_limit: size_t = limit as size_t;
    let mut reverse: bool = l_row > u_row || l_row == u_row && l_col > u_col;
    if reverse {
        limit = INT64_MAX as Integer;
        let mut row: ::core::ffi::c_int = l_row;
        l_row = u_row;
        u_row = row;
        let mut col: colnr_T = l_col;
        l_col = u_col;
        u_col = col;
    }
    let mut marks: ExtmarkInfoArray = extmark_get(
        b,
        ns_id as uint32_t,
        l_row,
        l_col,
        u_row,
        u_col,
        limit,
        type_0,
        (*opts).overlap as bool,
    );
    rv = arena_array(
        arena,
        if marks.size < rv_limit {
            marks.size
        } else {
            rv_limit
        },
    );
    if reverse {
        let mut i: ::core::ffi::c_int = marks.size as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int && rv.size < rv_limit {
            let c2rust_fresh20 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh20 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed {
                    array: extmark_to_array(
                        *marks.items.offset(i as isize),
                        true,
                        details,
                        hl_name,
                        arena,
                    ),
                },
            };
            i -= 1;
        }
    } else {
        let mut i_0: size_t = 0 as size_t;
        while i_0 < marks.size {
            let c2rust_fresh21 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh21 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed {
                    array: extmark_to_array(
                        *marks.items.offset(i_0 as isize),
                        true,
                        details,
                        hl_name,
                        arena,
                    ),
                },
            };
            i_0 = i_0.wrapping_add(1);
        }
    }
    xfree(marks.items as *mut ::core::ffi::c_void);
    marks.capacity = 0 as size_t;
    marks.size = marks.capacity;
    marks.items = ::core::ptr::null_mut::<MTPair>();
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_set_extmark(
    mut buf: Buffer,
    mut ns_id: Integer,
    mut line: Integer,
    mut col: Integer,
    mut opts: *mut KeyDict_set_extmark,
    mut err: *mut Error,
) -> Integer {
    let mut id: uint32_t = 0;
    let mut line2: ::core::ffi::c_int = 0;
    let mut did_end_line: bool = false;
    let mut strict: bool = false;
    let mut col2: colnr_T = 0;
    let mut virt_lines_flags: ::core::ffi::c_int = 0;
    let mut right_gravity: bool = false;
    let mut len: colnr_T = 0;
    let mut hl: DecorHighlightInline = DECOR_HIGHLIGHT_INLINE_INIT;
    let mut sign: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
    let mut virt_text: DecorVirtText = DecorVirtText {
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
    let mut virt_lines: DecorVirtText = DecorVirtText {
        flags: kVTIsLines as ::core::ffi::c_int as uint8_t,
        hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
        priority: DECOR_PRIORITY_BASE as DecorPriority,
        width: 0 as ::core::ffi::c_int,
        col: 0 as ::core::ffi::c_int,
        pos: kVPosEndOfLine,
        data: C2Rust_Unnamed_2 {
            virt_lines: VirtLines {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<virt_line>(),
            },
        },
        next: ::core::ptr::null_mut::<DecorVirtText>(),
    };
    let mut url: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut has_hl: bool = false_0 != 0;
    let mut has_hl_multiple: bool = false_0 != 0;
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    '_error: {
        if !b.is_null() {
            if !ns_initialized(ns_id as uint32_t) {
                api_err_invalid(
                    err,
                    b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    ns_id as int64_t,
                    false_0 != 0,
                );
            } else {
                id = 0 as uint32_t;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__id
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if !((*opts).id > 0 as Integer) {
                        api_err_exp(
                            err,
                            b"id\0".as_ptr() as *const ::core::ffi::c_char,
                            b"positive Integer\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        );
                        break '_error;
                    } else {
                        id = (*opts).id as uint32_t;
                    }
                }
                line2 = -1 as ::core::ffi::c_int;
                did_end_line = false_0 != 0;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__end_line
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 10 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                            b"cannot use both 'end_row' and 'end_line'\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_error;
                    } else {
                        (*opts).end_row = (*opts).end_line;
                        did_end_line = true_0 != 0;
                    }
                }
                strict = if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__strict
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*opts).strict as ::core::ffi::c_int
                } else {
                    true_0
                } != 0;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__end_row
                    != 0 as ::core::ffi::c_ulonglong
                    || did_end_line as ::core::ffi::c_int != 0
                {
                    let mut val: Integer = (*opts).end_row;
                    if !(val >= 0 as Integer
                        && !(val > (*b).b_ml.ml_line_count as Integer
                            && strict as ::core::ffi::c_int != 0))
                    {
                        api_err_invalid(
                            err,
                            b"end_row\0".as_ptr() as *const ::core::ffi::c_char,
                            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            false_0 != 0,
                        );
                        break '_error;
                    } else {
                        line2 = val as ::core::ffi::c_int;
                    }
                }
                col2 = -1 as colnr_T;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__end_col
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut val_0: Integer = (*opts).end_col;
                    if !(val_0 >= -1 as Integer && val_0 <= MAXCOL as ::core::ffi::c_int as Integer)
                    {
                        api_err_invalid(
                            err,
                            b"end_col\0".as_ptr() as *const ::core::ffi::c_char,
                            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            false_0 != 0,
                        );
                        break '_error;
                    } else {
                        if val_0 == -1 as Integer {
                            val_0 = MAXCOL as ::core::ffi::c_int as Integer;
                        }
                        col2 = val_0 as ::core::ffi::c_int as colnr_T;
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__hl_group
                    != 0 as ::core::ffi::c_ulonglong
                {
                    's_293: {
                        if (*opts).hl_group.type_0 as ::core::ffi::c_uint
                            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            let mut arr: Array = (*opts).hl_group.data.array;
                            if arr.size >= 1 as size_t {
                                hl.hl_id = object_to_hl_id(
                                    *arr.items.offset(0 as ::core::ffi::c_int as isize),
                                    b"hl_group item\0".as_ptr() as *const ::core::ffi::c_char,
                                    err,
                                );
                                if (*err).type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    break '_error;
                                }
                            }
                            let mut i: size_t = 1 as size_t;
                            loop {
                                if i >= arr.size {
                                    break 's_293;
                                }
                                let mut hl_id: ::core::ffi::c_int = object_to_hl_id(
                                    *arr.items.offset(i as isize),
                                    b"hl_group item\0".as_ptr() as *const ::core::ffi::c_char,
                                    err,
                                );
                                if (*err).type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    break '_error;
                                }
                                if hl_id != 0 {
                                    has_hl_multiple = true_0 != 0;
                                }
                                i = i.wrapping_add(1);
                            }
                        } else {
                            hl.hl_id = object_to_hl_id(
                                (*opts).hl_group,
                                b"hl_group\0".as_ptr() as *const ::core::ffi::c_char,
                                err,
                            );
                            if (*err).type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                break '_error;
                            }
                        }
                    }
                    has_hl = hl.hl_id > 0 as ::core::ffi::c_int;
                }
                sign.hl_id = (*opts).sign_hl_group as ::core::ffi::c_int;
                sign.cursorline_hl_id = (*opts).cursorline_hl_group as ::core::ffi::c_int;
                sign.number_hl_id = (*opts).number_hl_group as ::core::ffi::c_int;
                sign.line_hl_id = (*opts).line_hl_group as ::core::ffi::c_int;
                if sign.hl_id != 0
                    || sign.cursorline_hl_id != 0
                    || sign.number_hl_id != 0
                    || sign.line_hl_id != 0
                {
                    sign.flags = (sign.flags as ::core::ffi::c_int
                        | kSHIsSign as ::core::ffi::c_int)
                        as uint16_t;
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__conceal
                    != 0 as ::core::ffi::c_ulonglong
                {
                    hl.flags = (hl.flags as ::core::ffi::c_int | kSHConceal as ::core::ffi::c_int)
                        as uint16_t;
                    has_hl = true_0 != 0;
                    if (*opts).conceal.size > 0 as size_t {
                        let mut ch: ::core::ffi::c_int = 0;
                        hl.conceal_char = utfc_ptr2schar((*opts).conceal.data, &raw mut ch);
                        if !(hl.conceal_char != 0 && vim_isprintc(ch) as ::core::ffi::c_int != 0) {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                b"conceal char has to be printable\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_error;
                        }
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__conceal_lines
                    != 0 as ::core::ffi::c_ulonglong
                {
                    hl.flags = (hl.flags as ::core::ffi::c_int
                        | kSHConcealLines as ::core::ffi::c_int)
                        as uint16_t;
                    has_hl = true_0 != 0;
                    if (*opts).conceal_lines.size > 0 as size_t {
                        if !(*(*opts).conceal_lines.data as ::core::ffi::c_int
                            == '\0' as ::core::ffi::c_int)
                        {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                b"conceal_lines has to be an empty string\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_error;
                        }
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__virt_text
                    != 0 as ::core::ffi::c_ulonglong
                {
                    virt_text.data.virt_text =
                        parse_virt_text((*opts).virt_text, err, &raw mut virt_text.width);
                    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        break '_error;
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__virt_text_pos
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut str: String_0 = (*opts).virt_text_pos;
                    if strequal(b"eol\0".as_ptr() as *const ::core::ffi::c_char, str.data) {
                        virt_text.pos = kVPosEndOfLine;
                    } else if strequal(
                        b"overlay\0".as_ptr() as *const ::core::ffi::c_char,
                        str.data,
                    ) {
                        virt_text.pos = kVPosOverlay;
                    } else if strequal(
                        b"right_align\0".as_ptr() as *const ::core::ffi::c_char,
                        str.data,
                    ) {
                        virt_text.pos = kVPosRightAlign;
                    } else if strequal(
                        b"eol_right_align\0".as_ptr() as *const ::core::ffi::c_char,
                        str.data,
                    ) {
                        virt_text.pos = kVPosEndOfLineRightAlign;
                    } else if strequal(b"inline\0".as_ptr() as *const ::core::ffi::c_char, str.data)
                    {
                        virt_text.pos = kVPosInline;
                    } else if true {
                        api_err_invalid(
                            err,
                            b"virt_text_pos\0".as_ptr() as *const ::core::ffi::c_char,
                            str.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_error;
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong)
                        << KEYSET_OPTIDX_set_extmark__virt_text_win_col
                    != 0 as ::core::ffi::c_ulonglong
                {
                    virt_text.col = (*opts).virt_text_win_col as ::core::ffi::c_int;
                    virt_text.pos = kVPosWinCol;
                }
                hl.flags = (hl.flags as ::core::ffi::c_int
                    | if (*opts).hl_eol as ::core::ffi::c_int != 0 {
                        kSHHlEol as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as uint16_t;
                virt_text.flags = (virt_text.flags as ::core::ffi::c_int
                    | ((if (*opts).virt_text_hide as ::core::ffi::c_int != 0 {
                        kVTHide as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) | (if (*opts).virt_text_repeat_linebreak as ::core::ffi::c_int != 0 {
                        kVTRepeatLinebreak as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }))) as uint8_t;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__hl_mode
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut str_0: String_0 = (*opts).hl_mode;
                    if strequal(
                        b"replace\0".as_ptr() as *const ::core::ffi::c_char,
                        str_0.data,
                    ) {
                        virt_text.hl_mode = kHlModeReplace as ::core::ffi::c_int as uint8_t;
                    } else if strequal(
                        b"combine\0".as_ptr() as *const ::core::ffi::c_char,
                        str_0.data,
                    ) {
                        virt_text.hl_mode = kHlModeCombine as ::core::ffi::c_int as uint8_t;
                    } else if strequal(
                        b"blend\0".as_ptr() as *const ::core::ffi::c_char,
                        str_0.data,
                    ) {
                        if virt_text.pos as ::core::ffi::c_uint
                            == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if true {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"cannot use 'blend' hl_mode with inline virtual text\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                                break '_error;
                            }
                        }
                        virt_text.hl_mode = kHlModeBlend as ::core::ffi::c_int as uint8_t;
                    } else if true {
                        api_err_invalid(
                            err,
                            b"hl_mode\0".as_ptr() as *const ::core::ffi::c_char,
                            str_0.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_error;
                    }
                }
                virt_lines_flags = if (*opts).virt_lines_leftcol as ::core::ffi::c_int != 0 {
                    kVLLeftcol as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong)
                        << KEYSET_OPTIDX_set_extmark__virt_lines_overflow
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut str_1: String_0 = (*opts).virt_lines_overflow;
                    if strequal(
                        b"scroll\0".as_ptr() as *const ::core::ffi::c_char,
                        str_1.data,
                    ) {
                        virt_lines_flags |= kVLScroll as ::core::ffi::c_int;
                    } else if !strequal(
                        b"trunc\0".as_ptr() as *const ::core::ffi::c_char,
                        str_1.data,
                    ) {
                        if true {
                            api_err_invalid(
                                err,
                                b"virt_lines_overflow\0".as_ptr() as *const ::core::ffi::c_char,
                                str_1.data,
                                0 as int64_t,
                                true_0 != 0,
                            );
                            break '_error;
                        }
                    }
                }
                's_785: {
                    if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__virt_lines
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        let mut a: Array = (*opts).virt_lines;
                        let mut j: size_t = 0 as size_t;
                        loop {
                            if j >= a.size {
                                break 's_785;
                            }
                            if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                                != (*a.items.offset(j as isize)).type_0 as ::core::ffi::c_uint
                            {
                                api_err_exp(
                                    err,
                                    b"virt_text_line\0".as_ptr() as *const ::core::ffi::c_char,
                                    api_typename(kObjectTypeArray),
                                    api_typename((*a.items.offset(j as isize)).type_0),
                                );
                                break '_error;
                            } else {
                                let mut dummig: ::core::ffi::c_int = 0;
                                let mut jtem: VirtText = parse_virt_text(
                                    (*a.items.offset(j as isize)).data.array,
                                    err,
                                    &raw mut dummig,
                                );
                                if virt_lines.data.virt_lines.size
                                    == virt_lines.data.virt_lines.capacity
                                {
                                    virt_lines.data.virt_lines.capacity =
                                        if virt_lines.data.virt_lines.capacity != 0 {
                                            virt_lines.data.virt_lines.capacity
                                                << 1 as ::core::ffi::c_int
                                        } else {
                                            8 as size_t
                                        };
                                    virt_lines.data.virt_lines.items = xrealloc(
                                        virt_lines.data.virt_lines.items
                                            as *mut ::core::ffi::c_void,
                                        ::core::mem::size_of::<virt_line>()
                                            .wrapping_mul(virt_lines.data.virt_lines.capacity),
                                    )
                                        as *mut virt_line;
                                } else {
                                };
                                let c2rust_fresh22 = virt_lines.data.virt_lines.size;
                                virt_lines.data.virt_lines.size =
                                    virt_lines.data.virt_lines.size.wrapping_add(1);
                                *virt_lines
                                    .data
                                    .virt_lines
                                    .items
                                    .offset(c2rust_fresh22 as isize) = virt_line {
                                    line: jtem,
                                    flags: virt_lines_flags,
                                }
                                    as virt_line;
                                if (*err).type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    break '_error;
                                }
                                j = j.wrapping_add(1);
                            }
                        }
                    }
                }
                virt_lines.flags = (virt_lines.flags as ::core::ffi::c_int
                    | if (*opts).virt_lines_above as ::core::ffi::c_int != 0 {
                        kVTLinesAbove as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as uint8_t;
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__priority
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if !((*opts).priority >= 0 as Integer && (*opts).priority <= 65535 as Integer) {
                        api_err_invalid(
                            err,
                            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
                            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            false_0 != 0,
                        );
                        break '_error;
                    } else {
                        hl.priority = (*opts).priority as DecorPriority;
                        sign.priority = (*opts).priority as DecorPriority;
                        virt_text.priority = (*opts).priority as DecorPriority;
                        virt_lines.priority = (*opts).priority as DecorPriority;
                    }
                }
                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__sign_text
                    != 0 as ::core::ffi::c_ulonglong
                {
                    sign.text[0 as ::core::ffi::c_int as usize] = 0 as schar_T;
                    if init_sign_text(
                        ::core::ptr::null_mut::<sign_T>(),
                        &raw mut sign.text as *mut schar_T,
                        (*opts).sign_text.data,
                    ) == 0
                    {
                        api_err_invalid(
                            err,
                            b"sign_text\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_error;
                    } else {
                        sign.flags = (sign.flags as ::core::ffi::c_int
                            | kSHIsSign as ::core::ffi::c_int)
                            as uint16_t;
                    }
                }
                right_gravity = if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__right_gravity
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*opts).right_gravity as ::core::ffi::c_int
                } else {
                    true_0
                } != 0;
                if line2 == -1 as ::core::ffi::c_int
                    && col2 == -1 as ::core::ffi::c_int
                    && (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 30 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"cannot set end_right_gravity without end_row or end_col\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                } else {
                    len = 0 as colnr_T;
                    if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__spell
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        hl.flags = (hl.flags as ::core::ffi::c_int
                            | if (*opts).spell as ::core::ffi::c_int != 0 {
                                kSHSpellOn as ::core::ffi::c_int
                            } else {
                                kSHSpellOff as ::core::ffi::c_int
                            }) as uint16_t;
                        has_hl = true_0 != 0;
                    }
                    if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_set_extmark__url
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        url = string_to_cstr((*opts).url);
                        has_hl = true_0 != 0;
                    }
                    if (*opts).ui_watched {
                        hl.flags = (hl.flags as ::core::ffi::c_int
                            | kSHUIWatched as ::core::ffi::c_int)
                            as uint16_t;
                        if virt_text.pos as ::core::ffi::c_uint
                            == kVPosOverlay as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            hl.flags = (hl.flags as ::core::ffi::c_int
                                | kSHUIWatchedOverlay as ::core::ffi::c_int)
                                as uint16_t;
                        }
                        has_hl = true_0 != 0;
                    }
                    if !(line >= 0 as Integer) {
                        api_err_invalid(
                            err,
                            b"line\0".as_ptr() as *const ::core::ffi::c_char,
                            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            false_0 != 0,
                        );
                    } else {
                        if line > (*b).b_ml.ml_line_count as Integer {
                            if strict {
                                api_err_invalid(
                                    err,
                                    b"line\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    false_0 != 0,
                                );
                                break '_error;
                            } else {
                                line = (*b).b_ml.ml_line_count as Integer;
                            }
                        } else if line < (*b).b_ml.ml_line_count as Integer {
                            len = (if (*opts).ephemeral as ::core::ffi::c_int != 0 {
                                MAXCOL as ::core::ffi::c_int
                            } else {
                                ml_get_buf_len(b, line as linenr_T + 1 as linenr_T)
                            }) as colnr_T;
                        }
                        if col == -1 as Integer {
                            col = len as Integer;
                        } else if col > len as Integer {
                            if strict {
                                api_err_invalid(
                                    err,
                                    b"col\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    false_0 != 0,
                                );
                                break '_error;
                            } else {
                                col = len as Integer;
                            }
                        } else if col < -1 as Integer {
                            if true {
                                api_err_invalid(
                                    err,
                                    b"col\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    false_0 != 0,
                                );
                                break '_error;
                            }
                        }
                        if col2 >= 0 as ::core::ffi::c_int {
                            if line2 >= 0 as ::core::ffi::c_int
                                && (line2 as linenr_T) < (*b).b_ml.ml_line_count
                            {
                                len = (if (*opts).ephemeral as ::core::ffi::c_int != 0 {
                                    MAXCOL as ::core::ffi::c_int
                                } else {
                                    ml_get_buf_len(b, line2 as linenr_T + 1 as linenr_T)
                                }) as colnr_T;
                            } else if line2 as linenr_T == (*b).b_ml.ml_line_count {
                                len = 0 as ::core::ffi::c_int as colnr_T;
                            } else {
                                line2 = line as ::core::ffi::c_int;
                            }
                            if col2 > len {
                                if strict {
                                    api_err_invalid(
                                        err,
                                        b"end_col\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                                        0 as int64_t,
                                        false_0 != 0,
                                    );
                                    break '_error;
                                } else {
                                    col2 = len;
                                }
                            }
                        } else if line2 >= 0 as ::core::ffi::c_int {
                            col2 = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        if (*opts).ephemeral as ::core::ffi::c_int != 0
                            && !(*decor_state.ptr()).win.is_null()
                            && (*(*decor_state.ptr()).win).w_buffer == b
                        {
                            let mut r: ::core::ffi::c_int = line as ::core::ffi::c_int;
                            let mut c: ::core::ffi::c_int = col as ::core::ffi::c_int;
                            if line2 == -1 as ::core::ffi::c_int {
                                line2 = r;
                                col2 = c as colnr_T;
                            }
                            let mut subpriority: DecorPriority = 0 as DecorPriority;
                            if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                                & (1 as ::core::ffi::c_ulonglong)
                                    << KEYSET_OPTIDX_set_extmark___subpriority
                                != 0 as ::core::ffi::c_ulonglong
                            {
                                if !((*opts)._subpriority >= 0 as Integer
                                    && (*opts)._subpriority <= 65535 as Integer)
                                {
                                    api_err_invalid(
                                        err,
                                        b"_subpriority\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                                        0 as int64_t,
                                        false_0 != 0,
                                    );
                                    break '_error;
                                } else {
                                    subpriority = (*opts)._subpriority as DecorPriority;
                                }
                            }
                            if virt_text.data.virt_text.size != 0 {
                                decor_range_add_virt(
                                    decor_state.ptr(),
                                    r,
                                    c,
                                    line2,
                                    col2 as ::core::ffi::c_int,
                                    decor_put_vt(
                                        virt_text,
                                        ::core::ptr::null_mut::<DecorVirtText>(),
                                    ),
                                    true_0 != 0,
                                );
                            }
                            if virt_lines.data.virt_lines.size != 0 {
                                decor_range_add_virt(
                                    decor_state.ptr(),
                                    r,
                                    c,
                                    line2,
                                    col2 as ::core::ffi::c_int,
                                    decor_put_vt(
                                        virt_lines,
                                        ::core::ptr::null_mut::<DecorVirtText>(),
                                    ),
                                    true_0 != 0,
                                );
                            }
                            if has_hl {
                                let mut sh: DecorSignHighlight = decor_sh_from_inline(hl);
                                sh.url = url;
                                decor_range_add_sh(
                                    decor_state.ptr(),
                                    r,
                                    c,
                                    line2,
                                    col2 as ::core::ffi::c_int,
                                    &raw mut sh,
                                    true_0 != 0,
                                    ns_id as uint32_t,
                                    id,
                                    subpriority,
                                );
                            }
                        } else if (*opts).ephemeral {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                b"cannot set emphemeral mark outside of a decoration provider\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_error;
                        } else {
                            let mut decor_flags: uint16_t = 0 as uint16_t;
                            let mut decor_alloc: *mut DecorVirtText =
                                ::core::ptr::null_mut::<DecorVirtText>();
                            if virt_text.data.virt_text.size != 0 {
                                decor_alloc = decor_put_vt(virt_text, decor_alloc);
                                if virt_text.pos as ::core::ffi::c_uint
                                    == kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    decor_flags = (decor_flags as ::core::ffi::c_int
                                        | MT_FLAG_DECOR_VIRT_TEXT_INLINE)
                                        as uint16_t;
                                }
                            }
                            if virt_lines.data.virt_lines.size != 0 {
                                decor_alloc = decor_put_vt(virt_lines, decor_alloc);
                                decor_flags = (decor_flags as ::core::ffi::c_int
                                    | MT_FLAG_DECOR_VIRT_LINES)
                                    as uint16_t;
                            }
                            let mut decor_indexed: uint32_t = DECOR_ID_INVALID as uint32_t;
                            if sign.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int
                                != 0
                            {
                                sign.next = decor_indexed;
                                decor_indexed = decor_put_sh(sign);
                                if sign.text[0 as ::core::ffi::c_int as usize] != 0 {
                                    decor_flags = (decor_flags as ::core::ffi::c_int
                                        | MT_FLAG_DECOR_SIGNTEXT)
                                        as uint16_t;
                                }
                                if sign.number_hl_id != 0
                                    || sign.line_hl_id != 0
                                    || sign.cursorline_hl_id != 0
                                {
                                    decor_flags = (decor_flags as ::core::ffi::c_int
                                        | MT_FLAG_DECOR_SIGNHL)
                                        as uint16_t;
                                }
                            }
                            if has_hl_multiple {
                                let mut arr_0: Array = (*opts).hl_group.data.array;
                                let mut i_0: size_t = arr_0.size.wrapping_sub(1 as size_t);
                                while i_0 > 0 as size_t {
                                    let mut hl_id_0: ::core::ffi::c_int = object_to_hl_id(
                                        *arr_0.items.offset(i_0 as isize),
                                        b"hl_group item\0".as_ptr() as *const ::core::ffi::c_char,
                                        err,
                                    );
                                    if hl_id_0 > 0 as ::core::ffi::c_int {
                                        let mut sh_0: DecorSignHighlight =
                                            DECOR_SIGN_HIGHLIGHT_INIT;
                                        sh_0.hl_id = hl_id_0;
                                        sh_0.flags = (if (*opts).hl_eol as ::core::ffi::c_int != 0 {
                                            kSHHlEol as ::core::ffi::c_int
                                        } else {
                                            0 as ::core::ffi::c_int
                                        })
                                            as uint16_t;
                                        sh_0.next = decor_indexed;
                                        decor_indexed = decor_put_sh(sh_0);
                                        decor_flags = (decor_flags as ::core::ffi::c_int
                                            | MT_FLAG_DECOR_HL)
                                            as uint16_t;
                                    }
                                    i_0 = i_0.wrapping_sub(1);
                                }
                            }
                            if hl.flags as ::core::ffi::c_int
                                & kSHConcealLines as ::core::ffi::c_int
                                != 0
                            {
                                decor_flags = (decor_flags as ::core::ffi::c_int
                                    | MT_FLAG_DECOR_CONCEAL_LINES)
                                    as uint16_t;
                            }
                            let mut decor: DecorInline = DECOR_INLINE_INIT;
                            if !decor_alloc.is_null()
                                || decor_indexed != DECOR_ID_INVALID as uint32_t
                                || !url.is_null()
                                || schar_high(hl.conceal_char) as ::core::ffi::c_int != 0
                            {
                                if has_hl {
                                    let mut sh_1: DecorSignHighlight = decor_sh_from_inline(hl);
                                    sh_1.url = url;
                                    sh_1.next = decor_indexed;
                                    decor_indexed = decor_put_sh(sh_1);
                                }
                                decor.ext = true_0 != 0;
                                decor.data.ext = DecorExt {
                                    sh_idx: decor_indexed,
                                    vt: decor_alloc,
                                };
                            } else {
                                decor.data.hl = hl;
                            }
                            if has_hl {
                                decor_flags = (decor_flags as ::core::ffi::c_int | MT_FLAG_DECOR_HL)
                                    as uint16_t;
                            }
                            extmark_set(
                                b,
                                ns_id as uint32_t,
                                &raw mut id,
                                line as ::core::ffi::c_int,
                                col as colnr_T,
                                line2,
                                col2,
                                decor,
                                decor_flags,
                                right_gravity,
                                (*opts).end_right_gravity as bool,
                                if (*opts).is_set__set_extmark_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_set_extmark__undo_restore
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*opts).undo_restore as ::core::ffi::c_int
                                } else {
                                    true_0
                                } == 0,
                                (*opts).invalidate as bool,
                                err,
                            );
                            if (*err).type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                decor_free(decor);
                                return 0 as Integer;
                            }
                        }
                        return id as Integer;
                    }
                }
            }
        }
    }
    clear_virttext(&raw mut virt_text.data.virt_text);
    clear_virtlines(&raw mut virt_lines.data.virt_lines);
    if !url.is_null() {
        xfree(url as *mut ::core::ffi::c_void);
    }
    return 0 as Integer;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_del_extmark(
    mut buf: Buffer,
    mut ns_id: Integer,
    mut id: Integer,
    mut err: *mut Error,
) -> Boolean {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return false_0 != 0;
    }
    if !ns_initialized(ns_id as uint32_t) {
        api_err_invalid(
            err,
            b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return false;
    }
    return extmark_del_id(b, ns_id as uint32_t, id as uint32_t);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_clear_namespace(
    mut buf: Buffer,
    mut ns_id: Integer,
    mut line_start: Integer,
    mut line_end: Integer,
    mut err: *mut Error,
) {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return;
    }
    if !(line_start >= 0 as Integer && line_start < MAXLNUM as ::core::ffi::c_int as Integer) {
        api_err_invalid(
            err,
            b"line number\0".as_ptr() as *const ::core::ffi::c_char,
            b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
            0 as int64_t,
            false_0 != 0,
        );
        return;
    }
    if line_end < 0 as Integer || line_end > MAXLNUM as ::core::ffi::c_int as Integer {
        line_end = MAXLNUM as ::core::ffi::c_int as Integer;
    }
    extmark_clear(
        b,
        if ns_id < 0 as Integer {
            0 as uint32_t
        } else {
            ns_id as uint32_t
        },
        line_start as ::core::ffi::c_int,
        0 as colnr_T,
        line_end as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        MAXCOL as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim_set_decoration_provider(
    mut ns_id: Integer,
    mut opts: *mut KeyDict_set_decoration_provider,
    mut _err: *mut Error,
) {
    let mut p: *mut DecorProvider = get_decor_provider(ns_id as NS, true_0 != 0);
    '_c2rust_label: {
        if !p.is_null() {
        } else {
            __assert_fail(
                b"p != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/extmark.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1083 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    decor_provider_clear(p);
    redraw_all_later(UPD_NOT_VALID as ::core::ffi::c_int);
    let mut cbs: [C2Rust_Unnamed_26; 10] = [
        C2Rust_Unnamed_26 {
            name: b"on_start\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_start,
            dest: &raw mut (*p).redraw_start,
        },
        C2Rust_Unnamed_26 {
            name: b"on_buf\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_buf,
            dest: &raw mut (*p).redraw_buf,
        },
        C2Rust_Unnamed_26 {
            name: b"on_win\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_win,
            dest: &raw mut (*p).redraw_win,
        },
        C2Rust_Unnamed_26 {
            name: b"on_line\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_line,
            dest: &raw mut (*p).redraw_line,
        },
        C2Rust_Unnamed_26 {
            name: b"on_range\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_range,
            dest: &raw mut (*p).redraw_range,
        },
        C2Rust_Unnamed_26 {
            name: b"on_end\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts).on_end,
            dest: &raw mut (*p).redraw_end,
        },
        C2Rust_Unnamed_26 {
            name: b"_on_hl_def\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts)._on_hl_def,
            dest: &raw mut (*p).hl_def,
        },
        C2Rust_Unnamed_26 {
            name: b"_on_spell_nav\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts)._on_spell_nav,
            dest: &raw mut (*p).spell_nav,
        },
        C2Rust_Unnamed_26 {
            name: b"_on_conceal_line\0".as_ptr() as *const ::core::ffi::c_char,
            source: &raw mut (*opts)._on_conceal_line,
            dest: &raw mut (*p).conceal_line,
        },
        C2Rust_Unnamed_26 {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            source: ::core::ptr::null_mut::<LuaRef>(),
            dest: ::core::ptr::null_mut::<LuaRef>(),
        },
    ];
    let mut i: size_t = 0 as size_t;
    while !cbs[i as usize].source.is_null()
        && !cbs[i as usize].dest.is_null()
        && !cbs[i as usize].name.is_null()
    {
        let mut v: *mut LuaRef = cbs[i as usize].source;
        if *v > 0 as ::core::ffi::c_int {
            *cbs[i as usize].dest = *v;
            *v = LUA_NOREF as LuaRef;
        }
        i = i.wrapping_add(1);
    }
    (*p).state = kDecorProviderActive;
    (*p).hl_valid += 1;
    (*p).hl_cached = false_0 != 0;
}
unsafe extern "C" fn extmark_get_index_from_obj(
    mut buf: *mut buf_T,
    mut ns_id: Integer,
    mut obj: Object,
    mut row: *mut ::core::ffi::c_int,
    mut col: *mut colnr_T,
    mut err: *mut Error,
) -> bool {
    if obj.type_0 as ::core::ffi::c_uint
        == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut id: Integer = obj.data.integer;
        if id == 0 as Integer {
            *row = 0 as ::core::ffi::c_int;
            *col = 0 as ::core::ffi::c_int as colnr_T;
            return true_0 != 0;
        } else if id == -1 as Integer {
            *row = MAXLNUM as ::core::ffi::c_int;
            *col = MAXCOL as ::core::ffi::c_int as colnr_T;
            return true_0 != 0;
        } else if id < 0 as Integer {
            if true {
                api_err_invalid(
                    err,
                    b"mark id\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    id as int64_t,
                    false_0 != 0,
                );
                return false;
            }
        }
        let mut extmark: MTPair = extmark_from_id(buf, ns_id as uint32_t, id as uint32_t);
        if !(extmark.start.pos.row >= 0 as int32_t) {
            api_err_invalid(
                err,
                b"mark id (not found)\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
                id as int64_t,
                false_0 != 0,
            );
            return false;
        }
        *row = extmark.start.pos.row as ::core::ffi::c_int;
        *col = extmark.start.pos.col as colnr_T;
        return true_0 != 0;
    } else if obj.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut pos: Array = obj.data.array;
        if !(pos.size == 2 as size_t
            && (*pos.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*pos.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            api_err_exp(
                err,
                b"mark position\0".as_ptr() as *const ::core::ffi::c_char,
                b"2 Integer items\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return false;
        }
        let mut pos_row: Integer = (*pos.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .integer;
        let mut pos_col: Integer = (*pos.items.offset(1 as ::core::ffi::c_int as isize))
            .data
            .integer;
        *row = (if pos_row >= 0 as Integer {
            pos_row
        } else {
            MAXLNUM as ::core::ffi::c_int as Integer
        }) as ::core::ffi::c_int;
        *col = (if pos_col >= 0 as Integer {
            pos_col
        } else {
            MAXCOL as ::core::ffi::c_int as Integer
        }) as colnr_T;
        return true_0 != 0;
    } else if true {
        api_err_exp(
            err,
            b"mark position\0".as_ptr() as *const ::core::ffi::c_char,
            b"mark id Integer or 2-item Array\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        return false;
    }
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn parse_virt_text(
    mut chunks: Array,
    mut err: *mut Error,
    mut width: *mut ::core::ffi::c_int,
) -> VirtText {
    let mut virt_text: VirtText = VirtText {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    };
    let mut w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    '_free_exit: {
        while i < chunks.size {
            if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                != (*chunks.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
            {
                api_err_exp(
                    err,
                    b"chunk\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(kObjectTypeArray),
                    api_typename((*chunks.items.offset(i as isize)).type_0),
                );
                break '_free_exit;
            } else {
                let mut chunk: Array = (*chunks.items.offset(i as isize)).data.array;
                if !(chunk.size > 0 as size_t
                    && chunk.size <= 2 as size_t
                    && (*chunk.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                        as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"Invalid chunk: expected Array with 1 or 2 Strings\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                    break '_free_exit;
                } else {
                    let mut str: String_0 = (*chunk.items.offset(0 as ::core::ffi::c_int as isize))
                        .data
                        .string;
                    let mut hl_id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                    's_146: {
                        if chunk.size == 2 as size_t {
                            let mut hl: Object =
                                *chunk.items.offset(1 as ::core::ffi::c_int as isize);
                            if hl.type_0 as ::core::ffi::c_uint
                                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                let mut arr: Array = hl.data.array;
                                let mut j: size_t = 0 as size_t;
                                loop {
                                    if j >= arr.size {
                                        break 's_146;
                                    }
                                    hl_id = object_to_hl_id(
                                        *arr.items.offset(j as isize),
                                        b"virt_text highlight\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        err,
                                    );
                                    if (*err).type_0 as ::core::ffi::c_int
                                        != kErrorTypeNone as ::core::ffi::c_int
                                    {
                                        break '_free_exit;
                                    }
                                    if j < arr.size.wrapping_sub(1 as size_t) {
                                        if virt_text.size == virt_text.capacity {
                                            virt_text.capacity = if virt_text.capacity != 0 {
                                                virt_text.capacity << 1 as ::core::ffi::c_int
                                            } else {
                                                8 as size_t
                                            };
                                            virt_text.items = xrealloc(
                                                virt_text.items as *mut ::core::ffi::c_void,
                                                ::core::mem::size_of::<VirtTextChunk>()
                                                    .wrapping_mul(virt_text.capacity),
                                            )
                                                as *mut VirtTextChunk;
                                        } else {
                                        };
                                        let c2rust_fresh23 = virt_text.size;
                                        virt_text.size = virt_text.size.wrapping_add(1);
                                        *virt_text.items.offset(c2rust_fresh23 as isize) =
                                            VirtTextChunk {
                                                text: ::core::ptr::null_mut::<::core::ffi::c_char>(
                                                ),
                                                hl_id: hl_id,
                                            };
                                    }
                                    j = j.wrapping_add(1);
                                }
                            } else {
                                hl_id = object_to_hl_id(
                                    hl,
                                    b"virt_text highlight\0".as_ptr() as *const ::core::ffi::c_char,
                                    err,
                                );
                                if (*err).type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    break '_free_exit;
                                }
                            }
                        }
                    }
                    let mut text: *mut ::core::ffi::c_char = transstr(
                        if str.size > 0 as size_t {
                            str.data as *const ::core::ffi::c_char
                        } else {
                            b"\0".as_ptr() as *const ::core::ffi::c_char
                        },
                        false_0 != 0,
                    );
                    w += mb_string2cells(text) as ::core::ffi::c_int;
                    if virt_text.size == virt_text.capacity {
                        virt_text.capacity = if virt_text.capacity != 0 {
                            virt_text.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        virt_text.items = xrealloc(
                            virt_text.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<VirtTextChunk>()
                                .wrapping_mul(virt_text.capacity),
                        ) as *mut VirtTextChunk;
                    } else {
                    };
                    let c2rust_fresh24 = virt_text.size;
                    virt_text.size = virt_text.size.wrapping_add(1);
                    *virt_text.items.offset(c2rust_fresh24 as isize) = VirtTextChunk {
                        text: text,
                        hl_id: hl_id,
                    };
                    i = i.wrapping_add(1);
                }
            }
        }
        if !width.is_null() {
            *width = w;
        }
        return virt_text;
    }
    clear_virttext(&raw mut virt_text);
    return virt_text;
}
#[no_mangle]
pub unsafe extern "C" fn nvim__buf_debug_extmarks(
    mut buf: Buffer,
    mut keys: Boolean,
    mut dot: Boolean,
    mut err: *mut Error,
) -> String_0 {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return NULL_STRING;
    }
    return mt_inspect(
        &raw mut (*b).b_marktree as *mut MarkTree,
        keys as bool,
        dot as bool,
    );
}
#[no_mangle]
pub unsafe extern "C" fn nvim__ns_set(
    mut ns_id: Integer,
    mut opts: *mut KeyDict_ns_opts,
    mut err: *mut Error,
) {
    if !ns_initialized(ns_id as uint32_t) {
        api_err_invalid(
            err,
            b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return;
    }
    let mut set_scoped: bool = true_0 != 0;
    if (*opts).is_set__ns_opts_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_ns_opts__wins
        != 0 as ::core::ffi::c_ulonglong
    {
        if (*opts).wins.size == 0 as size_t {
            set_scoped = false_0 != 0;
        }
        let mut windows: Set_ptr_t = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        };
        let mut i: size_t = 0 as size_t;
        while i < (*opts).wins.size {
            let mut win: Integer = (*(*opts).wins.items.offset(i as isize)).data.integer;
            let mut wp: *mut win_T = find_window_by_handle(win as Window, err);
            if wp.is_null() {
                return;
            }
            set_put_ptr_t(
                &raw mut windows,
                wp as ptr_t,
                ::core::ptr::null_mut::<*mut ptr_t>(),
            );
            i = i.wrapping_add(1);
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp_0: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp_0.is_null() {
                if set_has_ptr_t(&raw mut windows, wp_0 as ptr_t) as ::core::ffi::c_int != 0
                    && !set_has_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t)
                {
                    set_put_uint32_t(
                        &raw mut (*wp_0).w_ns_set,
                        ns_id as uint32_t,
                        ::core::ptr::null_mut::<*mut uint32_t>(),
                    );
                    if set_has_uint32_t(
                        &raw mut (*(&raw mut (*(*wp_0).w_buffer).b_extmark_ns
                            as *mut Map_uint32_t_uint32_t))
                            .set,
                        ns_id as uint32_t,
                    ) {
                        changed_window_setting(wp_0);
                    }
                }
                if set_has_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t)
                    as ::core::ffi::c_int
                    != 0
                    && !set_has_ptr_t(&raw mut windows, wp_0 as ptr_t)
                {
                    set_del_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t);
                    if set_has_uint32_t(
                        &raw mut (*(&raw mut (*(*wp_0).w_buffer).b_extmark_ns
                            as *mut Map_uint32_t_uint32_t))
                            .set,
                        ns_id as uint32_t,
                    ) {
                        changed_window_setting(wp_0);
                    }
                }
                wp_0 = (*wp_0).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        xfree(windows.keys as *mut ::core::ffi::c_void);
        xfree(windows.h.hash as *mut ::core::ffi::c_void);
        windows = Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        };
    }
    if set_scoped as ::core::ffi::c_int != 0
        && !set_has_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t)
    {
        set_put_uint32_t(
            namespace_localscope.ptr(),
            ns_id as uint32_t,
            ::core::ptr::null_mut::<*mut uint32_t>(),
        );
        let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp_0.is_null() {
            let mut wp_1: *mut win_T = if tp_0 == curtab.get() {
                firstwin.get()
            } else {
                (*tp_0).tp_firstwin
            };
            while !wp_1.is_null() {
                if set_has_uint32_t(
                    &raw mut (*(&raw mut (*(*wp_1).w_buffer).b_extmark_ns
                        as *mut Map_uint32_t_uint32_t))
                        .set,
                    ns_id as uint32_t,
                ) {
                    changed_window_setting(wp_1);
                }
                wp_1 = (*wp_1).w_next;
            }
            tp_0 = (*tp_0).tp_next as *mut tabpage_T;
        }
    } else if !set_scoped
        && set_has_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t) as ::core::ffi::c_int
            != 0
    {
        set_del_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t);
        let mut tp_1: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp_1.is_null() {
            let mut wp_2: *mut win_T = if tp_1 == curtab.get() {
                firstwin.get()
            } else {
                (*tp_1).tp_firstwin
            };
            while !wp_2.is_null() {
                if set_has_uint32_t(
                    &raw mut (*(&raw mut (*(*wp_2).w_buffer).b_extmark_ns
                        as *mut Map_uint32_t_uint32_t))
                        .set,
                    ns_id as uint32_t,
                ) {
                    changed_window_setting(wp_2);
                }
                wp_2 = (*wp_2).w_next;
            }
            tp_1 = (*tp_1).tp_next as *mut tabpage_T;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim__ns_get(
    mut ns_id: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> KeyDict_ns_opts {
    let mut opts: KeyDict_ns_opts = KEYDICT_INIT;
    let mut windows: Array = ARRAY_DICT_INIT;
    opts.is_set__ns_opts_ = (opts.is_set__ns_opts_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_ns_opts__wins)
        as OptionalKeys;
    opts.wins = windows;
    if !ns_initialized(ns_id as uint32_t) {
        api_err_invalid(
            err,
            b"ns_id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ns_id as int64_t,
            false_0 != 0,
        );
        return opts;
    }
    if !set_has_uint32_t(namespace_localscope.ptr(), ns_id as uint32_t) {
        return opts;
    }
    let mut count: size_t = 0 as size_t;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if set_has_uint32_t(&raw mut (*wp).w_ns_set, ns_id as uint32_t) {
                count = count.wrapping_add(1);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    windows = arena_array(arena, count);
    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp_0.is_null() {
        let mut wp_0: *mut win_T = if tp_0 == curtab.get() {
            firstwin.get()
        } else {
            (*tp_0).tp_firstwin
        };
        while !wp_0.is_null() {
            if set_has_uint32_t(&raw mut (*wp_0).w_ns_set, ns_id as uint32_t) {
                if windows.size == windows.capacity {
                    windows.capacity = if windows.capacity != 0 {
                        windows.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    windows.items = xrealloc(
                        windows.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(windows.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh25 = windows.size;
                windows.size = windows.size.wrapping_add(1);
                *windows.items.offset(c2rust_fresh25 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*wp_0).handle as Integer,
                    },
                };
            }
            wp_0 = (*wp_0).w_next;
        }
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    opts.is_set__ns_opts_ = (opts.is_set__ns_opts_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_ns_opts__wins)
        as OptionalKeys;
    opts.wins = windows;
    return opts;
}
pub const KEYSET_OPTIDX_set_extmark__id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__url: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__spell: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__strict: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__end_col: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__conceal: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__hl_mode: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__end_row: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__end_line: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__hl_group: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__priority: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__sign_text: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_text: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_lines: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark___subpriority: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__undo_restore: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__conceal_lines: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__right_gravity: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_text_pos: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_text_win_col: ::core::ffi::c_int =
    31 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_set_extmark__virt_lines_overflow: ::core::ffi::c_int =
    34 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmark__hl_name: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmarks__type: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmarks__limit: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_extmarks__hl_name: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_ns_opts__wins: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NULL_STRING: String_0 = STRING_INIT;
pub const KEYDICT_INIT: KeyDict_ns_opts = KeyDict_ns_opts {
    is_set__ns_opts_: 0 as OptionalKeys,
    wins: Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    },
};
pub const MT_FLAG_PAIRED: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
pub const MT_FLAG_NO_UNDO: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
pub const MT_FLAG_INVALIDATE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 5 as ::core::ffi::c_int;
pub const MT_FLAG_INVALID: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 6 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_EXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_HL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 8 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNTEXT: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 9 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_SIGNHL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 10 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_VIRT_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 11 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_VIRT_TEXT_INLINE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 12 as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_CONCEAL_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 13 as ::core::ffi::c_int;
pub const MT_FLAG_RIGHT_GRAVITY: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mt_paired(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_PAIRED != 0;
}
#[inline]
unsafe extern "C" fn mt_right(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_RIGHT_GRAVITY != 0;
}
#[inline]
unsafe extern "C" fn mt_no_undo(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_NO_UNDO != 0;
}
#[inline]
unsafe extern "C" fn mt_invalidate(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_INVALIDATE != 0;
}
#[inline]
unsafe extern "C" fn mt_invalid(mut key: MTKey) -> bool {
    return key.flags as ::core::ffi::c_int & MT_FLAG_INVALID != 0;
}
#[inline]
unsafe extern "C" fn mt_decor(mut key: MTKey) -> DecorInline {
    return DecorInline {
        ext: key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0,
        data: key.decor_data,
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
