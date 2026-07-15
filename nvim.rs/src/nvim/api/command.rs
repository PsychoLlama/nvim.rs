extern "C" {
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn arena_alloc(
        arena: *mut Arena,
        size: size_t,
        align: bool,
    ) -> *mut ::core::ffi::c_void;
    fn arena_memdupz(
        arena: *mut Arena,
        buf: *const ::core::ffi::c_char,
        size: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn KeyDict_cmd_magic_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_cmd_mods_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
    fn KeyDict_cmd_mods_filter_get_field(
        str: *const ::core::ffi::c_char,
        len: size_t,
    ) -> *mut KeySetLink;
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
    fn api_err_required(err: *mut Error, name: *const ::core::ffi::c_char);
    fn try_enter(tstate: *mut TryState);
    fn try_leave(tstate: *const TryState, err: *mut Error);
    fn find_buffer_by_handle(buffer: Buffer, err: *mut Error) -> *mut buf_T;
    fn string_to_cstr(str: String_0) -> *mut ::core::ffi::c_char;
    fn cstrn_as_string(str: *mut ::core::ffi::c_char, maxsize: size_t) -> String_0;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn arena_array(arena: *mut Arena, max_size: size_t) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: size_t) -> Dict;
    fn arena_string(arena: *mut Arena, str: String_0) -> String_0;
    fn api_set_error(
        err: *mut Error,
        errType: ErrorType,
        format: *const ::core::ffi::c_char,
        ...
    );
    fn api_typename(t: ObjectType) -> *mut ::core::ffi::c_char;
    fn api_dict_to_keydict(
        retval: *mut ::core::ffi::c_void,
        hashy: FieldHashfn,
        dict: Dict,
        err: *mut Error,
    ) -> bool;
    fn api_set_sctx(channel_id: uint64_t) -> sctx_T;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn has_event(event: event_T) -> bool;
    fn kv_do_printf(
        str: *mut StringBuilder,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn set_cmd_addr_type(eap: *mut exarg_T, p: *mut ::core::ffi::c_char);
    fn get_cmd_default_range(eap: *mut exarg_T) -> linenr_T;
    fn set_cmd_dflall_range(eap: *mut exarg_T);
    fn set_cmd_count(eap: *mut exarg_T, count: linenr_T, validate: bool);
    fn is_cmd_ni(cmdidx: cmdidx_T) -> bool;
    fn parse_cmdline(
        cmdline: *mut *mut ::core::ffi::c_char,
        eap: *mut exarg_T,
        cmdinfo: *mut CmdParseInfo,
        errormsg: *mut *const ::core::ffi::c_char,
    ) -> bool;
    fn execute_cmd(
        eap: *mut exarg_T,
        cmdinfo: *mut CmdParseInfo,
        preview: bool,
    ) -> ::core::ffi::c_int;
    fn undo_cmdmod(cmod: *mut cmdmod_T);
    fn find_ex_command(
        eap: *mut exarg_T,
        full: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn excmd_get_argt(idx: cmdidx_T) -> uint32_t;
    fn invalid_range(eap: *mut exarg_T) -> *mut ::core::ffi::c_char;
    fn replace_makeprg(
        eap: *mut exarg_T,
        arg: *mut ::core::ffi::c_char,
        cmdlinep: *mut *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn getargcmd(argp: *mut *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getargopt(eap: *mut exarg_T) -> ::core::ffi::c_int;
    fn get_command_name(
        xp: *mut expand_T,
        idx: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn is_map_cmd(cmdidx: cmdidx_T) -> bool;
    fn aborting() -> bool;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(
        gap: *mut garray_T,
        itemsize: ::core::ffi::c_int,
        growsize: ::core::ffi::c_int,
    );
    static mut ucmds: garray_T;
    fn get_user_command_name(
        idx: ::core::ffi::c_int,
        cmdidx: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn parse_addr_type_arg(
        value: *mut ::core::ffi::c_char,
        vallen: ::core::ffi::c_int,
        addr_type_arg: *mut cmd_addr_T,
    ) -> ::core::ffi::c_int;
    fn parse_compl_arg(
        value: *const ::core::ffi::c_char,
        vallen: ::core::ffi::c_int,
        complp: *mut ::core::ffi::c_int,
        argt: *mut uint32_t,
        compl_arg: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn uc_validate_name(name: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn uc_add_command(
        name: *mut ::core::ffi::c_char,
        name_len: size_t,
        rep: *const ::core::ffi::c_char,
        argt: uint32_t,
        def: int64_t,
        flags: ::core::ffi::c_int,
        context: ::core::ffi::c_int,
        compl_arg: *mut ::core::ffi::c_char,
        compl_luaref: LuaRef,
        preview_luaref: LuaRef,
        addr_type: cmd_addr_T,
        luaref: LuaRef,
        force: bool,
    ) -> ::core::ffi::c_int;
    fn free_ucmd(cmd: *mut ucmd_T);
    fn uc_split_args_iter(
        arg: *const ::core::ffi::c_char,
        arglen: size_t,
        end: *mut size_t,
        buf: *mut ::core::ffi::c_char,
        len: *mut size_t,
    ) -> bool;
    fn uc_nargs_upper_bound(arg: *const ::core::ffi::c_char, arglen: size_t) -> size_t;
    fn commands_array(buf: *mut buf_T, arena: *mut Arena) -> Dict;
    static mut msg_col: ::core::ffi::c_int;
    static mut current_sctx: sctx_T;
    static mut curbuf: *mut buf_T;
    static mut msg_silent: ::core::ffi::c_int;
    static mut redir_off: bool;
    static mut capture_ga: *mut garray_T;
    fn api_free_luaref(ref_0: LuaRef);
    fn api_new_luaref(original_ref: LuaRef) -> LuaRef;
    fn mb_islower(a: ::core::ffi::c_int) -> bool;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn valid_yank_reg(regname: ::core::ffi::c_int, writing: bool) -> bool;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StringBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
pub type Buffer = handle_T;
pub type OptionalKeys = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeySetLink {
    pub str: *mut ::core::ffi::c_char,
    pub ptr_off: size_t,
    pub type_0: ::core::ffi::c_int,
    pub opt_index: ::core::ffi::c_int,
    pub is_hlgroup: bool,
}
pub type FieldHashfn = Option<
    unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_empty {
    pub is_set__empty_: OptionalKeys,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_get_commands {
    pub builtin: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_user_command {
    pub is_set__user_command_: OptionalKeys,
    pub addr: Object,
    pub bang: Boolean,
    pub bar: Boolean,
    pub complete: Object,
    pub count: Object,
    pub desc: Object,
    pub force: Boolean,
    pub keepscript: Boolean,
    pub nargs: Object,
    pub preview: Object,
    pub range: Object,
    pub register_: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd {
    pub is_set__cmd_: OptionalKeys,
    pub cmd: String_0,
    pub range: Array,
    pub count: Integer,
    pub reg: String_0,
    pub bang: Boolean,
    pub args: Array,
    pub magic: Dict,
    pub mods: Dict,
    pub nargs: Object,
    pub addr: String_0,
    pub nextcmd: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd_magic {
    pub is_set__cmd_magic_: OptionalKeys,
    pub file: Boolean,
    pub bar: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd_mods {
    pub is_set__cmd_mods_: OptionalKeys,
    pub silent: Boolean,
    pub emsg_silent: Boolean,
    pub unsilent: Boolean,
    pub filter: Dict,
    pub sandbox: Boolean,
    pub noautocmd: Boolean,
    pub browse: Boolean,
    pub confirm: Boolean,
    pub hide: Boolean,
    pub horizontal: Boolean,
    pub keepalt: Boolean,
    pub keepjumps: Boolean,
    pub keepmarks: Boolean,
    pub keeppatterns: Boolean,
    pub lockmarks: Boolean,
    pub noswapfile: Boolean,
    pub tab: Integer,
    pub verbose: Integer,
    pub vertical: Boolean,
    pub split: String_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd_mods_filter {
    pub is_set__cmd_mods_filter_: OptionalKeys,
    pub pattern: String_0,
    pub force: Boolean,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_cmd_opts {
    pub output: Boolean,
}
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
pub struct CmdParseInfo {
    pub cmdmod: cmdmod_T,
    pub magic: C2Rust_Unnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_13 {
    pub file: bool,
    pub bar: bool,
}
pub const WSP_ABOVE: C2Rust_Unnamed_19 = 128;
pub const WSP_BELOW: C2Rust_Unnamed_19 = 64;
pub const WSP_TOP: C2Rust_Unnamed_19 = 8;
pub const WSP_BOT: C2Rust_Unnamed_19 = 16;
pub const WSP_HOR: C2Rust_Unnamed_19 = 4;
pub const WSP_VERT: C2Rust_Unnamed_19 = 2;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_17 = 8192;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_17 = 2048;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_17 = 4096;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_17 = 512;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_17 = 1024;
pub const CMOD_KEEPALT: C2Rust_Unnamed_17 = 256;
pub const CMOD_HIDE: C2Rust_Unnamed_17 = 32;
pub const CMOD_CONFIRM: C2Rust_Unnamed_17 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_17 = 64;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_17 = 16;
pub const CMOD_SANDBOX: C2Rust_Unnamed_17 = 1;
pub const CMOD_UNSILENT: C2Rust_Unnamed_17 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_17 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_17 = 2;
pub type exarg_T = exarg;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cstack_T {
    pub cs_flags: [::core::ffi::c_int; 50],
    pub cs_pending: [::core::ffi::c_char; 50],
    pub cs_pend: C2Rust_Unnamed_14,
    pub cs_forinfo: [*mut ::core::ffi::c_void; 50],
    pub cs_line: [::core::ffi::c_int; 50],
    pub cs_idx: ::core::ffi::c_int,
    pub cs_looplevel: ::core::ffi::c_int,
    pub cs_trylevel: ::core::ffi::c_int,
    pub cs_emsg_silent_list: *mut eslist_T,
    pub cs_lflags: ::core::ffi::c_int,
}
pub type eslist_T = eslist_elem;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct eslist_elem {
    pub saved_emsg_silent: ::core::ffi::c_int,
    pub next: *mut eslist_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_14 {
    pub csp_rv: [*mut ::core::ffi::c_void; 50],
    pub csp_ex: [*mut ::core::ffi::c_void; 50],
}
pub type LineGetter = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
        ::core::ffi::c_int,
        bool,
    ) -> *mut ::core::ffi::c_char,
>;
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
pub type cmdidx_T = CMD_index;
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
pub struct TryState {
    pub current_exception: *mut except_T,
    pub private_msg_list: *mut msglist_T,
    pub msg_list: *const *const msglist_T,
    pub got_int: ::core::ffi::c_int,
    pub did_throw: bool,
    pub need_rethrow: ::core::ffi::c_int,
    pub did_emsg: ::core::ffi::c_int,
}
pub type msglist_T = msglist;
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
pub type except_T = vim_exception;
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
pub type except_type_T = ::core::ffi::c_uint;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
pub const NUMBUFLEN: C2Rust_Unnamed_15 = 65;
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
pub const EXPAND_NOTHING: C2Rust_Unnamed_16 = 0;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_16 = 32;
pub const UC_BUFFER: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_16 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_16 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_16 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_16 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_16 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_16 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_16 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_16 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_16 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_16 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_16 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_16 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_16 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_16 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_16 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_16 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_16 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_16 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_16 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_16 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_16 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_16 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_16 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_16 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_16 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_16 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_16 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_16 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_16 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_16 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_16 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_16 = 33;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_16 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_16 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_16 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_16 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_16 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_16 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_16 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_16 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_16 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_16 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_16 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_16 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_16 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_16 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_16 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_16 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_16 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_16 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_16 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_16 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_16 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_16 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_16 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_16 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_16 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_16 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_16 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_16 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_16 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_16 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_16 = 1;
pub const EXPAND_OK: C2Rust_Unnamed_16 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_16 = -2;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_19 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_19 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_19 = 256;
pub const WSP_HELP: C2Rust_Unnamed_19 = 32;
pub const WSP_ROOM: C2Rust_Unnamed_19 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__addr: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__count: ::core::ffi::c_int = 5
    as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__force: ::core::ffi::c_int = 6
    as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__nargs: ::core::ffi::c_int = 7
    as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__range: ::core::ffi::c_int = 8
    as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__preview: ::core::ffi::c_int = 9
    as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_user_command__complete: ::core::ffi::c_int = 10
    as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__cmd: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__reg: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__bang: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__addr: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__mods: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__args: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__count: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__magic: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__nargs: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__range: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd__nextcmd: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_magic__bar: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_magic__file: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__tab: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__split: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__filter: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods__verbose: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_cmd_mods_filter__pattern: ::core::ffi::c_int = 2
    as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
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
pub const EX_CMDARG: ::core::ffi::c_uint = 0x4000 as ::core::ffi::c_uint;
pub const EX_ARGOPT: ::core::ffi::c_uint = 0x20000 as ::core::ffi::c_uint;
pub const EX_SBOXOK: ::core::ffi::c_uint = 0x40000 as ::core::ffi::c_uint;
pub const EX_KEEPSCRIPT: ::core::ffi::c_uint = 0x4000000 as ::core::ffi::c_uint;
pub const EX_PREVIEW: ::core::ffi::c_uint = 0x8000000 as ::core::ffi::c_uint;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 66] = unsafe {
    ::core::mem::transmute::<
        [u8; 66],
        [::core::ffi::c_char; 66],
    >(*b"void build_cmdline_str(char **, exarg_T *, CmdParseInfo *, Array)\0")
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
unsafe extern "C" fn parse_map_cmd(
    mut arg_str: *const ::core::ffi::c_char,
    mut arena: *mut Arena,
) -> Array {
    let mut args: Array = arena_array(arena, 2 as size_t);
    let mut lhs_start: *mut ::core::ffi::c_char = arg_str as *mut ::core::ffi::c_char;
    let mut lhs_end: *mut ::core::ffi::c_char = skiptowhite(lhs_start);
    let mut lhs_len: size_t = lhs_end.offset_from(lhs_start) as size_t;
    let c2rust_fresh28 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh28 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstrn_as_string(lhs_start, lhs_len),
        },
    };
    let mut rhs_start: *mut ::core::ffi::c_char = skipwhite(lhs_end);
    if *rhs_start as ::core::ffi::c_int != NUL {
        let mut rhs_len: size_t = strlen(rhs_start);
        let c2rust_fresh29 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh29 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstrn_as_string(rhs_start, rhs_len),
            },
        };
    }
    return args;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_parse_cmd(
    mut str: String_0,
    mut opts: *mut KeyDict_empty,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> KeyDict_cmd {
    let mut args: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut length: size_t = 0;
    let mut cmd: *mut ucmd_T = ::core::ptr::null_mut::<ucmd_T>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut nargs: [::core::ffi::c_char; 2] = [0; 2];
    let mut addr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut mods: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut filter: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut split: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut magic: Dict = Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut result: KeyDict_cmd = KeyDict_cmd {
        is_set__cmd_: 0 as OptionalKeys,
        cmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        range: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        count: 0,
        reg: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        bang: false,
        args: Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        },
        magic: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        mods: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
        nargs: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        addr: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        nextcmd: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
    };
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    let mut cmdinfo: CmdParseInfo = CmdParseInfo {
        cmdmod: cmdmod_T {
            cmod_flags: 0,
            cmod_split: 0,
            cmod_tab: 0,
            cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_filter_regmatch: regmatch_T {
                regprog: ::core::ptr::null_mut::<regprog_T>(),
                startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                rm_matchcol: 0,
                rm_ic: false,
            },
            cmod_filter_force: false,
            cmod_verbose: 0,
            cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_did_sandbox: 0,
            cmod_verbose_save: 0,
            cmod_save_msg_silent: 0,
            cmod_save_msg_scroll: 0,
            cmod_did_esilent: 0,
        },
        magic: C2Rust_Unnamed_13 {
            file: false,
            bar: false,
        },
    };
    let mut cmdline: *mut ::core::ffi::c_char = arena_memdupz(arena, str.data, str.size);
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<
        ::core::ffi::c_char,
    >();
    if !parse_cmdline(
        &raw mut cmdline,
        &raw mut ea,
        &raw mut cmdinfo,
        &raw mut errormsg,
    ) {
        if !errormsg.is_null() {
            api_set_error(
                err,
                kErrorTypeException,
                b"Parsing command-line: %s\0".as_ptr() as *const ::core::ffi::c_char,
                errormsg,
            );
        } else {
            api_set_error(
                err,
                kErrorTypeException,
                b"Parsing command-line\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    } else {
        args = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        length = strlen(ea.arg);
        if ea.cmdidx as ::core::ffi::c_int != CMD_SIZE as ::core::ffi::c_int
            && is_map_cmd(ea.cmdidx) as ::core::ffi::c_int != 0
            && *ea.arg as ::core::ffi::c_int != NUL
        {
            args = parse_map_cmd(ea.arg, arena);
        } else if ea.argt & EX_NOSPC as uint32_t != 0 {
            if *ea.arg as ::core::ffi::c_int != NUL {
                args = arena_array(arena, 1 as size_t);
                let c2rust_fresh0 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh0 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstrn_as_string(ea.arg, length),
                    },
                };
            }
        } else {
            let mut end: size_t = 0 as size_t;
            let mut len: size_t = 0 as size_t;
            let mut buf: *mut ::core::ffi::c_char = arena_alloc(
                arena,
                length.wrapping_add(1 as size_t),
                false_0 != 0,
            ) as *mut ::core::ffi::c_char;
            let mut done: bool = false_0 != 0;
            args = arena_array(arena, uc_nargs_upper_bound(ea.arg, length));
            while !done {
                done = uc_split_args_iter(
                    ea.arg,
                    length,
                    &raw mut end,
                    buf,
                    &raw mut len,
                );
                if len > 0 as size_t {
                    let c2rust_fresh1 = args.size;
                    args.size = args.size.wrapping_add(1);
                    *args.items.offset(c2rust_fresh1 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: cstrn_as_string(buf, len),
                        },
                    };
                    buf = buf.offset(len.wrapping_add(1 as size_t) as isize);
                }
            }
        }
        cmd = ::core::ptr::null_mut::<ucmd_T>();
        if ea.cmdidx as ::core::ffi::c_int == CMD_USER as ::core::ffi::c_int {
            cmd = (ucmds.ga_data as *mut ucmd_T).offset(ea.useridx as isize);
        } else if ea.cmdidx as ::core::ffi::c_int == CMD_USER_BUF as ::core::ffi::c_int {
            cmd = ((*curbuf).b_ucmds.ga_data as *mut ucmd_T).offset(ea.useridx as isize);
        }
        name = (if ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            (if !cmd.is_null() {
                (*cmd).uc_name
            } else {
                get_command_name(
                    ::core::ptr::null_mut::<expand_T>(),
                    ea.cmdidx as ::core::ffi::c_int,
                )
            }) as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__cmd) as OptionalKeys;
        result.cmd = cstr_as_string(name);
        if ea.argt & EX_RANGE as uint32_t != 0 && ea.addr_count > 0 as ::core::ffi::c_int
        {
            let mut range: Array = arena_array(arena, 2 as size_t);
            if ea.addr_count > 1 as ::core::ffi::c_int {
                let c2rust_fresh2 = range.size;
                range.size = range.size.wrapping_add(1);
                *range.items.offset(c2rust_fresh2 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: ea.line1 as Integer,
                    },
                };
            }
            let c2rust_fresh3 = range.size;
            range.size = range.size.wrapping_add(1);
            *range.items.offset(c2rust_fresh3 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: ea.line2 as Integer,
                },
            };
            result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range)
                as OptionalKeys;
            result.range = range;
        }
        if ea.argt & EX_COUNT as uint32_t != 0 {
            let mut count: Integer = if ea.addr_count > 0 as ::core::ffi::c_int {
                ea.line2 as Integer
            } else if !cmd.is_null() {
                (*cmd).uc_def as Integer
            } else {
                0 as Integer
            };
            if ea.addr_count > 0 as ::core::ffi::c_int
                || !cmd.is_null() && (*cmd).uc_def != 0 as int64_t
                || count != 0 as Integer
            {
                result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__count)
                    as OptionalKeys;
                result.count = count;
            }
        }
        if ea.argt & EX_REGSTR as uint32_t != 0 {
            let mut reg: [::core::ffi::c_char; 2] = [
                ea.regname as ::core::ffi::c_char,
                NUL as ::core::ffi::c_char,
            ];
            result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__reg)
                as OptionalKeys;
            result.reg = arena_string(
                arena,
                cstr_as_string(&raw mut reg as *mut ::core::ffi::c_char),
            );
        }
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__bang)
            as OptionalKeys;
        result.bang = ea.forceit != 0;
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__args)
            as OptionalKeys;
        result.args = args;
        nargs = [0; 2];
        if ea.argt & EX_EXTRA as uint32_t != 0 {
            if ea.argt & EX_NOSPC as uint32_t != 0 {
                if ea.argt & EX_NEEDARG as uint32_t != 0 {
                    nargs[0 as ::core::ffi::c_int as usize] = '1' as ::core::ffi::c_char;
                } else {
                    nargs[0 as ::core::ffi::c_int as usize] = '?' as ::core::ffi::c_char;
                }
            } else if ea.argt & EX_NEEDARG as uint32_t != 0 {
                nargs[0 as ::core::ffi::c_int as usize] = '+' as ::core::ffi::c_char;
            } else {
                nargs[0 as ::core::ffi::c_int as usize] = '*' as ::core::ffi::c_char;
            }
        } else {
            nargs[0 as ::core::ffi::c_int as usize] = '0' as ::core::ffi::c_char;
        }
        nargs[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__nargs)
            as OptionalKeys;
        result.nargs = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: arena_string(
                    arena,
                    cstr_as_string(&raw mut nargs as *mut ::core::ffi::c_char),
                ),
            },
        };
        addr = ::core::ptr::null_mut::<::core::ffi::c_char>();
        match ea.addr_type as ::core::ffi::c_uint {
            0 => {
                addr = b"line\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            2 => {
                addr = b"arg\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            4 => {
                addr = b"buf\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            3 => {
                addr = b"load\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            1 => {
                addr = b"win\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            5 => {
                addr = b"tab\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            8 => {
                addr = b"qf\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            11 => {
                addr = b"none\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            _ => {
                addr = b"?\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
        }
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__addr)
            as OptionalKeys;
        result.addr = cstr_as_string(addr);
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__nextcmd)
            as OptionalKeys;
        result.nextcmd = cstr_as_string(ea.nextcmd);
        mods = arena_dict(arena, 20 as size_t);
        filter = arena_dict(arena, 2 as size_t);
        let c2rust_fresh4 = filter.size;
        filter.size = filter.size.wrapping_add(1);
        *filter.items.offset(c2rust_fresh4 as isize) = key_value_pair {
            key: cstr_as_string(b"pattern\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(
                        arena,
                        cstr_as_string(cmdinfo.cmdmod.cmod_filter_pat),
                    ),
                },
            },
        };
        let c2rust_fresh5 = filter.size;
        filter.size = filter.size.wrapping_add(1);
        *filter.items.offset(c2rust_fresh5 as isize) = key_value_pair {
            key: cstr_as_string(b"force\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_filter_force,
                },
            },
        };
        let c2rust_fresh6 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh6 as isize) = key_value_pair {
            key: cstr_as_string(b"filter\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: filter },
            },
        };
        let c2rust_fresh7 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh7 as isize) = key_value_pair {
            key: cstr_as_string(b"silent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_SILENT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh8 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh8 as isize) = key_value_pair {
            key: cstr_as_string(b"emsg_silent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_ERRSILENT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh9 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"unsilent\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_UNSILENT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh10 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"sandbox\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_SANDBOX as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh11 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh11 as isize) = key_value_pair {
            key: cstr_as_string(b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_NOAUTOCMD as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh12 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh12 as isize) = key_value_pair {
            key: cstr_as_string(b"tab\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (cmdinfo.cmdmod.cmod_tab - 1 as ::core::ffi::c_int)
                        as Integer,
                },
            },
        };
        let c2rust_fresh13 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh13 as isize) = key_value_pair {
            key: cstr_as_string(b"verbose\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (cmdinfo.cmdmod.cmod_verbose - 1 as ::core::ffi::c_int)
                        as Integer,
                },
            },
        };
        let c2rust_fresh14 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh14 as isize) = key_value_pair {
            key: cstr_as_string(b"browse\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_BROWSE as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh15 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh15 as isize) = key_value_pair {
            key: cstr_as_string(b"confirm\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_CONFIRM as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh16 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh16 as isize) = key_value_pair {
            key: cstr_as_string(b"hide\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags & CMOD_HIDE as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        let c2rust_fresh17 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh17 as isize) = key_value_pair {
            key: cstr_as_string(b"keepalt\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_KEEPALT as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh18 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh18 as isize) = key_value_pair {
            key: cstr_as_string(b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_KEEPJUMPS as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh19 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh19 as isize) = key_value_pair {
            key: cstr_as_string(b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_KEEPMARKS as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh20 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh20 as isize) = key_value_pair {
            key: cstr_as_string(
                b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char,
            ),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_KEEPPATTERNS as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh21 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh21 as isize) = key_value_pair {
            key: cstr_as_string(b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_LOCKMARKS as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh22 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh22 as isize) = key_value_pair {
            key: cstr_as_string(b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_flags
                        & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh23 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh23 as isize) = key_value_pair {
            key: cstr_as_string(b"vertical\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_split & WSP_VERT as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        let c2rust_fresh24 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh24 as isize) = key_value_pair {
            key: cstr_as_string(b"horizontal\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.cmdmod.cmod_split & WSP_HOR as ::core::ffi::c_int
                        != 0,
                },
            },
        };
        split = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if cmdinfo.cmdmod.cmod_split & WSP_BOT as ::core::ffi::c_int != 0 {
            split = b"botright\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        } else if cmdinfo.cmdmod.cmod_split & WSP_TOP as ::core::ffi::c_int != 0 {
            split = b"topleft\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        } else if cmdinfo.cmdmod.cmod_split & WSP_BELOW as ::core::ffi::c_int != 0 {
            split = b"belowright\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        } else if cmdinfo.cmdmod.cmod_split & WSP_ABOVE as ::core::ffi::c_int != 0 {
            split = b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        } else {
            split = b"\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char;
        }
        let c2rust_fresh25 = mods.size;
        mods.size = mods.size.wrapping_add(1);
        *mods.items.offset(c2rust_fresh25 as isize) = key_value_pair {
            key: cstr_as_string(b"split\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(split),
                },
            },
        };
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__mods)
            as OptionalKeys;
        result.mods = mods;
        magic = arena_dict(arena, 2 as size_t);
        let c2rust_fresh26 = magic.size;
        magic.size = magic.size.wrapping_add(1);
        *magic.items.offset(c2rust_fresh26 as isize) = key_value_pair {
            key: cstr_as_string(b"file\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.magic.file,
                },
            },
        };
        let c2rust_fresh27 = magic.size;
        magic.size = magic.size.wrapping_add(1);
        *magic.items.offset(c2rust_fresh27 as isize) = key_value_pair {
            key: cstr_as_string(b"bar\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: cmdinfo.magic.bar,
                },
            },
        };
        result.is_set__cmd_ = (result.is_set__cmd_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__magic)
            as OptionalKeys;
        result.magic = magic;
        undo_cmdmod(&raw mut cmdinfo.cmdmod);
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_cmd(
    mut channel_id: uint64_t,
    mut cmd: *mut KeyDict_cmd,
    mut opts: *mut KeyDict_cmd_opts,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut range_only: bool = false;
    let mut count_from_first_arg: bool = false;
    let mut capture_local: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut save_msg_silent: ::core::ffi::c_int = 0;
    let mut save_redir_off: bool = false;
    let mut save_capture_ga: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    let mut save_msg_col: ::core::ffi::c_int = 0;
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    memset(
        &raw mut ea as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<exarg_T>(),
    );
    let mut cmdinfo: CmdParseInfo = CmdParseInfo {
        cmdmod: cmdmod_T {
            cmod_flags: 0,
            cmod_split: 0,
            cmod_tab: 0,
            cmod_filter_pat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_filter_regmatch: regmatch_T {
                regprog: ::core::ptr::null_mut::<regprog_T>(),
                startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                rm_matchcol: 0,
                rm_ic: false,
            },
            cmod_filter_force: false,
            cmod_verbose: 0,
            cmod_save_ei: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmod_did_sandbox: 0,
            cmod_verbose_save: 0,
            cmod_save_msg_silent: 0,
            cmod_save_msg_scroll: 0,
            cmod_did_esilent: 0,
        },
        magic: C2Rust_Unnamed_13 {
            file: false,
            bar: false,
        },
    };
    memset(
        &raw mut cmdinfo as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<CmdParseInfo>(),
    );
    let mut cmdline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut cmdname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut retv: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    '_end: {
        if !((*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
        {
            api_err_required(err, b"cmd\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            if *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int == NUL
            {
                if !((*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 10 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong && (*cmd).range.size > 0 as size_t
                    || (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong)
                {
                    api_err_exp(
                        err,
                        b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                        b"non-empty String\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_end;
                }
            }
            cmdname = arena_string(arena, (*cmd).cmd).data;
            ea.cmd = cmdname;
            p = find_ex_command(
                &raw mut ea,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            if !p.is_null()
                && ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                && (*ea.cmd as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                    && *ea.cmd as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint)
                && has_event(EVENT_CMDUNDEFINED) as ::core::ffi::c_int != 0
            {
                p = arena_string(arena, (*cmd).cmd).data;
                let mut ret: ::core::ffi::c_int = apply_autocmds(
                    EVENT_CMDUNDEFINED,
                    p,
                    p,
                    true_0 != 0,
                    ::core::ptr::null_mut::<buf_T>(),
                ) as ::core::ffi::c_int;
                p = if ret != 0 && !aborting() {
                    find_ex_command(
                        &raw mut ea,
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    )
                } else {
                    ea.cmd
                };
            }
            range_only = ea.cmdidx as ::core::ffi::c_int
                == CMD_SIZE as ::core::ffi::c_int
                && *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int == NUL
                && (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range
                    != 0 as ::core::ffi::c_ulonglong && (*cmd).range.size > 0 as size_t;
            if !(ea.cmdidx as ::core::ffi::c_int == CMD_SIZE as ::core::ffi::c_int
                && *(*cmd).cmd.data.offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int == NUL
                && (!((*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range
                    != 0 as ::core::ffi::c_ulonglong)
                    || (*cmd).range.size == 0 as size_t)
                && (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__mods
                    != 0 as ::core::ffi::c_ulonglong)
            {
                if !(!p.is_null()
                    && ea.cmdidx as ::core::ffi::c_int != CMD_SIZE as ::core::ffi::c_int
                    || range_only as ::core::ffi::c_int != 0)
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Command not found: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        cmdname,
                    );
                } else if !(range_only as ::core::ffi::c_int != 0
                    || !is_cmd_ni(ea.cmdidx))
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Command not implemented: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        cmdname,
                    );
                } else {
                    if !range_only {
                        let mut fullname: *const ::core::ffi::c_char = if (ea.cmdidx
                            as ::core::ffi::c_int) < 0 as ::core::ffi::c_int
                        {
                            get_user_command_name(
                                ea.useridx,
                                ea.cmdidx as ::core::ffi::c_int,
                            )
                        } else {
                            get_command_name(
                                ::core::ptr::null_mut::<expand_T>(),
                                ea.cmdidx as ::core::ffi::c_int,
                            )
                        };
                        if !(strncmp(fullname, cmdname, strlen(cmdname))
                            == 0 as ::core::ffi::c_int)
                        {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Invalid command: \"%s\"\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                cmdname,
                            );
                            break '_end;
                        }
                    }
                    if range_only {
                        ea.argt = (EX_RANGE | EX_SBOXOK) as uint32_t;
                    } else if !((ea.cmdidx as ::core::ffi::c_int)
                        < 0 as ::core::ffi::c_int)
                    {
                        ea.argt = excmd_get_argt(ea.cmdidx);
                    }
                    count_from_first_arg = false_0 != 0;
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__args
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if (*cmd).args.size == 1 as size_t
                            && ea.argt & EX_COUNT as uint32_t != 0
                            && ea.argt & EX_EXTRA as uint32_t == 0
                        {
                            let mut first_arg: Object = *(*cmd)
                                .args
                                .items
                                .offset(0 as ::core::ffi::c_int as isize);
                            let mut is_numeric: bool = false_0 != 0;
                            let mut count_value: int64_t = 0 as int64_t;
                            if first_arg.type_0 as ::core::ffi::c_uint
                                == kObjectTypeInteger as ::core::ffi::c_int
                                    as ::core::ffi::c_uint
                            {
                                is_numeric = true_0 != 0;
                                count_value = first_arg.data.integer as int64_t;
                            } else if first_arg.type_0 as ::core::ffi::c_uint
                                == kObjectTypeString as ::core::ffi::c_int
                                    as ::core::ffi::c_uint
                            {
                                let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                                    ::core::ffi::c_char,
                                >();
                                let mut val: ::core::ffi::c_long = strtol(
                                    first_arg.data.string.data,
                                    &raw mut endptr,
                                    10 as ::core::ffi::c_int,
                                );
                                if *endptr as ::core::ffi::c_int
                                    == '\0' as ::core::ffi::c_int
                                    && first_arg.data.string.size > 0 as size_t
                                {
                                    is_numeric = true_0 != 0;
                                    count_value = val as int64_t;
                                }
                            }
                            if is_numeric as ::core::ffi::c_int != 0
                                && count_value >= 0 as int64_t
                            {
                                count_from_first_arg = true_0 != 0;
                                ea.addr_count = 1 as ::core::ffi::c_int;
                                ea.line2 = count_value as linenr_T;
                                ea.line1 = ea.line2;
                                args = arena_array(arena, 0 as size_t);
                            }
                        }
                        if !count_from_first_arg {
                            args = arena_array(arena, (*cmd).args.size);
                            let mut i: size_t = 0 as size_t;
                            while i < (*cmd).args.size {
                                let mut elem: Object = *(*cmd)
                                    .args
                                    .items
                                    .offset(i as isize);
                                let mut data_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                                    ::core::ffi::c_char,
                                >();
                                match elem.type_0 as ::core::ffi::c_uint {
                                    1 => {
                                        data_str = arena_alloc(arena, 2 as size_t, false_0 != 0)
                                            as *mut ::core::ffi::c_char;
                                        *data_str.offset(0 as ::core::ffi::c_int as isize) = (if elem
                                            .data
                                            .boolean as ::core::ffi::c_int != 0
                                        {
                                            '1' as ::core::ffi::c_int
                                        } else {
                                            '0' as ::core::ffi::c_int
                                        }) as ::core::ffi::c_char;
                                        *data_str.offset(1 as ::core::ffi::c_int as isize) = NUL
                                            as ::core::ffi::c_char;
                                        let c2rust_fresh30 = args.size;
                                        args.size = args.size.wrapping_add(1);
                                        *args.items.offset(c2rust_fresh30 as isize) = object {
                                            type_0: kObjectTypeString,
                                            data: C2Rust_Unnamed {
                                                string: cstr_as_string(data_str),
                                            },
                                        };
                                    }
                                    8 | 9 | 10 | 2 => {
                                        data_str = arena_alloc(
                                            arena,
                                            NUMBUFLEN as ::core::ffi::c_int as size_t,
                                            false_0 != 0,
                                        ) as *mut ::core::ffi::c_char;
                                        snprintf(
                                            data_str,
                                            NUMBUFLEN as ::core::ffi::c_int as size_t,
                                            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                                            elem.data.integer,
                                        );
                                        let c2rust_fresh31 = args.size;
                                        args.size = args.size.wrapping_add(1);
                                        *args.items.offset(c2rust_fresh31 as isize) = object {
                                            type_0: kObjectTypeString,
                                            data: C2Rust_Unnamed {
                                                string: cstr_as_string(data_str),
                                            },
                                        };
                                    }
                                    4 => {
                                        if string_iswhite(elem.data.string) {
                                            api_err_exp(
                                                err,
                                                b"command arg\0".as_ptr() as *const ::core::ffi::c_char,
                                                b"non-whitespace\0".as_ptr() as *const ::core::ffi::c_char,
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                            break '_end;
                                        } else {
                                            let c2rust_fresh32 = args.size;
                                            args.size = args.size.wrapping_add(1);
                                            *args.items.offset(c2rust_fresh32 as isize) = elem;
                                        }
                                    }
                                    _ => {
                                        if true {
                                            api_err_exp(
                                                err,
                                                b"command arg\0".as_ptr() as *const ::core::ffi::c_char,
                                                b"valid type\0".as_ptr() as *const ::core::ffi::c_char,
                                                api_typename(elem.type_0),
                                            );
                                            break '_end;
                                        }
                                    }
                                }
                                i = i.wrapping_add(1);
                            }
                            let mut argc_valid: bool = false;
                            match ea.argt
                                & (EX_EXTRA as uint32_t | EX_NOSPC as uint32_t
                                    | EX_NEEDARG as uint32_t)
                            {
                                148 => {
                                    argc_valid = args.size == 1 as size_t;
                                }
                                20 => {
                                    argc_valid = args.size <= 1 as size_t;
                                }
                                132 => {
                                    argc_valid = args.size >= 1 as size_t;
                                }
                                EX_EXTRA => {
                                    argc_valid = true_0 != 0;
                                }
                                _ => {
                                    argc_valid = args.size == 0 as size_t;
                                }
                            }
                            if !argc_valid {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Wrong number of arguments\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                                break '_end;
                            }
                        }
                    }
                    if !range_only {
                        set_cmd_addr_type(
                            &raw mut ea,
                            if args.size > 0 as size_t {
                                (*args.items.offset(0 as ::core::ffi::c_int as isize))
                                    .data
                                    .string
                                    .data
                            } else {
                                ::core::ptr::null_mut::<::core::ffi::c_char>()
                            },
                        );
                    }
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__range
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if ea.argt & 0x1 as uint32_t == 0 {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Command cannot accept %s: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).cmd.data,
                            );
                            break '_end;
                        } else if !((*cmd).range.size <= 2 as size_t) {
                            api_err_exp(
                                err,
                                b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                b"<=2 elements\0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                            break '_end;
                        } else {
                            let mut range: Array = (*cmd).range;
                            ea.addr_count = range.size as ::core::ffi::c_int;
                            let mut i_0: size_t = 0 as size_t;
                            while i_0 < range.size {
                                let mut elem_0: Object = *range.items.offset(i_0 as isize);
                                if !(elem_0.type_0 as ::core::ffi::c_uint
                                    == kObjectTypeInteger as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                    && elem_0.data.integer >= 0 as Integer)
                                {
                                    api_err_exp(
                                        err,
                                        b"range element\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"non-negative Integer\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                    );
                                    break '_end;
                                } else {
                                    i_0 = i_0.wrapping_add(1);
                                }
                            }
                            if range.size > 0 as size_t {
                                ea.line1 = (*range
                                    .items
                                    .offset(0 as ::core::ffi::c_int as isize))
                                    .data
                                    .integer as linenr_T;
                                ea.line2 = (*range
                                    .items
                                    .offset(range.size.wrapping_sub(1 as size_t) as isize))
                                    .data
                                    .integer as linenr_T;
                            }
                            if !invalid_range(&raw mut ea).is_null() {
                                api_err_invalid(
                                    err,
                                    b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_end;
                            }
                        }
                    }
                    if ea.addr_count == 0 as ::core::ffi::c_int {
                        if ea.argt & EX_DFLALL as uint32_t != 0 {
                            set_cmd_dflall_range(&raw mut ea);
                        } else {
                            ea.line2 = get_cmd_default_range(&raw mut ea);
                            ea.line1 = ea.line2;
                            if ea.addr_type as ::core::ffi::c_uint
                                == ADDR_OTHER as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                ea.line2 = 1 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                    }
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__count
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if count_from_first_arg {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                b"Cannot specify both 'count' and numeric argument\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                            break '_end;
                        } else if ea.argt & 0x400 as uint32_t == 0 {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Command cannot accept %s: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"count\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).cmd.data,
                            );
                            break '_end;
                        } else if !((*cmd).count >= 0 as Integer) {
                            api_err_exp(
                                err,
                                b"count\0".as_ptr() as *const ::core::ffi::c_char,
                                b"non-negative Integer\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                            break '_end;
                        } else {
                            set_cmd_count(
                                &raw mut ea,
                                (*cmd).count as linenr_T,
                                true_0 != 0,
                            );
                        }
                    }
                    if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__reg
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if ea.argt & 0x200 as uint32_t == 0 {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Command cannot accept %s: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"register\0".as_ptr() as *const ::core::ffi::c_char,
                                (*cmd).cmd.data,
                            );
                            break '_end;
                        } else if !((*cmd).reg.size == 1 as size_t) {
                            api_err_exp(
                                err,
                                b"reg\0".as_ptr() as *const ::core::ffi::c_char,
                                b"single character\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                (*cmd).reg.data,
                            );
                            break '_end;
                        } else {
                            let mut regname: ::core::ffi::c_char = *(*cmd)
                                .reg
                                .data
                                .offset(0 as ::core::ffi::c_int as isize);
                            if !(regname as ::core::ffi::c_int
                                != '=' as ::core::ffi::c_int)
                            {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Cannot use register \"=\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                                break '_end;
                            } else if !valid_yank_reg(
                                regname as ::core::ffi::c_int,
                                !((ea.cmdidx as ::core::ffi::c_int)
                                    < 0 as ::core::ffi::c_int)
                                    && ea.cmdidx as ::core::ffi::c_int
                                        != CMD_put as ::core::ffi::c_int
                                    && ea.cmdidx as ::core::ffi::c_int
                                        != CMD_iput as ::core::ffi::c_int,
                            ) {
                                api_set_error(
                                    err,
                                    kErrorTypeValidation,
                                    b"Invalid register: \"%c\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    regname as ::core::ffi::c_int,
                                );
                                break '_end;
                            } else {
                                ea.regname = regname as uint8_t as ::core::ffi::c_int;
                            }
                        }
                    }
                    ea.forceit = (*cmd).bang as ::core::ffi::c_int;
                    if !(ea.forceit == 0 || ea.argt & 0x2 as uint32_t != 0) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Command cannot accept %s: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"bang\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).cmd.data,
                        );
                    } else {
                        if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__magic
                            != 0 as ::core::ffi::c_ulonglong
                        {
                            let mut magic: [KeyDict_cmd_magic; 1] = [
                                KeyDict_cmd_magic {
                                    is_set__cmd_magic_: 0 as OptionalKeys,
                                    file: false,
                                    bar: false,
                                },
                            ];
                            if !api_dict_to_keydict(
                                &raw mut magic as *mut KeyDict_cmd_magic
                                    as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict_cmd_magic_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        ) -> *mut KeySetLink,
                                ),
                                (*cmd).magic,
                                err,
                            ) {
                                break '_end;
                            } else {
                                cmdinfo.magic.file = if (*(&raw mut magic
                                    as *mut KeyDict_cmd_magic))
                                    .is_set__cmd_magic_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_magic__file
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*(&raw mut magic as *mut KeyDict_cmd_magic)).file
                                        as uint32_t
                                } else {
                                    ea.argt & EX_XFILE as uint32_t
                                } != 0;
                                cmdinfo.magic.bar = if (*(&raw mut magic
                                    as *mut KeyDict_cmd_magic))
                                    .is_set__cmd_magic_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_magic__bar
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    (*(&raw mut magic as *mut KeyDict_cmd_magic)).bar
                                        as uint32_t
                                } else {
                                    ea.argt & EX_TRLBAR as uint32_t
                                } != 0;
                                if cmdinfo.magic.file {
                                    ea.argt = (ea.argt as ::core::ffi::c_uint | EX_XFILE)
                                        as uint32_t;
                                } else {
                                    ea.argt = (ea.argt as ::core::ffi::c_uint & !EX_XFILE)
                                        as uint32_t;
                                }
                            }
                        } else {
                            cmdinfo.magic.file = ea.argt & EX_XFILE as uint32_t != 0;
                            cmdinfo.magic.bar = ea.argt & EX_TRLBAR as uint32_t != 0;
                        }
                        if (*cmd).is_set__cmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_cmd__mods
                            != 0 as ::core::ffi::c_ulonglong
                        {
                            let mut mods: [KeyDict_cmd_mods; 1] = [
                                KeyDict_cmd_mods {
                                    is_set__cmd_mods_: 0 as OptionalKeys,
                                    silent: false,
                                    emsg_silent: false,
                                    unsilent: false,
                                    filter: Dict {
                                        size: 0,
                                        capacity: 0,
                                        items: ::core::ptr::null_mut::<KeyValuePair>(),
                                    },
                                    sandbox: false,
                                    noautocmd: false,
                                    browse: false,
                                    confirm: false,
                                    hide: false,
                                    horizontal: false,
                                    keepalt: false,
                                    keepjumps: false,
                                    keepmarks: false,
                                    keeppatterns: false,
                                    lockmarks: false,
                                    noswapfile: false,
                                    tab: 0,
                                    verbose: 0,
                                    vertical: false,
                                    split: String_0 {
                                        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        size: 0,
                                    },
                                },
                            ];
                            if !api_dict_to_keydict(
                                &raw mut mods as *mut KeyDict_cmd_mods
                                    as *mut ::core::ffi::c_void,
                                Some(
                                    KeyDict_cmd_mods_get_field
                                        as unsafe extern "C" fn(
                                            *const ::core::ffi::c_char,
                                            size_t,
                                        ) -> *mut KeySetLink,
                                ),
                                (*cmd).mods,
                                err,
                            ) {
                                break '_end;
                            } else {
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods))
                                    .is_set__cmd_mods_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_mods__filter
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    let mut filter: [KeyDict_cmd_mods_filter; 1] = [
                                        KeyDict_cmd_mods_filter {
                                            is_set__cmd_mods_filter_: 0 as OptionalKeys,
                                            pattern: String_0 {
                                                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                                size: 0,
                                            },
                                            force: false,
                                        },
                                    ];
                                    if !api_dict_to_keydict(
                                        &raw mut filter as *mut ::core::ffi::c_void,
                                        Some(
                                            KeyDict_cmd_mods_filter_get_field
                                                as unsafe extern "C" fn(
                                                    *const ::core::ffi::c_char,
                                                    size_t,
                                                ) -> *mut KeySetLink,
                                        ),
                                        (*(&raw mut mods as *mut KeyDict_cmd_mods)).filter,
                                        err,
                                    ) {
                                        break '_end;
                                    } else if (*(&raw mut filter
                                        as *mut KeyDict_cmd_mods_filter))
                                        .is_set__cmd_mods_filter_ as ::core::ffi::c_ulonglong
                                        & (1 as ::core::ffi::c_ulonglong)
                                            << KEYSET_OPTIDX_cmd_mods_filter__pattern
                                        != 0 as ::core::ffi::c_ulonglong
                                    {
                                        cmdinfo.cmdmod.cmod_filter_force = (*(&raw mut filter
                                            as *mut KeyDict_cmd_mods_filter))
                                            .force as bool;
                                        if *(*(&raw mut filter as *mut KeyDict_cmd_mods_filter))
                                            .pattern
                                            .data as ::core::ffi::c_int != NUL
                                            || cmdinfo.cmdmod.cmod_filter_force as ::core::ffi::c_int
                                                != 0
                                        {
                                            cmdinfo.cmdmod.cmod_filter_pat = string_to_cstr(
                                                (*(&raw mut filter as *mut KeyDict_cmd_mods_filter)).pattern,
                                            );
                                            cmdinfo.cmdmod.cmod_filter_regmatch.regprog = vim_regcomp(
                                                cmdinfo.cmdmod.cmod_filter_pat,
                                                RE_MAGIC,
                                            );
                                        }
                                    }
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods))
                                    .is_set__cmd_mods_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_mods__tab
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if (*(&raw mut mods as *mut KeyDict_cmd_mods)).tab
                                        as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
                                    {
                                        cmdinfo.cmdmod.cmod_tab = (*(&raw mut mods
                                            as *mut KeyDict_cmd_mods))
                                            .tab as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                                    }
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods))
                                    .is_set__cmd_mods_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_mods__verbose
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if (*(&raw mut mods as *mut KeyDict_cmd_mods)).verbose
                                        as ::core::ffi::c_int >= 0 as ::core::ffi::c_int
                                    {
                                        cmdinfo.cmdmod.cmod_verbose = (*(&raw mut mods
                                            as *mut KeyDict_cmd_mods))
                                            .verbose as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                                    }
                                }
                                cmdinfo.cmdmod.cmod_split
                                    |= if (*(&raw mut mods as *mut KeyDict_cmd_mods)).vertical
                                        as ::core::ffi::c_int != 0
                                    {
                                        WSP_VERT as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    };
                                cmdinfo.cmdmod.cmod_split
                                    |= if (*(&raw mut mods as *mut KeyDict_cmd_mods)).horizontal
                                        as ::core::ffi::c_int != 0
                                    {
                                        WSP_HOR as ::core::ffi::c_int
                                    } else {
                                        0 as ::core::ffi::c_int
                                    };
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods))
                                    .is_set__cmd_mods_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_cmd_mods__split
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    if *(*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data
                                        as ::core::ffi::c_int != NUL
                                    {
                                        if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                            || strcmp(
                                                (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                                b"leftabove\0".as_ptr() as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split
                                                |= WSP_ABOVE as ::core::ffi::c_int;
                                        } else if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"belowright\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                            || strcmp(
                                                (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                                b"rightbelow\0".as_ptr() as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split
                                                |= WSP_BELOW as ::core::ffi::c_int;
                                        } else if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"topleft\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |= WSP_TOP as ::core::ffi::c_int;
                                        } else if strcmp(
                                            (*(&raw mut mods as *mut KeyDict_cmd_mods)).split.data,
                                            b"botright\0".as_ptr() as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        {
                                            cmdinfo.cmdmod.cmod_split |= WSP_BOT as ::core::ffi::c_int;
                                        } else if true {
                                            api_err_invalid(
                                                err,
                                                b"mods.split\0".as_ptr() as *const ::core::ffi::c_char,
                                                b"\0".as_ptr() as *const ::core::ffi::c_char,
                                                0 as int64_t,
                                                true_0 != 0,
                                            );
                                            break '_end;
                                        }
                                    }
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).silent {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_SILENT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).emsg_silent {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_ERRSILENT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).unsilent {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_UNSILENT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).sandbox {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_SANDBOX as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).noautocmd {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_NOAUTOCMD as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).browse {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_BROWSE as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).confirm {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_CONFIRM as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).hide {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_HIDE as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keepalt {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_KEEPALT as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keepjumps {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_KEEPJUMPS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keepmarks {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_KEEPMARKS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).keeppatterns
                                {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_KEEPPATTERNS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).lockmarks {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_LOCKMARKS as ::core::ffi::c_int;
                                }
                                if (*(&raw mut mods as *mut KeyDict_cmd_mods)).noswapfile {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_NOSWAPFILE as ::core::ffi::c_int;
                                }
                                if cmdinfo.cmdmod.cmod_flags
                                    & CMOD_ERRSILENT as ::core::ffi::c_int != 0
                                {
                                    cmdinfo.cmdmod.cmod_flags
                                        |= CMOD_SILENT as ::core::ffi::c_int;
                                }
                                if cmdinfo.cmdmod.cmod_flags
                                    & CMOD_SANDBOX as ::core::ffi::c_int != 0
                                    && ea.argt & 0x40000 as uint32_t == 0
                                {
                                    api_set_error(
                                        err,
                                        kErrorTypeValidation,
                                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"Command cannot be run in sandbox\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                    break '_end;
                                }
                            }
                        }
                        build_cmdline_str(
                            &raw mut cmdline,
                            &raw mut ea,
                            &raw mut cmdinfo,
                            args,
                        );
                        ea.cmdlinep = &raw mut cmdline;
                        's_1442: {
                            if ea.argt & EX_ARGOPT as uint32_t != 0 {
                                loop {
                                    if !(*ea.arg.offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                                        && *ea.arg.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int == '+' as ::core::ffi::c_int)
                                    {
                                        break 's_1442;
                                    }
                                    let mut orig_arg: *mut ::core::ffi::c_char = ea.arg;
                                    let mut result: ::core::ffi::c_int = getargopt(&raw mut ea);
                                    if result != 0 as ::core::ffi::c_int
                                        || is_cmd_ni(ea.cmdidx) as ::core::ffi::c_int != 0
                                    {
                                        continue;
                                    }
                                    api_err_invalid(
                                        err,
                                        b"argument \0".as_ptr() as *const ::core::ffi::c_char,
                                        orig_arg,
                                        0 as int64_t,
                                        true_0 != 0,
                                    );
                                    break '_end;
                                }
                            }
                        }
                        if ea.argt & EX_CMDARG as uint32_t != 0 && ea.usefilter == 0 {
                            ea.do_ecmd_cmd = getargcmd(&raw mut ea.arg);
                        }
                        capture_local = garray_T {
                            ga_len: 0,
                            ga_maxlen: 0,
                            ga_itemsize: 0,
                            ga_growsize: 0,
                            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        };
                        save_msg_silent = msg_silent;
                        save_redir_off = redir_off;
                        save_capture_ga = capture_ga;
                        save_msg_col = msg_col;
                        if (*opts).output {
                            ga_init(
                                &raw mut capture_local,
                                1 as ::core::ffi::c_int,
                                80 as ::core::ffi::c_int,
                            );
                            capture_ga = &raw mut capture_local;
                        }
                        let mut tstate: TryState = TryState {
                            current_exception: ::core::ptr::null_mut::<except_T>(),
                            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                            msg_list: ::core::ptr::null::<*const msglist_T>(),
                            got_int: 0,
                            did_throw: false,
                            need_rethrow: 0,
                            did_emsg: 0,
                        };
                        try_enter(&raw mut tstate);
                        if (*opts).output {
                            msg_silent += 1;
                            redir_off = false;
                            msg_col = 0 as ::core::ffi::c_int;
                        }
                        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
                        execute_cmd(&raw mut ea, &raw mut cmdinfo, false);
                        current_sctx = save_current_sctx;
                        if (*opts).output {
                            capture_ga = save_capture_ga;
                            msg_silent = save_msg_silent;
                            redir_off = save_redir_off;
                            msg_col = save_msg_col;
                        }
                        try_leave(&raw mut tstate, err);
                        if (*err).type_0 as ::core::ffi::c_int
                            == kErrorTypeNone as ::core::ffi::c_int
                        {
                            if (*opts).output as ::core::ffi::c_int != 0
                                && capture_local.ga_len > 1 as ::core::ffi::c_int
                            {
                                retv = arena_string(
                                    arena,
                                    String_0 {
                                        data: capture_local.ga_data as *mut ::core::ffi::c_char,
                                        size: capture_local.ga_len as size_t,
                                    },
                                );
                                if *retv.data.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                                {
                                    retv.data = retv.data.offset(1);
                                    retv.size = retv.size.wrapping_sub(1);
                                }
                            }
                        }
                        if (*opts).output {
                            ga_clear(&raw mut capture_local);
                        }
                    }
                }
            }
        }
    }
    xfree(cmdline as *mut ::core::ffi::c_void);
    xfree(ea.args as *mut ::core::ffi::c_void);
    xfree(ea.arglens as *mut ::core::ffi::c_void);
    return retv;
}
unsafe extern "C" fn string_iswhite(mut str: String_0) -> bool {
    let mut i: size_t = 0 as size_t;
    while i < str.size {
        if !ascii_iswhite(*str.data.offset(i as isize) as ::core::ffi::c_int) {
            return false_0 != 0
        } else {
            if *str.data.offset(i as isize) as ::core::ffi::c_int == NUL {
                break;
            }
            i = i.wrapping_add(1);
        }
    }
    return true_0 != 0;
}
unsafe extern "C" fn build_cmdline_str(
    mut cmdlinep: *mut *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut cmdinfo: *mut CmdParseInfo,
    mut args: Array,
) {
    let mut argc: size_t = args.size;
    let mut cmdline: StringBuilder = StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    cmdline.capacity = 32 as size_t;
    cmdline.items = xrealloc(
        cmdline.items as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
    ) as *mut ::core::ffi::c_char;
    if (*cmdinfo).cmdmod.cmod_tab != 0 as ::core::ffi::c_int {
        kv_do_printf(
            &raw mut cmdline,
            b"%dtab \0".as_ptr() as *const ::core::ffi::c_char,
            (*cmdinfo).cmdmod.cmod_tab - 1 as ::core::ffi::c_int,
        );
    }
    if (*cmdinfo).cmdmod.cmod_verbose > 0 as ::core::ffi::c_int {
        kv_do_printf(
            &raw mut cmdline,
            b"%dverbose \0".as_ptr() as *const ::core::ffi::c_char,
            (*cmdinfo).cmdmod.cmod_verbose - 1 as ::core::ffi::c_int,
        );
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0 {
        if strlen(b"silent! \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"silent! \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"silent! \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        852 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"silent! \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"silent! \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"silent! \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    } else if (*cmdinfo).cmdmod.cmod_flags & CMOD_SILENT as ::core::ffi::c_int != 0 {
        if strlen(b"silent \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"silent \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"silent \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_0: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        854 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"silent \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"silent \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"silent \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_UNSILENT as ::core::ffi::c_int != 0 {
        if strlen(b"unsilent \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"unsilent \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"unsilent \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_1: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        858 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"unsilent \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"unsilent \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"unsilent \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    match (*cmdinfo).cmdmod.cmod_split
        & (WSP_ABOVE as ::core::ffi::c_int | WSP_BELOW as ::core::ffi::c_int
            | WSP_TOP as ::core::ffi::c_int | WSP_BOT as ::core::ffi::c_int)
    {
        128 => {
            if strlen(b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char)
                > 0 as size_t
            {
                if cmdline.capacity
                    < cmdline
                        .size
                        .wrapping_add(
                            strlen(
                                b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char,
                            ),
                        )
                {
                    cmdline.capacity = cmdline
                        .size
                        .wrapping_add(
                            strlen(
                                b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char,
                            ),
                        );
                    cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                    cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                    cmdline.capacity = cmdline.capacity.wrapping_add(1);
                    cmdline.capacity = cmdline.capacity;
                    cmdline.items = xrealloc(
                        cmdline.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(cmdline.capacity),
                    ) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label_2: {
                    if !cmdline.items.is_null() {} else {
                        __assert_fail(
                            b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            863 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                memcpy(
                    cmdline.items.offset(cmdline.size as isize)
                        as *mut ::core::ffi::c_void,
                    b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(
                            strlen(
                                b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char,
                            ),
                        ),
                );
                cmdline.size = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"aboveleft \0".as_ptr() as *const ::core::ffi::c_char),
                    );
            }
        }
        64 => {
            if strlen(b"belowright \0".as_ptr() as *const ::core::ffi::c_char)
                > 0 as size_t
            {
                if cmdline.capacity
                    < cmdline
                        .size
                        .wrapping_add(
                            strlen(
                                b"belowright \0".as_ptr() as *const ::core::ffi::c_char,
                            ),
                        )
                {
                    cmdline.capacity = cmdline
                        .size
                        .wrapping_add(
                            strlen(
                                b"belowright \0".as_ptr() as *const ::core::ffi::c_char,
                            ),
                        );
                    cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                    cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                    cmdline.capacity = cmdline.capacity.wrapping_add(1);
                    cmdline.capacity = cmdline.capacity;
                    cmdline.items = xrealloc(
                        cmdline.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(cmdline.capacity),
                    ) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label_3: {
                    if !cmdline.items.is_null() {} else {
                        __assert_fail(
                            b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            866 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                memcpy(
                    cmdline.items.offset(cmdline.size as isize)
                        as *mut ::core::ffi::c_void,
                    b"belowright \0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(
                            strlen(
                                b"belowright \0".as_ptr() as *const ::core::ffi::c_char,
                            ),
                        ),
                );
                cmdline.size = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"belowright \0".as_ptr() as *const ::core::ffi::c_char),
                    );
            }
        }
        8 => {
            if strlen(b"topleft \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t
            {
                if cmdline.capacity
                    < cmdline
                        .size
                        .wrapping_add(
                            strlen(b"topleft \0".as_ptr() as *const ::core::ffi::c_char),
                        )
                {
                    cmdline.capacity = cmdline
                        .size
                        .wrapping_add(
                            strlen(b"topleft \0".as_ptr() as *const ::core::ffi::c_char),
                        );
                    cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                    cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                    cmdline.capacity = cmdline.capacity.wrapping_add(1);
                    cmdline.capacity = cmdline.capacity;
                    cmdline.items = xrealloc(
                        cmdline.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(cmdline.capacity),
                    ) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label_4: {
                    if !cmdline.items.is_null() {} else {
                        __assert_fail(
                            b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            869 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                memcpy(
                    cmdline.items.offset(cmdline.size as isize)
                        as *mut ::core::ffi::c_void,
                    b"topleft \0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(
                            strlen(b"topleft \0".as_ptr() as *const ::core::ffi::c_char),
                        ),
                );
                cmdline.size = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"topleft \0".as_ptr() as *const ::core::ffi::c_char),
                    );
            }
        }
        16 => {
            if strlen(b"botright \0".as_ptr() as *const ::core::ffi::c_char)
                > 0 as size_t
            {
                if cmdline.capacity
                    < cmdline
                        .size
                        .wrapping_add(
                            strlen(b"botright \0".as_ptr() as *const ::core::ffi::c_char),
                        )
                {
                    cmdline.capacity = cmdline
                        .size
                        .wrapping_add(
                            strlen(b"botright \0".as_ptr() as *const ::core::ffi::c_char),
                        );
                    cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                    cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                    cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                    cmdline.capacity = cmdline.capacity.wrapping_add(1);
                    cmdline.capacity = cmdline.capacity;
                    cmdline.items = xrealloc(
                        cmdline.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(cmdline.capacity),
                    ) as *mut ::core::ffi::c_char;
                }
                '_c2rust_label_5: {
                    if !cmdline.items.is_null() {} else {
                        __assert_fail(
                            b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            872 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                memcpy(
                    cmdline.items.offset(cmdline.size as isize)
                        as *mut ::core::ffi::c_void,
                    b"botright \0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(
                            strlen(b"botright \0".as_ptr() as *const ::core::ffi::c_char),
                        ),
                );
                cmdline.size = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"botright \0".as_ptr() as *const ::core::ffi::c_char),
                    );
            }
        }
        _ => {}
    }
    if (*cmdinfo).cmdmod.cmod_split & WSP_VERT as ::core::ffi::c_int != 0 {
        if strlen(b"vertical \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"vertical \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"vertical \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_6: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        885 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"vertical \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"vertical \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"vertical \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_split & WSP_HOR as ::core::ffi::c_int != 0 {
        if strlen(b"horizontal \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t
        {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"horizontal \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"horizontal \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_7: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        886 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"horizontal \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"horizontal \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"horizontal \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int != 0 {
        if strlen(b"sandbox \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"sandbox \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"sandbox \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_8: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        887 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"sandbox \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"sandbox \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"sandbox \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_NOAUTOCMD as ::core::ffi::c_int != 0 {
        if strlen(b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_9: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        888 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"noautocmd \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_BROWSE as ::core::ffi::c_int != 0 {
        if strlen(b"browse \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"browse \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"browse \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_10: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        889 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"browse \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"browse \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"browse \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_CONFIRM as ::core::ffi::c_int != 0 {
        if strlen(b"confirm \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"confirm \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"confirm \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_11: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        890 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"confirm \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"confirm \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"confirm \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_HIDE as ::core::ffi::c_int != 0 {
        if strlen(b"hide \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"hide \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"hide \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_12: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        891 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"hide \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"hide \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"hide \0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPALT as ::core::ffi::c_int != 0 {
        if strlen(b"keepalt \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"keepalt \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"keepalt \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_13: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        892 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"keepalt \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"keepalt \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"keepalt \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int != 0 {
        if strlen(b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_14: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        893 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"keepjumps \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPMARKS as ::core::ffi::c_int != 0 {
        if strlen(b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_15: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        894 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"keepmarks \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_KEEPPATTERNS as ::core::ffi::c_int != 0 {
        if strlen(b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char)
            > 0 as size_t
        {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_16: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        895 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"keeppatterns \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
        if strlen(b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_17: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        896 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"lockmarks \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*cmdinfo).cmdmod.cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int != 0 {
        if strlen(b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t
        {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(
                        strlen(b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char),
                    )
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(
                        strlen(b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_18: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        897 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(
                        strlen(b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char),
                    ),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(
                    strlen(b"noswapfile \0".as_ptr() as *const ::core::ffi::c_char),
                );
        }
    }
    if (*eap).argt & EX_RANGE as uint32_t != 0 {
        if (*eap).addr_count == 1 as ::core::ffi::c_int {
            kv_do_printf(
                &raw mut cmdline,
                b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*eap).line2,
            );
        } else if (*eap).addr_count > 1 as ::core::ffi::c_int {
            kv_do_printf(
                &raw mut cmdline,
                b"%d,%d\0".as_ptr() as *const ::core::ffi::c_char,
                (*eap).line1,
                (*eap).line2,
            );
            (*eap).addr_count = 2 as ::core::ffi::c_int;
        }
    }
    let mut cmdname_idx: size_t = cmdline.size;
    if strlen((*eap).cmd) > 0 as size_t {
        if cmdline.capacity < cmdline.size.wrapping_add(strlen((*eap).cmd)) {
            cmdline.capacity = cmdline.size.wrapping_add(strlen((*eap).cmd));
            cmdline.capacity = cmdline.capacity.wrapping_sub(1);
            cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
            cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
            cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
            cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
            cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
            cmdline.capacity = cmdline.capacity.wrapping_add(1);
            cmdline.capacity = cmdline.capacity;
            cmdline.items = xrealloc(
                cmdline.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(cmdline.capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label_19: {
            if !cmdline.items.is_null() {} else {
                __assert_fail(
                    b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    912 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        memcpy(
            cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
            (*eap).cmd as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>()
                .wrapping_mul(strlen((*eap).cmd)),
        );
        cmdline.size = cmdline.size.wrapping_add(strlen((*eap).cmd));
    }
    if (*eap).argt & EX_BANG as uint32_t != 0 && (*eap).forceit != 0 {
        if strlen(b"!\0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b"!\0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b"!\0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_20: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        916 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b"!\0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b"!\0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b"!\0".as_ptr() as *const ::core::ffi::c_char));
        }
    }
    if (*eap).argt & EX_REGSTR as uint32_t != 0 && (*eap).regname != 0 {
        kv_do_printf(
            &raw mut cmdline,
            b" %c\0".as_ptr() as *const ::core::ffi::c_char,
            (*eap).regname,
        );
    }
    (*eap).argc = argc;
    (*eap).arglens = (if (*eap).argc > 0 as size_t {
        xcalloc(argc, ::core::mem::size_of::<size_t>())
    } else {
        NULL
    }) as *mut size_t;
    let mut argstart_idx: size_t = cmdline.size;
    let mut i: size_t = 0 as size_t;
    while i < argc {
        let mut s: String_0 = (*args.items.offset(i as isize)).data.string;
        *(*eap).arglens.offset(i as isize) = s.size;
        if strlen(b" \0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
            if cmdline.capacity
                < cmdline
                    .size
                    .wrapping_add(strlen(b" \0".as_ptr() as *const ::core::ffi::c_char))
            {
                cmdline.capacity = cmdline
                    .size
                    .wrapping_add(strlen(b" \0".as_ptr() as *const ::core::ffi::c_char));
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_21: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        930 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                b" \0".as_ptr() as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>()
                    .wrapping_mul(strlen(b" \0".as_ptr() as *const ::core::ffi::c_char)),
            );
            cmdline.size = cmdline
                .size
                .wrapping_add(strlen(b" \0".as_ptr() as *const ::core::ffi::c_char));
        }
        if s.size > 0 as size_t {
            if cmdline.capacity < cmdline.size.wrapping_add(s.size) {
                cmdline.capacity = cmdline.size.wrapping_add(s.size);
                cmdline.capacity = cmdline.capacity.wrapping_sub(1);
                cmdline.capacity |= cmdline.capacity >> 1 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 2 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 4 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 8 as ::core::ffi::c_int;
                cmdline.capacity |= cmdline.capacity >> 16 as ::core::ffi::c_int;
                cmdline.capacity = cmdline.capacity.wrapping_add(1);
                cmdline.capacity = cmdline.capacity;
                cmdline.items = xrealloc(
                    cmdline.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul(cmdline.capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label_22: {
                if !cmdline.items.is_null() {} else {
                    __assert_fail(
                        b"(cmdline).items\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/api/command.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        931 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            memcpy(
                cmdline.items.offset(cmdline.size as isize) as *mut ::core::ffi::c_void,
                s.data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(s.size),
            );
            cmdline.size = cmdline.size.wrapping_add(s.size);
        }
        i = i.wrapping_add(1);
    }
    if cmdline.size == cmdline.capacity {
        cmdline.capacity = (if cmdline.capacity != 0 {
            cmdline.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        });
        cmdline.items = xrealloc(
            cmdline.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(cmdline.capacity),
        ) as *mut ::core::ffi::c_char;
    } else {};
    let c2rust_fresh33 = cmdline.size;
    cmdline.size = cmdline.size.wrapping_add(1);
    *cmdline.items.offset(c2rust_fresh33 as isize) = '\0' as ::core::ffi::c_char;
    (*eap).cmd = cmdline.items.offset(cmdname_idx as isize);
    (*eap).args = (if (*eap).argc > 0 as size_t {
        xcalloc(argc, ::core::mem::size_of::<*mut ::core::ffi::c_char>())
    } else {
        NULL
    }) as *mut *mut ::core::ffi::c_char;
    let mut offset: size_t = argstart_idx;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < argc {
        offset = offset.wrapping_add(1);
        *(*eap).args.offset(i_0 as isize) = cmdline.items.offset(offset as isize);
        offset = offset.wrapping_add(*(*eap).arglens.offset(i_0 as isize));
        i_0 = i_0.wrapping_add(1);
    }
    (*eap).arg = if argc > 0 as size_t {
        *(*eap).args.offset(0 as ::core::ffi::c_int as isize)
    } else {
        cmdline
            .items
            .offset(cmdline.size as isize)
            .offset(-(1 as ::core::ffi::c_int as isize))
    };
    *cmdlinep = cmdline.items;
    let mut p: *mut ::core::ffi::c_char = replace_makeprg(eap, (*eap).arg, cmdlinep);
    if p != (*eap).arg {
        (*eap).arg = p;
        let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*eap).args
            as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
        let mut ptr__0: *mut *mut ::core::ffi::c_void = &raw mut (*eap).arglens
            as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        *ptr__0;
        (*eap).argc = 0 as size_t;
    }
}
#[no_mangle]
pub unsafe extern "C" fn nvim_create_user_command(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut err: *mut Error,
) {
    create_user_command(channel_id, name, cmd, opts, 0 as ::core::ffi::c_int, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_del_user_command(mut name: String_0, mut err: *mut Error) {
    nvim_buf_del_user_command(-1 as Buffer, name, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_create_user_command(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut err: *mut Error,
) {
    let mut target_buf: *mut buf_T = find_buffer_by_handle(buf, err);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    let mut save_curbuf: *mut buf_T = curbuf;
    curbuf = target_buf;
    create_user_command(
        channel_id,
        name,
        cmd,
        opts,
        UC_BUFFER as ::core::ffi::c_int,
        err,
    );
    curbuf = save_curbuf;
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_del_user_command(
    mut buf: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) {
    let mut gap: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    if buf == -1 as ::core::ffi::c_int {
        gap = &raw mut ucmds;
    } else {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
        gap = &raw mut (*b).b_ucmds;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut cmd: *mut ucmd_T = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
        if strcmp(name.data, (*cmd).uc_name) == 0 {
            free_ucmd(cmd);
            (*gap).ga_len -= 1 as ::core::ffi::c_int;
            if i < (*gap).ga_len {
                memmove(
                    cmd as *mut ::core::ffi::c_void,
                    cmd.offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    (((*gap).ga_len - i) as size_t)
                        .wrapping_mul(::core::mem::size_of::<ucmd_T>()),
                );
            }
            return;
        }
        i += 1;
    }
    api_set_error(
        err,
        kErrorTypeException,
        b"Invalid command (not found): %s\0".as_ptr() as *const ::core::ffi::c_char,
        name.data,
    );
}
#[no_mangle]
pub unsafe extern "C" fn create_user_command(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut flags: ::core::ffi::c_int,
    mut err: *mut Error,
) {
    let mut force: bool = false;
    let mut argt: uint32_t = 0 as uint32_t;
    let mut def: int64_t = -1 as int64_t;
    let mut addr_type_arg: cmd_addr_T = ADDR_NONE;
    let mut context: ::core::ffi::c_int = EXPAND_NOTHING as ::core::ffi::c_int;
    let mut compl_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut rep: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut luaref: LuaRef = LUA_NOREF;
    let mut compl_luaref: LuaRef = LUA_NOREF;
    let mut preview_luaref: LuaRef = LUA_NOREF;
    '_err: {
        if uc_validate_name(name.data).is_null() {
            api_err_invalid(
                err,
                b"command name\0".as_ptr() as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
        } else if mb_islower(
            *name.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        ) {
            api_err_invalid(
                err,
                b"command name (must start with uppercase)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name.data,
                0 as int64_t,
                true_0 != 0,
            );
        } else if !(!((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
            || !((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong))
        {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"Cannot use both 'range' and 'count'\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        } else {
            if (*opts).nargs.type_0 as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                match (*opts).nargs.data.integer {
                    0 => {}
                    1 => {
                        argt = (argt as ::core::ffi::c_uint
                            | (EX_EXTRA | EX_NOSPC | EX_NEEDARG)) as uint32_t;
                    }
                    _ => {
                        if true {
                            api_err_invalid(
                                err,
                                b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                                (*opts).nargs.data.integer,
                                false_0 != 0,
                            );
                            break '_err;
                        }
                    }
                }
            } else if (*opts).nargs.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if !((*opts).nargs.data.string.size <= 1 as size_t) {
                    api_err_invalid(
                        err,
                        b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                        (*opts).nargs.data.string.data,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_err;
                } else {
                    match *(*opts)
                        .nargs
                        .data
                        .string
                        .data
                        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    {
                        42 => {
                            argt = (argt as ::core::ffi::c_uint | EX_EXTRA) as uint32_t;
                        }
                        63 => {
                            argt = (argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NOSPC))
                                as uint32_t;
                        }
                        43 => {
                            argt = (argt as ::core::ffi::c_uint
                                | (EX_EXTRA | EX_NEEDARG)) as uint32_t;
                        }
                        _ => {
                            if true {
                                api_err_invalid(
                                    err,
                                    b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                                    (*opts).nargs.data.string.data,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_err;
                            }
                        }
                    }
                }
            } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__nargs
                != 0 as ::core::ffi::c_ulonglong
            {
                if true {
                    api_err_invalid(
                        err,
                        b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
                        b"\0".as_ptr() as *const ::core::ffi::c_char,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_err;
                }
            }
            if !(!((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 10 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong) || argt != 0)
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"'complete' used without 'nargs'\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            } else {
                if (*opts).range.type_0 as ::core::ffi::c_uint
                    == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if (*opts).range.data.boolean {
                        argt = (argt as ::core::ffi::c_uint | EX_RANGE) as uint32_t;
                        addr_type_arg = ADDR_LINES;
                    }
                } else if (*opts).range.type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if !(*(*opts)
                        .range
                        .data
                        .string
                        .data
                        .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '%' as ::core::ffi::c_int
                        && (*opts).range.data.string.size == 1 as size_t)
                    {
                        api_err_invalid(
                            err,
                            b"range\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    } else {
                        argt = (argt as ::core::ffi::c_uint | (EX_RANGE | EX_DFLALL))
                            as uint32_t;
                        addr_type_arg = ADDR_LINES;
                    }
                } else if (*opts).range.type_0 as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    argt = (argt as ::core::ffi::c_uint | (EX_RANGE | EX_ZEROR))
                        as uint32_t;
                    def = (*opts).range.data.integer as int64_t;
                    addr_type_arg = ADDR_LINES;
                } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong)
                        << KEYSET_OPTIDX_user_command__range
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if true {
                        api_err_invalid(
                            err,
                            b"range\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    }
                }
                if (*opts).count.type_0 as ::core::ffi::c_uint
                    == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if (*opts).count.data.boolean {
                        argt = (argt as ::core::ffi::c_uint
                            | (EX_COUNT | EX_ZEROR | EX_RANGE)) as uint32_t;
                        addr_type_arg = ADDR_OTHER;
                        def = 0 as int64_t;
                    }
                } else if (*opts).count.type_0 as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    argt = (argt as ::core::ffi::c_uint
                        | (EX_COUNT | EX_ZEROR | EX_RANGE)) as uint32_t;
                    addr_type_arg = ADDR_OTHER;
                    def = (*opts).count.data.integer as int64_t;
                } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong)
                        << KEYSET_OPTIDX_user_command__count
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if true {
                        api_err_invalid(
                            err,
                            b"count\0".as_ptr() as *const ::core::ffi::c_char,
                            b"\0".as_ptr() as *const ::core::ffi::c_char,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    }
                }
                if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__addr
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                        != (*opts).addr.type_0 as ::core::ffi::c_uint
                    {
                        api_err_exp(
                            err,
                            b"addr\0".as_ptr() as *const ::core::ffi::c_char,
                            api_typename(kObjectTypeString),
                            api_typename((*opts).addr.type_0),
                        );
                        break '_err;
                    } else if !(1 as ::core::ffi::c_int
                        == parse_addr_type_arg(
                            (*opts).addr.data.string.data,
                            (*opts).addr.data.string.size as ::core::ffi::c_int,
                            &raw mut addr_type_arg,
                        ))
                    {
                        api_err_invalid(
                            err,
                            b"addr\0".as_ptr() as *const ::core::ffi::c_char,
                            (*opts).addr.data.string.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    } else {
                        argt = (argt as ::core::ffi::c_uint | EX_RANGE) as uint32_t;
                        if addr_type_arg as ::core::ffi::c_uint
                            != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            argt = (argt as ::core::ffi::c_uint | EX_ZEROR) as uint32_t;
                        }
                    }
                }
                if (*opts).bang {
                    argt = (argt as ::core::ffi::c_uint | EX_BANG) as uint32_t;
                }
                if (*opts).bar {
                    argt = (argt as ::core::ffi::c_uint | EX_TRLBAR) as uint32_t;
                }
                if (*opts).register_ {
                    argt = (argt as ::core::ffi::c_uint | EX_REGSTR) as uint32_t;
                }
                if (*opts).keepscript {
                    argt = (argt as ::core::ffi::c_uint | EX_KEEPSCRIPT) as uint32_t;
                }
                force = if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong)
                        << KEYSET_OPTIDX_user_command__force
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*opts).force as ::core::ffi::c_int
                } else {
                    true_0
                } != 0;
                if (*err).type_0 as ::core::ffi::c_int
                    == kErrorTypeNone as ::core::ffi::c_int
                {
                    if (*opts).complete.type_0 as ::core::ffi::c_uint
                        == kObjectTypeLuaRef as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        context = EXPAND_USER_LUA as ::core::ffi::c_int;
                        compl_luaref = (*opts).complete.data.luaref;
                        (*opts).complete.data.luaref = LUA_NOREF as LuaRef;
                    } else if (*opts).complete.type_0 as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if !(1 as ::core::ffi::c_int
                            == parse_compl_arg(
                                (*opts).complete.data.string.data,
                                (*opts).complete.data.string.size as ::core::ffi::c_int,
                                &raw mut context,
                                &raw mut argt,
                                &raw mut compl_arg,
                            ))
                        {
                            api_err_invalid(
                                err,
                                b"complete\0".as_ptr() as *const ::core::ffi::c_char,
                                (*opts).complete.data.string.data,
                                0 as int64_t,
                                true_0 != 0,
                            );
                            break '_err;
                        }
                    } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong)
                            << KEYSET_OPTIDX_user_command__complete
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if true {
                            api_err_exp(
                                err,
                                b"complete\0".as_ptr() as *const ::core::ffi::c_char,
                                b"Function or String\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                            break '_err;
                        }
                    }
                    if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong)
                            << KEYSET_OPTIDX_user_command__preview
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if kObjectTypeLuaRef as ::core::ffi::c_int as ::core::ffi::c_uint
                            != (*opts).preview.type_0 as ::core::ffi::c_uint
                        {
                            api_err_exp(
                                err,
                                b"preview\0".as_ptr() as *const ::core::ffi::c_char,
                                api_typename(kObjectTypeLuaRef),
                                api_typename((*opts).preview.type_0),
                            );
                            break '_err;
                        } else {
                            argt = (argt as ::core::ffi::c_uint | EX_PREVIEW)
                                as uint32_t;
                            preview_luaref = (*opts).preview.data.luaref;
                            (*opts).preview.data.luaref = LUA_NOREF as LuaRef;
                        }
                    }
                    match cmd.type_0 as ::core::ffi::c_uint {
                        7 => {
                            luaref = api_new_luaref(cmd.data.luaref);
                            if (*opts).desc.type_0 as ::core::ffi::c_uint
                                == kObjectTypeString as ::core::ffi::c_int
                                    as ::core::ffi::c_uint
                            {
                                rep = (*opts).desc.data.string.data;
                            } else {
                                rep = b"\0".as_ptr() as *const ::core::ffi::c_char;
                            }
                        }
                        4 => {
                            rep = cmd.data.string.data;
                        }
                        _ => {
                            if true {
                                api_err_exp(
                                    err,
                                    b"command\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Function or String\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                                break '_err;
                            }
                        }
                    }
                    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
                    if uc_add_command(
                        name.data,
                        name.size,
                        rep,
                        argt,
                        def,
                        flags,
                        context,
                        compl_arg,
                        compl_luaref,
                        preview_luaref,
                        addr_type_arg,
                        luaref,
                        force,
                    ) != 1 as ::core::ffi::c_int
                    {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Failed to create user command\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                    current_sctx = save_current_sctx;
                    return;
                }
            }
        }
    }
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
    xfree(compl_arg as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_get_commands(
    mut opts: *mut KeyDict_get_commands,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    return nvim_buf_get_commands(-1 as Buffer, opts, arena, err);
}
#[no_mangle]
pub unsafe extern "C" fn nvim_buf_get_commands(
    mut buf: Buffer,
    mut opts: *mut KeyDict_get_commands,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    let mut global: bool = buf == -1 as ::core::ffi::c_int;
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    if global {
        if (*opts).builtin {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"builtin=true not implemented\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            };
        }
        return commands_array(::core::ptr::null_mut::<buf_T>(), arena);
    }
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if (*opts).builtin as ::core::ffi::c_int != 0 || b.is_null() {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
    }
    return commands_array(b, arena);
}
