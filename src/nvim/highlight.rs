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
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_mem_free(mem: ArenaMem);
    fn KeyDict_highlight_get_field(str: *const ::core::ffi::c_char, len: size_t)
        -> *mut KeySetLink;
    fn KeyDict_highlight_cterm_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn mh_clear(h: *mut MapHash);
    fn mh_get_int(set: *mut Set_int, key: ::core::ffi::c_int) -> uint32_t;
    fn mh_put_cstr_t(set: *mut Set_cstr_t, key: cstr_t, new: *mut MHPutStatus) -> uint32_t;
    fn mh_get_uint64_t(set: *mut Set_uint64_t, key: uint64_t) -> uint32_t;
    fn mh_put_HlEntry(set: *mut Set_HlEntry, key: HlEntry, new: *mut MHPutStatus) -> uint32_t;
    fn mh_get_ColorKey(set: *mut Set_ColorKey, key: ColorKey) -> uint32_t;
    fn map_put_ref_int_ptr_t(
        map: *mut Map_int_ptr_t,
        key: ::core::ffi::c_int,
        key_alloc: *mut *mut ::core::ffi::c_int,
        new_item: *mut bool,
    ) -> *mut ptr_t;
    fn map_put_ref_uint64_t_int(
        map: *mut Map_uint64_t_int,
        key: uint64_t,
        key_alloc: *mut *mut uint64_t,
        new_item: *mut bool,
    ) -> *mut ::core::ffi::c_int;
    fn map_put_ref_ColorKey_ColorItem(
        map: *mut Map_ColorKey_ColorItem,
        key: ColorKey,
        key_alloc: *mut *mut ColorKey,
        new_item: *mut bool,
    ) -> *mut ColorItem;
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
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn api_dict_to_keydict(
        retval: *mut ::core::ffi::c_void,
        hashy: FieldHashfn,
        dict: Dict,
        err: *mut Error,
    ) -> bool;
    fn remote_ui_hl_attr_define(
        ui: *mut RemoteUI,
        id: Integer,
        rgb_attrs: HlAttrs,
        cterm_attrs: HlAttrs,
        info: Array,
    );
    fn remote_ui_hl_group_set(ui: *mut RemoteUI, name: String_0, id: Integer);
    fn get_decor_provider(ns_id: NS, force: bool) -> *mut DecorProvider;
    fn screen_invalidate_highlights();
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static curwin: GlobalCell<*mut win_T>;
    static must_redraw_pum: GlobalCell<bool>;
    static need_highlight_changed: GlobalCell<bool>;
    static p_bg: GlobalCell<*mut ::core::ffi::c_char>;
    static p_pb: GlobalCell<OptInt>;
    static hlf_names: GlobalCell<[*const ::core::ffi::c_char; 0]>;
    static highlight_attr: GlobalCell<[::core::ffi::c_int; 76]>;
    static highlight_attr_last: GlobalCell<[::core::ffi::c_int; 76]>;
    static normal_fg: GlobalCell<RgbValue>;
    static normal_bg: GlobalCell<RgbValue>;
    static normal_sp: GlobalCell<RgbValue>;
    static ns_hl_global: GlobalCell<NS>;
    static ns_hl_win: GlobalCell<NS>;
    static ns_hl_fast: GlobalCell<NS>;
    static ns_hl_active: GlobalCell<NS>;
    static hl_attr_active: GlobalCell<*mut ::core::ffi::c_int>;
    fn set_hl_group(
        id: ::core::ffi::c_int,
        attrs: HlAttrs,
        dict: *mut KeyDict_highlight,
        link_id: ::core::ffi::c_int,
    );
    fn syn_id2name(id: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn syn_check_group(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn syn_ns_id2attr(
        ns_id: ::core::ffi::c_int,
        hl_id: ::core::ffi::c_int,
        optional: *mut bool,
    ) -> ::core::ffi::c_int;
    fn highlight_attr_set_all();
    fn highlight_changed();
    fn name_to_color(name: *const ::core::ffi::c_char, idx: *mut ::core::ffi::c_int) -> RgbValue;
    fn name_to_ctermcolor(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn nlua_call_ref(
        ref_0: LuaRef,
        name: *const ::core::ffi::c_char,
        args: Array,
        mode: LuaRetMode,
        arena: *mut Arena,
        err: *mut Error,
    ) -> Object;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn check_blending(wp: *mut win_T);
    fn pum_drawn() -> bool;
    fn ui_call_hl_attr_define(id: Integer, rgb_attrs: HlAttrs, cterm_attrs: HlAttrs, info: Array);
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct consumed_blk {
    pub prev: *mut consumed_blk,
}
pub type ArenaMem = *mut consumed_blk;
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
pub type FieldHashfn =
    Option<unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_highlight {
    pub is_set__highlight_: OptionalKeys,
    pub altfont: Boolean,
    pub blink: Boolean,
    pub bold: Boolean,
    pub conceal: Boolean,
    pub dim: Boolean,
    pub italic: Boolean,
    pub nocombine: Boolean,
    pub overline: Boolean,
    pub reverse: Boolean,
    pub standout: Boolean,
    pub strikethrough: Boolean,
    pub undercurl: Boolean,
    pub underdashed: Boolean,
    pub underdotted: Boolean,
    pub underdouble: Boolean,
    pub underline: Boolean,
    pub default_: Boolean,
    pub cterm: Dict,
    pub foreground: Object,
    pub fg: Object,
    pub background: Object,
    pub bg: Object,
    pub ctermfg: Object,
    pub ctermbg: Object,
    pub special: Object,
    pub sp: Object,
    pub link: HLGroupID,
    pub link_global: HLGroupID,
    pub fallback: Boolean,
    pub blend: Integer,
    pub fg_indexed: Boolean,
    pub bg_indexed: Boolean,
    pub force: Boolean,
    pub update: Boolean,
    pub url: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_highlight_cterm {
    pub bold: Boolean,
    pub standout: Boolean,
    pub strikethrough: Boolean,
    pub underline: Boolean,
    pub undercurl: Boolean,
    pub underdouble: Boolean,
    pub underdotted: Boolean,
    pub underdashed: Boolean,
    pub italic: Boolean,
    pub reverse: Boolean,
    pub altfont: Boolean,
    pub dim: Boolean,
    pub blink: Boolean,
    pub conceal: Boolean,
    pub overline: Boolean,
    pub nocombine: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DecorProvider {
    pub ns_id: NS,
    pub state: C2Rust_Unnamed_13,
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kDecorProviderDisabled: C2Rust_Unnamed_13 = 4;
pub const kDecorProviderRedrawDisabled: C2Rust_Unnamed_13 = 3;
pub const kDecorProviderWinDisabled: C2Rust_Unnamed_13 = 2;
pub const kDecorProviderActive: C2Rust_Unnamed_13 = 1;
pub type RgbValue = int32_t;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const HL_GLOBAL: C2Rust_Unnamed_14 = 16384;
pub const HL_DEFAULT: C2Rust_Unnamed_14 = 8192;
pub const HL_FG_INDEXED: C2Rust_Unnamed_14 = 4096;
pub const HL_BG_INDEXED: C2Rust_Unnamed_14 = 2048;
pub const HL_NOCOMBINE: C2Rust_Unnamed_14 = 1024;
pub const HL_OVERLINE: C2Rust_Unnamed_14 = 131072;
pub const HL_CONCEALED: C2Rust_Unnamed_14 = 65536;
pub const HL_BLINK: C2Rust_Unnamed_14 = 32768;
pub const HL_DIM: C2Rust_Unnamed_14 = 512;
pub const HL_ALTFONT: C2Rust_Unnamed_14 = 256;
pub const HL_STRIKETHROUGH: C2Rust_Unnamed_14 = 128;
pub const HL_STANDOUT: C2Rust_Unnamed_14 = 64;
pub const HL_UNDERDASHED: C2Rust_Unnamed_14 = 40;
pub const HL_UNDERDOTTED: C2Rust_Unnamed_14 = 32;
pub const HL_UNDERDOUBLE: C2Rust_Unnamed_14 = 24;
pub const HL_UNDERCURL: C2Rust_Unnamed_14 = 16;
pub const HL_UNDERLINE: C2Rust_Unnamed_14 = 8;
pub const HL_UNDERLINE_MASK: C2Rust_Unnamed_14 = 56;
pub const HL_ITALIC: C2Rust_Unnamed_14 = 4;
pub const HL_BOLD: C2Rust_Unnamed_14 = 2;
pub const HL_INVERSE: C2Rust_Unnamed_14 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlAttrs {
    pub rgb_ae_attr: int32_t,
    pub cterm_ae_attr: int32_t,
    pub rgb_fg_color: RgbValue,
    pub rgb_bg_color: RgbValue,
    pub rgb_sp_color: RgbValue,
    pub cterm_fg_color: int16_t,
    pub cterm_bg_color: int16_t,
    pub hl_blend: int32_t,
    pub url: int32_t,
}
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_15 = 76;
pub const HLF_PRE: C2Rust_Unnamed_15 = 75;
pub const HLF_OK: C2Rust_Unnamed_15 = 74;
pub const HLF_SO: C2Rust_Unnamed_15 = 73;
pub const HLF_SE: C2Rust_Unnamed_15 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_15 = 71;
pub const HLF_TS: C2Rust_Unnamed_15 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_15 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_15 = 68;
pub const HLF_CU: C2Rust_Unnamed_15 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_15 = 66;
pub const HLF_WBR: C2Rust_Unnamed_15 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_15 = 64;
pub const HLF_MSG: C2Rust_Unnamed_15 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_15 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_15 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_15 = 60;
pub const HLF_0: C2Rust_Unnamed_15 = 59;
pub const HLF_QFL: C2Rust_Unnamed_15 = 58;
pub const HLF_MC: C2Rust_Unnamed_15 = 57;
pub const HLF_CUL: C2Rust_Unnamed_15 = 56;
pub const HLF_CUC: C2Rust_Unnamed_15 = 55;
pub const HLF_TPF: C2Rust_Unnamed_15 = 54;
pub const HLF_TPS: C2Rust_Unnamed_15 = 53;
pub const HLF_TP: C2Rust_Unnamed_15 = 52;
pub const HLF_PBR: C2Rust_Unnamed_15 = 51;
pub const HLF_PST: C2Rust_Unnamed_15 = 50;
pub const HLF_PSB: C2Rust_Unnamed_15 = 49;
pub const HLF_PSX: C2Rust_Unnamed_15 = 48;
pub const HLF_PNX: C2Rust_Unnamed_15 = 47;
pub const HLF_PSK: C2Rust_Unnamed_15 = 46;
pub const HLF_PNK: C2Rust_Unnamed_15 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_15 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_15 = 43;
pub const HLF_PSI: C2Rust_Unnamed_15 = 42;
pub const HLF_PNI: C2Rust_Unnamed_15 = 41;
pub const HLF_SPL: C2Rust_Unnamed_15 = 40;
pub const HLF_SPR: C2Rust_Unnamed_15 = 39;
pub const HLF_SPC: C2Rust_Unnamed_15 = 38;
pub const HLF_SPB: C2Rust_Unnamed_15 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_15 = 36;
pub const HLF_SC: C2Rust_Unnamed_15 = 35;
pub const HLF_TXA: C2Rust_Unnamed_15 = 34;
pub const HLF_TXD: C2Rust_Unnamed_15 = 33;
pub const HLF_DED: C2Rust_Unnamed_15 = 32;
pub const HLF_CHD: C2Rust_Unnamed_15 = 31;
pub const HLF_ADD: C2Rust_Unnamed_15 = 30;
pub const HLF_FC: C2Rust_Unnamed_15 = 29;
pub const HLF_FL: C2Rust_Unnamed_15 = 28;
pub const HLF_WM: C2Rust_Unnamed_15 = 27;
pub const HLF_W: C2Rust_Unnamed_15 = 26;
pub const HLF_VNC: C2Rust_Unnamed_15 = 25;
pub const HLF_V: C2Rust_Unnamed_15 = 24;
pub const HLF_T: C2Rust_Unnamed_15 = 23;
pub const HLF_VSP: C2Rust_Unnamed_15 = 22;
pub const HLF_C: C2Rust_Unnamed_15 = 21;
pub const HLF_SNC: C2Rust_Unnamed_15 = 20;
pub const HLF_S: C2Rust_Unnamed_15 = 19;
pub const HLF_R: C2Rust_Unnamed_15 = 18;
pub const HLF_CLF: C2Rust_Unnamed_15 = 17;
pub const HLF_CLS: C2Rust_Unnamed_15 = 16;
pub const HLF_CLN: C2Rust_Unnamed_15 = 15;
pub const HLF_LNB: C2Rust_Unnamed_15 = 14;
pub const HLF_LNA: C2Rust_Unnamed_15 = 13;
pub const HLF_N: C2Rust_Unnamed_15 = 12;
pub const HLF_CM: C2Rust_Unnamed_15 = 11;
pub const HLF_M: C2Rust_Unnamed_15 = 10;
pub const HLF_LC: C2Rust_Unnamed_15 = 9;
pub const HLF_L: C2Rust_Unnamed_15 = 8;
pub const HLF_I: C2Rust_Unnamed_15 = 7;
pub const HLF_E: C2Rust_Unnamed_15 = 6;
pub const HLF_D: C2Rust_Unnamed_15 = 5;
pub const HLF_AT: C2Rust_Unnamed_15 = 4;
pub const HLF_TERM: C2Rust_Unnamed_15 = 3;
pub const HLF_EOB: C2Rust_Unnamed_15 = 2;
pub const HLF_8: C2Rust_Unnamed_15 = 1;
pub const HLF_NONE: C2Rust_Unnamed_15 = 0;
pub type HlKind = ::core::ffi::c_uint;
pub const kHlInvalid: HlKind = 7;
pub const kHlBlendThrough: HlKind = 6;
pub const kHlBlend: HlKind = 5;
pub const kHlCombine: HlKind = 4;
pub const kHlTerminal: HlKind = 3;
pub const kHlSyntax: HlKind = 2;
pub const kHlUI: HlKind = 1;
pub const kHlUnknown: HlKind = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlEntry {
    pub attr: HlAttrs,
    pub kind: HlKind,
    pub id1: ::core::ffi::c_int,
    pub id2: ::core::ffi::c_int,
    pub winid: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ColorKey {
    pub ns_id: ::core::ffi::c_int,
    pub syn_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ColorItem {
    pub attr_id: ::core::ffi::c_int,
    pub link_id: ::core::ffi::c_int,
    pub version: ::core::ffi::c_int,
    pub is_default: bool,
    pub link_global: bool,
}
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const HLATTRS_DICT_SIZE: C2Rust_Unnamed_16 = 24;
pub type cstr_t = *const ::core::ffi::c_char;
pub type MHPutStatus = ::core::ffi::c_uint;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_int {
    pub h: MapHash,
    pub keys: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_cstr_t {
    pub h: MapHash,
    pub keys: *mut cstr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_HlEntry {
    pub h: MapHash,
    pub keys: *mut HlEntry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Set_ColorKey {
    pub h: MapHash,
    pub keys: *mut ColorKey,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_int_ptr_t {
    pub set: Set_int,
    pub values: *mut ptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_uint64_t_int {
    pub set: Set_uint64_t,
    pub values: *mut ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Map_ColorKey_ColorItem {
    pub set: Set_ColorKey,
    pub values: *mut ColorItem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct packer_buffer_t {
    pub startptr: *mut ::core::ffi::c_char,
    pub ptr: *mut ::core::ffi::c_char,
    pub endptr: *mut ::core::ffi::c_char,
    pub anydata: *mut ::core::ffi::c_void,
    pub anyint: int64_t,
    pub packer_flush: PackerBufferFlush,
}
pub type PackerBufferFlush = Option<unsafe extern "C" fn(*mut PackerBuffer) -> ()>;
pub type PackerBuffer = packer_buffer_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RemoteUI {
    pub rgb: bool,
    pub override_0: bool,
    pub composed: bool,
    pub ui_ext: [bool; 10],
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub pum_nlines: ::core::ffi::c_int,
    pub pum_pos: bool,
    pub pum_row: ::core::ffi::c_double,
    pub pum_col: ::core::ffi::c_double,
    pub pum_height: ::core::ffi::c_double,
    pub pum_width: ::core::ffi::c_double,
    pub term_name: *mut ::core::ffi::c_char,
    pub term_background: *mut ::core::ffi::c_char,
    pub term_colors: ::core::ffi::c_int,
    pub stdin_tty: bool,
    pub stdout_tty: bool,
    pub channel_id: uint64_t,
    pub packer: PackerBuffer,
    pub cur_event: *const ::core::ffi::c_char,
    pub nevents_pos: *mut ::core::ffi::c_char,
    pub ncalls_pos: *mut ::core::ffi::c_char,
    pub nevents: uint32_t,
    pub ncalls: uint32_t,
    pub flushed_events: bool,
    pub incomplete_event: bool,
    pub ncells_pending: size_t,
    pub hl_id: ::core::ffi::c_int,
    pub cursor_row: Integer,
    pub cursor_col: Integer,
    pub client_row: Integer,
    pub client_col: Integer,
    pub wildmenu_active: bool,
}
pub type LuaRetMode = ::core::ffi::c_uint;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub type NSHlAttr = [::core::ffi::c_int; 76];
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const KEYSET_OPTIDX_highlight__bg: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__fg: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__sp: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__dim: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__bold: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__link: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__blend: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__blink: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__cterm: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__italic: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__reverse: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__default: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__altfont: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__conceal: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__special: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__ctermfg: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__ctermbg: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__fallback: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__overline: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__standout: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__nocombine: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__undercurl: ::core::ffi::c_int = 25 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__underline: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__background: ::core::ffi::c_int = 27 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__bg_indexed: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__foreground: ::core::ffi::c_int = 29 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__fg_indexed: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__link_global: ::core::ffi::c_int = 31 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__underdashed: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__underdotted: ::core::ffi::c_int = 33 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__underdouble: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__strikethrough: ::core::ffi::c_int = 35 as ::core::ffi::c_int;
pub const HLATTRS_INIT: HlAttrs = HlAttrs {
    rgb_ae_attr: 0 as int32_t,
    cterm_ae_attr: 0 as int32_t,
    rgb_fg_color: -1 as RgbValue,
    rgb_bg_color: -1 as RgbValue,
    rgb_sp_color: -1 as RgbValue,
    cterm_fg_color: 0 as int16_t,
    cterm_bg_color: 0 as int16_t,
    hl_blend: -1 as int32_t,
    url: -1 as int32_t,
};
pub const COLOR_ITEM_INITIALIZER: ColorItem = ColorItem {
    attr_id: -1 as ::core::ffi::c_int,
    link_id: -1 as ::core::ffi::c_int,
    version: -1 as ::core::ffi::c_int,
    is_default: false_0 != 0,
    link_global: false_0 != 0,
};
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
static value_init_ColorItem: GlobalCell<ColorItem> = GlobalCell::new(COLOR_ITEM_INITIALIZER);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MAP_INIT: Map_uint64_t_int = Map_uint64_t_int {
    set: Set_uint64_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<uint64_t>(),
    },
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_put_HlEntry(
    mut set: *mut Set_HlEntry,
    mut key: HlEntry,
    mut key_alloc: *mut *mut HlEntry,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_HlEntry(set, key, &raw mut status);
    if !key_alloc.is_null() {
        *key_alloc = (*set).keys.offset(k as isize);
    }
    return status as ::core::ffi::c_uint
        != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
}
#[inline]
unsafe extern "C" fn set_has_ColorKey(mut set: *mut Set_ColorKey, mut key: ColorKey) -> bool {
    return mh_get_ColorKey(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn map_get_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
) -> ptr_t {
    let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_get_uint64_t_int(
    mut map: *mut Map_uint64_t_int,
    mut key: uint64_t,
) -> ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_int.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_put_uint64_t_int(
    mut map: *mut Map_uint64_t_int,
    mut key: uint64_t,
    mut value: ::core::ffi::c_int,
) {
    let mut val: *mut ::core::ffi::c_int = map_put_ref_uint64_t_int(
        map,
        key,
        ::core::ptr::null_mut::<*mut uint64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_put_ColorKey_ColorItem(
    mut map: *mut Map_ColorKey_ColorItem,
    mut key: ColorKey,
    mut value: ColorItem,
) {
    let mut val: *mut ColorItem = map_put_ref_ColorKey_ColorItem(
        map,
        key,
        ::core::ptr::null_mut::<*mut ColorKey>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_ColorKey_ColorItem(
    mut map: *mut Map_ColorKey_ColorItem,
    mut key: ColorKey,
) -> ColorItem {
    let mut k: uint32_t = mh_get_ColorKey(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ColorItem.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const MAX_TYPENR: ::core::ffi::c_int = 65535 as ::core::ffi::c_int;
static hlstate_active: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static attr_entries: GlobalCell<Set_HlEntry> = GlobalCell::new(Set_HlEntry {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<HlEntry>(),
});
static combine_attr_entries: GlobalCell<Map_uint64_t_int> = GlobalCell::new(MAP_INIT);
static blend_attr_entries: GlobalCell<Map_uint64_t_int> = GlobalCell::new(MAP_INIT);
static blendthrough_attr_entries: GlobalCell<Map_uint64_t_int> = GlobalCell::new(MAP_INIT);
static urls: GlobalCell<Set_cstr_t> = GlobalCell::new(Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
});
static ns_hls: GlobalCell<Map_ColorKey_ColorItem> = GlobalCell::new(Map_ColorKey_ColorItem {
    set: Set_ColorKey {
        h: MapHash {
            n_buckets: 0,
            size: 0,
            n_occupied: 0,
            upper_bound: 0,
            n_keys: 0,
            keys_capacity: 0,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<ColorKey>(),
    },
    values: ::core::ptr::null_mut::<ColorItem>(),
});
static ns_hl_attr: GlobalCell<Map_int_ptr_t> = GlobalCell::new(Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0,
            size: 0,
            n_occupied: 0,
            upper_bound: 0,
            n_keys: 0,
            keys_capacity: 0,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
});
#[no_mangle]
pub unsafe extern "C" fn highlight_init() {
    set_put_HlEntry(
        attr_entries.ptr(),
        HlEntry {
            attr: HlAttrs {
                rgb_ae_attr: 0 as int32_t,
                cterm_ae_attr: 0 as int32_t,
                rgb_fg_color: -1 as RgbValue,
                rgb_bg_color: -1 as RgbValue,
                rgb_sp_color: -1 as RgbValue,
                cterm_fg_color: 0 as int16_t,
                cterm_bg_color: 0 as int16_t,
                hl_blend: -1 as int32_t,
                url: -1 as int32_t,
            },
            kind: kHlInvalid,
            id1: 0 as ::core::ffi::c_int,
            id2: 0 as ::core::ffi::c_int,
            winid: 0,
        },
        ::core::ptr::null_mut::<*mut HlEntry>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn highlight_use_hlstate() -> bool {
    if hlstate_active.get() {
        return false_0 != 0;
    }
    hlstate_active.set(true_0 != 0);
    clear_hl_tables(true_0 != 0);
    return true_0 != 0;
}
unsafe extern "C" fn get_attr_entry(mut entry: HlEntry) -> ::core::ffi::c_int {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = 0;
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut retried: bool = false_0 != 0;
    if !hlstate_active.get() {
        entry.kind = kHlUnknown;
        entry.id1 = 0 as ::core::ffi::c_int;
        entry.id2 = 0 as ::core::ffi::c_int;
    }
    loop {
        status = kMHExisting;
        k = mh_put_HlEntry(attr_entries.ptr(), entry, &raw mut status);
        if status as ::core::ffi::c_uint == kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return k as ::core::ffi::c_int;
        }
        if (*attr_entries.ptr()).h.size <= MAX_TYPENR as uint32_t {
            break;
        }
        if recursive.get() as ::core::ffi::c_int != 0 || retried as ::core::ffi::c_int != 0 {
            emsg(gettext(
                b"E424: Too many different highlighting attributes in use\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return 0 as ::core::ffi::c_int;
        }
        recursive.set(true_0 != 0);
        clear_hl_tables(true_0 != 0);
        recursive.set(false_0 != 0);
        if entry.kind as ::core::ffi::c_uint
            == kHlCombine as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return 0 as ::core::ffi::c_int;
        }
        retried = true_0 != 0;
    }
    let mut id: ::core::ffi::c_int = k as ::core::ffi::c_int;
    let mut arena: Arena = ARENA_EMPTY;
    let mut inspect: Array = hl_inspect(id, &raw mut arena);
    ui_call_hl_attr_define(id as Integer, entry.attr, entry.attr, inspect);
    arena_mem_free(arena_finish(&raw mut arena));
    return id;
}
#[no_mangle]
pub unsafe extern "C" fn ui_send_all_hls(mut ui: *mut RemoteUI) {
    let mut i: size_t = 1 as size_t;
    while i < (*attr_entries.ptr()).h.size as size_t {
        let mut arena: Arena = ARENA_EMPTY;
        let mut inspect: Array = hl_inspect(i as ::core::ffi::c_int, &raw mut arena);
        let mut attr: HlAttrs = (*(*attr_entries.ptr()).keys.offset(i as isize)).attr;
        remote_ui_hl_attr_define(ui, i as Integer, attr, attr, inspect);
        arena_mem_free(arena_finish(&raw mut arena));
        i = i.wrapping_add(1);
    }
    let mut hlf: size_t = 0 as size_t;
    while hlf < HLF_COUNT as ::core::ffi::c_int as size_t {
        remote_ui_hl_group_set(
            ui,
            cstr_as_string(
                *(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(hlf as isize),
            ),
            (*highlight_attr.ptr())[hlf as usize] as Integer,
        );
        hlf = hlf.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn hl_get_syn_attr(
    mut ns_id: ::core::ffi::c_int,
    mut idx: ::core::ffi::c_int,
    mut at_en: HlAttrs,
) -> ::core::ffi::c_int {
    if at_en.cterm_fg_color as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        || at_en.cterm_bg_color as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        || at_en.rgb_fg_color != -1 as RgbValue
        || at_en.rgb_bg_color != -1 as RgbValue
        || at_en.rgb_sp_color != -1 as RgbValue
        || at_en.cterm_ae_attr != 0 as int32_t
        || at_en.rgb_ae_attr != 0 as int32_t
        || ns_id != 0 as ::core::ffi::c_int
    {
        return get_attr_entry(HlEntry {
            attr: at_en,
            kind: kHlSyntax,
            id1: idx,
            id2: ns_id,
            winid: 0,
        });
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn ns_hl_def(
    mut ns_id: NS,
    mut hl_id: ::core::ffi::c_int,
    mut attrs: HlAttrs,
    mut link_id: ::core::ffi::c_int,
    mut dict: *mut KeyDict_highlight,
) {
    if ns_id == 0 as ::core::ffi::c_int {
        '_c2rust_label: {
            if !dict.is_null() {
            } else {
                __assert_fail(
                    b"dict\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/highlight.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    161 as ::core::ffi::c_uint,
                    b"void ns_hl_def(NS, int, HlAttrs, int, KeyDict_highlight *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        set_hl_group(hl_id, attrs, dict, link_id);
        return;
    }
    if attrs.rgb_ae_attr & HL_DEFAULT as ::core::ffi::c_int as int32_t != 0
        && set_has_ColorKey(
            &raw mut (*ns_hls.ptr()).set,
            ColorKey {
                ns_id: ns_id,
                syn_id: hl_id,
            },
        ) as ::core::ffi::c_int
            != 0
    {
        return;
    }
    let mut p: *mut DecorProvider = get_decor_provider(ns_id, true_0 != 0);
    let mut attr_id: ::core::ffi::c_int = if link_id > 0 as ::core::ffi::c_int {
        -1 as ::core::ffi::c_int
    } else {
        hl_get_syn_attr(ns_id as ::core::ffi::c_int, hl_id, attrs)
    };
    let mut it: ColorItem = ColorItem {
        attr_id: attr_id,
        link_id: link_id,
        version: (*p).hl_valid,
        is_default: attrs.rgb_ae_attr & HL_DEFAULT as ::core::ffi::c_int as int32_t != 0,
        link_global: attrs.rgb_ae_attr & HL_GLOBAL as ::core::ffi::c_int as int32_t != 0,
    };
    map_put_ColorKey_ColorItem(
        ns_hls.ptr(),
        ColorKey {
            ns_id: ns_id,
            syn_id: hl_id,
        },
        it,
    );
    (*p).hl_cached = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ns_get_hl(
    mut ns_hl: *mut NS,
    mut hl_id: ::core::ffi::c_int,
    mut link: bool,
    mut nodefault: bool,
) -> ::core::ffi::c_int {
    static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if *ns_hl == 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if *ns_hl < 0 as ::core::ffi::c_int {
        if ns_hl_active.get() <= 0 as ::core::ffi::c_int {
            return -1 as ::core::ffi::c_int;
        }
        *ns_hl = ns_hl_active.get();
    }
    let mut ns_id: ::core::ffi::c_int = *ns_hl as ::core::ffi::c_int;
    let mut p: *mut DecorProvider = get_decor_provider(ns_id as NS, true_0 != 0);
    let mut it: ColorItem = map_get_ColorKey_ColorItem(
        ns_hls.ptr(),
        ColorKey {
            ns_id: ns_id,
            syn_id: hl_id,
        },
    );
    let mut valid_item: bool = it.version >= (*p).hl_valid;
    if !valid_item && (*p).hl_def != LUA_NOREF && recursive.get() == 0 {
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
        let c2rust_fresh8 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh8 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: ns_id as Integer,
            },
        };
        let c2rust_fresh9 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh9 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(syn_id2name(hl_id)),
            },
        };
        let c2rust_fresh10 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh10 as isize) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: link },
        };
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        (*recursive.ptr()) += 1;
        let mut ret: Object = nlua_call_ref(
            (*p).hl_def,
            b"hl_def\0".as_ptr() as *const ::core::ffi::c_char,
            args,
            kRetObject,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        (*recursive.ptr()) -= 1;
        let mut fallback: bool = true_0 != 0;
        let mut tmp: bool = false_0 != 0;
        let mut attrs: HlAttrs = HLATTRS_INIT;
        if ret.type_0 as ::core::ffi::c_uint
            == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            fallback = false_0 != 0;
            let mut dict: KeyDict_highlight = KeyDict_highlight {
                is_set__highlight_: 0 as OptionalKeys,
                altfont: false,
                blink: false,
                bold: false,
                conceal: false,
                dim: false,
                italic: false,
                nocombine: false,
                overline: false,
                reverse: false,
                standout: false,
                strikethrough: false,
                undercurl: false,
                underdashed: false,
                underdotted: false,
                underdouble: false,
                underline: false,
                default_: false,
                cterm: Dict {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                },
                foreground: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                fg: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                background: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                bg: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                ctermfg: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                ctermbg: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                special: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                sp: Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                },
                link: 0,
                link_global: 0,
                fallback: false,
                blend: 0,
                fg_indexed: false,
                bg_indexed: false,
                force: false,
                update: false,
                url: String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                },
            };
            if api_dict_to_keydict(
                &raw mut dict as *mut ::core::ffi::c_void,
                Some(
                    KeyDict_highlight_get_field
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            size_t,
                        ) -> *mut KeySetLink,
                ),
                ret.data.dict,
                &raw mut err,
            ) {
                attrs = dict2hlattrs(
                    &raw mut dict,
                    true_0 != 0,
                    &raw mut it.link_id,
                    ::core::ptr::null_mut::<HlAttrs>(),
                    &raw mut err,
                );
                fallback = if dict.is_set__highlight_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__fallback
                    != 0 as ::core::ffi::c_ulonglong
                {
                    dict.fallback as ::core::ffi::c_int
                } else {
                    true_0
                } != 0;
                tmp = dict.fallback as bool;
                if it.link_id >= 0 as ::core::ffi::c_int {
                    fallback = true_0 != 0;
                }
            }
        }
        it.attr_id = if fallback as ::core::ffi::c_int != 0 {
            -1 as ::core::ffi::c_int
        } else {
            hl_get_syn_attr(ns_id, hl_id, attrs)
        };
        it.version = (*p).hl_valid - tmp as ::core::ffi::c_int;
        it.is_default = attrs.rgb_ae_attr & HL_DEFAULT as ::core::ffi::c_int as int32_t != 0;
        it.link_global = attrs.rgb_ae_attr & HL_GLOBAL as ::core::ffi::c_int as int32_t != 0;
        map_put_ColorKey_ColorItem(
            ns_hls.ptr(),
            ColorKey {
                ns_id: ns_id,
                syn_id: hl_id,
            },
            it,
        );
        valid_item = true_0 != 0;
    }
    if it.is_default as ::core::ffi::c_int != 0 && nodefault as ::core::ffi::c_int != 0
        || !valid_item
    {
        return -1 as ::core::ffi::c_int;
    }
    if link {
        if it.attr_id >= 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if it.link_global {
            *ns_hl = 0 as ::core::ffi::c_int as NS;
        }
        return it.link_id;
    } else {
        return it.attr_id;
    };
}
#[no_mangle]
pub unsafe extern "C" fn hl_check_ns() -> bool {
    let mut ns: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if ns_hl_fast.get() > 0 as ::core::ffi::c_int {
        ns = ns_hl_fast.get() as ::core::ffi::c_int;
    } else if ns_hl_win.get() >= 0 as ::core::ffi::c_int {
        ns = ns_hl_win.get() as ::core::ffi::c_int;
    } else {
        ns = ns_hl_global.get() as ::core::ffi::c_int;
    }
    if ns_hl_active.get() == ns {
        return false_0 != 0;
    }
    ns_hl_active.set(ns as NS);
    hl_attr_active.set(highlight_attr.ptr() as *mut ::core::ffi::c_int);
    if ns > 0 as ::core::ffi::c_int {
        update_ns_hl(ns);
        let mut hl_def: *mut NSHlAttr = map_get_int_ptr_t(ns_hl_attr.ptr(), ns) as *mut NSHlAttr;
        if !hl_def.is_null() {
            hl_attr_active.set(&raw mut *hl_def as *mut ::core::ffi::c_int);
        }
    }
    need_highlight_changed.set(true_0 != 0);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn win_check_ns_hl(mut wp: *mut win_T) -> bool {
    ns_hl_win.set(
        (if !wp.is_null() {
            (*wp).w_ns_hl
        } else {
            -1 as ::core::ffi::c_int
        }) as NS,
    );
    return hl_check_ns();
}
#[no_mangle]
pub unsafe extern "C" fn hl_ns_get_attrs(
    mut ns_id: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
    mut optional: *mut bool,
    mut attrs: *mut HlAttrs,
) -> bool {
    let mut opt: bool = if !optional.is_null() {
        *optional as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut syn_attr: ::core::ffi::c_int = syn_ns_id2attr(ns_id, hl_id, &raw mut opt);
    if !optional.is_null() {
        *optional = opt;
    }
    if syn_attr <= 0 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    *attrs = syn_attr2entry(syn_attr);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn hl_get_ui_attr(
    mut ns_id: ::core::ffi::c_int,
    mut idx: ::core::ffi::c_int,
    mut final_id: ::core::ffi::c_int,
    mut optional: bool,
) -> ::core::ffi::c_int {
    let mut attrs: HlAttrs = HLATTRS_INIT;
    let mut available: bool = false_0 != 0;
    if final_id > 0 as ::core::ffi::c_int {
        available = hl_ns_get_attrs(ns_id, final_id, &raw mut optional, &raw mut attrs);
    }
    if HLF_PNI as ::core::ffi::c_int <= idx && idx <= HLF_PST as ::core::ffi::c_int {
        if attrs.hl_blend == -1 as int32_t && p_pb.get() > 0 as OptInt {
            attrs.hl_blend = p_pb.get() as ::core::ffi::c_int as int32_t;
        }
        if pum_drawn() {
            must_redraw_pum.set(true_0 != 0);
        }
    }
    if optional as ::core::ffi::c_int != 0 && !available {
        return 0 as ::core::ffi::c_int;
    }
    return get_attr_entry(HlEntry {
        attr: attrs,
        kind: kHlUI,
        id1: idx,
        id2: final_id,
        winid: 0,
    });
}
#[no_mangle]
pub unsafe extern "C" fn hl_apply_winblend(
    mut winbl: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut entry: HlEntry = *(*attr_entries.ptr()).keys.offset(attr as isize);
    if entry.attr.hl_blend == -1 as int32_t && winbl > 0 as ::core::ffi::c_int {
        entry.attr.hl_blend = winbl as int32_t;
        attr = get_attr_entry(entry);
    }
    return attr;
}
#[no_mangle]
pub unsafe extern "C" fn update_window_hl(mut wp: *mut win_T, mut invalid: bool) {
    let mut ns_id: ::core::ffi::c_int = (*wp).w_ns_hl;
    update_ns_hl(ns_id);
    if ns_id != (*wp).w_ns_hl_active || (*wp).w_ns_hl_attr.is_null() {
        (*wp).w_ns_hl_active = ns_id;
        let mut hl_def_ptr: *mut NSHlAttr =
            map_get_int_ptr_t(ns_hl_attr.ptr(), ns_id) as *mut NSHlAttr;
        if !hl_def_ptr.is_null() {
            (*wp).w_ns_hl_attr = &raw mut *hl_def_ptr as *mut ::core::ffi::c_int;
        } else {
            (*wp).w_ns_hl_attr = highlight_attr.ptr() as *mut ::core::ffi::c_int;
        }
    }
    let mut hl_def: *mut ::core::ffi::c_int = (*wp).w_ns_hl_attr;
    if (*wp).w_hl_needs_update == 0 && !invalid {
        return;
    }
    (*wp).w_hl_needs_update = false_0;
    let mut float_win: bool =
        (*wp).w_floating as ::core::ffi::c_int != 0 && !(*wp).w_config.external;
    if float_win as ::core::ffi::c_int != 0
        && *hl_def.offset(HLF_NFLOAT as ::core::ffi::c_int as isize) != 0 as ::core::ffi::c_int
        && ns_id > 0 as ::core::ffi::c_int
    {
        (*wp).w_hl_attr_normal = *hl_def.offset(HLF_NFLOAT as ::core::ffi::c_int as isize);
    } else if *hl_def.offset(HLF_NONE as ::core::ffi::c_int as isize) > 0 as ::core::ffi::c_int {
        (*wp).w_hl_attr_normal = *hl_def.offset(HLF_NONE as ::core::ffi::c_int as isize);
    } else if float_win {
        (*wp).w_hl_attr_normal = if *(*hl_attr_active.ptr())
            .offset(HLF_NFLOAT as ::core::ffi::c_int as isize)
            > 0 as ::core::ffi::c_int
        {
            *(*hl_attr_active.ptr()).offset(HLF_NFLOAT as ::core::ffi::c_int as isize)
        } else {
            (*highlight_attr.ptr())[HLF_NFLOAT as ::core::ffi::c_int as usize]
        };
    } else {
        (*wp).w_hl_attr_normal = 0 as ::core::ffi::c_int;
    }
    if (*wp).w_floating {
        (*wp).w_hl_attr_normal = hl_apply_winblend(
            (*wp).w_onebuf_opt.wo_winbl as ::core::ffi::c_int,
            (*wp).w_hl_attr_normal,
        );
    }
    (*wp).w_config.shadow = false_0 != 0;
    if (*wp).w_floating as ::core::ffi::c_int != 0
        && (*wp).w_config.border as ::core::ffi::c_int != 0
    {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 8 as ::core::ffi::c_int {
            let mut attr: ::core::ffi::c_int =
                *hl_def.offset(HLF_BORDER as ::core::ffi::c_int as isize);
            if (*wp).w_config.border_hl_ids[i as usize] != 0 {
                attr = hl_get_ui_attr(
                    ns_id,
                    HLF_BORDER as ::core::ffi::c_int,
                    (*wp).w_config.border_hl_ids[i as usize],
                    false_0 != 0,
                );
            }
            attr = hl_apply_winblend((*wp).w_onebuf_opt.wo_winbl as ::core::ffi::c_int, attr);
            if syn_attr2entry(attr).hl_blend > 0 as int32_t {
                (*wp).w_config.shadow = true_0 != 0;
            }
            (*wp).w_config.border_attr[i as usize] = attr;
            i += 1;
        }
    }
    check_blending(wp);
    if *hl_def.offset(HLF_INACTIVE as ::core::ffi::c_int as isize) == 0 as ::core::ffi::c_int {
        (*wp).w_hl_attr_normalnc = hl_combine_attr(
            *(*hl_attr_active.ptr()).offset(HLF_INACTIVE as ::core::ffi::c_int as isize),
            (*wp).w_hl_attr_normal,
        );
    } else {
        (*wp).w_hl_attr_normalnc = *hl_def.offset(HLF_INACTIVE as ::core::ffi::c_int as isize);
    }
    if (*wp).w_floating {
        (*wp).w_hl_attr_normalnc = hl_apply_winblend(
            (*wp).w_onebuf_opt.wo_winbl as ::core::ffi::c_int,
            (*wp).w_hl_attr_normalnc,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn update_ns_hl(mut ns_id: ::core::ffi::c_int) {
    if ns_id <= 0 as ::core::ffi::c_int {
        return;
    }
    let mut p: *mut DecorProvider = get_decor_provider(ns_id as NS, true_0 != 0);
    if (*p).hl_cached {
        return;
    }
    let mut alloc: *mut *mut NSHlAttr = map_put_ref_int_ptr_t(
        ns_hl_attr.ptr(),
        ns_id,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
        ::core::ptr::null_mut::<bool>(),
    ) as *mut *mut NSHlAttr;
    if (*alloc).is_null() {
        *alloc = xmalloc(::core::mem::size_of::<NSHlAttr>()) as *mut NSHlAttr;
    }
    let mut hl_attrs: *mut ::core::ffi::c_int = &raw mut **alloc as *mut ::core::ffi::c_int;
    let mut hlf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while hlf < HLF_COUNT as ::core::ffi::c_int {
        let mut id: ::core::ffi::c_int = syn_check_group(
            *(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(hlf as isize),
            strlen(*(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(hlf as isize)),
        );
        let mut optional: bool =
            hlf == HLF_INACTIVE as ::core::ffi::c_int || hlf == HLF_NFLOAT as ::core::ffi::c_int;
        *hl_attrs.offset(hlf as isize) = hl_get_ui_attr(ns_id, hlf, id, optional);
        hlf += 1;
    }
    let mut normality: ::core::ffi::c_int = syn_check_group(
        b"Normal\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
    );
    *hl_attrs.offset(HLF_NONE as ::core::ffi::c_int as isize) =
        hl_get_ui_attr(ns_id, -1 as ::core::ffi::c_int, normality, true_0 != 0);
    p = get_decor_provider(ns_id as NS, true_0 != 0);
    (*p).hl_cached = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn win_bg_attr(mut wp: *mut win_T) -> ::core::ffi::c_int {
    if ns_hl_fast.get() < 0 as ::core::ffi::c_int {
        let mut local: ::core::ffi::c_int = if wp == curwin.get() {
            (*wp).w_hl_attr_normal
        } else {
            (*wp).w_hl_attr_normalnc
        };
        if local != 0 {
            return local;
        }
    }
    if wp == curwin.get()
        || *(*hl_attr_active.ptr()).offset(HLF_INACTIVE as ::core::ffi::c_int as isize)
            == 0 as ::core::ffi::c_int
    {
        return *(*hl_attr_active.ptr()).offset(HLF_NONE as ::core::ffi::c_int as isize);
    } else {
        return *(*hl_attr_active.ptr()).offset(HLF_INACTIVE as ::core::ffi::c_int as isize);
    };
}
#[no_mangle]
pub unsafe extern "C" fn hl_get_underline() -> ::core::ffi::c_int {
    let mut attrs: HlAttrs = HLATTRS_INIT;
    attrs.cterm_ae_attr = HL_UNDERLINE as ::core::ffi::c_int as int16_t as int32_t;
    attrs.rgb_ae_attr = HL_UNDERLINE as ::core::ffi::c_int as int16_t as int32_t;
    return get_attr_entry(HlEntry {
        attr: attrs,
        kind: kHlUI,
        id1: 0 as ::core::ffi::c_int,
        id2: 0 as ::core::ffi::c_int,
        winid: 0,
    });
}
#[no_mangle]
pub unsafe extern "C" fn hl_add_url(
    mut attr: ::core::ffi::c_int,
    mut url: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut attrs: HlAttrs = HLATTRS_INIT;
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_cstr_t(urls.ptr(), url as cstr_t, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        *(*urls.ptr()).keys.offset(k as isize) = xstrdup(url) as cstr_t;
    }
    attrs.url = k as int32_t;
    let mut new: ::core::ffi::c_int = get_attr_entry(HlEntry {
        attr: attrs,
        kind: kHlUI,
        id1: 0 as ::core::ffi::c_int,
        id2: 0 as ::core::ffi::c_int,
        winid: 0,
    });
    return hl_combine_attr(attr, new);
}
#[no_mangle]
pub unsafe extern "C" fn hl_get_url(mut index: uint32_t) -> *const ::core::ffi::c_char {
    '_c2rust_label: {
        if !(*urls.ptr()).keys.is_null() {
        } else {
            __assert_fail(
                b"urls.keys\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/highlight.rs\0".as_ptr() as *const ::core::ffi::c_char,
                535 as ::core::ffi::c_uint,
                b"const char *hl_get_url(uint32_t)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return *(*urls.ptr()).keys.offset(index as isize) as *const ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn hl_get_term_attr(mut aep: *mut HlAttrs) -> ::core::ffi::c_int {
    return get_attr_entry(HlEntry {
        attr: *aep,
        kind: kHlTerminal,
        id1: 0 as ::core::ffi::c_int,
        id2: 0 as ::core::ffi::c_int,
        winid: 0,
    });
}
#[no_mangle]
pub unsafe extern "C" fn clear_hl_tables(mut reinit: bool) {
    let mut url: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*urls.ptr()).h.n_keys {
        url = *(*urls.ptr()).keys.offset(__i as isize) as *const ::core::ffi::c_char;
        xfree(url as *mut ::core::ffi::c_void);
        __i = __i.wrapping_add(1);
    }
    if reinit {
        mh_clear(&raw mut (*attr_entries.ptr()).h);
        highlight_init();
        mh_clear(&raw mut (*combine_attr_entries.ptr()).set.h);
        mh_clear(&raw mut (*blend_attr_entries.ptr()).set.h);
        mh_clear(&raw mut (*blendthrough_attr_entries.ptr()).set.h);
        mh_clear(&raw mut (*urls.ptr()).h);
        memset(
            highlight_attr_last.ptr() as *mut ::core::ffi::c_int as *mut ::core::ffi::c_void,
            -1 as ::core::ffi::c_int,
            ::core::mem::size_of::<[::core::ffi::c_int; 76]>(),
        );
        highlight_attr_set_all();
        highlight_changed();
        screen_invalidate_highlights();
    } else {
        xfree((*attr_entries.ptr()).keys as *mut ::core::ffi::c_void);
        xfree((*attr_entries.ptr()).h.hash as *mut ::core::ffi::c_void);
        attr_entries.set(Set_HlEntry {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<HlEntry>(),
        });
        xfree((*combine_attr_entries.ptr()).set.keys as *mut ::core::ffi::c_void);
        xfree((*combine_attr_entries.ptr()).set.h.hash as *mut ::core::ffi::c_void);
        (*combine_attr_entries.ptr()).set = Set_uint64_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<uint64_t>(),
        };
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*combine_attr_entries.ptr()).values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        *ptr_;
        xfree((*blend_attr_entries.ptr()).set.keys as *mut ::core::ffi::c_void);
        xfree((*blend_attr_entries.ptr()).set.h.hash as *mut ::core::ffi::c_void);
        (*blend_attr_entries.ptr()).set = Set_uint64_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<uint64_t>(),
        };
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*blend_attr_entries.ptr()).values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        *ptr__0;
        xfree((*blendthrough_attr_entries.ptr()).set.keys as *mut ::core::ffi::c_void);
        xfree((*blendthrough_attr_entries.ptr()).set.h.hash as *mut ::core::ffi::c_void);
        (*blendthrough_attr_entries.ptr()).set = Set_uint64_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<uint64_t>(),
        };
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            &raw mut (*blendthrough_attr_entries.ptr()).values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL_0;
        *ptr__1;
        xfree((*ns_hls.ptr()).set.keys as *mut ::core::ffi::c_void);
        xfree((*ns_hls.ptr()).set.h.hash as *mut ::core::ffi::c_void);
        (*ns_hls.ptr()).set = Set_ColorKey {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ColorKey>(),
        };
        let mut ptr__2: *mut *mut ::core::ffi::c_void =
            &raw mut (*ns_hls.ptr()).values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__2);
        *ptr__2 = NULL_0;
        *ptr__2;
        xfree((*urls.ptr()).keys as *mut ::core::ffi::c_void);
        xfree((*urls.ptr()).h.hash as *mut ::core::ffi::c_void);
        urls.set(Set_cstr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<cstr_t>(),
        });
    };
}
#[no_mangle]
pub unsafe extern "C" fn hl_invalidate_blends() {
    mh_clear(&raw mut (*blend_attr_entries.ptr()).set.h);
    mh_clear(&raw mut (*blendthrough_attr_entries.ptr()).set.h);
    highlight_changed();
    update_window_hl(curwin.get(), true_0 != 0);
}
unsafe extern "C" fn hl_combine_ae(mut char_ae: int32_t, mut prim_ae: int32_t) -> int32_t {
    let mut char_ul: int32_t = char_ae & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t;
    let mut prim_ul: int32_t = prim_ae & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t;
    let mut new_ul: int32_t = if prim_ul != 0 { prim_ul } else { char_ul };
    return char_ae & !(HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t)
        | prim_ae & !(HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t)
        | new_ul;
}
#[no_mangle]
pub unsafe extern "C" fn hl_combine_attr(
    mut char_attr: ::core::ffi::c_int,
    mut prim_attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if char_attr == 0 as ::core::ffi::c_int {
        return prim_attr;
    } else if prim_attr == 0 as ::core::ffi::c_int {
        return char_attr;
    }
    let mut combine_tag: uint64_t = (char_attr as uint32_t as uint64_t) << 32 as ::core::ffi::c_int
        | prim_attr as uint32_t as uint64_t;
    let mut id: ::core::ffi::c_int = map_get_uint64_t_int(combine_attr_entries.ptr(), combine_tag);
    if id > 0 as ::core::ffi::c_int {
        return id;
    }
    let mut char_aep: HlAttrs = syn_attr2entry(char_attr);
    let mut prim_aep: HlAttrs = syn_attr2entry(prim_attr);
    let mut new_en: HlAttrs = char_aep;
    if prim_aep.cterm_ae_attr & HL_NOCOMBINE as ::core::ffi::c_int as int32_t != 0 {
        new_en.cterm_ae_attr = prim_aep.cterm_ae_attr;
    } else {
        new_en.cterm_ae_attr = hl_combine_ae(new_en.cterm_ae_attr, prim_aep.cterm_ae_attr);
    }
    if prim_aep.rgb_ae_attr & HL_NOCOMBINE as ::core::ffi::c_int as int32_t != 0 {
        new_en.rgb_ae_attr = prim_aep.rgb_ae_attr;
    } else {
        new_en.rgb_ae_attr = hl_combine_ae(new_en.rgb_ae_attr, prim_aep.rgb_ae_attr);
    }
    if prim_aep.cterm_fg_color as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        new_en.cterm_fg_color = prim_aep.cterm_fg_color;
        new_en.rgb_ae_attr = (new_en.rgb_ae_attr as ::core::ffi::c_int
            & (!(HL_FG_INDEXED as ::core::ffi::c_int as int32_t)
                | prim_aep.rgb_ae_attr & HL_FG_INDEXED as ::core::ffi::c_int as int32_t)
                as ::core::ffi::c_int) as int32_t;
    }
    if prim_aep.cterm_bg_color as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        new_en.cterm_bg_color = prim_aep.cterm_bg_color;
        new_en.rgb_ae_attr = (new_en.rgb_ae_attr as ::core::ffi::c_int
            & (!(HL_BG_INDEXED as ::core::ffi::c_int as int32_t)
                | prim_aep.rgb_ae_attr & HL_BG_INDEXED as ::core::ffi::c_int as int32_t)
                as ::core::ffi::c_int) as int32_t;
    }
    if prim_aep.rgb_fg_color >= 0 as RgbValue {
        new_en.rgb_fg_color = prim_aep.rgb_fg_color;
        new_en.rgb_ae_attr = (new_en.rgb_ae_attr as ::core::ffi::c_int
            & (!(HL_FG_INDEXED as ::core::ffi::c_int as int32_t)
                | prim_aep.rgb_ae_attr & HL_FG_INDEXED as ::core::ffi::c_int as int32_t)
                as ::core::ffi::c_int) as int32_t;
    }
    if prim_aep.rgb_bg_color >= 0 as RgbValue {
        new_en.rgb_bg_color = prim_aep.rgb_bg_color;
        new_en.rgb_ae_attr = (new_en.rgb_ae_attr as ::core::ffi::c_int
            & (!(HL_BG_INDEXED as ::core::ffi::c_int as int32_t)
                | prim_aep.rgb_ae_attr & HL_BG_INDEXED as ::core::ffi::c_int as int32_t)
                as ::core::ffi::c_int) as int32_t;
    }
    if prim_aep.rgb_sp_color >= 0 as RgbValue {
        new_en.rgb_sp_color = prim_aep.rgb_sp_color;
    }
    if prim_aep.hl_blend >= 0 as int32_t {
        new_en.hl_blend = prim_aep.hl_blend;
    }
    if new_en.url == -1 as int32_t && prim_aep.url >= 0 as int32_t {
        new_en.url = prim_aep.url;
    }
    id = get_attr_entry(HlEntry {
        attr: new_en,
        kind: kHlCombine,
        id1: char_attr,
        id2: prim_attr,
        winid: 0,
    });
    if id > 0 as ::core::ffi::c_int {
        map_put_uint64_t_int(combine_attr_entries.ptr(), combine_tag, id);
    }
    return id;
}
unsafe extern "C" fn get_colors_force(mut attrs: HlAttrs) -> HlAttrs {
    if attrs.rgb_bg_color == -1 as RgbValue {
        attrs.rgb_bg_color = normal_bg.get();
    }
    if attrs.rgb_fg_color == -1 as RgbValue {
        attrs.rgb_fg_color = normal_fg.get();
    }
    if attrs.rgb_sp_color == -1 as RgbValue {
        attrs.rgb_sp_color = normal_sp.get();
    }
    let mut dark_: bool = *p_bg.get() as ::core::ffi::c_int == 'd' as ::core::ffi::c_int;
    attrs.rgb_fg_color = if attrs.rgb_fg_color != -1 as RgbValue {
        attrs.rgb_fg_color
    } else if dark_ as ::core::ffi::c_int != 0 {
        0xffffff as RgbValue
    } else {
        0 as RgbValue
    };
    attrs.rgb_bg_color = if attrs.rgb_bg_color != -1 as RgbValue {
        attrs.rgb_bg_color
    } else if dark_ as ::core::ffi::c_int != 0 {
        0 as RgbValue
    } else {
        0xffffff as RgbValue
    };
    attrs.rgb_sp_color = if attrs.rgb_sp_color != -1 as RgbValue {
        attrs.rgb_sp_color
    } else {
        0xff0000 as RgbValue
    };
    if attrs.rgb_ae_attr & HL_INVERSE as ::core::ffi::c_int as int32_t != 0 {
        let mut temp: ::core::ffi::c_int = attrs.rgb_bg_color as ::core::ffi::c_int;
        attrs.rgb_bg_color = attrs.rgb_fg_color;
        attrs.rgb_fg_color = temp as RgbValue;
        attrs.rgb_ae_attr = (attrs.rgb_ae_attr as ::core::ffi::c_int
            & !(HL_INVERSE as ::core::ffi::c_int)) as int32_t;
    }
    return attrs;
}
#[no_mangle]
pub unsafe extern "C" fn hl_blend_attrs(
    mut back_attr: ::core::ffi::c_int,
    mut front_attr: ::core::ffi::c_int,
    mut through: *mut bool,
) -> ::core::ffi::c_int {
    if front_attr < 0 as ::core::ffi::c_int || back_attr < 0 as ::core::ffi::c_int {
        return front_attr;
    }
    let mut fattrs_raw: HlAttrs = syn_attr2entry(front_attr);
    let mut fattrs: HlAttrs = get_colors_force(fattrs_raw);
    let mut ratio: ::core::ffi::c_int = fattrs.hl_blend as ::core::ffi::c_int;
    if ratio <= 0 as ::core::ffi::c_int {
        *through = false_0 != 0;
        return front_attr;
    }
    let mut combine_tag: uint64_t = (back_attr as uint32_t as uint64_t) << 32 as ::core::ffi::c_int
        | front_attr as uint32_t as uint64_t;
    let mut map: *mut Map_uint64_t_int = if *through as ::core::ffi::c_int != 0 {
        blendthrough_attr_entries.ptr()
    } else {
        blend_attr_entries.ptr()
    };
    let mut id: ::core::ffi::c_int = map_get_uint64_t_int(map, combine_tag);
    if id > 0 as ::core::ffi::c_int {
        return id;
    }
    let mut battrs_raw: HlAttrs = syn_attr2entry(back_attr);
    let mut battrs: HlAttrs = get_colors_force(battrs_raw);
    let mut cattrs: HlAttrs = HlAttrs {
        rgb_ae_attr: 0,
        cterm_ae_attr: 0,
        rgb_fg_color: 0,
        rgb_bg_color: 0,
        rgb_sp_color: 0,
        cterm_fg_color: 0,
        cterm_bg_color: 0,
        hl_blend: 0,
        url: 0,
    };
    if *through {
        cattrs = battrs;
        cattrs.rgb_fg_color = rgb_blend(
            ratio,
            battrs.rgb_fg_color as ::core::ffi::c_int,
            fattrs.rgb_bg_color as ::core::ffi::c_int,
        ) as RgbValue;
        if cattrs.rgb_ae_attr & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0
            && battrs_raw.rgb_sp_color != -1 as RgbValue
        {
            cattrs.rgb_sp_color = rgb_blend(
                ratio,
                battrs.rgb_sp_color as ::core::ffi::c_int,
                fattrs.rgb_bg_color as ::core::ffi::c_int,
            ) as RgbValue;
        } else {
            cattrs.rgb_sp_color = -1 as ::core::ffi::c_int as RgbValue;
        }
        cattrs.cterm_bg_color = fattrs.cterm_bg_color;
        cattrs.cterm_fg_color =
            cterm_blend(ratio, battrs.cterm_fg_color, fattrs.cterm_bg_color) as int16_t;
        cattrs.rgb_ae_attr = (cattrs.rgb_ae_attr as ::core::ffi::c_int
            & !(HL_FG_INDEXED as ::core::ffi::c_int | HL_BG_INDEXED as ::core::ffi::c_int))
            as int32_t;
    } else {
        cattrs = fattrs;
        cattrs.rgb_fg_color = rgb_blend(
            ratio / 2 as ::core::ffi::c_int,
            battrs.rgb_fg_color as ::core::ffi::c_int,
            fattrs.rgb_fg_color as ::core::ffi::c_int,
        ) as RgbValue;
        if cattrs.rgb_ae_attr & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
            cattrs.rgb_sp_color = rgb_blend(
                ratio / 2 as ::core::ffi::c_int,
                battrs.rgb_bg_color as ::core::ffi::c_int,
                fattrs.rgb_sp_color as ::core::ffi::c_int,
            ) as RgbValue;
        } else {
            cattrs.rgb_sp_color = -1 as ::core::ffi::c_int as RgbValue;
        }
        cattrs.rgb_ae_attr = (cattrs.rgb_ae_attr as ::core::ffi::c_int
            & !(HL_FG_INDEXED as ::core::ffi::c_int | HL_BG_INDEXED as ::core::ffi::c_int))
            as int32_t;
    }
    if ratio == 100 as ::core::ffi::c_int && battrs_raw.rgb_bg_color == -1 as RgbValue {
        cattrs.rgb_bg_color = -1 as ::core::ffi::c_int as RgbValue;
    } else {
        cattrs.rgb_bg_color = (if battrs_raw.rgb_bg_color == -1 as RgbValue
            && fattrs_raw.rgb_bg_color == -1 as RgbValue
        {
            -1 as ::core::ffi::c_int
        } else {
            rgb_blend(
                ratio,
                battrs.rgb_bg_color as ::core::ffi::c_int,
                fattrs.rgb_bg_color as ::core::ffi::c_int,
            )
        }) as RgbValue;
    }
    cattrs.hl_blend = -1 as ::core::ffi::c_int as int32_t;
    let mut kind: HlKind = (if *through as ::core::ffi::c_int != 0 {
        kHlBlendThrough as ::core::ffi::c_int
    } else {
        kHlBlend as ::core::ffi::c_int
    }) as HlKind;
    id = get_attr_entry(HlEntry {
        attr: cattrs,
        kind: kind,
        id1: back_attr,
        id2: front_attr,
        winid: 0,
    });
    if id > 0 as ::core::ffi::c_int {
        map_put_uint64_t_int(map, combine_tag, id);
    }
    return id;
}
unsafe extern "C" fn rgb_blend(
    mut ratio: ::core::ffi::c_int,
    mut rgb1: ::core::ffi::c_int,
    mut rgb2: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut a: ::core::ffi::c_int = ratio;
    let mut b: ::core::ffi::c_int = 100 as ::core::ffi::c_int - ratio;
    let mut r1: ::core::ffi::c_int =
        (rgb1 & 0xff0000 as ::core::ffi::c_int) >> 16 as ::core::ffi::c_int;
    let mut g1: ::core::ffi::c_int =
        (rgb1 & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
    let mut b1: ::core::ffi::c_int = (rgb1 & 0xff as ::core::ffi::c_int) >> 0 as ::core::ffi::c_int;
    let mut r2: ::core::ffi::c_int =
        (rgb2 & 0xff0000 as ::core::ffi::c_int) >> 16 as ::core::ffi::c_int;
    let mut g2: ::core::ffi::c_int =
        (rgb2 & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
    let mut b2: ::core::ffi::c_int = (rgb2 & 0xff as ::core::ffi::c_int) >> 0 as ::core::ffi::c_int;
    let mut mr: ::core::ffi::c_int = (a * r1 + b * r2) / 100 as ::core::ffi::c_int;
    let mut mg: ::core::ffi::c_int = (a * g1 + b * g2) / 100 as ::core::ffi::c_int;
    let mut mb: ::core::ffi::c_int = (a * b1 + b * b2) / 100 as ::core::ffi::c_int;
    return (mr << 16 as ::core::ffi::c_int) + (mg << 8 as ::core::ffi::c_int) + mb;
}
unsafe extern "C" fn cterm_blend(
    mut ratio: ::core::ffi::c_int,
    mut c1: int16_t,
    mut c2: int16_t,
) -> ::core::ffi::c_int {
    let mut rgb1: ::core::ffi::c_int = hl_cterm2rgb_color(c1 as ::core::ffi::c_int);
    let mut rgb2: ::core::ffi::c_int = hl_cterm2rgb_color(c2 as ::core::ffi::c_int);
    let mut rgb_blended: ::core::ffi::c_int = rgb_blend(ratio, rgb1, rgb2);
    return hl_rgb2cterm_color(rgb_blended);
}
unsafe extern "C" fn hl_rgb2cterm_color(mut rgb: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int =
        (rgb & 0xff0000 as ::core::ffi::c_int) >> 16 as ::core::ffi::c_int;
    let mut g: ::core::ffi::c_int = (rgb & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
    let mut b: ::core::ffi::c_int = (rgb & 0xff as ::core::ffi::c_int) >> 0 as ::core::ffi::c_int;
    return r * 6 as ::core::ffi::c_int / 256 as ::core::ffi::c_int * 36 as ::core::ffi::c_int
        + g * 6 as ::core::ffi::c_int / 256 as ::core::ffi::c_int * 6 as ::core::ffi::c_int
        + b * 6 as ::core::ffi::c_int / 256 as ::core::ffi::c_int;
}
unsafe extern "C" fn hl_cterm2rgb_color(mut nr: ::core::ffi::c_int) -> ::core::ffi::c_int {
    static cube_value: GlobalCell<[::core::ffi::c_int; 6]> = GlobalCell::new([
        0 as ::core::ffi::c_int,
        0x5f as ::core::ffi::c_int,
        0x87 as ::core::ffi::c_int,
        0xaf as ::core::ffi::c_int,
        0xd7 as ::core::ffi::c_int,
        0xff as ::core::ffi::c_int,
    ]);
    static grey_ramp: GlobalCell<[::core::ffi::c_int; 24]> = GlobalCell::new([
        0x8 as ::core::ffi::c_int,
        0x12 as ::core::ffi::c_int,
        0x1c as ::core::ffi::c_int,
        0x26 as ::core::ffi::c_int,
        0x30 as ::core::ffi::c_int,
        0x3a as ::core::ffi::c_int,
        0x44 as ::core::ffi::c_int,
        0x4e as ::core::ffi::c_int,
        0x58 as ::core::ffi::c_int,
        0x62 as ::core::ffi::c_int,
        0x6c as ::core::ffi::c_int,
        0x76 as ::core::ffi::c_int,
        0x80 as ::core::ffi::c_int,
        0x8a as ::core::ffi::c_int,
        0x94 as ::core::ffi::c_int,
        0x9e as ::core::ffi::c_int,
        0xa8 as ::core::ffi::c_int,
        0xb2 as ::core::ffi::c_int,
        0xbc as ::core::ffi::c_int,
        0xc6 as ::core::ffi::c_int,
        0xd0 as ::core::ffi::c_int,
        0xda as ::core::ffi::c_int,
        0xe4 as ::core::ffi::c_int,
        0xee as ::core::ffi::c_int,
    ]);
    static ansi_table: GlobalCell<[[uint8_t; 4]; 16]> = GlobalCell::new([
        [0 as uint8_t, 0 as uint8_t, 0 as uint8_t, 1 as uint8_t],
        [224 as uint8_t, 0 as uint8_t, 0 as uint8_t, 2 as uint8_t],
        [0 as uint8_t, 224 as uint8_t, 0 as uint8_t, 3 as uint8_t],
        [224 as uint8_t, 224 as uint8_t, 0 as uint8_t, 4 as uint8_t],
        [0 as uint8_t, 0 as uint8_t, 224 as uint8_t, 5 as uint8_t],
        [224 as uint8_t, 0 as uint8_t, 224 as uint8_t, 6 as uint8_t],
        [0 as uint8_t, 224 as uint8_t, 224 as uint8_t, 7 as uint8_t],
        [224 as uint8_t, 224 as uint8_t, 224 as uint8_t, 8 as uint8_t],
        [128 as uint8_t, 128 as uint8_t, 128 as uint8_t, 9 as uint8_t],
        [255 as uint8_t, 64 as uint8_t, 64 as uint8_t, 10 as uint8_t],
        [64 as uint8_t, 255 as uint8_t, 64 as uint8_t, 11 as uint8_t],
        [255 as uint8_t, 255 as uint8_t, 64 as uint8_t, 12 as uint8_t],
        [64 as uint8_t, 64 as uint8_t, 255 as uint8_t, 13 as uint8_t],
        [255 as uint8_t, 64 as uint8_t, 255 as uint8_t, 14 as uint8_t],
        [64 as uint8_t, 255 as uint8_t, 255 as uint8_t, 15 as uint8_t],
        [
            255 as uint8_t,
            255 as uint8_t,
            255 as uint8_t,
            16 as uint8_t,
        ],
    ]);
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut g: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut b: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut idx: ::core::ffi::c_int = 0;
    if nr < 16 as ::core::ffi::c_int {
        r = (*ansi_table.ptr())[nr as usize][0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int;
        g = (*ansi_table.ptr())[nr as usize][1 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int;
        b = (*ansi_table.ptr())[nr as usize][2 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int;
    } else if nr < 232 as ::core::ffi::c_int {
        idx = nr - 16 as ::core::ffi::c_int;
        r = (*cube_value.ptr())
            [(idx / 36 as ::core::ffi::c_int % 6 as ::core::ffi::c_int) as usize];
        g = (*cube_value.ptr())[(idx / 6 as ::core::ffi::c_int % 6 as ::core::ffi::c_int) as usize];
        b = (*cube_value.ptr())[(idx % 6 as ::core::ffi::c_int) as usize];
    } else if nr < 256 as ::core::ffi::c_int {
        idx = nr - 232 as ::core::ffi::c_int;
        r = (*grey_ramp.ptr())[idx as usize];
        g = (*grey_ramp.ptr())[idx as usize];
        b = (*grey_ramp.ptr())[idx as usize];
    }
    return (r << 16 as ::core::ffi::c_int) + (g << 8 as ::core::ffi::c_int) + b;
}
#[no_mangle]
pub unsafe extern "C" fn syn_attr2entry(mut attr: ::core::ffi::c_int) -> HlAttrs {
    if attr <= 0 as ::core::ffi::c_int || attr >= (*attr_entries.ptr()).h.size as ::core::ffi::c_int
    {
        return HLATTRS_INIT;
    }
    return (*(*attr_entries.ptr()).keys.offset(attr as isize)).attr;
}
#[no_mangle]
pub unsafe extern "C" fn hl_get_attr_by_id(
    mut attr_id: Integer,
    mut rgb: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut dic: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    if attr_id == 0 as Integer {
        return dic;
    }
    if attr_id < 0 as Integer
        || attr_id >= (*attr_entries.ptr()).h.size as ::core::ffi::c_int as Integer
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"Invalid attribute id: %ld\0".as_ptr() as *const ::core::ffi::c_char,
            attr_id,
        );
        return dic;
    }
    let mut retval: Dict = arena_dict(arena, HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t);
    hlattrs2dict(
        &raw mut retval,
        ::core::ptr::null_mut::<Dict>(),
        syn_attr2entry(attr_id as ::core::ffi::c_int),
        rgb as bool,
        false_0 != 0,
    );
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn hlattrs2dict(
    mut hl: *mut Dict,
    mut hl_attrs: *mut Dict,
    mut ae: HlAttrs,
    mut use_rgb: bool,
    mut short_keys: bool,
) {
    hl_attrs = if !hl_attrs.is_null() { hl_attrs } else { hl };
    '_c2rust_label: {
        if (*hl).capacity >= HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t {
        } else {
            __assert_fail(
                b"hl->capacity >= HLATTRS_DICT_SIZE\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/highlight.rs\0".as_ptr() as *const ::core::ffi::c_char,
                919 as ::core::ffi::c_uint,
                b"void hlattrs2dict(Dict *, Dict *, HlAttrs, _Bool, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if (*hl_attrs).capacity >= HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t {
        } else {
            __assert_fail(
                b"hl_attrs->capacity >= HLATTRS_DICT_SIZE\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/highlight.rs\0".as_ptr() as *const ::core::ffi::c_char,
                920 as ::core::ffi::c_uint,
                b"void hlattrs2dict(Dict *, Dict *, HlAttrs, _Bool, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut mask: ::core::ffi::c_int = if use_rgb as ::core::ffi::c_int != 0 {
        ae.rgb_ae_attr as ::core::ffi::c_int
    } else {
        ae.cterm_ae_attr as ::core::ffi::c_int
    };
    if mask & HL_INVERSE as ::core::ffi::c_int != 0 {
        let c2rust_fresh11 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh11 as isize) = key_value_pair {
            key: cstr_as_string(b"reverse\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    if mask & HL_BOLD as ::core::ffi::c_int != 0 {
        let c2rust_fresh12 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh12 as isize) = key_value_pair {
            key: cstr_as_string(b"bold\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    if mask & HL_ITALIC as ::core::ffi::c_int != 0 {
        let c2rust_fresh13 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh13 as isize) = key_value_pair {
            key: cstr_as_string(b"italic\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    match mask & HL_UNDERLINE_MASK as ::core::ffi::c_int {
        8 => {
            let c2rust_fresh14 = (*hl_attrs).size;
            (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
            *(*hl_attrs).items.offset(c2rust_fresh14 as isize) = key_value_pair {
                key: cstr_as_string(b"underline\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        16 => {
            let c2rust_fresh15 = (*hl_attrs).size;
            (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
            *(*hl_attrs).items.offset(c2rust_fresh15 as isize) = key_value_pair {
                key: cstr_as_string(b"undercurl\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        24 => {
            let c2rust_fresh16 = (*hl_attrs).size;
            (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
            *(*hl_attrs).items.offset(c2rust_fresh16 as isize) = key_value_pair {
                key: cstr_as_string(b"underdouble\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        32 => {
            let c2rust_fresh17 = (*hl_attrs).size;
            (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
            *(*hl_attrs).items.offset(c2rust_fresh17 as isize) = key_value_pair {
                key: cstr_as_string(b"underdotted\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        40 => {
            let c2rust_fresh18 = (*hl_attrs).size;
            (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
            *(*hl_attrs).items.offset(c2rust_fresh18 as isize) = key_value_pair {
                key: cstr_as_string(b"underdashed\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        _ => {}
    }
    if mask & HL_STANDOUT as ::core::ffi::c_int != 0 {
        let c2rust_fresh19 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh19 as isize) = key_value_pair {
            key: cstr_as_string(b"standout\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    if mask & HL_STRIKETHROUGH as ::core::ffi::c_int != 0 {
        let c2rust_fresh20 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh20 as isize) = key_value_pair {
            key: cstr_as_string(b"strikethrough\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    if mask & HL_ALTFONT as ::core::ffi::c_int != 0 {
        let c2rust_fresh21 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"altfont\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    if mask & HL_DIM as ::core::ffi::c_int != 0 {
        let c2rust_fresh22 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh22 as isize) = key_value_pair {
            key: cstr_as_string(b"dim\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    if mask & HL_BLINK as ::core::ffi::c_int != 0 {
        let c2rust_fresh23 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"blink\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    if mask & HL_CONCEALED as ::core::ffi::c_int != 0 {
        let c2rust_fresh24 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh24 as isize) = key_value_pair {
            key: cstr_as_string(b"conceal\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    if mask & HL_OVERLINE as ::core::ffi::c_int != 0 {
        let c2rust_fresh25 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh25 as isize) = key_value_pair {
            key: cstr_as_string(b"overline\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    if mask & HL_NOCOMBINE as ::core::ffi::c_int != 0 {
        let c2rust_fresh26 = (*hl_attrs).size;
        (*hl_attrs).size = (*hl_attrs).size.wrapping_add(1);
        *(*hl_attrs).items.offset(c2rust_fresh26 as isize) = key_value_pair {
            key: cstr_as_string(b"nocombine\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: true },
            },
        };
    }
    if use_rgb {
        if ae.rgb_fg_color != -1 as RgbValue {
            let c2rust_fresh27 = (*hl).size;
            (*hl).size = (*hl).size.wrapping_add(1);
            *(*hl).items.offset(c2rust_fresh27 as isize) = key_value_pair {
                key: cstr_as_string(if short_keys as ::core::ffi::c_int != 0 {
                    b"fg\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"foreground\0".as_ptr() as *const ::core::ffi::c_char
                }),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: ae.rgb_fg_color as Integer,
                    },
                },
            };
        }
        if ae.rgb_bg_color != -1 as RgbValue {
            let c2rust_fresh28 = (*hl).size;
            (*hl).size = (*hl).size.wrapping_add(1);
            *(*hl).items.offset(c2rust_fresh28 as isize) = key_value_pair {
                key: cstr_as_string(if short_keys as ::core::ffi::c_int != 0 {
                    b"bg\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"background\0".as_ptr() as *const ::core::ffi::c_char
                }),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: ae.rgb_bg_color as Integer,
                    },
                },
            };
        }
        if ae.rgb_sp_color != -1 as RgbValue {
            let c2rust_fresh29 = (*hl).size;
            (*hl).size = (*hl).size.wrapping_add(1);
            *(*hl).items.offset(c2rust_fresh29 as isize) = key_value_pair {
                key: cstr_as_string(if short_keys as ::core::ffi::c_int != 0 {
                    b"sp\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"special\0".as_ptr() as *const ::core::ffi::c_char
                }),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: ae.rgb_sp_color as Integer,
                    },
                },
            };
        }
        if mask & HL_FG_INDEXED as ::core::ffi::c_int != 0 {
            let c2rust_fresh30 = (*hl).size;
            (*hl).size = (*hl).size.wrapping_add(1);
            *(*hl).items.offset(c2rust_fresh30 as isize) = key_value_pair {
                key: cstr_as_string(b"fg_indexed\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
        if mask & HL_BG_INDEXED as ::core::ffi::c_int != 0 {
            let c2rust_fresh31 = (*hl).size;
            (*hl).size = (*hl).size.wrapping_add(1);
            *(*hl).items.offset(c2rust_fresh31 as isize) = key_value_pair {
                key: cstr_as_string(b"bg_indexed\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed { boolean: true },
                },
            };
        }
    } else {
        if ae.cterm_fg_color as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            let c2rust_fresh32 = (*hl).size;
            (*hl).size = (*hl).size.wrapping_add(1);
            *(*hl).items.offset(c2rust_fresh32 as isize) = key_value_pair {
                key: cstr_as_string(if short_keys as ::core::ffi::c_int != 0 {
                    b"ctermfg\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"foreground\0".as_ptr() as *const ::core::ffi::c_char
                }),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (ae.cterm_fg_color as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            as Integer,
                    },
                },
            };
        }
        if ae.cterm_bg_color as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            let c2rust_fresh33 = (*hl).size;
            (*hl).size = (*hl).size.wrapping_add(1);
            *(*hl).items.offset(c2rust_fresh33 as isize) = key_value_pair {
                key: cstr_as_string(if short_keys as ::core::ffi::c_int != 0 {
                    b"ctermbg\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"background\0".as_ptr() as *const ::core::ffi::c_char
                }),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (ae.cterm_bg_color as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            as Integer,
                    },
                },
            };
        }
    }
    if ae.hl_blend > -1 as int32_t && (use_rgb as ::core::ffi::c_int != 0 || !short_keys) {
        let c2rust_fresh34 = (*hl).size;
        (*hl).size = (*hl).size.wrapping_add(1);
        *(*hl).items.offset(c2rust_fresh34 as isize) = key_value_pair {
            key: cstr_as_string(b"blend\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: ae.hl_blend as Integer,
                },
            },
        };
    }
}
#[no_mangle]
pub unsafe extern "C" fn dict2hlattrs(
    mut dict: *mut KeyDict_highlight,
    mut use_rgb: bool,
    mut link_id: *mut ::core::ffi::c_int,
    mut base: *mut HlAttrs,
    mut err: *mut Error,
) -> HlAttrs {
    let mut hlattrs: HlAttrs = HLATTRS_INIT;
    let mut fg: int32_t = if !base.is_null() {
        (*base).rgb_fg_color as int32_t
    } else {
        -1 as int32_t
    };
    let mut bg: int32_t = if !base.is_null() {
        (*base).rgb_bg_color as int32_t
    } else {
        -1 as int32_t
    };
    let mut ctermfg: int32_t = if !base.is_null() {
        if (*base).cterm_fg_color as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            -1 as int32_t
        } else {
            (*base).cterm_fg_color as int32_t - 1 as int32_t
        }
    } else {
        -1 as int32_t
    };
    let mut ctermbg: int32_t = if !base.is_null() {
        if (*base).cterm_bg_color as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            -1 as int32_t
        } else {
            (*base).cterm_bg_color as int32_t - 1 as int32_t
        }
    } else {
        -1 as int32_t
    };
    let mut sp: int32_t = if !base.is_null() {
        (*base).rgb_sp_color as int32_t
    } else {
        -1 as int32_t
    };
    let mut blend: ::core::ffi::c_int = if !base.is_null() {
        (*base).hl_blend as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
    let mut mask: int32_t = if !base.is_null() {
        (*base).rgb_ae_attr
    } else {
        0 as int32_t
    };
    let mut cterm_mask: int32_t = if !base.is_null() {
        (*base).cterm_ae_attr
    } else {
        0 as int32_t
    };
    let mut cterm_mask_provided: bool = false_0 != 0;
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__reverse
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag_: int32_t = HL_INVERSE as ::core::ffi::c_int as int32_t;
        let mut cmask_: int32_t = if flag_ & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0
        {
            HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
        } else {
            flag_
        };
        if (*dict).reverse {
            mask = mask & !cmask_ | flag_;
        } else if mask & cmask_ == flag_ {
            mask &= !cmask_;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__bold
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__0: int32_t = HL_BOLD as ::core::ffi::c_int as int32_t;
        let mut cmask__0: int32_t =
            if flag__0 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__0
            };
        if (*dict).bold {
            mask = mask & !cmask__0 | flag__0;
        } else if mask & cmask__0 == flag__0 {
            mask &= !cmask__0;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__italic
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__1: int32_t = HL_ITALIC as ::core::ffi::c_int as int32_t;
        let mut cmask__1: int32_t =
            if flag__1 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__1
            };
        if (*dict).italic {
            mask = mask & !cmask__1 | flag__1;
        } else if mask & cmask__1 == flag__1 {
            mask &= !cmask__1;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__underline
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__2: int32_t = HL_UNDERLINE as ::core::ffi::c_int as int32_t;
        let mut cmask__2: int32_t =
            if flag__2 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__2
            };
        if (*dict).underline {
            mask = mask & !cmask__2 | flag__2;
        } else if mask & cmask__2 == flag__2 {
            mask &= !cmask__2;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__undercurl
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__3: int32_t = HL_UNDERCURL as ::core::ffi::c_int as int32_t;
        let mut cmask__3: int32_t =
            if flag__3 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__3
            };
        if (*dict).undercurl {
            mask = mask & !cmask__3 | flag__3;
        } else if mask & cmask__3 == flag__3 {
            mask &= !cmask__3;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__underdouble
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__4: int32_t = HL_UNDERDOUBLE as ::core::ffi::c_int as int32_t;
        let mut cmask__4: int32_t =
            if flag__4 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__4
            };
        if (*dict).underdouble {
            mask = mask & !cmask__4 | flag__4;
        } else if mask & cmask__4 == flag__4 {
            mask &= !cmask__4;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__underdotted
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__5: int32_t = HL_UNDERDOTTED as ::core::ffi::c_int as int32_t;
        let mut cmask__5: int32_t =
            if flag__5 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__5
            };
        if (*dict).underdotted {
            mask = mask & !cmask__5 | flag__5;
        } else if mask & cmask__5 == flag__5 {
            mask &= !cmask__5;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__underdashed
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__6: int32_t = HL_UNDERDASHED as ::core::ffi::c_int as int32_t;
        let mut cmask__6: int32_t =
            if flag__6 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__6
            };
        if (*dict).underdashed {
            mask = mask & !cmask__6 | flag__6;
        } else if mask & cmask__6 == flag__6 {
            mask &= !cmask__6;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__standout
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__7: int32_t = HL_STANDOUT as ::core::ffi::c_int as int32_t;
        let mut cmask__7: int32_t =
            if flag__7 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__7
            };
        if (*dict).standout {
            mask = mask & !cmask__7 | flag__7;
        } else if mask & cmask__7 == flag__7 {
            mask &= !cmask__7;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__strikethrough
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__8: int32_t = HL_STRIKETHROUGH as ::core::ffi::c_int as int32_t;
        let mut cmask__8: int32_t =
            if flag__8 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__8
            };
        if (*dict).strikethrough {
            mask = mask & !cmask__8 | flag__8;
        } else if mask & cmask__8 == flag__8 {
            mask &= !cmask__8;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__altfont
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__9: int32_t = HL_ALTFONT as ::core::ffi::c_int as int32_t;
        let mut cmask__9: int32_t =
            if flag__9 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__9
            };
        if (*dict).altfont {
            mask = mask & !cmask__9 | flag__9;
        } else if mask & cmask__9 == flag__9 {
            mask &= !cmask__9;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__dim
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__10: int32_t = HL_DIM as ::core::ffi::c_int as int32_t;
        let mut cmask__10: int32_t =
            if flag__10 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__10
            };
        if (*dict).dim {
            mask = mask & !cmask__10 | flag__10;
        } else if mask & cmask__10 == flag__10 {
            mask &= !cmask__10;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__blink
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__11: int32_t = HL_BLINK as ::core::ffi::c_int as int32_t;
        let mut cmask__11: int32_t =
            if flag__11 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__11
            };
        if (*dict).blink {
            mask = mask & !cmask__11 | flag__11;
        } else if mask & cmask__11 == flag__11 {
            mask &= !cmask__11;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__conceal
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__12: int32_t = HL_CONCEALED as ::core::ffi::c_int as int32_t;
        let mut cmask__12: int32_t =
            if flag__12 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__12
            };
        if (*dict).conceal {
            mask = mask & !cmask__12 | flag__12;
        } else if mask & cmask__12 == flag__12 {
            mask &= !cmask__12;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__overline
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__13: int32_t = HL_OVERLINE as ::core::ffi::c_int as int32_t;
        let mut cmask__13: int32_t =
            if flag__13 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__13
            };
        if (*dict).overline {
            mask = mask & !cmask__13 | flag__13;
        } else if mask & cmask__13 == flag__13 {
            mask &= !cmask__13;
        }
    }
    if use_rgb {
        if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__fg_indexed
            != 0 as ::core::ffi::c_ulonglong
        {
            let mut flag__14: int32_t = HL_FG_INDEXED as ::core::ffi::c_int as int32_t;
            let mut cmask__14: int32_t =
                if flag__14 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                    HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
                } else {
                    flag__14
                };
            if (*dict).fg_indexed {
                mask = mask & !cmask__14 | flag__14;
            } else if mask & cmask__14 == flag__14 {
                mask &= !cmask__14;
            }
        }
        if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__bg_indexed
            != 0 as ::core::ffi::c_ulonglong
        {
            let mut flag__15: int32_t = HL_BG_INDEXED as ::core::ffi::c_int as int32_t;
            let mut cmask__15: int32_t =
                if flag__15 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                    HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
                } else {
                    flag__15
                };
            if (*dict).bg_indexed {
                mask = mask & !cmask__15 | flag__15;
            } else if mask & cmask__15 == flag__15 {
                mask &= !cmask__15;
            }
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__nocombine
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__16: int32_t = HL_NOCOMBINE as ::core::ffi::c_int as int32_t;
        let mut cmask__16: int32_t =
            if flag__16 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__16
            };
        if (*dict).nocombine {
            mask = mask & !cmask__16 | flag__16;
        } else if mask & cmask__16 == flag__16 {
            mask &= !cmask__16;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__default
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut flag__17: int32_t = HL_DEFAULT as ::core::ffi::c_int as int32_t;
        let mut cmask__17: int32_t =
            if flag__17 & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0 {
                HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t
            } else {
                flag__17
            };
        if (*dict).default_ {
            mask = mask & !cmask__17 | flag__17;
        } else if mask & cmask__17 == flag__17 {
            mask &= !cmask__17;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__fg
        != 0 as ::core::ffi::c_ulonglong
    {
        fg = object_to_color(
            (*dict).fg,
            b"fg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            use_rgb,
            err,
        ) as int32_t;
    } else if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__foreground
        != 0 as ::core::ffi::c_ulonglong
    {
        fg = object_to_color(
            (*dict).foreground,
            b"foreground\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            use_rgb,
            err,
        ) as int32_t;
    }
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return hlattrs;
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__bg
        != 0 as ::core::ffi::c_ulonglong
    {
        bg = object_to_color(
            (*dict).bg,
            b"bg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            use_rgb,
            err,
        ) as int32_t;
    } else if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__background
        != 0 as ::core::ffi::c_ulonglong
    {
        bg = object_to_color(
            (*dict).background,
            b"background\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            use_rgb,
            err,
        ) as int32_t;
    }
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return hlattrs;
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__sp
        != 0 as ::core::ffi::c_ulonglong
    {
        sp = object_to_color(
            (*dict).sp,
            b"sp\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            true_0 != 0,
            err,
        ) as int32_t;
    } else if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__special
        != 0 as ::core::ffi::c_ulonglong
    {
        sp = object_to_color(
            (*dict).special,
            b"special\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            true_0 != 0,
            err,
        ) as int32_t;
    }
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return hlattrs;
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__blend
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut blend0: Integer = (*dict).blend;
        if !(blend0 >= 0 as Integer && blend0 <= 100 as Integer) {
            api_err_invalid(
                err,
                b"blend\0".as_ptr() as *const ::core::ffi::c_char,
                b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                0 as int64_t,
                false_0 != 0,
            );
            return hlattrs;
        }
        blend = blend0 as ::core::ffi::c_int;
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__link
        != 0 as ::core::ffi::c_ulonglong
        || (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__link_global
            != 0 as ::core::ffi::c_ulonglong
    {
        if link_id.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Invalid Key: '%s'\0".as_ptr() as *const ::core::ffi::c_char,
                if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__link_global
                    != 0 as ::core::ffi::c_ulonglong
                {
                    b"link_global\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"link\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            return hlattrs;
        }
        if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__link_global
            != 0 as ::core::ffi::c_ulonglong
        {
            *link_id = (*dict).link_global as ::core::ffi::c_int;
            mask = (mask as ::core::ffi::c_int | HL_GLOBAL as ::core::ffi::c_int) as int32_t;
        } else {
            *link_id = (*dict).link as ::core::ffi::c_int;
        }
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return hlattrs;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__cterm
        != 0 as ::core::ffi::c_ulonglong
    {
        let mut cterm: [KeyDict_highlight_cterm; 1] = [KeyDict_highlight_cterm {
            bold: false,
            standout: false,
            strikethrough: false,
            underline: false,
            undercurl: false,
            underdouble: false,
            underdotted: false,
            underdashed: false,
            italic: false,
            reverse: false,
            altfont: false,
            dim: false,
            blink: false,
            conceal: false,
            overline: false,
            nocombine: false,
        }];
        if !api_dict_to_keydict(
            &raw mut cterm as *mut KeyDict_highlight_cterm as *mut ::core::ffi::c_void,
            Some(
                KeyDict_highlight_cterm_get_field
                    as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
            ),
            (*dict).cterm,
            err,
        ) {
            return hlattrs;
        }
        cterm_mask_provided = true_0 != 0;
        cterm_mask = 0 as ::core::ffi::c_int as int32_t;
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).reverse {
            if HL_INVERSE as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_INVERSE as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).bold {
            if HL_BOLD as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_BOLD as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).italic {
            if HL_ITALIC as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_ITALIC as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).underline {
            if HL_UNDERLINE as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_UNDERLINE as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).undercurl {
            if HL_UNDERCURL as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_UNDERCURL as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).underdouble {
            if HL_UNDERDOUBLE as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask = (cterm_mask as ::core::ffi::c_int | HL_UNDERDOUBLE as ::core::ffi::c_int)
                as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).underdotted {
            if HL_UNDERDOTTED as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask = (cterm_mask as ::core::ffi::c_int | HL_UNDERDOTTED as ::core::ffi::c_int)
                as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).underdashed {
            if HL_UNDERDASHED as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask = (cterm_mask as ::core::ffi::c_int | HL_UNDERDASHED as ::core::ffi::c_int)
                as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).standout {
            if HL_STANDOUT as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_STANDOUT as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).strikethrough {
            if HL_STRIKETHROUGH as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0
            {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask = (cterm_mask as ::core::ffi::c_int | HL_STRIKETHROUGH as ::core::ffi::c_int)
                as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).altfont {
            if HL_ALTFONT as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_ALTFONT as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).dim {
            if HL_DIM as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_DIM as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).blink {
            if HL_BLINK as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_BLINK as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).conceal {
            if HL_CONCEALED as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_CONCEALED as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).overline {
            if HL_OVERLINE as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_OVERLINE as ::core::ffi::c_int) as int32_t;
        }
        if (*(&raw mut cterm as *mut KeyDict_highlight_cterm)).nocombine {
            if HL_NOCOMBINE as ::core::ffi::c_int & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0 {
                cterm_mask = (cterm_mask as ::core::ffi::c_int
                    & !(HL_UNDERLINE_MASK as ::core::ffi::c_int))
                    as int32_t;
            }
            cterm_mask =
                (cterm_mask as ::core::ffi::c_int | HL_NOCOMBINE as ::core::ffi::c_int) as int32_t;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__ctermfg
        != 0 as ::core::ffi::c_ulonglong
    {
        ctermfg = object_to_color(
            (*dict).ctermfg,
            b"ctermfg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            false_0 != 0,
            err,
        ) as int32_t;
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return hlattrs;
        }
    }
    if (*dict).is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__ctermbg
        != 0 as ::core::ffi::c_ulonglong
    {
        ctermbg = object_to_color(
            (*dict).ctermbg,
            b"ctermbg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            false_0 != 0,
            err,
        ) as int32_t;
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return hlattrs;
        }
    }
    if use_rgb {
        if !cterm_mask_provided {
            cterm_mask = mask;
        }
        hlattrs.rgb_ae_attr = mask;
        hlattrs.rgb_bg_color = bg as RgbValue;
        hlattrs.rgb_fg_color = fg as RgbValue;
        hlattrs.rgb_sp_color = sp as RgbValue;
        hlattrs.hl_blend = blend as int32_t;
        hlattrs.cterm_bg_color = (if ctermbg == -1 as int32_t {
            0 as ::core::ffi::c_int
        } else {
            (ctermbg + 1 as int32_t) as int16_t as ::core::ffi::c_int
        }) as int16_t;
        hlattrs.cterm_fg_color = (if ctermfg == -1 as int32_t {
            0 as ::core::ffi::c_int
        } else {
            (ctermfg + 1 as int32_t) as int16_t as ::core::ffi::c_int
        }) as int16_t;
        hlattrs.cterm_ae_attr = cterm_mask;
    } else {
        hlattrs.cterm_bg_color = (if bg == -1 as int32_t {
            0 as ::core::ffi::c_int
        } else {
            (bg + 1 as int32_t) as int16_t as ::core::ffi::c_int
        }) as int16_t;
        hlattrs.cterm_fg_color = (if fg == -1 as int32_t {
            0 as ::core::ffi::c_int
        } else {
            (fg + 1 as int32_t) as int16_t as ::core::ffi::c_int
        }) as int16_t;
        hlattrs.cterm_ae_attr = mask;
    }
    return hlattrs;
}
#[no_mangle]
pub unsafe extern "C" fn object_to_color(
    mut val: Object,
    mut key: *mut ::core::ffi::c_char,
    mut rgb: bool,
    mut err: *mut Error,
) -> ::core::ffi::c_int {
    if val.type_0 as ::core::ffi::c_uint
        == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return val.data.integer as ::core::ffi::c_int;
    } else if val.type_0 as ::core::ffi::c_uint
        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut str: String_0 = val.data.string;
        if str.size == 0
            || strcasecmp(
                str.data,
                b"NONE\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            return -1 as ::core::ffi::c_int;
        }
        let mut color: ::core::ffi::c_int = 0;
        if rgb {
            let mut dummy: ::core::ffi::c_int = 0;
            color = name_to_color(str.data, &raw mut dummy) as ::core::ffi::c_int;
        } else {
            color = name_to_ctermcolor(str.data);
        }
        if !(color >= 0 as ::core::ffi::c_int) {
            api_err_invalid(
                err,
                b"highlight color\0".as_ptr() as *const ::core::ffi::c_char,
                str.data,
                0 as int64_t,
                true_0 != 0,
            );
            return color;
        }
        return color;
    } else if true {
        api_err_exp(
            err,
            key,
            b"String or Integer\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        return 0 as ::core::ffi::c_int;
    }
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn hl_inspect(mut attr: ::core::ffi::c_int, mut arena: *mut Arena) -> Array {
    if !hlstate_active.get() {
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
    let mut ret: Array = arena_array(arena, hl_inspect_size(attr));
    hl_inspect_impl(&raw mut ret, attr, arena);
    return ret;
}
unsafe extern "C" fn hl_inspect_size(mut attr: ::core::ffi::c_int) -> size_t {
    if attr <= 0 as ::core::ffi::c_int || attr >= (*attr_entries.ptr()).h.size as ::core::ffi::c_int
    {
        return 0 as size_t;
    }
    let mut e: HlEntry = *(*attr_entries.ptr()).keys.offset(attr as isize);
    if e.kind as ::core::ffi::c_uint == kHlCombine as ::core::ffi::c_int as ::core::ffi::c_uint
        || e.kind as ::core::ffi::c_uint == kHlBlend as ::core::ffi::c_int as ::core::ffi::c_uint
        || e.kind as ::core::ffi::c_uint
            == kHlBlendThrough as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return hl_inspect_size(e.id1).wrapping_add(hl_inspect_size(e.id2));
    }
    return 1 as size_t;
}
unsafe extern "C" fn hl_inspect_impl(
    mut arr: *mut Array,
    mut attr: ::core::ffi::c_int,
    mut arena: *mut Arena,
) {
    let mut item: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    if attr <= 0 as ::core::ffi::c_int || attr >= (*attr_entries.ptr()).h.size as ::core::ffi::c_int
    {
        return;
    }
    let mut e: HlEntry = *(*attr_entries.ptr()).keys.offset(attr as isize);
    let mut ui_name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    match e.kind as ::core::ffi::c_uint {
        2 => {
            item = arena_dict(arena, 3 as size_t);
            let c2rust_fresh0 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh0 as isize) = key_value_pair {
                key: cstr_as_string(b"kind\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(b"syntax\0".as_ptr() as *const ::core::ffi::c_char),
                    },
                },
            };
            let c2rust_fresh1 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh1 as isize) = key_value_pair {
                key: cstr_as_string(b"hi_name\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(syn_id2name(e.id1)),
                    },
                },
            };
        }
        1 => {
            item = arena_dict(arena, 4 as size_t);
            let c2rust_fresh2 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh2 as isize) = key_value_pair {
                key: cstr_as_string(b"kind\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(b"ui\0".as_ptr() as *const ::core::ffi::c_char),
                    },
                },
            };
            ui_name = if e.id1 == -1 as ::core::ffi::c_int {
                b"Normal\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                *(hlf_names.ptr() as *mut *const ::core::ffi::c_char).offset(e.id1 as isize)
            };
            let c2rust_fresh3 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh3 as isize) = key_value_pair {
                key: cstr_as_string(b"ui_name\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(ui_name),
                    },
                },
            };
            let c2rust_fresh4 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh4 as isize) = key_value_pair {
                key: cstr_as_string(b"hi_name\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(syn_id2name(e.id2)),
                    },
                },
            };
        }
        3 => {
            item = arena_dict(arena, 2 as size_t);
            let c2rust_fresh5 = item.size;
            item.size = item.size.wrapping_add(1);
            *item.items.offset(c2rust_fresh5 as isize) = key_value_pair {
                key: cstr_as_string(b"kind\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(b"term\0".as_ptr() as *const ::core::ffi::c_char),
                    },
                },
            };
        }
        4 | 5 | 6 => {
            hl_inspect_impl(arr, e.id1, arena);
            hl_inspect_impl(arr, e.id2, arena);
            return;
        }
        0 | 7 => return,
        _ => {}
    }
    let c2rust_fresh6 = item.size;
    item.size = item.size.wrapping_add(1);
    *item.items.offset(c2rust_fresh6 as isize) = key_value_pair {
        key: cstr_as_string(b"id\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: attr as Integer,
            },
        },
    };
    let c2rust_fresh7 = (*arr).size;
    (*arr).size = (*arr).size.wrapping_add(1);
    *(*arr).items.offset(c2rust_fresh7 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: item },
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
