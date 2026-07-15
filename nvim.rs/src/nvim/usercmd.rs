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
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcat(
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
    fn strchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn arena_string(arena: *mut Arena, str: String_0) -> String_0;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_verbose: OptInt;
    fn xstrnsave(
        string: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn arena_printf(arena: *mut Arena, fmt: *const ::core::ffi::c_char, ...) -> String_0;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits_int(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn last_set_msg(script_ctx: sctx_T);
    fn do_cmdline(
        cmdline: *mut ::core::ffi::c_char,
        fgetline: LineGetter,
        cookie: *mut ::core::ffi::c_void,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ends_excmd(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(
        gap: *mut garray_T,
        itemsize: ::core::ffi::c_int,
        growsize: ::core::ffi::c_int,
    );
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut Columns: ::core::ffi::c_int;
    static mut current_sctx: sctx_T;
    static mut curtab: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut cmdmod: cmdmod_T;
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut got_int: bool;
    fn replace_termcodes(
        from: *const ::core::ffi::c_char,
        from_len: size_t,
        bufp: *mut *mut ::core::ffi::c_char,
        sid_arg: scid_T,
        flags: ::core::ffi::c_int,
        did_simplify: *mut bool,
        cpo_val: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn api_free_luaref(ref_0: LuaRef);
    fn api_new_luaref(original_ref: LuaRef) -> LuaRef;
    fn nlua_set_sctx(current: *mut sctx_T);
    fn nlua_do_ucmd(
        cmd: *mut ucmd_T,
        eap: *mut exarg_T,
        preview: bool,
    ) -> ::core::ffi::c_int;
    fn nlua_funcref_str(ref_0: LuaRef, arena: *mut Arena) -> *mut ::core::ffi::c_char;
    fn set_context_in_map_cmd(
        xp: *mut expand_T,
        cmd: *mut ::core::ffi::c_char,
        arg: *mut ::core::ffi::c_char,
        forceit: bool,
        isabbrev: bool,
        isunmap: bool,
        cmdidx: cmdidx_T,
    ) -> *mut ::core::ffi::c_char;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_copy_char(
        fp: *mut *const ::core::ffi::c_char,
        tp: *mut *mut ::core::ffi::c_char,
    );
    fn set_context_in_menu_cmd(
        xp: *mut expand_T,
        cmd: *const ::core::ffi::c_char,
        arg: *mut ::core::ffi::c_char,
        forceit: bool,
    ) -> *mut ::core::ffi::c_char;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn msg_ext_set_kind(msg_kind: *const ::core::ffi::c_char);
    fn msg_putchar(c: ::core::ffi::c_int);
    fn msg_outtrans(
        str: *const ::core::ffi::c_char,
        hl_id: ::core::ffi::c_int,
        hist: bool,
    ) -> ::core::ffi::c_int;
    fn msg_outtrans_special(
        strstart: *const ::core::ffi::c_char,
        from: bool,
        maxlen: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn msg_puts(s: *const ::core::ffi::c_char);
    fn msg_puts_title(s: *const ::core::ffi::c_char);
    fn msg_puts_hl(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int, hist: bool);
    fn message_filtered(msg_0: *const ::core::ffi::c_char) -> bool;
    static mut exestack: garray_T;
    fn line_breakcheck();
    fn prevwin_curwin() -> *mut win_T;
    fn tabpage_index(ftp: *mut tabpage_T) -> ::core::ffi::c_int;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_13 = 76;
pub const HLF_PRE: C2Rust_Unnamed_13 = 75;
pub const HLF_OK: C2Rust_Unnamed_13 = 74;
pub const HLF_SO: C2Rust_Unnamed_13 = 73;
pub const HLF_SE: C2Rust_Unnamed_13 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_13 = 71;
pub const HLF_TS: C2Rust_Unnamed_13 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_13 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_13 = 68;
pub const HLF_CU: C2Rust_Unnamed_13 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_13 = 66;
pub const HLF_WBR: C2Rust_Unnamed_13 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_13 = 64;
pub const HLF_MSG: C2Rust_Unnamed_13 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_13 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_13 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_13 = 60;
pub const HLF_0: C2Rust_Unnamed_13 = 59;
pub const HLF_QFL: C2Rust_Unnamed_13 = 58;
pub const HLF_MC: C2Rust_Unnamed_13 = 57;
pub const HLF_CUL: C2Rust_Unnamed_13 = 56;
pub const HLF_CUC: C2Rust_Unnamed_13 = 55;
pub const HLF_TPF: C2Rust_Unnamed_13 = 54;
pub const HLF_TPS: C2Rust_Unnamed_13 = 53;
pub const HLF_TP: C2Rust_Unnamed_13 = 52;
pub const HLF_PBR: C2Rust_Unnamed_13 = 51;
pub const HLF_PST: C2Rust_Unnamed_13 = 50;
pub const HLF_PSB: C2Rust_Unnamed_13 = 49;
pub const HLF_PSX: C2Rust_Unnamed_13 = 48;
pub const HLF_PNX: C2Rust_Unnamed_13 = 47;
pub const HLF_PSK: C2Rust_Unnamed_13 = 46;
pub const HLF_PNK: C2Rust_Unnamed_13 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_13 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_13 = 43;
pub const HLF_PSI: C2Rust_Unnamed_13 = 42;
pub const HLF_PNI: C2Rust_Unnamed_13 = 41;
pub const HLF_SPL: C2Rust_Unnamed_13 = 40;
pub const HLF_SPR: C2Rust_Unnamed_13 = 39;
pub const HLF_SPC: C2Rust_Unnamed_13 = 38;
pub const HLF_SPB: C2Rust_Unnamed_13 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_13 = 36;
pub const HLF_SC: C2Rust_Unnamed_13 = 35;
pub const HLF_TXA: C2Rust_Unnamed_13 = 34;
pub const HLF_TXD: C2Rust_Unnamed_13 = 33;
pub const HLF_DED: C2Rust_Unnamed_13 = 32;
pub const HLF_CHD: C2Rust_Unnamed_13 = 31;
pub const HLF_ADD: C2Rust_Unnamed_13 = 30;
pub const HLF_FC: C2Rust_Unnamed_13 = 29;
pub const HLF_FL: C2Rust_Unnamed_13 = 28;
pub const HLF_WM: C2Rust_Unnamed_13 = 27;
pub const HLF_W: C2Rust_Unnamed_13 = 26;
pub const HLF_VNC: C2Rust_Unnamed_13 = 25;
pub const HLF_V: C2Rust_Unnamed_13 = 24;
pub const HLF_T: C2Rust_Unnamed_13 = 23;
pub const HLF_VSP: C2Rust_Unnamed_13 = 22;
pub const HLF_C: C2Rust_Unnamed_13 = 21;
pub const HLF_SNC: C2Rust_Unnamed_13 = 20;
pub const HLF_S: C2Rust_Unnamed_13 = 19;
pub const HLF_R: C2Rust_Unnamed_13 = 18;
pub const HLF_CLF: C2Rust_Unnamed_13 = 17;
pub const HLF_CLS: C2Rust_Unnamed_13 = 16;
pub const HLF_CLN: C2Rust_Unnamed_13 = 15;
pub const HLF_LNB: C2Rust_Unnamed_13 = 14;
pub const HLF_LNA: C2Rust_Unnamed_13 = 13;
pub const HLF_N: C2Rust_Unnamed_13 = 12;
pub const HLF_CM: C2Rust_Unnamed_13 = 11;
pub const HLF_M: C2Rust_Unnamed_13 = 10;
pub const HLF_LC: C2Rust_Unnamed_13 = 9;
pub const HLF_L: C2Rust_Unnamed_13 = 8;
pub const HLF_I: C2Rust_Unnamed_13 = 7;
pub const HLF_E: C2Rust_Unnamed_13 = 6;
pub const HLF_D: C2Rust_Unnamed_13 = 5;
pub const HLF_AT: C2Rust_Unnamed_13 = 4;
pub const HLF_TERM: C2Rust_Unnamed_13 = 3;
pub const HLF_EOB: C2Rust_Unnamed_13 = 2;
pub const HLF_8: C2Rust_Unnamed_13 = 1;
pub const HLF_NONE: C2Rust_Unnamed_13 = 0;
pub type Direction = ::core::ffi::c_int;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub type xp_prefix_T = ::core::ffi::c_uint;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expand_T {
    pub xp_pattern: *mut ::core::ffi::c_char,
    pub xp_context: ::core::ffi::c_int,
    pub xp_pattern_len: size_t,
    pub xp_prefix: xp_prefix_T,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub xp_luaref: LuaRef,
    pub xp_script_ctx: sctx_T,
    pub xp_backslash: ::core::ffi::c_int,
    pub xp_shell: bool,
    pub xp_numfiles: ::core::ffi::c_int,
    pub xp_col: ::core::ffi::c_int,
    pub xp_selected: ::core::ffi::c_int,
    pub xp_orig: *mut ::core::ffi::c_char,
    pub xp_files: *mut *mut ::core::ffi::c_char,
    pub xp_line: *mut ::core::ffi::c_char,
    pub xp_buf: [::core::ffi::c_char; 256],
    pub xp_search_dir: Direction,
    pub xp_pre_incsearch_pos: pos_T,
}
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_14 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_14 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_14 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_14 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_14 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_14 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_14 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_14 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_14 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_14 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_14 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_14 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_14 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_14 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_14 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_14 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_14 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_14 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_14 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_14 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_14 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_14 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_14 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_14 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_14 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_14 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_14 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_14 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_14 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_14 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_14 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_14 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_14 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_14 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_14 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_14 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_14 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_14 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_14 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_14 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_14 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_14 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_14 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_14 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_14 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_14 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_14 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_14 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_14 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_14 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_14 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_14 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_14 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_14 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_14 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_14 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_14 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_14 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_14 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_14 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_14 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_14 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_14 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_14 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_14 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_14 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_14 = -2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_T {
    pub regprog: *mut regprog_T,
    pub startp: [*mut ::core::ffi::c_char; 10],
    pub endp: [*mut ::core::ffi::c_char; 10],
    pub rm_matchcol: colnr_T,
    pub rm_ic: bool,
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
pub struct eslist_elem {
    pub saved_emsg_silent: ::core::ffi::c_int,
    pub next: *mut eslist_T,
}
pub type eslist_T = eslist_elem;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cstack_T {
    pub cs_flags: [::core::ffi::c_int; 50],
    pub cs_pending: [::core::ffi::c_char; 50],
    pub cs_pend: C2Rust_Unnamed_15,
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
pub union C2Rust_Unnamed_15 {
    pub csp_rv: [*mut ::core::ffi::c_void; 50],
    pub csp_ex: [*mut ::core::ffi::c_void; 50],
}
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_16 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_16 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_16 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_16 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_16 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_16 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_16 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_16 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_16 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_16 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_16 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_16 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_16 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_16 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdmod_T {
    pub cmod_flags: ::core::ffi::c_int,
    pub cmod_split: ::core::ffi::c_int,
    pub cmod_tab: ::core::ffi::c_int,
    pub cmod_filter_pat: *mut ::core::ffi::c_char,
    pub cmod_filter_regmatch: regmatch_T,
    pub cmod_filter_force: bool,
    pub cmod_verbose: ::core::ffi::c_int,
    pub cmod_save_ei: *mut ::core::ffi::c_char,
    pub cmod_did_sandbox: ::core::ffi::c_int,
    pub cmod_verbose_save: OptInt,
    pub cmod_save_msg_silent: ::core::ffi::c_int,
    pub cmod_save_msg_scroll: ::core::ffi::c_int,
    pub cmod_did_esilent: ::core::ffi::c_int,
}
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_17 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_17 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_17 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_17 = 4;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_17 = 2;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_17 = 1;
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
pub type event_T = auto_event;
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
pub type AutoPatCmd = AutoPatCmd_S;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: C2Rust_Unnamed_18,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_18 {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ucmd_T {
    pub uc_name: *mut ::core::ffi::c_char,
    pub uc_argt: uint32_t,
    pub uc_rep: *mut ::core::ffi::c_char,
    pub uc_def: int64_t,
    pub uc_compl: ::core::ffi::c_int,
    pub uc_addr_type: cmd_addr_T,
    pub uc_script_ctx: sctx_T,
    pub uc_compl_arg: *mut ::core::ffi::c_char,
    pub uc_compl_luaref: LuaRef,
    pub uc_preview_luaref: LuaRef,
    pub uc_luaref: LuaRef,
}
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const UC_BUFFER: C2Rust_Unnamed_19 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_20 {
    pub expand: cmd_addr_T,
    pub name: *mut ::core::ffi::c_char,
    pub shortname: *mut ::core::ffi::c_char,
}
pub const WSP_HOR: C2Rust_Unnamed_22 = 4;
pub const WSP_VERT: C2Rust_Unnamed_22 = 2;
pub const WSP_TOP: C2Rust_Unnamed_22 = 8;
pub const WSP_BOT: C2Rust_Unnamed_22 = 16;
pub const WSP_BELOW: C2Rust_Unnamed_22 = 64;
pub const WSP_ABOVE: C2Rust_Unnamed_22 = 128;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mod_entry_T {
    pub flag: ::core::ffi::c_int,
    pub name: *mut ::core::ffi::c_char,
}
pub const ct_LT: C2Rust_Unnamed_21 = 8;
pub const ct_REGISTER: C2Rust_Unnamed_21 = 7;
pub const ct_MODS: C2Rust_Unnamed_21 = 6;
pub const ct_RANGE: C2Rust_Unnamed_21 = 5;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const ct_NONE: C2Rust_Unnamed_21 = 9;
pub const ct_LINE2: C2Rust_Unnamed_21 = 4;
pub const ct_LINE1: C2Rust_Unnamed_21 = 3;
pub const ct_COUNT: C2Rust_Unnamed_21 = 2;
pub const ct_BANG: C2Rust_Unnamed_21 = 1;
pub const ct_ARGS: C2Rust_Unnamed_21 = 0;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_22 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_22 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_22 = 256;
pub const WSP_HELP: C2Rust_Unnamed_22 = 32;
pub const WSP_ROOM: C2Rust_Unnamed_22 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn ascii_isdigit(mut c: ::core::ffi::c_int) -> bool {
    return c >= '0' as ::core::ffi::c_int && c <= '9' as ::core::ffi::c_int;
}
pub const EX_RANGE: ::core::ffi::c_uint = 0x1 as ::core::ffi::c_uint;
pub const EX_BANG: ::core::ffi::c_uint = 0x2 as ::core::ffi::c_uint;
pub const EX_EXTRA: ::core::ffi::c_uint = 0x4 as ::core::ffi::c_uint;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const EX_DFLALL: ::core::ffi::c_uint = 0x20 as ::core::ffi::c_uint;
pub const EX_NEEDARG: ::core::ffi::c_uint = 0x80 as ::core::ffi::c_uint;
pub const EX_TRLBAR: ::core::ffi::c_uint = 0x100 as ::core::ffi::c_uint;
pub const EX_REGSTR: ::core::ffi::c_uint = 0x200 as ::core::ffi::c_uint;
pub const EX_COUNT: ::core::ffi::c_uint = 0x400 as ::core::ffi::c_uint;
pub const EX_ZEROR: ::core::ffi::c_uint = 0x1000 as ::core::ffi::c_uint;
pub const EX_BUFNAME: ::core::ffi::c_uint = 0x8000 as ::core::ffi::c_uint;
pub const EX_KEEPSCRIPT: ::core::ffi::c_uint = 0x4000000 as ::core::ffi::c_uint;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int
    + 1 as ::core::ffi::c_int;
pub const K_SPECIAL: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
#[no_mangle]
pub static mut ucmds: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<ucmd_T>() as ::core::ffi::c_int,
    ga_growsize: 4 as ::core::ffi::c_int,
    ga_data: NULL,
};
static mut e_argument_required_for_str: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<
        [u8; 31],
        [::core::ffi::c_char; 31],
    >(*b"E179: Argument required for %s\0")
};
static mut e_no_such_user_defined_command_str: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 39],
        [::core::ffi::c_char; 39],
    >(*b"E184: No such user-defined command: %s\0")
};
static mut e_complete_used_without_allowing_arguments: [::core::ffi::c_char; 49] = unsafe {
    ::core::mem::transmute::<
        [u8; 49],
        [::core::ffi::c_char; 49],
    >(*b"E1208: -complete used without allowing arguments\0")
};
static mut e_no_such_user_defined_command_in_current_buffer_str: [::core::ffi::c_char; 58] = unsafe {
    ::core::mem::transmute::<
        [u8; 58],
        [::core::ffi::c_char; 58],
    >(*b"E1237: No such user-defined command in current buffer: %s\0")
};
static mut command_complete: [*const ::core::ffi::c_char; 64] = [
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"command\0".as_ptr() as *const ::core::ffi::c_char,
    b"file\0".as_ptr() as *const ::core::ffi::c_char,
    b"dir\0".as_ptr() as *const ::core::ffi::c_char,
    b"option\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"tag\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"help\0".as_ptr() as *const ::core::ffi::c_char,
    b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
    b"event\0".as_ptr() as *const ::core::ffi::c_char,
    b"menu\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"highlight\0".as_ptr() as *const ::core::ffi::c_char,
    b"augroup\0".as_ptr() as *const ::core::ffi::c_char,
    b"var\0".as_ptr() as *const ::core::ffi::c_char,
    b"mapping\0".as_ptr() as *const ::core::ffi::c_char,
    b"tag_listfiles\0".as_ptr() as *const ::core::ffi::c_char,
    b"function\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"expression\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"environment\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"color\0".as_ptr() as *const ::core::ffi::c_char,
    b"compiler\0".as_ptr() as *const ::core::ffi::c_char,
    b"custom\0".as_ptr() as *const ::core::ffi::c_char,
    b"customlist\0".as_ptr() as *const ::core::ffi::c_char,
    b"<Lua function>\0".as_ptr() as *const ::core::ffi::c_char,
    b"shellcmd\0".as_ptr() as *const ::core::ffi::c_char,
    b"sign\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"filetype\0".as_ptr() as *const ::core::ffi::c_char,
    b"file_in_path\0".as_ptr() as *const ::core::ffi::c_char,
    b"syntax\0".as_ptr() as *const ::core::ffi::c_char,
    b"locale\0".as_ptr() as *const ::core::ffi::c_char,
    b"history\0".as_ptr() as *const ::core::ffi::c_char,
    b"user\0".as_ptr() as *const ::core::ffi::c_char,
    b"syntime\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"packadd\0".as_ptr() as *const ::core::ffi::c_char,
    b"messages\0".as_ptr() as *const ::core::ffi::c_char,
    b"mapclear\0".as_ptr() as *const ::core::ffi::c_char,
    b"arglist\0".as_ptr() as *const ::core::ffi::c_char,
    b"diff_buffer\0".as_ptr() as *const ::core::ffi::c_char,
    b"breakpoint\0".as_ptr() as *const ::core::ffi::c_char,
    b"scriptnames\0".as_ptr() as *const ::core::ffi::c_char,
    b"runtime\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"keymap\0".as_ptr() as *const ::core::ffi::c_char,
    b"dir_in_path\0".as_ptr() as *const ::core::ffi::c_char,
    b"shellcmdline\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"filetypecmd\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"retab\0".as_ptr() as *const ::core::ffi::c_char,
    b"checkhealth\0".as_ptr() as *const ::core::ffi::c_char,
    b"lua\0".as_ptr() as *const ::core::ffi::c_char,
];
static mut addr_type_complete: [C2Rust_Unnamed_20; 9] = [
    C2Rust_Unnamed_20 {
        expand: ADDR_ARGUMENTS,
        name: b"arguments\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shortname: b"arg\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_LINES,
        name: b"lines\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shortname: b"line\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_LOADED_BUFFERS,
        name: b"loaded_buffers\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shortname: b"load\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_TABS,
        name: b"tabs\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shortname: b"tab\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_BUFFERS,
        name: b"buffers\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shortname: b"buf\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_WINDOWS,
        name: b"windows\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shortname: b"win\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_QUICKFIX,
        name: b"quickfix\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shortname: b"qf\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_OTHER,
        name: b"other\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shortname: b"?\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_NONE,
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        shortname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
];
#[no_mangle]
pub unsafe extern "C" fn find_ucmd(
    mut eap: *mut exarg_T,
    mut p: *mut ::core::ffi::c_char,
    mut full: *mut ::core::ffi::c_int,
    mut xp: *mut expand_T,
    mut complp: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut len: ::core::ffi::c_int = p.offset_from((*eap).cmd) as ::core::ffi::c_int;
    let mut matchlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found: bool = false_0 != 0;
    let mut possible: bool = false_0 != 0;
    let mut amb_local: bool = false_0 != 0;
    let mut gap: *mut garray_T = &raw mut (*(*(prevwin_curwin
        as unsafe extern "C" fn() -> *mut win_T)())
        .w_buffer)
        .b_ucmds;
    loop {
        let mut j: ::core::ffi::c_int = 0;
        j = 0 as ::core::ffi::c_int;
        while j < (*gap).ga_len {
            let mut uc: *mut ucmd_T = ((*gap).ga_data as *mut ucmd_T).offset(j as isize);
            let mut cp: *mut ::core::ffi::c_char = (*eap).cmd;
            let mut np: *mut ::core::ffi::c_char = (*uc).uc_name;
            let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while k < len && *np as ::core::ffi::c_int != NUL
                && {
                    let c2rust_fresh0 = cp;
                    cp = cp.offset(1);
                    let c2rust_fresh1 = np;
                    np = np.offset(1);
                    *c2rust_fresh0 as ::core::ffi::c_int
                        == *c2rust_fresh1 as ::core::ffi::c_int
                }
            {
                k += 1;
            }
            if k == len
                || *np as ::core::ffi::c_int == NUL
                    && ascii_isdigit(
                        *(*eap).cmd.offset(k as isize) as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int != 0
            {
                if k == len && found as ::core::ffi::c_int != 0
                    && *np as ::core::ffi::c_int != NUL
                {
                    if gap == &raw mut ucmds {
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    amb_local = true_0 != 0;
                }
                if !found || k == len && *np as ::core::ffi::c_int == NUL {
                    if k == len {
                        found = true_0 != 0;
                    } else {
                        possible = true_0 != 0;
                    }
                    if gap == &raw mut ucmds {
                        (*eap).cmdidx = CMD_USER;
                    } else {
                        (*eap).cmdidx = CMD_USER_BUF;
                    }
                    (*eap).argt = (*uc).uc_argt;
                    (*eap).useridx = j;
                    (*eap).addr_type = (*uc).uc_addr_type;
                    if !complp.is_null() {
                        *complp = (*uc).uc_compl;
                    }
                    if !xp.is_null() {
                        (*xp).xp_luaref = (*uc).uc_compl_luaref;
                        (*xp).xp_arg = (*uc).uc_compl_arg;
                        (*xp).xp_script_ctx = (*uc).uc_script_ctx;
                        (*xp).xp_script_ctx.sc_lnum
                            += (*(exestack.ga_data as *mut estack_T)
                                .offset(
                                    (exestack.ga_len - 1 as ::core::ffi::c_int) as isize,
                                ))
                                .es_lnum;
                    }
                    matchlen = k;
                    if k == len && *np as ::core::ffi::c_int == NUL {
                        if !full.is_null() {
                            *full = true_0;
                        }
                        amb_local = false_0 != 0;
                        break;
                    }
                }
            }
            j += 1;
        }
        if j < (*gap).ga_len || gap == &raw mut ucmds {
            break;
        }
        gap = &raw mut ucmds;
    }
    if amb_local {
        if !xp.is_null() {
            (*xp).xp_context = EXPAND_UNSUCCESSFUL as ::core::ffi::c_int;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if found as ::core::ffi::c_int != 0 || possible as ::core::ffi::c_int != 0 {
        return p.offset((matchlen - len) as isize);
    }
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn set_context_in_user_cmd(
    mut xp: *mut expand_T,
    mut arg_in: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut arg: *const ::core::ffi::c_char = arg_in;
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    while *arg as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
        arg = arg.offset(1);
        p = skiptowhite(arg);
        if *p as ::core::ffi::c_int == NUL {
            p = strchr(arg, '=' as ::core::ffi::c_int);
            if p.is_null() {
                (*xp).xp_context = EXPAND_USER_CMD_FLAGS as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            if strncasecmp(
                arg as *mut ::core::ffi::c_char,
                b"complete\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                (*xp).xp_context = EXPAND_USER_COMPLETE as ::core::ffi::c_int;
                (*xp).xp_pattern = (p as *mut ::core::ffi::c_char)
                    .offset(1 as ::core::ffi::c_int as isize);
                return ::core::ptr::null::<::core::ffi::c_char>();
            } else if strncasecmp(
                arg as *mut ::core::ffi::c_char,
                b"nargs\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                (*xp).xp_context = EXPAND_USER_NARGS as ::core::ffi::c_int;
                (*xp).xp_pattern = (p as *mut ::core::ffi::c_char)
                    .offset(1 as ::core::ffi::c_int as isize);
                return ::core::ptr::null::<::core::ffi::c_char>();
            } else if strncasecmp(
                arg as *mut ::core::ffi::c_char,
                b"addr\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                (*xp).xp_context = EXPAND_USER_ADDR_TYPE as ::core::ffi::c_int;
                (*xp).xp_pattern = (p as *mut ::core::ffi::c_char)
                    .offset(1 as ::core::ffi::c_int as isize);
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        arg = skipwhite(p);
    }
    p = skiptowhite(arg);
    if *p as ::core::ffi::c_int == NUL {
        (*xp).xp_context = EXPAND_USER_COMMANDS as ::core::ffi::c_int;
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return skipwhite(p);
}
#[no_mangle]
pub unsafe extern "C" fn set_context_in_user_cmdarg(
    mut cmd: *const ::core::ffi::c_char,
    mut arg: *const ::core::ffi::c_char,
    mut argt: uint32_t,
    mut context: ::core::ffi::c_int,
    mut xp: *mut expand_T,
    mut forceit: bool,
) -> *const ::core::ffi::c_char {
    if context == EXPAND_NOTHING as ::core::ffi::c_int {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if argt & EX_XFILE as uint32_t != 0 {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if context == EXPAND_MENUS as ::core::ffi::c_int {
        return set_context_in_menu_cmd(
            xp,
            cmd,
            arg as *mut ::core::ffi::c_char,
            forceit,
        );
    }
    if context == EXPAND_COMMANDS as ::core::ffi::c_int {
        return arg;
    }
    if context == EXPAND_MAPPINGS as ::core::ffi::c_int {
        return set_context_in_map_cmd(
            xp,
            b"map\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            arg as *mut ::core::ffi::c_char,
            forceit,
            false_0 != 0,
            false_0 != 0,
            CMD_map,
        );
    }
    let mut p: *const ::core::ffi::c_char = arg;
    while *p != 0 {
        if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
            arg = p.offset(1 as ::core::ffi::c_int as isize);
        } else if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    (*xp).xp_context = context;
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn expand_user_command_name(
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return get_user_commands(
        ::core::ptr::null_mut::<expand_T>(),
        idx - CMD_SIZE as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn get_user_commands(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let buf: *const buf_T = (*prevwin_curwin()).w_buffer;
    if idx < (*buf).b_ucmds.ga_len {
        return (*((*buf).b_ucmds.ga_data as *mut ucmd_T).offset(idx as isize)).uc_name;
    }
    idx -= (*buf).b_ucmds.ga_len;
    if idx < ucmds.ga_len {
        let mut name: *mut ::core::ffi::c_char = (*(ucmds.ga_data as *mut ucmd_T)
            .offset(idx as isize))
            .uc_name;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*buf).b_ucmds.ga_len {
            if strcmp(
                name,
                (*((*buf).b_ucmds.ga_data as *mut ucmd_T).offset(i as isize)).uc_name,
            ) == 0 as ::core::ffi::c_int
            {
                return b"\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            i += 1;
        }
        return name;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn get_user_command_name(
    mut idx: ::core::ffi::c_int,
    mut cmdidx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if cmdidx == CMD_USER as ::core::ffi::c_int && idx < ucmds.ga_len {
        return (*(ucmds.ga_data as *mut ucmd_T).offset(idx as isize)).uc_name;
    }
    if cmdidx == CMD_USER_BUF as ::core::ffi::c_int {
        let buf: *const buf_T = (*prevwin_curwin()).w_buffer;
        if idx < (*buf).b_ucmds.ga_len {
            return (*((*buf).b_ucmds.ga_data as *mut ucmd_T).offset(idx as isize))
                .uc_name;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn get_user_cmd_addr_type(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return addr_type_complete[idx as usize].name;
}
#[no_mangle]
pub unsafe extern "C" fn get_user_cmd_flags(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static mut user_cmd_flags: [*mut ::core::ffi::c_char; 10] = [
        b"addr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"bang\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"bar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"complete\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"count\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"nargs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"range\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"register\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"keepscript\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
    ];
    if idx
        >= ::core::mem::size_of::<[*mut ::core::ffi::c_char; 10]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 10]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return user_cmd_flags[idx as usize];
}
#[no_mangle]
pub unsafe extern "C" fn get_user_cmd_nargs(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static mut user_cmd_nargs: [*mut ::core::ffi::c_char; 5] = [
        b"0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"?\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ];
    if idx
        >= ::core::mem::size_of::<[*mut ::core::ffi::c_char; 5]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 5]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return user_cmd_nargs[idx as usize];
}
unsafe extern "C" fn get_command_complete(
    mut arg: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if arg < 0 as ::core::ffi::c_int
        || arg
            >= ::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                        .wrapping_rem(
                            ::core::mem::size_of::<*const ::core::ffi::c_char>(),
                        ) == 0) as ::core::ffi::c_int as usize,
                ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return command_complete[arg as usize] as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn get_user_cmd_complete(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx
        >= ::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut cmd_compl: *mut ::core::ffi::c_char = get_command_complete(idx);
    if cmd_compl.is_null() || idx == EXPAND_USER_LUA as ::core::ffi::c_int {
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return cmd_compl;
}
#[no_mangle]
pub unsafe extern "C" fn cmdcomplete_type_to_str(
    mut expand: ::core::ffi::c_int,
    mut compl_arg: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut cmd_compl: *mut ::core::ffi::c_char = get_command_complete(expand);
    if cmd_compl.is_null() || expand == EXPAND_USER_LUA as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if expand == EXPAND_USER_LIST as ::core::ffi::c_int
        || expand == EXPAND_USER_DEFINED as ::core::ffi::c_int
    {
        let mut buflen: size_t = strlen(cmd_compl)
            .wrapping_add(strlen(compl_arg))
            .wrapping_add(2 as size_t);
        let mut buffer: *mut ::core::ffi::c_char = xmalloc(buflen)
            as *mut ::core::ffi::c_char;
        snprintf(
            buffer,
            buflen,
            b"%s,%s\0".as_ptr() as *const ::core::ffi::c_char,
            cmd_compl,
            compl_arg,
        );
        return buffer;
    }
    return xstrdup(cmd_compl);
}
#[no_mangle]
pub unsafe extern "C" fn cmdcomplete_str_to_type(
    mut complete_str: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if strncmp(
        complete_str,
        b"custom,\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return EXPAND_USER_DEFINED as ::core::ffi::c_int;
    }
    if strncmp(
        complete_str,
        b"customlist,\0".as_ptr() as *const ::core::ffi::c_char,
        11 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return EXPAND_USER_LIST as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i
        < ::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        let mut cmd_compl: *mut ::core::ffi::c_char = get_command_complete(i);
        if !cmd_compl.is_null() {
            if strcmp(complete_str, command_complete[i as usize])
                == 0 as ::core::ffi::c_int
            {
                return i;
            }
        }
        i += 1;
    }
    return EXPAND_NOTHING as ::core::ffi::c_int;
}
unsafe extern "C" fn uc_list(mut name: *mut ::core::ffi::c_char, mut name_len: size_t) {
    let mut found: bool = false_0 != 0;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    let mut gap: *const garray_T = &raw mut (*(*(prevwin_curwin
        as unsafe extern "C" fn() -> *mut win_T)())
        .w_buffer)
        .b_ucmds;
    loop {
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < (*gap).ga_len {
            let mut cmd: *mut ucmd_T = ((*gap).ga_data as *mut ucmd_T)
                .offset(i as isize);
            let mut a: uint32_t = (*cmd).uc_argt;
            if !(strncmp(name, (*cmd).uc_name, name_len) != 0 as ::core::ffi::c_int
                || message_filtered((*cmd).uc_name) as ::core::ffi::c_int != 0)
            {
                if !found {
                    msg_puts_title(
                        gettext(
                            b"\n    Name              Args Address Complete    Definition\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                    );
                }
                found = true_0 != 0;
                msg_putchar('\n' as ::core::ffi::c_int);
                if got_int {
                    break;
                }
                let mut len: size_t = 4 as size_t;
                if a & EX_BANG as uint32_t != 0 {
                    msg_putchar('!' as ::core::ffi::c_int);
                    len = len.wrapping_sub(1);
                }
                if a & EX_REGSTR as uint32_t != 0 {
                    msg_putchar('"' as ::core::ffi::c_int);
                    len = len.wrapping_sub(1);
                }
                if gap != &raw mut ucmds as *const garray_T {
                    msg_putchar('b' as ::core::ffi::c_int);
                    len = len.wrapping_sub(1);
                }
                if a & EX_TRLBAR as uint32_t != 0 {
                    msg_putchar('|' as ::core::ffi::c_int);
                    len = len.wrapping_sub(1);
                }
                if len != 0 as size_t {
                    msg_puts(
                        (b"    \0".as_ptr() as *const ::core::ffi::c_char)
                            .offset((4 as size_t).wrapping_sub(len) as isize),
                    );
                }
                msg_outtrans((*cmd).uc_name, HLF_D as ::core::ffi::c_int, false_0 != 0);
                len = strlen((*cmd).uc_name).wrapping_add(4 as size_t);
                if len < 21 as size_t {
                    static mut spaces: [::core::ffi::c_char; 18] = unsafe {
                        ::core::mem::transmute::<
                            [u8; 18],
                            [::core::ffi::c_char; 18],
                        >(*b"                 \0")
                    };
                    msg_puts(
                        (&raw mut spaces as *mut ::core::ffi::c_char)
                            .offset(len.wrapping_sub(4 as size_t) as isize),
                    );
                    len = 21 as size_t;
                }
                msg_putchar(' ' as ::core::ffi::c_int);
                len = len.wrapping_add(1);
                let over: int64_t = len as int64_t - 22 as int64_t;
                len = 0 as size_t;
                match a
                    & (EX_EXTRA as uint32_t | EX_NOSPC as uint32_t
                        | EX_NEEDARG as uint32_t)
                {
                    0 => {
                        let c2rust_fresh2 = len;
                        len = len.wrapping_add(1);
                        IObuff[c2rust_fresh2 as usize] = '0' as ::core::ffi::c_char;
                    }
                    4 => {
                        let c2rust_fresh3 = len;
                        len = len.wrapping_add(1);
                        IObuff[c2rust_fresh3 as usize] = '*' as ::core::ffi::c_char;
                    }
                    20 => {
                        let c2rust_fresh4 = len;
                        len = len.wrapping_add(1);
                        IObuff[c2rust_fresh4 as usize] = '?' as ::core::ffi::c_char;
                    }
                    132 => {
                        let c2rust_fresh5 = len;
                        len = len.wrapping_add(1);
                        IObuff[c2rust_fresh5 as usize] = '+' as ::core::ffi::c_char;
                    }
                    148 => {
                        let c2rust_fresh6 = len;
                        len = len.wrapping_add(1);
                        IObuff[c2rust_fresh6 as usize] = '1' as ::core::ffi::c_char;
                    }
                    _ => {}
                }
                loop {
                    let c2rust_fresh7 = len;
                    len = len.wrapping_add(1);
                    IObuff[c2rust_fresh7 as usize] = ' ' as ::core::ffi::c_char;
                    if (len as int64_t) >= 5 as int64_t - over {
                        break;
                    }
                }
                if a & (EX_RANGE as uint32_t | EX_COUNT as uint32_t) != 0 {
                    if a & EX_COUNT as uint32_t != 0 {
                        let mut rc: ::core::ffi::c_int = snprintf(
                            (&raw mut IObuff as *mut ::core::ffi::c_char)
                                .offset(len as isize),
                            (IOSIZE as size_t).wrapping_sub(len),
                            b"%ldc\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).uc_def,
                        );
                        '_c2rust_label: {
                            if rc > 0 as ::core::ffi::c_int {} else {
                                __assert_fail(
                                    b"rc > 0\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"/home/overlord/projects/neovim/neovim/src/nvim/usercmd.c\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    563 as ::core::ffi::c_uint,
                                    b"void uc_list(char *, size_t)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        len = len.wrapping_add(rc as size_t);
                    } else if a & EX_DFLALL as uint32_t != 0 {
                        let c2rust_fresh8 = len;
                        len = len.wrapping_add(1);
                        IObuff[c2rust_fresh8 as usize] = '%' as ::core::ffi::c_char;
                    } else if (*cmd).uc_def >= 0 as int64_t {
                        let mut rc_0: ::core::ffi::c_int = snprintf(
                            (&raw mut IObuff as *mut ::core::ffi::c_char)
                                .offset(len as isize),
                            (IOSIZE as size_t).wrapping_sub(len),
                            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).uc_def,
                        );
                        '_c2rust_label_0: {
                            if rc_0 > 0 as ::core::ffi::c_int {} else {
                                __assert_fail(
                                    b"rc > 0\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"/home/overlord/projects/neovim/neovim/src/nvim/usercmd.c\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    570 as ::core::ffi::c_uint,
                                    b"void uc_list(char *, size_t)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        len = len.wrapping_add(rc_0 as size_t);
                    } else {
                        let c2rust_fresh9 = len;
                        len = len.wrapping_add(1);
                        IObuff[c2rust_fresh9 as usize] = '.' as ::core::ffi::c_char;
                    }
                }
                loop {
                    let c2rust_fresh10 = len;
                    len = len.wrapping_add(1);
                    IObuff[c2rust_fresh10 as usize] = ' ' as ::core::ffi::c_char;
                    if (len as int64_t) >= 8 as int64_t - over {
                        break;
                    }
                }
                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while addr_type_complete[j as usize].expand as ::core::ffi::c_uint
                    != ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if addr_type_complete[j as usize].expand as ::core::ffi::c_uint
                        != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                        && addr_type_complete[j as usize].expand as ::core::ffi::c_uint
                            == (*cmd).uc_addr_type as ::core::ffi::c_uint
                    {
                        let mut rc_1: ::core::ffi::c_int = snprintf(
                            (&raw mut IObuff as *mut ::core::ffi::c_char)
                                .offset(len as isize),
                            (IOSIZE as size_t).wrapping_sub(len),
                            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                            addr_type_complete[j as usize].shortname,
                        );
                        '_c2rust_label_1: {
                            if rc_1 > 0 as ::core::ffi::c_int {} else {
                                __assert_fail(
                                    b"rc > 0\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"/home/overlord/projects/neovim/neovim/src/nvim/usercmd.c\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    586 as ::core::ffi::c_uint,
                                    b"void uc_list(char *, size_t)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        len = len.wrapping_add(rc_1 as size_t);
                        break;
                    } else {
                        j += 1;
                    }
                }
                loop {
                    let c2rust_fresh11 = len;
                    len = len.wrapping_add(1);
                    IObuff[c2rust_fresh11 as usize] = ' ' as ::core::ffi::c_char;
                    if (len as int64_t) >= 13 as int64_t - over {
                        break;
                    }
                }
                let mut cmd_compl: *mut ::core::ffi::c_char = get_command_complete(
                    (*cmd).uc_compl,
                );
                if !cmd_compl.is_null() {
                    let mut rc_2: ::core::ffi::c_int = snprintf(
                        (&raw mut IObuff as *mut ::core::ffi::c_char)
                            .offset(len as isize),
                        (IOSIZE as size_t).wrapping_sub(len),
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        get_command_complete((*cmd).uc_compl),
                    );
                    '_c2rust_label_2: {
                        if rc_2 > 0 as ::core::ffi::c_int {} else {
                            __assert_fail(
                                b"rc > 0\0".as_ptr() as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/usercmd.c\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                600 as ::core::ffi::c_uint,
                                b"void uc_list(char *, size_t)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    len = len.wrapping_add(rc_2 as size_t);
                }
                loop {
                    let c2rust_fresh12 = len;
                    len = len.wrapping_add(1);
                    IObuff[c2rust_fresh12 as usize] = ' ' as ::core::ffi::c_char;
                    if (len as int64_t) >= 25 as int64_t - over {
                        break;
                    }
                }
                IObuff[len as usize] = NUL as ::core::ffi::c_char;
                msg_outtrans(
                    &raw mut IObuff as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                if (*cmd).uc_luaref != LUA_NOREF {
                    let mut fn_0: *mut ::core::ffi::c_char = nlua_funcref_str(
                        (*cmd).uc_luaref,
                        ::core::ptr::null_mut::<Arena>(),
                    );
                    msg_puts_hl(fn_0, HLF_8 as ::core::ffi::c_int, false_0 != 0);
                    xfree(fn_0 as *mut ::core::ffi::c_void);
                    if *(*cmd).uc_rep as ::core::ffi::c_int != NUL {
                        msg_puts(
                            b"\n                                               \0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                }
                msg_outtrans_special(
                    (*cmd).uc_rep,
                    false_0 != 0,
                    if name_len == 0 as size_t {
                        Columns - 47 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                );
                if p_verbose > 0 as OptInt {
                    last_set_msg((*cmd).uc_script_ctx);
                }
                line_breakcheck();
                if got_int {
                    break;
                }
            }
            i += 1;
        }
        if gap == &raw mut ucmds as *const garray_T || i < (*gap).ga_len {
            break;
        }
        gap = &raw mut ucmds;
    }
    if !found {
        msg(
            gettext(
                b"No user-defined commands found\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            0 as ::core::ffi::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn parse_addr_type_arg(
    mut value: *mut ::core::ffi::c_char,
    mut vallen: ::core::ffi::c_int,
    mut addr_type_arg: *mut cmd_addr_T,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while addr_type_complete[i as usize].expand as ::core::ffi::c_uint
        != ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut a: ::core::ffi::c_int = (strlen(addr_type_complete[i as usize].name)
            as ::core::ffi::c_int == vallen) as ::core::ffi::c_int;
        let mut b: ::core::ffi::c_int = (strncmp(
            value,
            addr_type_complete[i as usize].name,
            vallen as size_t,
        ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        if a != 0 && b != 0 {
            *addr_type_arg = addr_type_complete[i as usize].expand;
            break;
        } else {
            i += 1;
        }
    }
    if addr_type_complete[i as usize].expand as ::core::ffi::c_uint
        == ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut err: *mut ::core::ffi::c_char = value;
        i = 0 as ::core::ffi::c_int;
        while *err.offset(i as isize) as ::core::ffi::c_int != NUL
            && !ascii_iswhite(*err.offset(i as isize) as ::core::ffi::c_int)
        {
            i += 1;
        }
        *err.offset(i as isize) = NUL as ::core::ffi::c_char;
        semsg(
            gettext(
                b"E180: Invalid address type value: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            err,
        );
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn parse_compl_arg(
    mut value: *const ::core::ffi::c_char,
    mut vallen: ::core::ffi::c_int,
    mut complp: *mut ::core::ffi::c_int,
    mut argt: *mut uint32_t,
    mut compl_arg: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut arg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut arglen: size_t = 0 as size_t;
    let mut valend: ::core::ffi::c_int = vallen;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < vallen {
        if *value.offset(i as isize) as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            arg = value.offset((i + 1 as ::core::ffi::c_int) as isize)
                as *mut ::core::ffi::c_char;
            arglen = (vallen - i - 1 as ::core::ffi::c_int) as size_t;
            valend = i;
            break;
        } else {
            i += 1;
        }
    }
    let mut i_0: ::core::ffi::c_int = 0;
    i_0 = 0 as ::core::ffi::c_int;
    while i_0
        < ::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        if !get_command_complete(i_0).is_null() {
            if strlen(command_complete[i_0 as usize]) as ::core::ffi::c_int == valend
                && strncmp(value, command_complete[i_0 as usize], valend as size_t)
                    == 0 as ::core::ffi::c_int
            {
                *complp = i_0;
                if i_0 == EXPAND_BUFFERS as ::core::ffi::c_int {
                    *argt = (*argt as ::core::ffi::c_uint | EX_BUFNAME) as uint32_t;
                } else if i_0 == EXPAND_DIRECTORIES as ::core::ffi::c_int
                    || i_0 == EXPAND_FILES as ::core::ffi::c_int
                    || i_0 == EXPAND_SHELLCMDLINE as ::core::ffi::c_int
                {
                    *argt = (*argt as ::core::ffi::c_uint | EX_XFILE) as uint32_t;
                }
                break;
            }
        }
        i_0 += 1;
    }
    if i_0
        == ::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        semsg(
            gettext(
                b"E180: Invalid complete value: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            value,
        );
        return FAIL;
    }
    if *complp != EXPAND_USER_DEFINED as ::core::ffi::c_int
        && *complp != EXPAND_USER_LIST as ::core::ffi::c_int && !arg.is_null()
    {
        emsg(
            gettext(
                b"E468: Completion argument only allowed for custom completion\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            ),
        );
        return FAIL;
    }
    if (*complp == EXPAND_USER_DEFINED as ::core::ffi::c_int
        || *complp == EXPAND_USER_LIST as ::core::ffi::c_int) && arg.is_null()
    {
        emsg(
            gettext(
                b"E467: Custom completion requires a function argument\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        return FAIL;
    }
    if !arg.is_null() {
        *compl_arg = xstrnsave(arg, arglen);
    }
    return OK;
}
unsafe extern "C" fn uc_scan_attr(
    mut attr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut argt: *mut uint32_t,
    mut def: *mut ::core::ffi::c_int,
    mut flags: *mut ::core::ffi::c_int,
    mut complp: *mut ::core::ffi::c_int,
    mut compl_arg: *mut *mut ::core::ffi::c_char,
    mut addr_type_arg: *mut cmd_addr_T,
) -> ::core::ffi::c_int {
    if len == 0 as size_t {
        emsg(
            gettext(
                b"E175: No attribute specified\0".as_ptr() as *const ::core::ffi::c_char,
            ),
        );
        return FAIL;
    }
    if strncasecmp(
        attr,
        b"bang\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        len,
    ) == 0 as ::core::ffi::c_int
    {
        *argt = (*argt as ::core::ffi::c_uint | EX_BANG) as uint32_t;
    } else if strncasecmp(
        attr,
        b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        len,
    ) == 0 as ::core::ffi::c_int
    {
        *flags |= UC_BUFFER as ::core::ffi::c_int;
    } else if strncasecmp(
        attr,
        b"register\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        len,
    ) == 0 as ::core::ffi::c_int
    {
        *argt = (*argt as ::core::ffi::c_uint | EX_REGSTR) as uint32_t;
    } else if strncasecmp(
        attr,
        b"keepscript\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        len,
    ) == 0 as ::core::ffi::c_int
    {
        *argt = (*argt as ::core::ffi::c_uint | EX_KEEPSCRIPT) as uint32_t;
    } else if strncasecmp(
        attr,
        b"bar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        len,
    ) == 0 as ::core::ffi::c_int
    {
        *argt = (*argt as ::core::ffi::c_uint | EX_TRLBAR) as uint32_t;
    } else {
        let mut val: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
            ::core::ffi::c_char,
        >();
        let mut vallen: size_t = 0 as size_t;
        let mut attrlen: size_t = len;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < len as ::core::ffi::c_int {
            if *attr.offset(i as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
            {
                val = attr.offset((i + 1 as ::core::ffi::c_int) as isize);
                vallen = len.wrapping_sub(i as size_t).wrapping_sub(1 as size_t);
                attrlen = i as size_t;
                break;
            } else {
                i += 1;
            }
        }
        if strncasecmp(
            attr,
            b"nargs\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            attrlen,
        ) == 0 as ::core::ffi::c_int
        {
            's_180: {
                '_wrong_nargs: {
                    if vallen == 1 as size_t {
                        if *val as ::core::ffi::c_int != '0' as ::core::ffi::c_int {
                            if *val as ::core::ffi::c_int == '1' as ::core::ffi::c_int {
                                *argt = (*argt as ::core::ffi::c_uint
                                    | (EX_EXTRA | EX_NOSPC | EX_NEEDARG)) as uint32_t;
                            } else if *val as ::core::ffi::c_int
                                == '*' as ::core::ffi::c_int
                            {
                                *argt = (*argt as ::core::ffi::c_uint | EX_EXTRA)
                                    as uint32_t;
                            } else if *val as ::core::ffi::c_int
                                == '?' as ::core::ffi::c_int
                            {
                                *argt = (*argt as ::core::ffi::c_uint
                                    | (EX_EXTRA | EX_NOSPC)) as uint32_t;
                            } else if *val as ::core::ffi::c_int
                                == '+' as ::core::ffi::c_int
                            {
                                *argt = (*argt as ::core::ffi::c_uint
                                    | (EX_EXTRA | EX_NEEDARG)) as uint32_t;
                            } else {
                                break '_wrong_nargs;
                            }
                        }
                        break 's_180;
                    }
                }
                emsg(
                    gettext(
                        b"E176: Invalid number of arguments\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                );
                return FAIL;
            }
        } else {
            's_409: {
                let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                    ::core::ffi::c_char,
                >();
                '_two_count: {
                    '_invalid_count: {
                        if strncasecmp(
                            attr,
                            b"range\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            attrlen,
                        ) == 0 as ::core::ffi::c_int
                        {
                            *argt = (*argt as ::core::ffi::c_uint | EX_RANGE)
                                as uint32_t;
                            if vallen == 1 as size_t
                                && *val as ::core::ffi::c_int == '%' as ::core::ffi::c_int
                            {
                                *argt = (*argt as ::core::ffi::c_uint | EX_DFLALL)
                                    as uint32_t;
                            } else if !val.is_null() {
                                p = val;
                                if *def >= 0 as ::core::ffi::c_int {
                                    break '_two_count;
                                } else {
                                    *def = getdigits_int(
                                        &raw mut p,
                                        true_0 != 0,
                                        0 as ::core::ffi::c_int,
                                    );
                                    *argt = (*argt as ::core::ffi::c_uint | EX_ZEROR)
                                        as uint32_t;
                                    if p != val.offset(vallen as isize) || vallen == 0 as size_t
                                    {
                                        break '_invalid_count;
                                    }
                                }
                            }
                            if *addr_type_arg as ::core::ffi::c_uint
                                == ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                *addr_type_arg = ADDR_LINES;
                            }
                        } else if strncasecmp(
                            attr,
                            b"count\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            attrlen,
                        ) == 0 as ::core::ffi::c_int
                        {
                            *argt = (*argt as ::core::ffi::c_uint
                                | (EX_COUNT | EX_ZEROR | EX_RANGE)) as uint32_t;
                            if *addr_type_arg as ::core::ffi::c_uint
                                == ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                *addr_type_arg = ADDR_OTHER;
                            }
                            if !val.is_null() {
                                let mut p_0: *mut ::core::ffi::c_char = val;
                                if *def >= 0 as ::core::ffi::c_int {
                                    break '_two_count;
                                } else {
                                    *def = getdigits_int(
                                        &raw mut p_0,
                                        true_0 != 0,
                                        0 as ::core::ffi::c_int,
                                    );
                                    if p_0 != val.offset(vallen as isize) {
                                        break '_invalid_count;
                                    }
                                }
                            }
                            *def = if *def > 0 as ::core::ffi::c_int {
                                *def
                            } else {
                                0 as ::core::ffi::c_int
                            };
                        } else if strncasecmp(
                            attr,
                            b"complete\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            attrlen,
                        ) == 0 as ::core::ffi::c_int
                        {
                            if val.is_null() {
                                semsg(
                                    gettext(
                                        &raw const e_argument_required_for_str
                                            as *const ::core::ffi::c_char,
                                    ),
                                    b"-complete\0".as_ptr() as *const ::core::ffi::c_char,
                                );
                                return FAIL;
                            }
                            if parse_compl_arg(
                                val,
                                vallen as ::core::ffi::c_int,
                                complp,
                                argt,
                                compl_arg,
                            ) == FAIL
                            {
                                return FAIL;
                            }
                        } else if strncasecmp(
                            attr,
                            b"addr\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            attrlen,
                        ) == 0 as ::core::ffi::c_int
                        {
                            *argt = (*argt as ::core::ffi::c_uint | EX_RANGE)
                                as uint32_t;
                            if val.is_null() {
                                semsg(
                                    gettext(
                                        &raw const e_argument_required_for_str
                                            as *const ::core::ffi::c_char,
                                    ),
                                    b"-addr\0".as_ptr() as *const ::core::ffi::c_char,
                                );
                                return FAIL;
                            }
                            if parse_addr_type_arg(
                                val,
                                vallen as ::core::ffi::c_int,
                                addr_type_arg,
                            ) == FAIL
                            {
                                return FAIL;
                            }
                            if *addr_type_arg as ::core::ffi::c_uint
                                != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                *argt = (*argt as ::core::ffi::c_uint | EX_ZEROR)
                                    as uint32_t;
                            }
                        } else {
                            let mut ch: ::core::ffi::c_char = *attr.offset(len as isize);
                            *attr.offset(len as isize) = NUL as ::core::ffi::c_char;
                            semsg(
                                gettext(
                                    b"E181: Invalid attribute: %s\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ),
                                attr,
                            );
                            *attr.offset(len as isize) = ch;
                            return FAIL;
                        }
                        break 's_409;
                    }
                    emsg(
                        gettext(
                            b"E178: Invalid default value for count\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                    );
                    return FAIL;
                }
                emsg(
                    gettext(
                        b"E177: Count cannot be specified twice\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                );
                return FAIL;
            }
        }
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn uc_validate_name(
    mut name: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if *name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *name as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *name as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        while *name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *name as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *name as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*name as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            name = name.offset(1);
        }
    }
    if ends_excmd(*name as ::core::ffi::c_int) == 0
        && !ascii_iswhite(*name as ::core::ffi::c_int)
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return name;
}
#[no_mangle]
pub unsafe extern "C" fn uc_add_command(
    mut name: *mut ::core::ffi::c_char,
    mut name_len: size_t,
    mut rep: *const ::core::ffi::c_char,
    mut argt: uint32_t,
    mut def: int64_t,
    mut flags: ::core::ffi::c_int,
    mut context: ::core::ffi::c_int,
    mut compl_arg: *mut ::core::ffi::c_char,
    mut compl_luaref: LuaRef,
    mut preview_luaref: LuaRef,
    mut addr_type: cmd_addr_T,
    mut luaref: LuaRef,
    mut force: bool,
) -> ::core::ffi::c_int {
    let mut cmd: *mut ucmd_T = ::core::ptr::null_mut::<ucmd_T>();
    let mut cmp: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut rep_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut gap: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    replace_termcodes(
        rep,
        strlen(rep),
        &raw mut rep_buf,
        0 as scid_T,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<bool>(),
        p_cpo,
    );
    if rep_buf.is_null() {
        rep_buf = xstrdup(rep);
    }
    if flags & UC_BUFFER as ::core::ffi::c_int != 0 {
        gap = &raw mut (*curbuf).b_ucmds;
        if (*gap).ga_itemsize == 0 as ::core::ffi::c_int {
            ga_init(
                gap,
                ::core::mem::size_of::<ucmd_T>() as ::core::ffi::c_int,
                4 as ::core::ffi::c_int,
            );
        }
    } else {
        gap = &raw mut ucmds;
    }
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    '_fail: {
        while i < (*gap).ga_len {
            cmd = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
            let mut len: size_t = strlen((*cmd).uc_name);
            cmp = strncmp(name, (*cmd).uc_name, name_len);
            if cmp == 0 as ::core::ffi::c_int {
                if name_len < len {
                    cmp = -1 as ::core::ffi::c_int;
                } else if name_len > len {
                    cmp = 1 as ::core::ffi::c_int;
                }
            }
            if cmp == 0 as ::core::ffi::c_int {
                if !force
                    && ((*cmd).uc_script_ctx.sc_sid != current_sctx.sc_sid
                        || (*cmd).uc_script_ctx.sc_seq == current_sctx.sc_seq)
                {
                    semsg(
                        gettext(
                            b"E174: Command already exists: add ! to replace it: %s\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        name,
                    );
                    break '_fail;
                } else {
                    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*cmd).uc_rep
                        as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    *ptr_;
                    let mut ptr__0: *mut *mut ::core::ffi::c_void = &raw mut (*cmd)
                        .uc_compl_arg as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__0);
                    *ptr__0 = NULL;
                    *ptr__0;
                    if (*cmd).uc_luaref != LUA_NOREF {
                        api_free_luaref((*cmd).uc_luaref);
                        (*cmd).uc_luaref = LUA_NOREF as LuaRef;
                    }
                    if (*cmd).uc_compl_luaref != LUA_NOREF {
                        api_free_luaref((*cmd).uc_compl_luaref);
                        (*cmd).uc_compl_luaref = LUA_NOREF as LuaRef;
                    }
                    if (*cmd).uc_preview_luaref != LUA_NOREF {
                        api_free_luaref((*cmd).uc_preview_luaref);
                        (*cmd).uc_preview_luaref = LUA_NOREF as LuaRef;
                    }
                    break;
                }
            } else {
                if cmp < 0 as ::core::ffi::c_int {
                    break;
                }
                i += 1;
            }
        }
        if cmp != 0 as ::core::ffi::c_int {
            ga_grow(gap, 1 as ::core::ffi::c_int);
            let p: *mut ::core::ffi::c_char = xstrnsave(name, name_len);
            cmd = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
            memmove(
                cmd.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                cmd as *const ::core::ffi::c_void,
                (((*gap).ga_len - i) as size_t)
                    .wrapping_mul(::core::mem::size_of::<ucmd_T>()),
            );
            (*gap).ga_len += 1;
            (*cmd).uc_name = p;
        }
        (*cmd).uc_rep = rep_buf;
        (*cmd).uc_argt = argt;
        (*cmd).uc_def = def;
        (*cmd).uc_compl = context;
        (*cmd).uc_script_ctx = current_sctx;
        (*cmd).uc_script_ctx.sc_lnum
            += (*(exestack.ga_data as *mut estack_T)
                .offset((exestack.ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum;
        nlua_set_sctx(&raw mut (*cmd).uc_script_ctx);
        (*cmd).uc_compl_arg = compl_arg;
        (*cmd).uc_compl_luaref = compl_luaref;
        (*cmd).uc_preview_luaref = preview_luaref;
        (*cmd).uc_addr_type = addr_type;
        (*cmd).uc_luaref = luaref;
        return OK;
    }
    xfree(rep_buf as *mut ::core::ffi::c_void);
    xfree(compl_arg as *mut ::core::ffi::c_void);
    if luaref != LUA_NOREF {
        api_free_luaref(luaref);
        luaref = LUA_NOREF as LuaRef;
    }
    if compl_luaref != LUA_NOREF {
        api_free_luaref(compl_luaref);
        compl_luaref = LUA_NOREF as LuaRef;
    }
    if preview_luaref != LUA_NOREF {
        api_free_luaref(preview_luaref);
        preview_luaref = LUA_NOREF as LuaRef;
    }
    return FAIL;
}
#[no_mangle]
pub unsafe extern "C" fn ex_command(mut eap: *mut exarg_T) {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut name_len: size_t = 0;
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut argt: uint32_t = 0 as uint32_t;
    let mut def: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut context: ::core::ffi::c_int = EXPAND_NOTHING as ::core::ffi::c_int;
    let mut compl_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut addr_type_arg: cmd_addr_T = ADDR_NONE;
    let mut has_attr: ::core::ffi::c_int = (*(*eap)
        .arg
        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '-' as ::core::ffi::c_int) as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    '_theend: {
        while *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
            p = p.offset(1);
            end = skiptowhite(p);
            if uc_scan_attr(
                p,
                end.offset_from(p) as size_t,
                &raw mut argt,
                &raw mut def,
                &raw mut flags,
                &raw mut context,
                &raw mut compl_arg,
                &raw mut addr_type_arg,
            ) == FAIL
            {
                break '_theend;
            }
            p = skipwhite(end);
        }
        name = p;
        end = uc_validate_name(name);
        if end.is_null() {
            emsg(
                gettext(
                    b"E182: Invalid command name\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
            );
        } else {
            name_len = end.offset_from(name) as size_t;
            p = skipwhite(end);
            if has_attr == 0 && ends_excmd(*p as ::core::ffi::c_int) != 0 {
                uc_list(name, name_len);
            } else if !(*name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint)
            {
                emsg(
                    gettext(
                        b"E183: User defined commands must start with an uppercase letter\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                );
            } else if name_len <= 4 as size_t
                && strncmp(
                    name,
                    b"Next\0".as_ptr() as *const ::core::ffi::c_char,
                    name_len,
                ) == 0 as ::core::ffi::c_int
            {
                emsg(
                    gettext(
                        b"E841: Reserved name, cannot be used for user defined command\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    ),
                );
            } else if context > 0 as ::core::ffi::c_int
                && argt & EX_EXTRA as uint32_t == 0 as uint32_t
            {
                emsg(
                    gettext(
                        &raw const e_complete_used_without_allowing_arguments
                            as *const ::core::ffi::c_char,
                    ),
                );
            } else {
                uc_add_command(
                    name,
                    name_len,
                    p,
                    argt,
                    def as int64_t,
                    flags,
                    context,
                    compl_arg,
                    LUA_NOREF,
                    LUA_NOREF,
                    addr_type_arg,
                    LUA_NOREF,
                    (*eap).forceit != 0,
                );
                return;
            }
        }
    }
    xfree(compl_arg as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ex_comclear(mut eap: *mut exarg_T) {
    uc_clear(&raw mut ucmds);
    if !curbuf.is_null() {
        uc_clear(&raw mut (*curbuf).b_ucmds);
    }
}
#[no_mangle]
pub unsafe extern "C" fn free_ucmd(mut cmd: *mut ucmd_T) {
    xfree((*cmd).uc_name as *mut ::core::ffi::c_void);
    xfree((*cmd).uc_rep as *mut ::core::ffi::c_void);
    xfree((*cmd).uc_compl_arg as *mut ::core::ffi::c_void);
    if (*cmd).uc_compl_luaref != LUA_NOREF {
        api_free_luaref((*cmd).uc_compl_luaref);
        (*cmd).uc_compl_luaref = LUA_NOREF as LuaRef;
    }
    if (*cmd).uc_luaref != LUA_NOREF {
        api_free_luaref((*cmd).uc_luaref);
        (*cmd).uc_luaref = LUA_NOREF as LuaRef;
    }
    if (*cmd).uc_preview_luaref != LUA_NOREF {
        api_free_luaref((*cmd).uc_preview_luaref);
        (*cmd).uc_preview_luaref = LUA_NOREF as LuaRef;
    }
}
#[no_mangle]
pub unsafe extern "C" fn uc_clear(mut gap: *mut garray_T) {
    let mut _gap: *mut garray_T = gap;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut ucmd_T = ((*_gap).ga_data as *mut ucmd_T)
                .offset(i as isize);
            free_ucmd(_item);
            i += 1;
        }
    }
    ga_clear(_gap);
}
#[no_mangle]
pub unsafe extern "C" fn ex_delcommand(mut eap: *mut exarg_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cmd: *mut ucmd_T = ::core::ptr::null_mut::<ucmd_T>();
    let mut res: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut arg: *const ::core::ffi::c_char = (*eap).arg;
    let mut buffer_only: bool = false_0 != 0;
    if strncmp(arg, b"-buffer\0".as_ptr() as *const ::core::ffi::c_char, 7 as size_t)
        == 0 as ::core::ffi::c_int
        && ascii_iswhite(
            *arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        ) as ::core::ffi::c_int != 0
    {
        buffer_only = true_0 != 0;
        arg = skipwhite(arg.offset(7 as ::core::ffi::c_int as isize));
    }
    let mut gap: *mut garray_T = &raw mut (*curbuf).b_ucmds;
    loop {
        i = 0 as ::core::ffi::c_int;
        while i < (*gap).ga_len {
            cmd = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
            res = strcmp(arg, (*cmd).uc_name);
            if res <= 0 as ::core::ffi::c_int {
                break;
            }
            i += 1;
        }
        if gap == &raw mut ucmds || res == 0 as ::core::ffi::c_int
            || buffer_only as ::core::ffi::c_int != 0
        {
            break;
        }
        gap = &raw mut ucmds;
    }
    if res != 0 as ::core::ffi::c_int {
        semsg(
            gettext(
                if buffer_only as ::core::ffi::c_int != 0 {
                    &raw const e_no_such_user_defined_command_in_current_buffer_str
                        as *const ::core::ffi::c_char
                } else {
                    &raw const e_no_such_user_defined_command_str
                        as *const ::core::ffi::c_char
                },
            ),
            arg,
        );
        return;
    }
    free_ucmd(cmd);
    (*gap).ga_len -= 1;
    if i < (*gap).ga_len {
        memmove(
            cmd as *mut ::core::ffi::c_void,
            cmd.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            (((*gap).ga_len - i) as size_t)
                .wrapping_mul(::core::mem::size_of::<ucmd_T>()),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn uc_split_args_iter(
    mut arg: *const ::core::ffi::c_char,
    mut arglen: size_t,
    mut end: *mut size_t,
    mut buf: *mut ::core::ffi::c_char,
    mut len: *mut size_t,
) -> bool {
    if arglen == 0 {
        return true_0 != 0;
    }
    let mut pos: size_t = *end;
    while pos < arglen
        && ascii_iswhite(*arg.offset(pos as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int != 0
    {
        pos = pos.wrapping_add(1);
    }
    let mut l: size_t = 0 as size_t;
    while pos < arglen.wrapping_sub(1 as size_t) {
        if *arg.offset(pos as isize) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && (*arg.offset(pos.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                || ascii_iswhite(
                    *arg.offset(pos.wrapping_add(1 as size_t) as isize)
                        as ::core::ffi::c_int,
                ) as ::core::ffi::c_int != 0)
        {
            pos = pos.wrapping_add(1);
            let c2rust_fresh13 = l;
            l = l.wrapping_add(1);
            *buf.offset(c2rust_fresh13 as isize) = *arg.offset(pos as isize);
        } else {
            let c2rust_fresh14 = l;
            l = l.wrapping_add(1);
            *buf.offset(c2rust_fresh14 as isize) = *arg.offset(pos as isize);
        }
        if ascii_iswhite(
            *arg.offset(pos.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int,
        ) {
            *end = pos.wrapping_add(1 as size_t);
            *len = l;
            return false_0 != 0;
        }
        pos = pos.wrapping_add(1);
    }
    if pos < arglen && !ascii_iswhite(*arg.offset(pos as isize) as ::core::ffi::c_int) {
        let c2rust_fresh15 = l;
        l = l.wrapping_add(1);
        *buf.offset(c2rust_fresh15 as isize) = *arg.offset(pos as isize);
    }
    *len = l;
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn uc_nargs_upper_bound(
    mut arg: *const ::core::ffi::c_char,
    mut arglen: size_t,
) -> size_t {
    let mut was_white: bool = true_0 != 0;
    let mut nargs: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < arglen {
        let mut is_white: bool = ascii_iswhite(
            *arg.offset(i as isize) as ::core::ffi::c_int,
        );
        if was_white as ::core::ffi::c_int != 0 && !is_white {
            nargs = nargs.wrapping_add(1);
        }
        was_white = is_white;
        i = i.wrapping_add(1);
    }
    return nargs;
}
unsafe extern "C" fn uc_split_args(
    mut arg: *const ::core::ffi::c_char,
    mut args: *mut *mut ::core::ffi::c_char,
    mut arglens: *const size_t,
    mut argc: size_t,
    mut lenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut len: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    if args.is_null() {
        let mut p: *const ::core::ffi::c_char = arg;
        while *p != 0 {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
            {
                len += 2 as ::core::ffi::c_int;
                p = p.offset(2 as ::core::ffi::c_int as isize);
            } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && ascii_iswhite(
                    *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                ) as ::core::ffi::c_int != 0
            {
                len += 1 as ::core::ffi::c_int;
                p = p.offset(2 as ::core::ffi::c_int as isize);
            } else if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
            {
                len += 2 as ::core::ffi::c_int;
                p = p.offset(1 as ::core::ffi::c_int as isize);
            } else if ascii_iswhite(*p as ::core::ffi::c_int) {
                p = skipwhite(p);
                if *p as ::core::ffi::c_int == NUL {
                    break;
                }
                len += 4 as ::core::ffi::c_int;
            } else {
                let charlen: ::core::ffi::c_int = utfc_ptr2len(p);
                len += charlen;
                p = p.offset(charlen as isize);
            }
        }
    } else {
        let mut i: size_t = 0 as size_t;
        while i < argc {
            let mut p_0: *const ::core::ffi::c_char = *args.offset(i as isize);
            let mut arg_end: *const ::core::ffi::c_char = (*args.offset(i as isize))
                .offset(*arglens.offset(i as isize) as isize);
            while p_0 < arg_end {
                if *p_0 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    || *p_0 as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                {
                    len += 2 as ::core::ffi::c_int;
                    p_0 = p_0.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    let charlen_0: ::core::ffi::c_int = utfc_ptr2len(p_0);
                    len += charlen_0;
                    p_0 = p_0.offset(charlen_0 as isize);
                }
            }
            if i != argc.wrapping_sub(1 as size_t) {
                len += 4 as ::core::ffi::c_int;
            }
            i = i.wrapping_add(1);
        }
    }
    let mut buf: *mut ::core::ffi::c_char = xmalloc(
        (len as size_t).wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    let mut q: *mut ::core::ffi::c_char = buf;
    let c2rust_fresh26 = q;
    q = q.offset(1);
    *c2rust_fresh26 = '"' as ::core::ffi::c_char;
    if args.is_null() {
        let mut p_1: *const ::core::ffi::c_char = arg;
        while *p_1 != 0 {
            if *p_1.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p_1.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
            {
                let c2rust_fresh27 = q;
                q = q.offset(1);
                *c2rust_fresh27 = '\\' as ::core::ffi::c_char;
                let c2rust_fresh28 = q;
                q = q.offset(1);
                *c2rust_fresh28 = '\\' as ::core::ffi::c_char;
                p_1 = p_1.offset(2 as ::core::ffi::c_int as isize);
            } else if *p_1.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && ascii_iswhite(
                    *p_1.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                ) as ::core::ffi::c_int != 0
            {
                let c2rust_fresh29 = q;
                q = q.offset(1);
                *c2rust_fresh29 = *p_1.offset(1 as ::core::ffi::c_int as isize);
                p_1 = p_1.offset(2 as ::core::ffi::c_int as isize);
            } else if *p_1 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                || *p_1 as ::core::ffi::c_int == '"' as ::core::ffi::c_int
            {
                let c2rust_fresh30 = q;
                q = q.offset(1);
                *c2rust_fresh30 = '\\' as ::core::ffi::c_char;
                let c2rust_fresh31 = p_1;
                p_1 = p_1.offset(1);
                let c2rust_fresh32 = q;
                q = q.offset(1);
                *c2rust_fresh32 = *c2rust_fresh31;
            } else if ascii_iswhite(*p_1 as ::core::ffi::c_int) {
                p_1 = skipwhite(p_1);
                if *p_1 as ::core::ffi::c_int == NUL {
                    break;
                }
                let c2rust_fresh33 = q;
                q = q.offset(1);
                *c2rust_fresh33 = '"' as ::core::ffi::c_char;
                let c2rust_fresh34 = q;
                q = q.offset(1);
                *c2rust_fresh34 = ',' as ::core::ffi::c_char;
                let c2rust_fresh35 = q;
                q = q.offset(1);
                *c2rust_fresh35 = ' ' as ::core::ffi::c_char;
                let c2rust_fresh36 = q;
                q = q.offset(1);
                *c2rust_fresh36 = '"' as ::core::ffi::c_char;
            } else {
                mb_copy_char(&raw mut p_1, &raw mut q);
            }
        }
    } else {
        let mut i_0: size_t = 0 as size_t;
        while i_0 < argc {
            let mut p_2: *const ::core::ffi::c_char = *args.offset(i_0 as isize);
            let mut arg_end_0: *const ::core::ffi::c_char = (*args.offset(i_0 as isize))
                .offset(*arglens.offset(i_0 as isize) as isize);
            while p_2 < arg_end_0 {
                if *p_2 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    || *p_2 as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                {
                    let c2rust_fresh37 = q;
                    q = q.offset(1);
                    *c2rust_fresh37 = '\\' as ::core::ffi::c_char;
                    let c2rust_fresh38 = p_2;
                    p_2 = p_2.offset(1);
                    let c2rust_fresh39 = q;
                    q = q.offset(1);
                    *c2rust_fresh39 = *c2rust_fresh38;
                } else {
                    mb_copy_char(&raw mut p_2, &raw mut q);
                }
            }
            if i_0 != argc.wrapping_sub(1 as size_t) {
                let c2rust_fresh40 = q;
                q = q.offset(1);
                *c2rust_fresh40 = '"' as ::core::ffi::c_char;
                let c2rust_fresh41 = q;
                q = q.offset(1);
                *c2rust_fresh41 = ',' as ::core::ffi::c_char;
                let c2rust_fresh42 = q;
                q = q.offset(1);
                *c2rust_fresh42 = ' ' as ::core::ffi::c_char;
                let c2rust_fresh43 = q;
                q = q.offset(1);
                *c2rust_fresh43 = '"' as ::core::ffi::c_char;
            }
            i_0 = i_0.wrapping_add(1);
        }
    }
    let c2rust_fresh44 = q;
    q = q.offset(1);
    *c2rust_fresh44 = '"' as ::core::ffi::c_char;
    *q = 0 as ::core::ffi::c_char;
    *lenp = len as size_t;
    return buf;
}
unsafe extern "C" fn add_cmd_modifier(
    mut buf: *mut ::core::ffi::c_char,
    mut mod_str: *mut ::core::ffi::c_char,
    mut multi_mods: *mut bool,
) -> size_t {
    let mut result: size_t = strlen(mod_str);
    if *multi_mods {
        result = result.wrapping_add(1);
    }
    if !buf.is_null() {
        if *multi_mods {
            strcat(buf, b" \0".as_ptr() as *const ::core::ffi::c_char);
        }
        strcat(buf, mod_str);
    }
    *multi_mods = true_0 != 0;
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn add_win_cmd_modifiers(
    mut buf: *mut ::core::ffi::c_char,
    mut cmod: *const cmdmod_T,
    mut multi_mods: *mut bool,
) -> size_t {
    let mut result: size_t = 0 as size_t;
    if (*cmod).cmod_split & WSP_ABOVE as ::core::ffi::c_int != 0 {
        result = result
            .wrapping_add(
                add_cmd_modifier(
                    buf,
                    b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    multi_mods,
                ),
            );
    }
    if (*cmod).cmod_split & WSP_BELOW as ::core::ffi::c_int != 0 {
        result = result
            .wrapping_add(
                add_cmd_modifier(
                    buf,
                    b"belowright\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    multi_mods,
                ),
            );
    }
    if (*cmod).cmod_split & WSP_BOT as ::core::ffi::c_int != 0 {
        result = result
            .wrapping_add(
                add_cmd_modifier(
                    buf,
                    b"botright\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    multi_mods,
                ),
            );
    }
    if (*cmod).cmod_tab > 0 as ::core::ffi::c_int {
        let mut tabnr: ::core::ffi::c_int = (*cmod).cmod_tab - 1 as ::core::ffi::c_int;
        if tabnr == tabpage_index(curtab) {
            result = result
                .wrapping_add(
                    add_cmd_modifier(
                        buf,
                        b"tab\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        multi_mods,
                    ),
                );
        } else {
            let mut tab_buf: [::core::ffi::c_char; 68] = [0; 68];
            snprintf(
                &raw mut tab_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 68]>(),
                b"%dtab\0".as_ptr() as *const ::core::ffi::c_char,
                tabnr,
            );
            result = result
                .wrapping_add(
                    add_cmd_modifier(
                        buf,
                        &raw mut tab_buf as *mut ::core::ffi::c_char,
                        multi_mods,
                    ),
                );
        }
    }
    if (*cmod).cmod_split & WSP_TOP as ::core::ffi::c_int != 0 {
        result = result
            .wrapping_add(
                add_cmd_modifier(
                    buf,
                    b"topleft\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    multi_mods,
                ),
            );
    }
    if (*cmod).cmod_split & WSP_VERT as ::core::ffi::c_int != 0 {
        result = result
            .wrapping_add(
                add_cmd_modifier(
                    buf,
                    b"vertical\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    multi_mods,
                ),
            );
    }
    if (*cmod).cmod_split & WSP_HOR as ::core::ffi::c_int != 0 {
        result = result
            .wrapping_add(
                add_cmd_modifier(
                    buf,
                    b"horizontal\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    multi_mods,
                ),
            );
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn uc_mods(
    mut buf: *mut ::core::ffi::c_char,
    mut cmod: *const cmdmod_T,
    mut quote: bool,
) -> size_t {
    let mut result: size_t = 0 as size_t;
    let mut multi_mods: bool = false_0 != 0;
    static mut mod_entries: [mod_entry_T; 12] = [
        mod_entry_T {
            flag: CMOD_BROWSE as ::core::ffi::c_int,
            name: b"browse\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_CONFIRM as ::core::ffi::c_int,
            name: b"confirm\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_HIDE as ::core::ffi::c_int,
            name: b"hide\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPALT as ::core::ffi::c_int,
            name: b"keepalt\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPJUMPS as ::core::ffi::c_int,
            name: b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPMARKS as ::core::ffi::c_int,
            name: b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPPATTERNS as ::core::ffi::c_int,
            name: b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_LOCKMARKS as ::core::ffi::c_int,
            name: b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_NOSWAPFILE as ::core::ffi::c_int,
            name: b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_UNSILENT as ::core::ffi::c_int,
            name: b"unsilent\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_NOAUTOCMD as ::core::ffi::c_int,
            name: b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_SANDBOX as ::core::ffi::c_int,
            name: b"sandbox\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
    ];
    result = (if quote as ::core::ffi::c_int != 0 {
        2 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as size_t;
    if !buf.is_null() {
        if quote {
            let c2rust_fresh16 = buf;
            buf = buf.offset(1);
            *c2rust_fresh16 = '"' as ::core::ffi::c_char;
        }
        *buf = NUL as ::core::ffi::c_char;
    }
    let mut i: size_t = 0 as size_t;
    while i
        < ::core::mem::size_of::<[mod_entry_T; 12]>()
            .wrapping_div(::core::mem::size_of::<mod_entry_T>())
            .wrapping_div(
                (::core::mem::size_of::<[mod_entry_T; 12]>()
                    .wrapping_rem(::core::mem::size_of::<mod_entry_T>()) == 0)
                    as ::core::ffi::c_int as usize,
            )
    {
        if (*cmod).cmod_flags & mod_entries[i as usize].flag != 0 {
            result = result
                .wrapping_add(
                    add_cmd_modifier(
                        buf,
                        mod_entries[i as usize].name,
                        &raw mut multi_mods,
                    ),
                );
        }
        i = i.wrapping_add(1);
    }
    if (*cmod).cmod_flags & CMOD_SILENT as ::core::ffi::c_int != 0 {
        result = result
            .wrapping_add(
                add_cmd_modifier(
                    buf,
                    (if (*cmod).cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0 {
                        b"silent!\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"silent\0".as_ptr() as *const ::core::ffi::c_char
                    }) as *mut ::core::ffi::c_char,
                    &raw mut multi_mods,
                ),
            );
    }
    if (*cmod).cmod_verbose > 0 as ::core::ffi::c_int {
        let mut verbose_value: ::core::ffi::c_int = (*cmod).cmod_verbose
            - 1 as ::core::ffi::c_int;
        if verbose_value == 1 as ::core::ffi::c_int {
            result = result
                .wrapping_add(
                    add_cmd_modifier(
                        buf,
                        b"verbose\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        &raw mut multi_mods,
                    ),
                );
        } else {
            let mut verbose_buf: [::core::ffi::c_char; 65] = [0; 65];
            snprintf(
                &raw mut verbose_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                b"%dverbose\0".as_ptr() as *const ::core::ffi::c_char,
                verbose_value,
            );
            result = result
                .wrapping_add(
                    add_cmd_modifier(
                        buf,
                        &raw mut verbose_buf as *mut ::core::ffi::c_char,
                        &raw mut multi_mods,
                    ),
                );
        }
    }
    result = result.wrapping_add(add_win_cmd_modifiers(buf, cmod, &raw mut multi_mods));
    if quote as ::core::ffi::c_int != 0 && !buf.is_null() {
        buf = buf.offset(result.wrapping_sub(2 as size_t) as isize);
        *buf = '"' as ::core::ffi::c_char;
    }
    return result;
}
unsafe extern "C" fn uc_check_code(
    mut code: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut buf: *mut ::core::ffi::c_char,
    mut cmd: *mut ucmd_T,
    mut eap: *mut exarg_T,
    mut split_buf: *mut *mut ::core::ffi::c_char,
    mut split_len: *mut size_t,
) -> size_t {
    let mut result: size_t = 0 as size_t;
    let mut p: *mut ::core::ffi::c_char = code.offset(1 as ::core::ffi::c_int as isize);
    let mut l: size_t = len.wrapping_sub(2 as size_t);
    let mut quote: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut type_0: C2Rust_Unnamed_21 = ct_NONE;
    if !vim_strchr(
            b"qQfF\0".as_ptr() as *const ::core::ffi::c_char,
            *p as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
    {
        quote = if *p as ::core::ffi::c_int == 'q' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == 'Q' as ::core::ffi::c_int
        {
            1 as ::core::ffi::c_int
        } else {
            2 as ::core::ffi::c_int
        };
        p = p.offset(2 as ::core::ffi::c_int as isize);
        l = l.wrapping_sub(2 as size_t);
    }
    l = l.wrapping_add(1);
    if l > 1 as size_t {
        if strncasecmp(
            p,
            b"args>\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_ARGS;
        } else if strncasecmp(
            p,
            b"bang>\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_BANG;
        } else if strncasecmp(
            p,
            b"count>\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_COUNT;
        } else if strncasecmp(
            p,
            b"line1>\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_LINE1;
        } else if strncasecmp(
            p,
            b"line2>\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_LINE2;
        } else if strncasecmp(
            p,
            b"range>\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_RANGE;
        } else if strncasecmp(
            p,
            b"lt>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_LT;
        } else if strncasecmp(
            p,
            b"reg>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
            || strncasecmp(
                p,
                b"register>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                l,
            ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_REGISTER;
        } else if strncasecmp(
            p,
            b"mods>\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_MODS;
        }
    }
    match type_0 as ::core::ffi::c_uint {
        0 => {
            if *(*eap).arg as ::core::ffi::c_int == NUL {
                if quote == 1 as ::core::ffi::c_int {
                    result = 2 as size_t;
                    if !buf.is_null() {
                        strcpy(
                            buf,
                            b"''\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                    }
                } else {
                    result = 0 as size_t;
                }
            } else {
                if (*eap).argt & EX_NOSPC as uint32_t != 0
                    && quote == 2 as ::core::ffi::c_int
                {
                    quote = 1 as ::core::ffi::c_int;
                }
                match quote {
                    0 => {
                        result = strlen((*eap).arg);
                        if !buf.is_null() {
                            strcpy(buf, (*eap).arg);
                        }
                    }
                    1 => {
                        result = strlen((*eap).arg).wrapping_add(2 as size_t);
                        p = (*eap).arg;
                        while *p != 0 {
                            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                || *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                            {
                                result = result.wrapping_add(1);
                            }
                            p = p.offset(1);
                        }
                        if !buf.is_null() {
                            let c2rust_fresh18 = buf;
                            buf = buf.offset(1);
                            *c2rust_fresh18 = '"' as ::core::ffi::c_char;
                            p = (*eap).arg;
                            while *p != 0 {
                                if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                    || *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                                {
                                    let c2rust_fresh19 = buf;
                                    buf = buf.offset(1);
                                    *c2rust_fresh19 = '\\' as ::core::ffi::c_char;
                                }
                                let c2rust_fresh20 = buf;
                                buf = buf.offset(1);
                                *c2rust_fresh20 = *p;
                                p = p.offset(1);
                            }
                            *buf = '"' as ::core::ffi::c_char;
                        }
                    }
                    2 => {
                        if (*split_buf).is_null() {
                            *split_buf = uc_split_args(
                                (*eap).arg,
                                (*eap).args,
                                (*eap).arglens,
                                (*eap).argc,
                                split_len,
                            );
                        }
                        result = *split_len;
                        if !buf.is_null() && result != 0 as size_t {
                            strcpy(buf, *split_buf);
                        }
                    }
                    _ => {}
                }
            }
        }
        1 => {
            result = (if (*eap).forceit != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as size_t;
            if quote != 0 {
                result = result.wrapping_add(2 as size_t);
            }
            if !buf.is_null() {
                if quote != 0 {
                    let c2rust_fresh21 = buf;
                    buf = buf.offset(1);
                    *c2rust_fresh21 = '"' as ::core::ffi::c_char;
                }
                if (*eap).forceit != 0 {
                    let c2rust_fresh22 = buf;
                    buf = buf.offset(1);
                    *c2rust_fresh22 = '!' as ::core::ffi::c_char;
                }
                if quote != 0 {
                    *buf = '"' as ::core::ffi::c_char;
                }
            }
        }
        3 | 4 | 5 | 2 => {
            let mut num_buf: [::core::ffi::c_char; 20] = [0; 20];
            let mut num: int64_t = if type_0 as ::core::ffi::c_uint
                == ct_LINE1 as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*eap).line1 as int64_t
            } else if type_0 as ::core::ffi::c_uint
                == ct_LINE2 as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*eap).line2 as int64_t
            } else if type_0 as ::core::ffi::c_uint
                == ct_RANGE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*eap).addr_count as int64_t
            } else if (*eap).addr_count > 0 as ::core::ffi::c_int {
                (*eap).line2 as int64_t
            } else {
                (*cmd).uc_def
            };
            let mut num_len: size_t = 0;
            snprintf(
                &raw mut num_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                num,
            );
            num_len = strlen(&raw mut num_buf as *mut ::core::ffi::c_char);
            result = num_len;
            if quote != 0 {
                result = result.wrapping_add(2 as size_t);
            }
            if !buf.is_null() {
                if quote != 0 {
                    let c2rust_fresh23 = buf;
                    buf = buf.offset(1);
                    *c2rust_fresh23 = '"' as ::core::ffi::c_char;
                }
                strcpy(buf, &raw mut num_buf as *mut ::core::ffi::c_char);
                buf = buf.offset(num_len as isize);
                if quote != 0 {
                    *buf = '"' as ::core::ffi::c_char;
                }
            }
        }
        6 => {
            result = uc_mods(buf, &raw mut cmdmod, quote != 0);
        }
        7 => {
            result = (if (*eap).regname != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as size_t;
            if quote != 0 {
                result = result.wrapping_add(2 as size_t);
            }
            if !buf.is_null() {
                if quote != 0 {
                    let c2rust_fresh24 = buf;
                    buf = buf.offset(1);
                    *c2rust_fresh24 = '\'' as ::core::ffi::c_char;
                }
                if (*eap).regname != 0 {
                    let c2rust_fresh25 = buf;
                    buf = buf.offset(1);
                    *c2rust_fresh25 = (*eap).regname as ::core::ffi::c_char;
                }
                if quote != 0 {
                    *buf = '\'' as ::core::ffi::c_char;
                }
            }
        }
        8 => {
            result = 1 as size_t;
            if !buf.is_null() {
                *buf = '<' as ::core::ffi::c_char;
            }
        }
        _ => {
            result = -1 as ::core::ffi::c_int as size_t;
            if !buf.is_null() {
                *buf = '<' as ::core::ffi::c_char;
            }
        }
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn do_ucmd(
    mut eap: *mut exarg_T,
    mut preview: bool,
) -> ::core::ffi::c_int {
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut split_len: size_t = 0 as size_t;
    let mut split_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut cmd: *mut ucmd_T = ::core::ptr::null_mut::<ucmd_T>();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_USER as ::core::ffi::c_int {
        cmd = (ucmds.ga_data as *mut ucmd_T).offset((*eap).useridx as isize);
    } else {
        cmd = ((*(*(prevwin_curwin as unsafe extern "C" fn() -> *mut win_T)()).w_buffer)
            .b_ucmds
            .ga_data as *mut ucmd_T)
            .offset((*eap).useridx as isize);
    }
    if preview {
        '_c2rust_label: {
            if (*cmd).uc_preview_luaref > 0 as ::core::ffi::c_int {} else {
                __assert_fail(
                    b"cmd->uc_preview_luaref > 0\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/usercmd.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    1663 as ::core::ffi::c_uint,
                    b"int do_ucmd(exarg_T *, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return nlua_do_ucmd(cmd, eap, true_0 != 0);
    }
    if (*cmd).uc_luaref > 0 as ::core::ffi::c_int {
        nlua_do_ucmd(cmd, eap, false_0 != 0);
        return 0 as ::core::ffi::c_int;
    }
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    loop {
        let mut p: *mut ::core::ffi::c_char = (*cmd).uc_rep;
        let mut q: *mut ::core::ffi::c_char = buf;
        let mut totlen: size_t = 0 as size_t;
        loop {
            let mut start: *mut ::core::ffi::c_char = vim_strchr(
                p,
                '<' as ::core::ffi::c_int,
            );
            if !start.is_null() {
                end = vim_strchr(
                    start.offset(1 as ::core::ffi::c_int as isize),
                    '>' as ::core::ffi::c_int,
                );
            }
            if !buf.is_null() {
                let mut ksp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                    ::core::ffi::c_char,
                >();
                ksp = p;
                while *ksp as ::core::ffi::c_int != NUL
                    && *ksp as uint8_t as ::core::ffi::c_int != K_SPECIAL
                {
                    ksp = ksp.offset(1);
                }
                if *ksp as uint8_t as ::core::ffi::c_int == K_SPECIAL
                    && (start.is_null() || ksp < start || end.is_null())
                    && (*ksp.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int == KS_SPECIAL
                        && *ksp.offset(2 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int == KE_FILLER)
                {
                    let mut len: size_t = ksp.offset_from(p) as size_t;
                    if len > 0 as size_t {
                        memmove(
                            q as *mut ::core::ffi::c_void,
                            p as *const ::core::ffi::c_void,
                            len,
                        );
                        q = q.offset(len as isize);
                    }
                    let c2rust_fresh17 = q;
                    q = q.offset(1);
                    *c2rust_fresh17 = K_SPECIAL as ::core::ffi::c_char;
                    p = ksp.offset(3 as ::core::ffi::c_int as isize);
                    continue;
                }
            }
            if start.is_null() || end.is_null() {
                break;
            }
            end = end.offset(1);
            let mut len_0: size_t = start.offset_from(p) as size_t;
            if buf.is_null() {
                totlen = totlen.wrapping_add(len_0);
            } else {
                memmove(
                    q as *mut ::core::ffi::c_void,
                    p as *const ::core::ffi::c_void,
                    len_0,
                );
                q = q.offset(len_0 as isize);
            }
            len_0 = uc_check_code(
                start,
                end.offset_from(start) as size_t,
                q,
                cmd,
                eap,
                &raw mut split_buf,
                &raw mut split_len,
            );
            if len_0 == -1 as ::core::ffi::c_int as size_t {
                p = start.offset(1 as ::core::ffi::c_int as isize);
                len_0 = 1 as size_t;
            } else {
                p = end;
            }
            if buf.is_null() {
                totlen = totlen.wrapping_add(len_0);
            } else {
                q = q.offset(len_0 as isize);
            }
        }
        if !buf.is_null() {
            strcpy(q, p);
            break;
        } else {
            totlen = totlen.wrapping_add(strlen(p));
            buf = xmalloc(totlen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        }
    }
    let mut save_current_sctx: sctx_T = sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    let mut restore_current_sctx: bool = false_0 != 0;
    if (*cmd).uc_argt & EX_KEEPSCRIPT as uint32_t == 0 as uint32_t {
        restore_current_sctx = true_0 != 0;
        save_current_sctx = current_sctx;
        current_sctx.sc_sid = (*cmd).uc_script_ctx.sc_sid;
    }
    do_cmdline(
        buf,
        (*eap).ea_getline,
        (*eap).cookie,
        DOCMD_VERBOSE as ::core::ffi::c_int | DOCMD_NOWAIT as ::core::ffi::c_int
            | DOCMD_KEYTYPED as ::core::ffi::c_int,
    );
    if restore_current_sctx {
        current_sctx = save_current_sctx;
    }
    xfree(buf as *mut ::core::ffi::c_void);
    xfree(split_buf as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn commands_array(
    mut buf: *mut buf_T,
    mut arena: *mut Arena,
) -> Dict {
    let mut gap: *mut garray_T = if buf.is_null() {
        &raw mut ucmds
    } else {
        &raw mut (*buf).b_ucmds
    };
    let mut rv: Dict = arena_dict(arena, (*gap).ga_len as size_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut arg: [::core::ffi::c_char; 2] = [
            0 as ::core::ffi::c_char,
            0 as ::core::ffi::c_char,
        ];
        let mut d: Dict = arena_dict(arena, 16 as size_t);
        let mut cmd: *mut ucmd_T = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
        let c2rust_fresh45 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh45 as isize) = key_value_pair {
            key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*cmd).uc_name),
                },
            },
        };
        let c2rust_fresh46 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh46 as isize) = key_value_pair {
            key: cstr_as_string(b"definition\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*cmd).uc_rep),
                },
            },
        };
        let c2rust_fresh47 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh47 as isize) = key_value_pair {
            key: cstr_as_string(b"script_id\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*cmd).uc_script_ctx.sc_sid as Integer,
                },
            },
        };
        let c2rust_fresh48 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh48 as isize) = key_value_pair {
            key: cstr_as_string(b"bang\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: (*cmd).uc_argt & 0x2 as uint32_t != 0,
                },
            },
        };
        let c2rust_fresh49 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh49 as isize) = key_value_pair {
            key: cstr_as_string(b"bar\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: (*cmd).uc_argt & 0x100 as uint32_t != 0,
                },
            },
        };
        let c2rust_fresh50 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh50 as isize) = key_value_pair {
            key: cstr_as_string(b"register\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: (*cmd).uc_argt & 0x200 as uint32_t != 0,
                },
            },
        };
        let c2rust_fresh51 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh51 as isize) = key_value_pair {
            key: cstr_as_string(b"keepscript\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: (*cmd).uc_argt & 0x4000000 as uint32_t != 0,
                },
            },
        };
        if (*cmd).uc_preview_luaref != LUA_NOREF {
            let c2rust_fresh52 = d.size;
            d.size = d.size.wrapping_add(1);
            *d.items.offset(c2rust_fresh52 as isize) = key_value_pair {
                key: cstr_as_string(b"preview\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeLuaRef,
                    data: C2Rust_Unnamed {
                        luaref: api_new_luaref((*cmd).uc_preview_luaref),
                    },
                },
            };
        }
        if (*cmd).uc_luaref != LUA_NOREF {
            let c2rust_fresh53 = d.size;
            d.size = d.size.wrapping_add(1);
            *d.items.offset(c2rust_fresh53 as isize) = key_value_pair {
                key: cstr_as_string(
                    b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                ),
                value: object {
                    type_0: kObjectTypeLuaRef,
                    data: C2Rust_Unnamed {
                        luaref: api_new_luaref((*cmd).uc_luaref),
                    },
                },
            };
        }
        match (*cmd).uc_argt
            & (EX_EXTRA as uint32_t | EX_NOSPC as uint32_t | EX_NEEDARG as uint32_t)
        {
            0 => {
                arg[0 as ::core::ffi::c_int as usize] = '0' as ::core::ffi::c_char;
            }
            4 => {
                arg[0 as ::core::ffi::c_int as usize] = '*' as ::core::ffi::c_char;
            }
            20 => {
                arg[0 as ::core::ffi::c_int as usize] = '?' as ::core::ffi::c_char;
            }
            132 => {
                arg[0 as ::core::ffi::c_int as usize] = '+' as ::core::ffi::c_char;
            }
            148 => {
                arg[0 as ::core::ffi::c_int as usize] = '1' as ::core::ffi::c_char;
            }
            _ => {}
        }
        let c2rust_fresh54 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh54 as isize) = key_value_pair {
            key: cstr_as_string(b"nargs\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(
                        arena,
                        cstr_as_string(&raw mut arg as *mut ::core::ffi::c_char),
                    ),
                },
            },
        };
        if (*cmd).uc_compl_luaref != LUA_NOREF {
            let c2rust_fresh55 = d.size;
            d.size = d.size.wrapping_add(1);
            *d.items.offset(c2rust_fresh55 as isize) = key_value_pair {
                key: cstr_as_string(
                    b"complete\0".as_ptr() as *const ::core::ffi::c_char,
                ),
                value: object {
                    type_0: kObjectTypeLuaRef,
                    data: C2Rust_Unnamed {
                        luaref: api_new_luaref((*cmd).uc_compl_luaref),
                    },
                },
            };
        } else {
            let mut cmd_compl: *mut ::core::ffi::c_char = get_command_complete(
                (*cmd).uc_compl,
            );
            let c2rust_fresh56 = d.size;
            d.size = d.size.wrapping_add(1);
            *d.items.offset(c2rust_fresh56 as isize) = key_value_pair {
                key: cstr_as_string(
                    b"complete\0".as_ptr() as *const ::core::ffi::c_char,
                ),
                value: if cmd_compl.is_null() {
                    object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    }
                } else {
                    object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: cstr_as_string(cmd_compl),
                        },
                    }
                },
            };
        }
        let c2rust_fresh57 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh57 as isize) = key_value_pair {
            key: cstr_as_string(
                b"complete_arg\0".as_ptr() as *const ::core::ffi::c_char,
            ),
            value: if (*cmd).uc_compl_arg.is_null() {
                object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                }
            } else {
                object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string((*cmd).uc_compl_arg),
                    },
                }
            },
        };
        let mut obj: Object = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        if (*cmd).uc_argt & EX_COUNT as uint32_t != 0 {
            if (*cmd).uc_def >= 0 as int64_t {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: arena_printf(
                            arena,
                            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).uc_def,
                        ),
                    },
                };
            } else {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(
                            b"0\0".as_ptr() as *const ::core::ffi::c_char,
                        ),
                    },
                };
            }
        }
        let c2rust_fresh58 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh58 as isize) = key_value_pair {
            key: cstr_as_string(b"count\0".as_ptr() as *const ::core::ffi::c_char),
            value: obj,
        };
        obj = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        if (*cmd).uc_argt & EX_RANGE as uint32_t != 0 {
            if (*cmd).uc_argt & EX_DFLALL as uint32_t != 0 {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: String_0 {
                            data: b"%\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                };
            } else if (*cmd).uc_def >= 0 as int64_t {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: arena_printf(
                            arena,
                            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).uc_def,
                        ),
                    },
                };
            } else {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: String_0 {
                            data: b".\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                };
            }
        }
        let c2rust_fresh59 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh59 as isize) = key_value_pair {
            key: cstr_as_string(b"range\0".as_ptr() as *const ::core::ffi::c_char),
            value: obj,
        };
        obj = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while addr_type_complete[j as usize].expand as ::core::ffi::c_uint
            != ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if addr_type_complete[j as usize].expand as ::core::ffi::c_uint
                != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                && addr_type_complete[j as usize].expand as ::core::ffi::c_uint
                    == (*cmd).uc_addr_type as ::core::ffi::c_uint
            {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(addr_type_complete[j as usize].name),
                    },
                };
                break;
            } else {
                j += 1;
            }
        }
        let c2rust_fresh60 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.offset(c2rust_fresh60 as isize) = key_value_pair {
            key: cstr_as_string(b"addr\0".as_ptr() as *const ::core::ffi::c_char),
            value: obj,
        };
        let c2rust_fresh61 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh61 as isize) = key_value_pair {
            key: cstr_as_string((*cmd).uc_name),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: d },
            },
        };
        i += 1;
    }
    return rv;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
