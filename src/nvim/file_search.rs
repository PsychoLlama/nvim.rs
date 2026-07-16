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
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    fn abort() -> !;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcpy(
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
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xmemcpyz(
        dst: *mut ::core::ffi::c_void,
        src: *const ::core::ffi::c_void,
        len: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn cstr_to_string(str: *const ::core::ffi::c_char) -> String_0;
    fn cbuf_to_string(buf: *const ::core::ffi::c_char, size: size_t) -> String_0;
    fn cstr_as_string(str: *const ::core::ffi::c_char) -> String_0;
    fn copy_string(str: String_0, arena: *mut Arena) -> String_0;
    fn apply_autocmds(
        event: event_T,
        fname: *mut ::core::ffi::c_char,
        fname_io: *mut ::core::ffi::c_char,
        force: bool,
        buf: *mut buf_T,
    ) -> bool;
    fn has_event(event: event_T) -> bool;
    static mut p_cpo: *mut ::core::ffi::c_char;
    static mut p_fic: ::core::ffi::c_int;
    static mut p_path: *mut ::core::ffi::c_char;
    static mut p_cdpath: *mut ::core::ffi::c_char;
    fn xstrnsave(string: *const ::core::ffi::c_char, len: size_t) -> *mut ::core::ffi::c_char;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_snprintf(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn vim_isfilec(c: ::core::ffi::c_int) -> bool;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn getdigits_long(
        pp: *mut *mut ::core::ffi::c_char,
        strict: bool,
        def: ::core::ffi::c_long,
    ) -> ::core::ffi::c_long;
    fn getdigits_int32(pp: *mut *mut ::core::ffi::c_char, strict: bool, def: int32_t) -> int32_t;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn get_cursor_line_ptr() -> *mut ::core::ffi::c_char;
    static e_cant_find_directory_str_in_cdpath: [::core::ffi::c_char; 0];
    static e_cant_find_file_str_in_path: [::core::ffi::c_char; 0];
    static e_no_more_directory_str_found_in_cdpath: [::core::ffi::c_char; 0];
    static e_no_more_file_str_found_in_path: [::core::ffi::c_char; 0];
    static line_msg: [::core::ffi::c_char; 0];
    fn get_v_event(sve: *mut save_v_event_T) -> *mut dict_T;
    fn restore_v_event(v_event: *mut dict_T, sve: *mut save_v_event_T);
    fn eval_to_string_safe(
        arg: *mut ::core::ffi::c_char,
        use_sandbox: bool,
        use_simple_function: bool,
    ) -> *mut ::core::ffi::c_char;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_dict_add_bool(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        val: BoolVarValue,
    ) -> ::core::ffi::c_int;
    fn tv_dict_add_str(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        val: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tv_dict_set_keys_readonly(dict: *mut dict_T);
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    static mut current_sctx: sctx_T;
    static mut curwin: *mut win_T;
    static mut curbuf: *mut buf_T;
    static mut VIsual_active: bool;
    static mut NameBuff: [::core::ffi::c_char; 4096];
    static mut got_int: bool;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mb_tolower(a: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_head_off(
        base_in: *const ::core::ffi::c_char,
        p_in: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn get_visual_text(
        cap: *mut cmdarg_T,
        pp: *mut *mut ::core::ffi::c_char,
        lenp: *mut size_t,
    ) -> bool;
    fn os_chdir(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_dirname(buf: *mut ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn os_isdir(name: *const ::core::ffi::c_char) -> bool;
    fn os_path_exists(path: *const ::core::ffi::c_char) -> bool;
    fn os_fileid(path: *const ::core::ffi::c_char, file_id: *mut FileID) -> bool;
    fn os_fileid_equal(file_id_1: *const FileID, file_id_2: *const FileID) -> bool;
    fn was_set_insecurely(
        wp: *mut win_T,
        opt_idx: OptIndex,
        opt_flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn copy_option_part(
        option: *mut *mut ::core::ffi::c_char,
        buf: *mut ::core::ffi::c_char,
        maxlen: size_t,
        sep_chars: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn expand_env_esc(
        srcp: *const ::core::ffi::c_char,
        dst: *mut ::core::ffi::c_char,
        dstlen: ::core::ffi::c_int,
        esc: bool,
        one: bool,
        prefix: *mut ::core::ffi::c_char,
    ) -> size_t;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn path_tail_with_sep(fname: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn vim_ispathsep(c: ::core::ffi::c_int) -> bool;
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn path_fnamencmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
        len: size_t,
    ) -> ::core::ffi::c_int;
    fn FullName_save(fname: *const ::core::ffi::c_char, force: bool) -> *mut ::core::ffi::c_char;
    fn FreeWild(count: ::core::ffi::c_int, files: *mut *mut ::core::ffi::c_char);
    fn simplify_filename(filename: *mut ::core::ffi::c_char) -> size_t;
    fn path_has_drive_letter(p: *const ::core::ffi::c_char, path_len: size_t) -> bool;
    fn path_is_url(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn path_with_url(fname: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn vim_isAbsName(name: *const ::core::ffi::c_char) -> bool;
    fn after_pathsep(
        b: *const ::core::ffi::c_char,
        p: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn pathcmp(
        p: *const ::core::ffi::c_char,
        q: *const ::core::ffi::c_char,
        maxlen: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn path_shorten_fname(
        full_path: *mut ::core::ffi::c_char,
        dir_name: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn expand_wildcards(
        num_pat: ::core::ffi::c_int,
        pat: *mut *mut ::core::ffi::c_char,
        num_files: *mut ::core::ffi::c_int,
        files: *mut *mut *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn os_breakcheck();
}
pub type __time_t = ::core::ffi::c_long;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type ptrdiff_t = isize;
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
pub struct String_0 {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
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
pub type CdScope = ::core::ffi::c_int;
pub const kCdScopeGlobal: CdScope = 2;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdScopeInvalid: CdScope = -1;
pub type CdCause = ::core::ffi::c_int;
pub const kCdCauseAuto: CdCause = 2;
pub const kCdCauseWindow: CdCause = 1;
pub const kCdCauseManual: CdCause = 0;
pub const kCdCauseOther: CdCause = -1;
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_int;
pub const kBufOptWrapmargin: C2Rust_Unnamed_13 = 91;
pub const kBufOptVartabstop: C2Rust_Unnamed_13 = 90;
pub const kBufOptVarsofttabstop: C2Rust_Unnamed_13 = 89;
pub const kBufOptUndolevels: C2Rust_Unnamed_13 = 88;
pub const kBufOptUndofile: C2Rust_Unnamed_13 = 87;
pub const kBufOptThesaurusfunc: C2Rust_Unnamed_13 = 86;
pub const kBufOptThesaurus: C2Rust_Unnamed_13 = 85;
pub const kBufOptTextwidth: C2Rust_Unnamed_13 = 84;
pub const kBufOptTags: C2Rust_Unnamed_13 = 83;
pub const kBufOptTagfunc: C2Rust_Unnamed_13 = 82;
pub const kBufOptTagcase: C2Rust_Unnamed_13 = 81;
pub const kBufOptTabstop: C2Rust_Unnamed_13 = 80;
pub const kBufOptSyntax: C2Rust_Unnamed_13 = 79;
pub const kBufOptSynmaxcol: C2Rust_Unnamed_13 = 78;
pub const kBufOptSwapfile: C2Rust_Unnamed_13 = 77;
pub const kBufOptSuffixesadd: C2Rust_Unnamed_13 = 76;
pub const kBufOptSpelloptions: C2Rust_Unnamed_13 = 75;
pub const kBufOptSpelllang: C2Rust_Unnamed_13 = 74;
pub const kBufOptSpellfile: C2Rust_Unnamed_13 = 73;
pub const kBufOptSpellcapcheck: C2Rust_Unnamed_13 = 72;
pub const kBufOptSofttabstop: C2Rust_Unnamed_13 = 71;
pub const kBufOptSmartindent: C2Rust_Unnamed_13 = 70;
pub const kBufOptShiftwidth: C2Rust_Unnamed_13 = 69;
pub const kBufOptScrollback: C2Rust_Unnamed_13 = 68;
pub const kBufOptReadonly: C2Rust_Unnamed_13 = 67;
pub const kBufOptQuoteescape: C2Rust_Unnamed_13 = 66;
pub const kBufOptPreserveindent: C2Rust_Unnamed_13 = 65;
pub const kBufOptPath: C2Rust_Unnamed_13 = 64;
pub const kBufOptOmnifunc: C2Rust_Unnamed_13 = 63;
pub const kBufOptNrformats: C2Rust_Unnamed_13 = 62;
pub const kBufOptModified: C2Rust_Unnamed_13 = 61;
pub const kBufOptModifiable: C2Rust_Unnamed_13 = 60;
pub const kBufOptModeline: C2Rust_Unnamed_13 = 59;
pub const kBufOptMatchpairs: C2Rust_Unnamed_13 = 58;
pub const kBufOptMakeprg: C2Rust_Unnamed_13 = 57;
pub const kBufOptMakeencoding: C2Rust_Unnamed_13 = 56;
pub const kBufOptLispwords: C2Rust_Unnamed_13 = 55;
pub const kBufOptLispoptions: C2Rust_Unnamed_13 = 54;
pub const kBufOptLisp: C2Rust_Unnamed_13 = 53;
pub const kBufOptKeywordprg: C2Rust_Unnamed_13 = 52;
pub const kBufOptKeymap: C2Rust_Unnamed_13 = 51;
pub const kBufOptIskeyword: C2Rust_Unnamed_13 = 50;
pub const kBufOptInfercase: C2Rust_Unnamed_13 = 49;
pub const kBufOptIndentkeys: C2Rust_Unnamed_13 = 48;
pub const kBufOptIndentexpr: C2Rust_Unnamed_13 = 47;
pub const kBufOptIncludeexpr: C2Rust_Unnamed_13 = 46;
pub const kBufOptInclude: C2Rust_Unnamed_13 = 45;
pub const kBufOptImsearch: C2Rust_Unnamed_13 = 44;
pub const kBufOptIminsert: C2Rust_Unnamed_13 = 43;
pub const kBufOptGrepprg: C2Rust_Unnamed_13 = 42;
pub const kBufOptGrepformat: C2Rust_Unnamed_13 = 41;
pub const kBufOptFsync: C2Rust_Unnamed_13 = 40;
pub const kBufOptFormatprg: C2Rust_Unnamed_13 = 39;
pub const kBufOptFormatoptions: C2Rust_Unnamed_13 = 38;
pub const kBufOptFormatlistpat: C2Rust_Unnamed_13 = 37;
pub const kBufOptFormatexpr: C2Rust_Unnamed_13 = 36;
pub const kBufOptFixendofline: C2Rust_Unnamed_13 = 35;
pub const kBufOptFindfunc: C2Rust_Unnamed_13 = 34;
pub const kBufOptFiletype: C2Rust_Unnamed_13 = 33;
pub const kBufOptFileformat: C2Rust_Unnamed_13 = 32;
pub const kBufOptFileencoding: C2Rust_Unnamed_13 = 31;
pub const kBufOptExpandtab: C2Rust_Unnamed_13 = 30;
pub const kBufOptErrorformat: C2Rust_Unnamed_13 = 29;
pub const kBufOptEqualprg: C2Rust_Unnamed_13 = 28;
pub const kBufOptEndofline: C2Rust_Unnamed_13 = 27;
pub const kBufOptEndoffile: C2Rust_Unnamed_13 = 26;
pub const kBufOptDiffanchors: C2Rust_Unnamed_13 = 25;
pub const kBufOptDictionary: C2Rust_Unnamed_13 = 24;
pub const kBufOptDefine: C2Rust_Unnamed_13 = 23;
pub const kBufOptCopyindent: C2Rust_Unnamed_13 = 22;
pub const kBufOptCompleteslash: C2Rust_Unnamed_13 = 21;
pub const kBufOptCompleteopt: C2Rust_Unnamed_13 = 20;
pub const kBufOptCompletefunc: C2Rust_Unnamed_13 = 19;
pub const kBufOptComplete: C2Rust_Unnamed_13 = 18;
pub const kBufOptCommentstring: C2Rust_Unnamed_13 = 17;
pub const kBufOptComments: C2Rust_Unnamed_13 = 16;
pub const kBufOptCinwords: C2Rust_Unnamed_13 = 15;
pub const kBufOptCinscopedecls: C2Rust_Unnamed_13 = 14;
pub const kBufOptCinoptions: C2Rust_Unnamed_13 = 13;
pub const kBufOptCinkeys: C2Rust_Unnamed_13 = 12;
pub const kBufOptCindent: C2Rust_Unnamed_13 = 11;
pub const kBufOptChannel: C2Rust_Unnamed_13 = 10;
pub const kBufOptBusy: C2Rust_Unnamed_13 = 9;
pub const kBufOptBuftype: C2Rust_Unnamed_13 = 8;
pub const kBufOptBuflisted: C2Rust_Unnamed_13 = 7;
pub const kBufOptBufhidden: C2Rust_Unnamed_13 = 6;
pub const kBufOptBomb: C2Rust_Unnamed_13 = 5;
pub const kBufOptBinary: C2Rust_Unnamed_13 = 4;
pub const kBufOptBackupcopy: C2Rust_Unnamed_13 = 3;
pub const kBufOptAutoread: C2Rust_Unnamed_13 = 2;
pub const kBufOptAutoindent: C2Rust_Unnamed_13 = 1;
pub const kBufOptAutocomplete: C2Rust_Unnamed_13 = 0;
pub const kBufOptInvalid: C2Rust_Unnamed_13 = -1;
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
pub struct save_v_event_T {
    pub sve_did_save: bool,
    pub sve_hashtab: hashtab_T,
}
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const FINDFILE_BOTH: C2Rust_Unnamed_14 = 2;
pub const FINDFILE_DIR: C2Rust_Unnamed_14 = 1;
pub const FINDFILE_FILE: C2Rust_Unnamed_14 = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const FNAME_UNESC: C2Rust_Unnamed_15 = 32;
pub const FNAME_REL: C2Rust_Unnamed_15 = 16;
pub const FNAME_INCL: C2Rust_Unnamed_15 = 8;
pub const FNAME_HYP: C2Rust_Unnamed_15 = 4;
pub const FNAME_EXP: C2Rust_Unnamed_15 = 2;
pub const FNAME_MESS: C2Rust_Unnamed_15 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_search_ctx_T {
    pub ffsc_stack_ptr: *mut ff_stack_T,
    pub ffsc_visited_list: *mut ff_visited_list_hdr_T,
    pub ffsc_dir_visited_list: *mut ff_visited_list_hdr_T,
    pub ffsc_visited_lists_list: *mut ff_visited_list_hdr_T,
    pub ffsc_dir_visited_lists_list: *mut ff_visited_list_hdr_T,
    pub ffsc_file_to_search: String_0,
    pub ffsc_start_dir: String_0,
    pub ffsc_fix_path: String_0,
    pub ffsc_wc_path: String_0,
    pub ffsc_level: ::core::ffi::c_int,
    pub ffsc_stopdirs_v: *mut String_0,
    pub ffsc_find_what: ::core::ffi::c_int,
    pub ffsc_tagfile: ::core::ffi::c_int,
}
pub type ff_visited_list_hdr_T = ff_visited_list_hdr;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_visited_list_hdr {
    pub ffvl_next: *mut ff_visited_list_hdr,
    pub ffvl_filename: *mut ::core::ffi::c_char,
    pub ffvl_visited_list: *mut ff_visited_T,
}
pub type ff_visited_T = ff_visited;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_visited {
    pub ffv_next: *mut ff_visited,
    pub ffv_wc_path: *mut ::core::ffi::c_char,
    pub file_id_valid: bool,
    pub file_id: FileID,
    pub ffv_fname: [::core::ffi::c_char; 0],
}
pub type ff_stack_T = ff_stack;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ff_stack {
    pub ffs_prev: *mut ff_stack,
    pub ffs_fix_path: String_0,
    pub ffs_wc_path: String_0,
    pub ffs_filearray: *mut *mut ::core::ffi::c_char,
    pub ffs_filearray_size: ::core::ffi::c_int,
    pub ffs_filearray_cur: ::core::ffi::c_int,
    pub ffs_stage: ::core::ffi::c_int,
    pub ffs_level: ::core::ffi::c_int,
    pub ffs_star_star_empty: ::core::ffi::c_int,
}
pub const EW_NOTWILD: C2Rust_Unnamed_17 = 1024;
pub const EW_SILENT: C2Rust_Unnamed_17 = 32;
pub const EW_ADDSLASH: C2Rust_Unnamed_17 = 8;
pub const EW_DIR: C2Rust_Unnamed_17 = 1;
pub const OPT_LOCAL: C2Rust_Unnamed_16 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdarg_T {
    pub oap: *mut oparg_T,
    pub prechar: ::core::ffi::c_int,
    pub cmdchar: ::core::ffi::c_int,
    pub nchar: ::core::ffi::c_int,
    pub nchar_composing: [::core::ffi::c_char; 32],
    pub nchar_len: ::core::ffi::c_int,
    pub extra_char: ::core::ffi::c_int,
    pub opcount: ::core::ffi::c_int,
    pub count0: ::core::ffi::c_int,
    pub count1: ::core::ffi::c_int,
    pub arg: ::core::ffi::c_int,
    pub retval: ::core::ffi::c_int,
    pub searchbuf: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct oparg_T {
    pub op_type: ::core::ffi::c_int,
    pub regname: ::core::ffi::c_int,
    pub motion_type: MotionType,
    pub motion_force: ::core::ffi::c_int,
    pub use_reg_one: bool,
    pub inclusive: bool,
    pub end_adjusted: bool,
    pub start: pos_T,
    pub end: pos_T,
    pub cursor_start: pos_T,
    pub line_count: linenr_T,
    pub empty: bool,
    pub is_VIsual: bool,
    pub start_vcol: colnr_T,
    pub end_vcol: colnr_T,
    pub prev_opcount: ::core::ffi::c_int,
    pub prev_count0: ::core::ffi::c_int,
    pub excl_tr_ws: bool,
}
pub type MotionType = ::core::ffi::c_int;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const OPT_SKIPRTP: C2Rust_Unnamed_16 = 128;
pub const OPT_NO_REDRAW: C2Rust_Unnamed_16 = 64;
pub const OPT_ONECOLUMN: C2Rust_Unnamed_16 = 32;
pub const OPT_NOWIN: C2Rust_Unnamed_16 = 16;
pub const OPT_WINONLY: C2Rust_Unnamed_16 = 8;
pub const OPT_MODELINE: C2Rust_Unnamed_16 = 4;
pub const OPT_GLOBAL: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_17 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_17 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_17 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_17 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_17 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_17 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_17 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_17 = 2048;
pub const EW_NOERROR: C2Rust_Unnamed_17 = 512;
pub const EW_ICASE: C2Rust_Unnamed_17 = 256;
pub const EW_PATH: C2Rust_Unnamed_17 = 128;
pub const EW_EXEC: C2Rust_Unnamed_17 = 64;
pub const EW_KEEPALL: C2Rust_Unnamed_17 = 16;
pub const EW_NOTFOUND: C2Rust_Unnamed_17 = 4;
pub const EW_FILE: C2Rust_Unnamed_17 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const PATHSEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"/\0") };
pub const CPO_DOTTAG: ::core::ffi::c_int = 'd' as ::core::ffi::c_int;
static mut ff_expand_buffer: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const FF_MAX_STAR_STAR_EXPAND: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
static mut e_path_too_long_for_completion: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"E854: Path too long for completion\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn vim_findfile_init(
    mut path: *mut ::core::ffi::c_char,
    mut filename: *mut ::core::ffi::c_char,
    mut filenamelen: size_t,
    mut stopdirs: *mut ::core::ffi::c_char,
    mut level: ::core::ffi::c_int,
    mut free_visited: ::core::ffi::c_int,
    mut find_what: ::core::ffi::c_int,
    mut search_ctx_arg: *mut ::core::ffi::c_void,
    mut tagfile: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_void {
    let mut wc_part: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut add_sep: bool = false;
    let mut sptr: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
    let mut search_ctx: *mut ff_search_ctx_T = ::core::ptr::null_mut::<ff_search_ctx_T>();
    if !search_ctx_arg.is_null() {
        search_ctx = search_ctx_arg as *mut ff_search_ctx_T;
    } else {
        search_ctx =
            xcalloc(1 as size_t, ::core::mem::size_of::<ff_search_ctx_T>()) as *mut ff_search_ctx_T;
    }
    (*search_ctx).ffsc_find_what = find_what;
    (*search_ctx).ffsc_tagfile = tagfile;
    ff_clear(search_ctx);
    '_error_return: {
        if free_visited == true_0 {
            vim_findfile_free_visited(search_ctx as *mut ::core::ffi::c_void);
        } else {
            (*search_ctx).ffsc_visited_list = ff_get_visited_list(
                filename,
                filenamelen,
                &raw mut (*search_ctx).ffsc_visited_lists_list,
            );
            if (*search_ctx).ffsc_visited_list.is_null() {
                break '_error_return;
            } else {
                (*search_ctx).ffsc_dir_visited_list = ff_get_visited_list(
                    filename,
                    filenamelen,
                    &raw mut (*search_ctx).ffsc_dir_visited_lists_list,
                );
                if (*search_ctx).ffsc_dir_visited_list.is_null() {
                    break '_error_return;
                }
            }
        }
        if ff_expand_buffer.data.is_null() {
            ff_expand_buffer.size = 0 as size_t;
            ff_expand_buffer.data = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        }
        if *path.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (vim_ispathsep(*path.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *path.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
            && (tagfile == 0 || vim_strchr(p_cpo, CPO_DOTTAG).is_null())
            && !rel_fname.is_null()
        {
            let mut len: size_t = path_tail(rel_fname).offset_from(rel_fname) as size_t;
            if !vim_isAbsName(rel_fname) && len.wrapping_add(1 as size_t) < MAXPATHL as size_t {
                xmemcpyz(
                    ff_expand_buffer.data as *mut ::core::ffi::c_void,
                    rel_fname as *const ::core::ffi::c_void,
                    len,
                );
                ff_expand_buffer.size = len;
                (*search_ctx).ffsc_start_dir =
                    cstr_as_string(FullName_save(ff_expand_buffer.data, false_0 != 0));
            } else {
                (*search_ctx).ffsc_start_dir = cbuf_to_string(rel_fname, len);
            }
            path = path.offset(1);
            if *path as ::core::ffi::c_int != NUL {
                path = path.offset(1);
            }
        } else if *path as ::core::ffi::c_int == NUL || !vim_isAbsName(path) {
            if os_dirname(ff_expand_buffer.data, MAXPATHL as size_t) == FAIL {
                break '_error_return;
            } else {
                ff_expand_buffer.size = strlen(ff_expand_buffer.data);
                (*search_ctx).ffsc_start_dir =
                    copy_string(ff_expand_buffer, ::core::ptr::null_mut::<Arena>());
            }
        }
        if !stopdirs.is_null() {
            let mut walker: *mut ::core::ffi::c_char = stopdirs;
            while *walker as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                walker = walker.offset(1);
            }
            let mut dircount: size_t = 1 as size_t;
            (*search_ctx).ffsc_stopdirs_v =
                xmalloc(::core::mem::size_of::<String_0>()) as *mut String_0;
            loop {
                let mut helper: *mut ::core::ffi::c_char = walker;
                let mut ptr: *mut ::core::ffi::c_void = xrealloc(
                    (*search_ctx).ffsc_stopdirs_v as *mut ::core::ffi::c_void,
                    dircount
                        .wrapping_add(1 as size_t)
                        .wrapping_mul(::core::mem::size_of::<String_0>()),
                );
                (*search_ctx).ffsc_stopdirs_v = ptr as *mut String_0;
                walker = vim_strchr(walker, ';' as ::core::ffi::c_int);
                '_c2rust_label: {
                    if walker.is_null() || walker.offset_from(helper) >= 0 as isize {
                    } else {
                        __assert_fail(
                            b"!walker || walker - helper >= 0\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            359 as ::core::ffi::c_uint,
                            b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                let mut len_0: size_t = if !walker.is_null() {
                    walker.offset_from(helper) as size_t
                } else {
                    strlen(helper)
                };
                if *helper as ::core::ffi::c_int != NUL
                    && !vim_isAbsName(helper)
                    && len_0.wrapping_add(1 as size_t) < MAXPATHL as size_t
                {
                    xmemcpyz(
                        ff_expand_buffer.data as *mut ::core::ffi::c_void,
                        helper as *const ::core::ffi::c_void,
                        len_0,
                    );
                    ff_expand_buffer.size = len_0;
                    *(*search_ctx)
                        .ffsc_stopdirs_v
                        .offset(dircount.wrapping_sub(1 as size_t) as isize) =
                        cstr_as_string(FullName_save(helper, len_0 != 0));
                } else {
                    *(*search_ctx)
                        .ffsc_stopdirs_v
                        .offset(dircount.wrapping_sub(1 as size_t) as isize) =
                        cbuf_to_string(helper, len_0);
                }
                if !walker.is_null() {
                    walker = walker.offset(1);
                }
                dircount = dircount.wrapping_add(1);
                if walker.is_null() {
                    break;
                }
            }
            *(*search_ctx)
                .ffsc_stopdirs_v
                .offset(dircount.wrapping_sub(1 as size_t) as isize) = NULL_STRING;
        }
        (*search_ctx).ffsc_level = level;
        wc_part = vim_strchr(path, '*' as ::core::ffi::c_int);
        if !wc_part.is_null() {
            let mut llevel: int64_t = 0;
            let mut errpt: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            '_c2rust_label_0: {
                if wc_part.offset_from(path) >= 0 as isize {
                } else {
                    __assert_fail(
                        b"wc_part - path >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        390 as ::core::ffi::c_uint,
                        b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            (*search_ctx).ffsc_fix_path = cbuf_to_string(path, wc_part.offset_from(path) as size_t);
            ff_expand_buffer.size = 0 as size_t;
            while *wc_part as ::core::ffi::c_int != NUL {
                if ff_expand_buffer.size.wrapping_add(5 as size_t) >= MAXPATHL as size_t {
                    emsg(gettext(
                        &raw const e_path_too_long_for_completion as *const ::core::ffi::c_char,
                    ));
                    break;
                } else if strncmp(
                    wc_part,
                    b"**\0".as_ptr() as *const ::core::ffi::c_char,
                    2 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    let c2rust_fresh0 = wc_part;
                    wc_part = wc_part.offset(1);
                    let c2rust_fresh1 = ff_expand_buffer.size;
                    ff_expand_buffer.size = ff_expand_buffer.size.wrapping_add(1);
                    *ff_expand_buffer.data.offset(c2rust_fresh1 as isize) = *c2rust_fresh0;
                    let c2rust_fresh2 = wc_part;
                    wc_part = wc_part.offset(1);
                    let c2rust_fresh3 = ff_expand_buffer.size;
                    ff_expand_buffer.size = ff_expand_buffer.size.wrapping_add(1);
                    *ff_expand_buffer.data.offset(c2rust_fresh3 as isize) = *c2rust_fresh2;
                    llevel = strtol(wc_part, &raw mut errpt, 10 as ::core::ffi::c_int) as int64_t;
                    if errpt != wc_part && llevel > 0 as int64_t && llevel < 255 as int64_t {
                        let c2rust_fresh4 = ff_expand_buffer.size;
                        ff_expand_buffer.size = ff_expand_buffer.size.wrapping_add(1);
                        *ff_expand_buffer.data.offset(c2rust_fresh4 as isize) =
                            llevel as ::core::ffi::c_char;
                    } else if errpt != wc_part && llevel == 0 as int64_t {
                        ff_expand_buffer.size = ff_expand_buffer.size.wrapping_sub(2 as size_t);
                    } else {
                        let c2rust_fresh5 = ff_expand_buffer.size;
                        ff_expand_buffer.size = ff_expand_buffer.size.wrapping_add(1);
                        *ff_expand_buffer.data.offset(c2rust_fresh5 as isize) =
                            FF_MAX_STAR_STAR_EXPAND as ::core::ffi::c_char;
                    }
                    wc_part = errpt;
                    if !(*wc_part as ::core::ffi::c_int != NUL
                        && !vim_ispathsep(*wc_part as ::core::ffi::c_int))
                    {
                        continue;
                    }
                    semsg(
                        gettext(
                            b"E343: Invalid path: '**[number]' must be at the end of the path or be followed by '%s'.\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        PATHSEPSTR.as_ptr(),
                    );
                    break '_error_return;
                } else {
                    let c2rust_fresh6 = wc_part;
                    wc_part = wc_part.offset(1);
                    let c2rust_fresh7 = ff_expand_buffer.size;
                    ff_expand_buffer.size = ff_expand_buffer.size.wrapping_add(1);
                    *ff_expand_buffer.data.offset(c2rust_fresh7 as isize) = *c2rust_fresh6;
                }
            }
            *ff_expand_buffer.data.offset(ff_expand_buffer.size as isize) =
                NUL as ::core::ffi::c_char;
            (*search_ctx).ffsc_wc_path =
                copy_string(ff_expand_buffer, ::core::ptr::null_mut::<Arena>());
        } else {
            (*search_ctx).ffsc_fix_path = cstr_to_string(path);
        }
        if (*search_ctx).ffsc_start_dir.data.is_null() {
            (*search_ctx).ffsc_start_dir = copy_string(
                (*search_ctx).ffsc_fix_path,
                ::core::ptr::null_mut::<Arena>(),
            );
            *(*search_ctx)
                .ffsc_fix_path
                .data
                .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
            (*search_ctx).ffsc_fix_path.size = 0 as size_t;
        }
        if (*search_ctx)
            .ffsc_start_dir
            .size
            .wrapping_add((*search_ctx).ffsc_fix_path.size)
            .wrapping_add(3 as size_t)
            >= MAXPATHL as size_t
        {
            emsg(gettext(
                &raw const e_path_too_long_for_completion as *const ::core::ffi::c_char,
            ));
        } else {
            add_sep = after_pathsep(
                (*search_ctx).ffsc_start_dir.data,
                (*search_ctx)
                    .ffsc_start_dir
                    .data
                    .offset((*search_ctx).ffsc_start_dir.size as isize),
            ) == 0;
            ff_expand_buffer.size = vim_snprintf(
                ff_expand_buffer.data,
                MAXPATHL as size_t,
                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*search_ctx).ffsc_start_dir.data,
                if add_sep as ::core::ffi::c_int != 0 {
                    PATHSEPSTR.as_ptr()
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                },
            ) as size_t;
            '_c2rust_label_1: {
                if ff_expand_buffer.size < 4096 as size_t {
                } else {
                    __assert_fail(
                        b"ff_expand_buffer.size < MAXPATHL\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        458 as ::core::ffi::c_uint,
                        b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut bufsize: size_t = ff_expand_buffer
                .size
                .wrapping_add((*search_ctx).ffsc_fix_path.size)
                .wrapping_add(1 as size_t);
            let mut buf: *mut ::core::ffi::c_char = xmalloc(bufsize) as *mut ::core::ffi::c_char;
            vim_snprintf(
                buf,
                bufsize,
                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                ff_expand_buffer.data,
                (*search_ctx).ffsc_fix_path.data,
            );
            if os_isdir(buf) {
                if (*search_ctx).ffsc_fix_path.size > 0 as size_t {
                    add_sep = after_pathsep(
                        (*search_ctx).ffsc_fix_path.data,
                        (*search_ctx)
                            .ffsc_fix_path
                            .data
                            .offset((*search_ctx).ffsc_fix_path.size as isize),
                    ) == 0;
                    ff_expand_buffer.size = ff_expand_buffer.size.wrapping_add(vim_snprintf(
                        ff_expand_buffer.data.offset(ff_expand_buffer.size as isize),
                        (MAXPATHL as size_t).wrapping_sub(ff_expand_buffer.size),
                        b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                        (*search_ctx).ffsc_fix_path.data,
                        if add_sep as ::core::ffi::c_int != 0 {
                            PATHSEPSTR.as_ptr()
                        } else {
                            b"\0".as_ptr() as *const ::core::ffi::c_char
                        },
                    )
                        as size_t);
                    '_c2rust_label_2: {
                        if ff_expand_buffer.size < 4096 as size_t {
                        } else {
                            __assert_fail(
                                b"ff_expand_buffer.size < MAXPATHL\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                478 as ::core::ffi::c_uint,
                                b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
            } else {
                let mut p: *mut ::core::ffi::c_char = path_tail((*search_ctx).ffsc_fix_path.data);
                let mut len_1: ::core::ffi::c_int =
                    (*search_ctx).ffsc_fix_path.size as ::core::ffi::c_int;
                if p > (*search_ctx).ffsc_fix_path.data {
                    len_1 = p.offset_from((*search_ctx).ffsc_fix_path.data) as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int;
                    if len_1 >= 2 as ::core::ffi::c_int
                        && strncmp(
                            (*search_ctx).ffsc_fix_path.data,
                            b"..\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                        && (len_1 == 2 as ::core::ffi::c_int
                            || *(*search_ctx)
                                .ffsc_fix_path
                                .data
                                .offset(2 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == PATHSEP)
                    {
                        xfree(buf as *mut ::core::ffi::c_void);
                        break '_error_return;
                    } else {
                        add_sep = after_pathsep(
                            (*search_ctx).ffsc_fix_path.data,
                            (*search_ctx)
                                .ffsc_fix_path
                                .data
                                .offset((*search_ctx).ffsc_fix_path.size as isize),
                        ) == 0;
                        ff_expand_buffer.size = ff_expand_buffer.size.wrapping_add(vim_snprintf(
                            ff_expand_buffer.data.offset(ff_expand_buffer.size as isize),
                            (MAXPATHL as size_t).wrapping_sub(ff_expand_buffer.size),
                            b"%.*s%s\0".as_ptr() as *const ::core::ffi::c_char,
                            len_1,
                            (*search_ctx).ffsc_fix_path.data,
                            if add_sep as ::core::ffi::c_int != 0 {
                                PATHSEPSTR.as_ptr()
                            } else {
                                b"\0".as_ptr() as *const ::core::ffi::c_char
                            },
                        )
                            as size_t);
                        '_c2rust_label_3: {
                            if ff_expand_buffer.size < 4096 as size_t {
                            } else {
                                __assert_fail(
                                    b"ff_expand_buffer.size < MAXPATHL\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    501 as ::core::ffi::c_uint,
                                    b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                    }
                }
                if !(*search_ctx).ffsc_wc_path.data.is_null() {
                    let mut tempsize: size_t = (*search_ctx)
                        .ffsc_fix_path
                        .size
                        .wrapping_sub(len_1 as size_t)
                        .wrapping_add((*search_ctx).ffsc_wc_path.size)
                        .wrapping_add(1 as size_t);
                    let mut temp: *mut ::core::ffi::c_char =
                        xmalloc(tempsize) as *mut ::core::ffi::c_char;
                    (*search_ctx).ffsc_wc_path.size = vim_snprintf(
                        temp,
                        tempsize,
                        b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                        (*search_ctx).ffsc_fix_path.data.offset(len_1 as isize),
                        (*search_ctx).ffsc_wc_path.data,
                    ) as size_t;
                    '_c2rust_label_4: {
                        if (*search_ctx).ffsc_wc_path.size < tempsize {
                        } else {
                            __assert_fail(
                                b"search_ctx->ffsc_wc_path.size < tempsize\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                513 as ::core::ffi::c_uint,
                                b"void *vim_findfile_init(char *, char *, size_t, char *, int, int, int, void *, int, char *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    xfree((*search_ctx).ffsc_wc_path.data as *mut ::core::ffi::c_void);
                    (*search_ctx).ffsc_wc_path.data = temp;
                }
            }
            xfree(buf as *mut ::core::ffi::c_void);
            sptr = ff_create_stack_element(
                ff_expand_buffer.data,
                ff_expand_buffer.size,
                (*search_ctx).ffsc_wc_path.data,
                (*search_ctx).ffsc_wc_path.size,
                level,
                0 as ::core::ffi::c_int,
            );
            ff_push(search_ctx, sptr);
            (*search_ctx).ffsc_file_to_search = cbuf_to_string(filename, filenamelen);
            return search_ctx as *mut ::core::ffi::c_void;
        }
    }
    vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
    return NULL;
}
#[no_mangle]
pub unsafe extern "C" fn vim_findfile_stopdir(
    mut buf: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    while *buf as ::core::ffi::c_int != NUL
        && *buf as ::core::ffi::c_int != ';' as ::core::ffi::c_int
        && (*buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '\\' as ::core::ffi::c_int
            || *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ';' as ::core::ffi::c_int)
    {
        buf = buf.offset(1);
    }
    let mut dst: *mut ::core::ffi::c_char = buf;
    's_91: {
        '_is_semicolon: {
            if *buf as ::core::ffi::c_int != ';' as ::core::ffi::c_int {
                if *buf as ::core::ffi::c_int != NUL {
                    's_61: loop {
                        let c2rust_fresh8 = dst;
                        dst = dst.offset(1);
                        *c2rust_fresh8 = ';' as ::core::ffi::c_char;
                        buf = buf.offset(2 as ::core::ffi::c_int as isize);
                        loop {
                            if !(*buf as ::core::ffi::c_int != NUL
                                && *buf as ::core::ffi::c_int != ';' as ::core::ffi::c_int)
                            {
                                break 's_61;
                            }
                            if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '\\' as ::core::ffi::c_int
                                && *buf.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ';' as ::core::ffi::c_int
                            {
                                break;
                            }
                            let c2rust_fresh9 = buf;
                            buf = buf.offset(1);
                            let c2rust_fresh10 = dst;
                            dst = dst.offset(1);
                            *c2rust_fresh10 = *c2rust_fresh9;
                        }
                    }
                    '_c2rust_label: {
                        if dst < buf {
                        } else {
                            __assert_fail(
                                b"dst < buf\0".as_ptr() as *const ::core::ffi::c_char,
                                b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                                561 as ::core::ffi::c_uint,
                                b"char *vim_findfile_stopdir(char *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    *dst = NUL as ::core::ffi::c_char;
                    if *buf as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                        break '_is_semicolon;
                    }
                }
                buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                break 's_91;
            }
        }
        *buf = NUL as ::core::ffi::c_char;
        buf = buf.offset(1);
    }
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn vim_findfile_cleanup(mut ctx: *mut ::core::ffi::c_void) {
    if ctx.is_null() {
        return;
    }
    vim_findfile_free_visited(ctx);
    ff_clear(ctx as *mut ff_search_ctx_T);
    xfree(ctx);
}
#[no_mangle]
pub unsafe extern "C" fn vim_findfile(
    mut search_ctx_arg: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    let mut rest_of_wildcards: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut path_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut stackp: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
    if search_ctx_arg.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut search_ctx: *mut ff_search_ctx_T = search_ctx_arg as *mut ff_search_ctx_T;
    let mut file_path: String_0 = String_0 {
        data: xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char,
        size: 0,
    };
    if !(*search_ctx).ffsc_start_dir.data.is_null() {
        path_end = (*search_ctx)
            .ffsc_start_dir
            .data
            .offset((*search_ctx).ffsc_start_dir.size as isize);
    }
    '_fail: loop {
        os_breakcheck();
        if !got_int {
            stackp = ff_pop(search_ctx);
            if !stackp.is_null() {
                if (*stackp).ffs_filearray.is_null()
                    && ff_check_visited(
                        &raw mut (*(*search_ctx).ffsc_dir_visited_list).ffvl_visited_list,
                        (*stackp).ffs_fix_path.data,
                        (*stackp).ffs_fix_path.size,
                        (*stackp).ffs_wc_path.data,
                        (*stackp).ffs_wc_path.size,
                    ) == FAIL
                {
                    ff_free_stack_element(stackp);
                    continue;
                } else if (*stackp).ffs_level <= 0 as ::core::ffi::c_int {
                    ff_free_stack_element(stackp);
                    continue;
                } else {
                    *file_path.data.offset(0 as ::core::ffi::c_int as isize) =
                        NUL as ::core::ffi::c_char;
                    file_path.size = 0 as size_t;
                    if (*stackp).ffs_filearray.is_null() {
                        let mut dirptrs: [*mut ::core::ffi::c_char; 2] =
                            [::core::ptr::null_mut::<::core::ffi::c_char>(); 2];
                        dirptrs[0 as ::core::ffi::c_int as usize] = file_path.data;
                        dirptrs[1 as ::core::ffi::c_int as usize] =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        if !vim_isAbsName((*stackp).ffs_fix_path.data)
                            && !(*search_ctx).ffsc_start_dir.data.is_null()
                        {
                            if (*search_ctx).ffsc_start_dir.size.wrapping_add(1 as size_t)
                                >= MAXPATHL as size_t
                            {
                                ff_free_stack_element(stackp);
                                break;
                            } else {
                                let mut add_sep: bool = after_pathsep(
                                    (*search_ctx).ffsc_start_dir.data,
                                    (*search_ctx)
                                        .ffsc_start_dir
                                        .data
                                        .offset((*search_ctx).ffsc_start_dir.size as isize),
                                ) == 0;
                                file_path.size = vim_snprintf(
                                    file_path.data,
                                    MAXPATHL as size_t,
                                    b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    (*search_ctx).ffsc_start_dir.data,
                                    if add_sep as ::core::ffi::c_int != 0 {
                                        PATHSEPSTR.as_ptr()
                                    } else {
                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                    },
                                ) as size_t;
                                if file_path.size >= MAXPATHL as size_t {
                                    ff_free_stack_element(stackp);
                                    break;
                                }
                            }
                        }
                        if file_path
                            .size
                            .wrapping_add((*stackp).ffs_fix_path.size)
                            .wrapping_add(1 as size_t)
                            >= MAXPATHL as size_t
                        {
                            ff_free_stack_element(stackp);
                            break;
                        } else {
                            let mut add_sep_0: bool = after_pathsep(
                                (*stackp).ffs_fix_path.data,
                                (*stackp)
                                    .ffs_fix_path
                                    .data
                                    .offset((*stackp).ffs_fix_path.size as isize),
                            ) == 0;
                            file_path.size = file_path.size.wrapping_add(vim_snprintf(
                                file_path.data.offset(file_path.size as isize),
                                (MAXPATHL as size_t).wrapping_sub(file_path.size),
                                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                (*stackp).ffs_fix_path.data,
                                if add_sep_0 as ::core::ffi::c_int != 0 {
                                    PATHSEPSTR.as_ptr()
                                } else {
                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                },
                            )
                                as size_t);
                            if file_path.size >= MAXPATHL as size_t {
                                ff_free_stack_element(stackp);
                                break;
                            } else {
                                rest_of_wildcards = (*stackp).ffs_wc_path;
                                if *rest_of_wildcards.data as ::core::ffi::c_int != NUL {
                                    if strncmp(
                                        rest_of_wildcards.data,
                                        b"**\0".as_ptr() as *const ::core::ffi::c_char,
                                        2 as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                    {
                                        let mut p: *mut ::core::ffi::c_char = rest_of_wildcards
                                            .data
                                            .offset(2 as ::core::ffi::c_int as isize);
                                        if *p as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                                            *p -= 1;
                                            if file_path.size.wrapping_add(1 as size_t)
                                                >= MAXPATHL as size_t
                                            {
                                                ff_free_stack_element(stackp);
                                                break;
                                            } else {
                                                let c2rust_fresh11 = file_path.size;
                                                file_path.size = file_path.size.wrapping_add(1);
                                                *file_path.data.offset(c2rust_fresh11 as isize) =
                                                    '*' as ::core::ffi::c_char;
                                            }
                                        }
                                        if *p as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                                            memmove(
                                                rest_of_wildcards.data as *mut ::core::ffi::c_void,
                                                rest_of_wildcards
                                                    .data
                                                    .offset(3 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                rest_of_wildcards
                                                    .size
                                                    .wrapping_sub(3 as size_t)
                                                    .wrapping_add(1 as size_t),
                                            );
                                            rest_of_wildcards.size =
                                                rest_of_wildcards.size.wrapping_sub(3 as size_t);
                                            (*stackp).ffs_wc_path.size = rest_of_wildcards.size;
                                        } else {
                                            rest_of_wildcards.data = rest_of_wildcards
                                                .data
                                                .offset(3 as ::core::ffi::c_int as isize);
                                            rest_of_wildcards.size =
                                                rest_of_wildcards.size.wrapping_sub(3 as size_t);
                                        }
                                        if (*stackp).ffs_star_star_empty == 0 as ::core::ffi::c_int
                                        {
                                            (*stackp).ffs_star_star_empty = 1 as ::core::ffi::c_int;
                                            dirptrs[1 as ::core::ffi::c_int as usize] =
                                                (*stackp).ffs_fix_path.data;
                                        }
                                    }
                                    while *rest_of_wildcards.data as ::core::ffi::c_int != 0
                                        && !vim_ispathsep(
                                            *rest_of_wildcards.data as ::core::ffi::c_int,
                                        )
                                    {
                                        if file_path.size.wrapping_add(1 as size_t)
                                            >= MAXPATHL as size_t
                                        {
                                            ff_free_stack_element(stackp);
                                            break '_fail;
                                        } else {
                                            let c2rust_fresh12 = rest_of_wildcards.data;
                                            rest_of_wildcards.data =
                                                rest_of_wildcards.data.offset(1);
                                            let c2rust_fresh13 = file_path.size;
                                            file_path.size = file_path.size.wrapping_add(1);
                                            *file_path.data.offset(c2rust_fresh13 as isize) =
                                                *c2rust_fresh12;
                                            rest_of_wildcards.size =
                                                rest_of_wildcards.size.wrapping_sub(1);
                                        }
                                    }
                                    *file_path.data.offset(file_path.size as isize) =
                                        NUL as ::core::ffi::c_char;
                                    if vim_ispathsep(*rest_of_wildcards.data as ::core::ffi::c_int)
                                    {
                                        rest_of_wildcards.data = rest_of_wildcards.data.offset(1);
                                        rest_of_wildcards.size =
                                            rest_of_wildcards.size.wrapping_sub(1);
                                    }
                                }
                                if path_with_url(dirptrs[0 as ::core::ffi::c_int as usize]) != 0 {
                                    (*stackp).ffs_filearray =
                                        xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                                            as *mut *mut ::core::ffi::c_char;
                                    *(*stackp)
                                        .ffs_filearray
                                        .offset(0 as ::core::ffi::c_int as isize) = xmemdupz(
                                        dirptrs[0 as ::core::ffi::c_int as usize]
                                            as *const ::core::ffi::c_void,
                                        file_path.size,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    (*stackp).ffs_filearray_size = 1 as ::core::ffi::c_int;
                                } else {
                                    expand_wildcards(
                                        if dirptrs[1 as ::core::ffi::c_int as usize].is_null() {
                                            1 as ::core::ffi::c_int
                                        } else {
                                            2 as ::core::ffi::c_int
                                        },
                                        &raw mut dirptrs as *mut *mut ::core::ffi::c_char,
                                        &raw mut (*stackp).ffs_filearray_size,
                                        &raw mut (*stackp).ffs_filearray,
                                        EW_DIR as ::core::ffi::c_int
                                            | EW_ADDSLASH as ::core::ffi::c_int
                                            | EW_SILENT as ::core::ffi::c_int
                                            | EW_NOTWILD as ::core::ffi::c_int,
                                    );
                                }
                                (*stackp).ffs_filearray_cur = 0 as ::core::ffi::c_int;
                                (*stackp).ffs_stage = 0 as ::core::ffi::c_int;
                            }
                        }
                    } else {
                        rest_of_wildcards.data = (*stackp)
                            .ffs_wc_path
                            .data
                            .offset((*stackp).ffs_wc_path.size as isize);
                        rest_of_wildcards.size = 0 as size_t;
                    }
                    if (*stackp).ffs_stage == 0 as ::core::ffi::c_int {
                        's_500: {
                            if *rest_of_wildcards.data as ::core::ffi::c_int == NUL {
                                let mut i: ::core::ffi::c_int = (*stackp).ffs_filearray_cur;
                                loop {
                                    if i >= (*stackp).ffs_filearray_size {
                                        break 's_500;
                                    }
                                    if !(path_with_url(*(*stackp).ffs_filearray.offset(i as isize))
                                        == 0
                                        && !os_isdir(*(*stackp).ffs_filearray.offset(i as isize)))
                                    {
                                        let mut len: size_t =
                                            strlen(*(*stackp).ffs_filearray.offset(i as isize));
                                        if len
                                            .wrapping_add(1 as size_t)
                                            .wrapping_add((*search_ctx).ffsc_file_to_search.size)
                                            >= MAXPATHL as size_t
                                        {
                                            ff_free_stack_element(stackp);
                                            break '_fail;
                                        } else {
                                            let mut add_sep_1: bool = after_pathsep(
                                                *(*stackp).ffs_filearray.offset(i as isize),
                                                (*(*stackp).ffs_filearray.offset(i as isize))
                                                    .offset(len as isize),
                                            ) == 0;
                                            file_path.size = vim_snprintf(
                                                file_path.data,
                                                MAXPATHL as size_t,
                                                b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                                *(*stackp).ffs_filearray.offset(i as isize),
                                                if add_sep_1 as ::core::ffi::c_int != 0 {
                                                    PATHSEPSTR.as_ptr()
                                                } else {
                                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                                },
                                                (*search_ctx).ffsc_file_to_search.data,
                                            )
                                                as size_t;
                                            if file_path.size >= MAXPATHL as size_t {
                                                ff_free_stack_element(stackp);
                                                break '_fail;
                                            } else {
                                                len = file_path.size;
                                                let mut suf: *mut ::core::ffi::c_char =
                                                    (if (*search_ctx).ffsc_tagfile != 0 {
                                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                                    } else {
                                                        (*curbuf).b_p_sua
                                                            as *const ::core::ffi::c_char
                                                    })
                                                        as *mut ::core::ffi::c_char;
                                                loop {
                                                    if (path_with_url(file_path.data) != 0
                                                        || os_path_exists(file_path.data)
                                                            as ::core::ffi::c_int
                                                            != 0
                                                            && ((*search_ctx).ffsc_find_what
                                                                == FINDFILE_BOTH
                                                                    as ::core::ffi::c_int
                                                                || ((*search_ctx).ffsc_find_what
                                                                    == FINDFILE_DIR
                                                                        as ::core::ffi::c_int)
                                                                    as ::core::ffi::c_int
                                                                    == os_isdir(file_path.data)
                                                                        as ::core::ffi::c_int))
                                                        && ff_check_visited(
                                                            &raw mut (*(*search_ctx)
                                                                .ffsc_visited_list)
                                                                .ffvl_visited_list,
                                                            file_path.data,
                                                            file_path.size,
                                                            b"\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                                as *mut ::core::ffi::c_char,
                                                            0 as size_t,
                                                        ) == OK
                                                    {
                                                        '_c2rust_label: {
                                                            if i < 2147483647 as ::core::ffi::c_int
                                                            {
                                                            } else {
                                                                __assert_fail(
                                                                    b"i < INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                                                    b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                                    875 as ::core::ffi::c_uint,
                                                                    b"char *vim_findfile(void *)\0".as_ptr()
                                                                        as *const ::core::ffi::c_char,
                                                                );
                                                            }
                                                        };
                                                        (*stackp).ffs_filearray_cur =
                                                            i + 1 as ::core::ffi::c_int;
                                                        ff_push(search_ctx, stackp);
                                                        if path_with_url(file_path.data) == 0 {
                                                            file_path.size =
                                                                simplify_filename(file_path.data);
                                                        }
                                                        if os_dirname(
                                                            ff_expand_buffer.data,
                                                            MAXPATHL as size_t,
                                                        ) == OK
                                                        {
                                                            ff_expand_buffer.size =
                                                                strlen(ff_expand_buffer.data);
                                                            let mut p_0: *mut ::core::ffi::c_char =
                                                                path_shorten_fname(
                                                                    file_path.data,
                                                                    ff_expand_buffer.data,
                                                                );
                                                            if !p_0.is_null() {
                                                                memmove(
                                                                    file_path.data as *mut ::core::ffi::c_void,
                                                                    p_0 as *const ::core::ffi::c_void,
                                                                    (file_path
                                                                        .data
                                                                        .offset(file_path.size as isize)
                                                                        .offset_from(p_0) as size_t)
                                                                        .wrapping_add(1 as size_t),
                                                                );
                                                                file_path.size =
                                                                    file_path.size.wrapping_sub(
                                                                        p_0.offset_from(
                                                                            file_path.data,
                                                                        )
                                                                            as size_t,
                                                                    );
                                                            }
                                                        }
                                                        return file_path.data;
                                                    }
                                                    if *suf as ::core::ffi::c_int == NUL {
                                                        break;
                                                    }
                                                    '_c2rust_label_0: {
                                                        if 4096 as size_t >= file_path.size {
                                                        } else {
                                                            __assert_fail(
                                                                b"MAXPATHL >= file_path.size\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                                b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                                907 as ::core::ffi::c_uint,
                                                                b"char *vim_findfile(void *)\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                        }
                                                    };
                                                    file_path.size =
                                                        len.wrapping_add(copy_option_part(
                                                            &raw mut suf,
                                                            file_path.data.offset(len as isize),
                                                            (MAXPATHL as size_t).wrapping_sub(len),
                                                            b",\0".as_ptr()
                                                                as *const ::core::ffi::c_char
                                                                as *mut ::core::ffi::c_char,
                                                        ));
                                                }
                                            }
                                        }
                                    }
                                    i += 1;
                                }
                            } else {
                                let mut i_0: ::core::ffi::c_int = (*stackp).ffs_filearray_cur;
                                while i_0 < (*stackp).ffs_filearray_size {
                                    if os_isdir(*(*stackp).ffs_filearray.offset(i_0 as isize)) {
                                        ff_push(
                                            search_ctx,
                                            ff_create_stack_element(
                                                *(*stackp).ffs_filearray.offset(i_0 as isize),
                                                strlen(
                                                    *(*stackp).ffs_filearray.offset(i_0 as isize),
                                                ),
                                                rest_of_wildcards.data,
                                                rest_of_wildcards.size,
                                                (*stackp).ffs_level - 1 as ::core::ffi::c_int,
                                                0 as ::core::ffi::c_int,
                                            ),
                                        );
                                    }
                                    i_0 += 1;
                                }
                            }
                        }
                        (*stackp).ffs_filearray_cur = 0 as ::core::ffi::c_int;
                        (*stackp).ffs_stage = 1 as ::core::ffi::c_int;
                    }
                    if strncmp(
                        (*stackp).ffs_wc_path.data,
                        b"**\0".as_ptr() as *const ::core::ffi::c_char,
                        2 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        let mut i_1: ::core::ffi::c_int = (*stackp).ffs_filearray_cur;
                        while i_1 < (*stackp).ffs_filearray_size {
                            if path_fnamecmp(
                                *(*stackp).ffs_filearray.offset(i_1 as isize),
                                (*stackp).ffs_fix_path.data,
                            ) != 0 as ::core::ffi::c_int
                            {
                                if os_isdir(*(*stackp).ffs_filearray.offset(i_1 as isize)) {
                                    ff_push(
                                        search_ctx,
                                        ff_create_stack_element(
                                            *(*stackp).ffs_filearray.offset(i_1 as isize),
                                            strlen(*(*stackp).ffs_filearray.offset(i_1 as isize)),
                                            (*stackp).ffs_wc_path.data,
                                            (*stackp).ffs_wc_path.size,
                                            (*stackp).ffs_level - 1 as ::core::ffi::c_int,
                                            1 as ::core::ffi::c_int,
                                        ),
                                    );
                                }
                            }
                            i_1 += 1;
                        }
                    }
                    ff_free_stack_element(stackp);
                    continue;
                }
            }
        }
        if !(!(*search_ctx).ffsc_start_dir.data.is_null()
            && !(*search_ctx).ffsc_stopdirs_v.is_null()
            && !got_int)
        {
            break;
        }
        let mut sptr: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
        let mut plen: ptrdiff_t = path_end.offset_from((*search_ctx).ffsc_start_dir.data)
            + (*path_end as ::core::ffi::c_int != NUL) as ::core::ffi::c_int as ptrdiff_t;
        if ff_path_in_stoplist(
            (*search_ctx).ffsc_start_dir.data,
            plen as size_t,
            (*search_ctx).ffsc_stopdirs_v,
        ) {
            break;
        }
        while path_end > (*search_ctx).ffsc_start_dir.data
            && vim_ispathsep(*path_end as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            path_end = path_end.offset(-1);
        }
        while path_end > (*search_ctx).ffsc_start_dir.data
            && !vim_ispathsep(
                *path_end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            )
        {
            path_end = path_end.offset(-1);
        }
        *path_end = NUL as ::core::ffi::c_char;
        (*search_ctx).ffsc_start_dir.size =
            path_end.offset_from((*search_ctx).ffsc_start_dir.data) as size_t;
        path_end = path_end.offset(-1);
        if *(*search_ctx).ffsc_start_dir.data as ::core::ffi::c_int == NUL {
            break;
        }
        if (*search_ctx)
            .ffsc_start_dir
            .size
            .wrapping_add(1 as size_t)
            .wrapping_add((*search_ctx).ffsc_fix_path.size)
            >= MAXPATHL as size_t
        {
            break;
        }
        let mut add_sep_2: bool = after_pathsep(
            (*search_ctx).ffsc_start_dir.data,
            (*search_ctx)
                .ffsc_start_dir
                .data
                .offset((*search_ctx).ffsc_start_dir.size as isize),
        ) == 0;
        file_path.size = vim_snprintf(
            file_path.data,
            MAXPATHL as size_t,
            b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            (*search_ctx).ffsc_start_dir.data,
            if add_sep_2 as ::core::ffi::c_int != 0 {
                PATHSEPSTR.as_ptr()
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            (*search_ctx).ffsc_fix_path.data,
        ) as size_t;
        if file_path.size >= MAXPATHL as size_t {
            break;
        }
        sptr = ff_create_stack_element(
            file_path.data,
            file_path.size,
            (*search_ctx).ffsc_wc_path.data,
            (*search_ctx).ffsc_wc_path.size,
            (*search_ctx).ffsc_level,
            0 as ::core::ffi::c_int,
        );
        ff_push(search_ctx, sptr);
    }
    xfree(file_path.data as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn vim_findfile_free_visited(mut search_ctx_arg: *mut ::core::ffi::c_void) {
    if search_ctx_arg.is_null() {
        return;
    }
    let mut search_ctx: *mut ff_search_ctx_T = search_ctx_arg as *mut ff_search_ctx_T;
    vim_findfile_free_visited_list(&raw mut (*search_ctx).ffsc_visited_lists_list);
    vim_findfile_free_visited_list(&raw mut (*search_ctx).ffsc_dir_visited_lists_list);
}
unsafe extern "C" fn vim_findfile_free_visited_list(
    mut list_headp: *mut *mut ff_visited_list_hdr_T,
) {
    let mut vp: *mut ff_visited_list_hdr_T = ::core::ptr::null_mut::<ff_visited_list_hdr_T>();
    while !(*list_headp).is_null() {
        vp = (**list_headp).ffvl_next as *mut ff_visited_list_hdr_T;
        ff_free_visited_list((**list_headp).ffvl_visited_list);
        xfree((**list_headp).ffvl_filename as *mut ::core::ffi::c_void);
        xfree(*list_headp as *mut ::core::ffi::c_void);
        *list_headp = vp;
    }
    *list_headp = ::core::ptr::null_mut::<ff_visited_list_hdr_T>();
}
unsafe extern "C" fn ff_free_visited_list(mut vl: *mut ff_visited_T) {
    let mut vp: *mut ff_visited_T = ::core::ptr::null_mut::<ff_visited_T>();
    while !vl.is_null() {
        vp = (*vl).ffv_next as *mut ff_visited_T;
        xfree((*vl).ffv_wc_path as *mut ::core::ffi::c_void);
        xfree(vl as *mut ::core::ffi::c_void);
        vl = vp;
    }
    vl = ::core::ptr::null_mut::<ff_visited_T>();
}
unsafe extern "C" fn ff_get_visited_list(
    mut filename: *mut ::core::ffi::c_char,
    mut filenamelen: size_t,
    mut list_headp: *mut *mut ff_visited_list_hdr_T,
) -> *mut ff_visited_list_hdr_T {
    let mut retptr: *mut ff_visited_list_hdr_T = ::core::ptr::null_mut::<ff_visited_list_hdr_T>();
    if !(*list_headp).is_null() {
        retptr = *list_headp;
        while !retptr.is_null() {
            if path_fnamecmp(filename, (*retptr).ffvl_filename) == 0 as ::core::ffi::c_int {
                return retptr;
            }
            retptr = (*retptr).ffvl_next as *mut ff_visited_list_hdr_T;
        }
    }
    retptr = xmalloc(::core::mem::size_of::<ff_visited_list_hdr_T>()) as *mut ff_visited_list_hdr_T;
    (*retptr).ffvl_visited_list = ::core::ptr::null_mut::<ff_visited_T>();
    (*retptr).ffvl_filename =
        xmemdupz(filename as *const ::core::ffi::c_void, filenamelen) as *mut ::core::ffi::c_char;
    (*retptr).ffvl_next = *list_headp as *mut ff_visited_list_hdr;
    *list_headp = retptr;
    return retptr;
}
unsafe extern "C" fn ff_wc_equal(
    mut s1: *mut ::core::ffi::c_char,
    mut s2: *mut ::core::ffi::c_char,
) -> bool {
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut prev1: ::core::ffi::c_int = NUL;
    let mut prev2: ::core::ffi::c_int = NUL;
    if s1 == s2 {
        return true_0 != 0;
    }
    if s1.is_null() || s2.is_null() {
        return false_0 != 0;
    }
    i = 0 as ::core::ffi::c_int;
    j = 0 as ::core::ffi::c_int;
    while *s1.offset(i as isize) as ::core::ffi::c_int != NUL
        && *s2.offset(j as isize) as ::core::ffi::c_int != NUL
    {
        let mut c1: ::core::ffi::c_int = utf_ptr2char(s1.offset(i as isize));
        let mut c2: ::core::ffi::c_int = utf_ptr2char(s2.offset(j as isize));
        if (if p_fic != 0 {
            (mb_tolower(c1) != mb_tolower(c2)) as ::core::ffi::c_int
        } else {
            (c1 != c2) as ::core::ffi::c_int
        }) != 0
            && (prev1 != '*' as ::core::ffi::c_int || prev2 != '*' as ::core::ffi::c_int)
        {
            return false_0 != 0;
        }
        prev2 = prev1;
        prev1 = c1;
        i += utfc_ptr2len(s1.offset(i as isize));
        j += utfc_ptr2len(s2.offset(j as isize));
    }
    return *s1.offset(i as isize) as ::core::ffi::c_int
        == *s2.offset(j as isize) as ::core::ffi::c_int;
}
unsafe extern "C" fn ff_check_visited(
    mut visited_list: *mut *mut ff_visited_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fnamelen: size_t,
    mut wc_path: *mut ::core::ffi::c_char,
    mut wc_pathlen: size_t,
) -> ::core::ffi::c_int {
    let mut vp: *mut ff_visited_T = ::core::ptr::null_mut::<ff_visited_T>();
    let mut url: bool = false_0 != 0;
    let mut file_id: FileID = FileID {
        inode: 0,
        device_id: 0,
    };
    if path_with_url(fname) != 0 {
        xmemcpyz(
            ff_expand_buffer.data as *mut ::core::ffi::c_void,
            fname as *const ::core::ffi::c_void,
            fnamelen,
        );
        ff_expand_buffer.size = fnamelen;
        url = true_0 != 0;
    } else {
        *ff_expand_buffer
            .data
            .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        ff_expand_buffer.size = 0 as size_t;
        if !os_fileid(fname, &raw mut file_id) {
            return FAIL;
        }
    }
    vp = *visited_list;
    while !vp.is_null() {
        if url as ::core::ffi::c_int != 0
            && path_fnamecmp(
                &raw mut (*vp).ffv_fname as *mut ::core::ffi::c_char,
                ff_expand_buffer.data,
            ) == 0 as ::core::ffi::c_int
            || !url
                && (*vp).file_id_valid as ::core::ffi::c_int != 0
                && os_fileid_equal(&raw mut (*vp).file_id, &raw mut file_id) as ::core::ffi::c_int
                    != 0
        {
            if ff_wc_equal((*vp).ffv_wc_path, wc_path) {
                return FAIL;
            }
        }
        vp = (*vp).ffv_next as *mut ff_visited_T;
    }
    vp = xmalloc(
        (40 as size_t)
            .wrapping_add(ff_expand_buffer.size)
            .wrapping_add(1 as size_t),
    ) as *mut ff_visited_T;
    if !url {
        (*vp).file_id_valid = true_0 != 0;
        (*vp).file_id = file_id;
        *(&raw mut (*vp).ffv_fname as *mut ::core::ffi::c_char)
            .offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
    } else {
        (*vp).file_id_valid = false_0 != 0;
        strcpy(
            &raw mut (*vp).ffv_fname as *mut ::core::ffi::c_char,
            ff_expand_buffer.data,
        );
    }
    if !wc_path.is_null() {
        (*vp).ffv_wc_path =
            xmemdupz(wc_path as *const ::core::ffi::c_void, wc_pathlen) as *mut ::core::ffi::c_char;
    } else {
        (*vp).ffv_wc_path = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*vp).ffv_next = *visited_list as *mut ff_visited;
    *visited_list = vp;
    return OK;
}
unsafe extern "C" fn ff_create_stack_element(
    mut fix_part: *mut ::core::ffi::c_char,
    mut fix_partlen: size_t,
    mut wc_part: *mut ::core::ffi::c_char,
    mut wc_partlen: size_t,
    mut level: ::core::ffi::c_int,
    mut star_star_empty: ::core::ffi::c_int,
) -> *mut ff_stack_T {
    let mut stack: *mut ff_stack_T =
        xmalloc(::core::mem::size_of::<ff_stack_T>()) as *mut ff_stack_T;
    (*stack).ffs_prev = ::core::ptr::null_mut::<ff_stack>();
    (*stack).ffs_filearray = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    (*stack).ffs_filearray_size = 0 as ::core::ffi::c_int;
    (*stack).ffs_filearray_cur = 0 as ::core::ffi::c_int;
    (*stack).ffs_stage = 0 as ::core::ffi::c_int;
    (*stack).ffs_level = level;
    (*stack).ffs_star_star_empty = star_star_empty;
    if fix_part.is_null() {
        fix_part = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        fix_partlen = 0 as size_t;
    }
    (*stack).ffs_fix_path = cbuf_to_string(fix_part, fix_partlen);
    if wc_part.is_null() {
        wc_part = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        wc_partlen = 0 as size_t;
    }
    (*stack).ffs_wc_path = cbuf_to_string(wc_part, wc_partlen);
    return stack;
}
unsafe extern "C" fn ff_push(mut search_ctx: *mut ff_search_ctx_T, mut stack_ptr: *mut ff_stack_T) {
    if stack_ptr.is_null() {
        return;
    }
    (*stack_ptr).ffs_prev = (*search_ctx).ffsc_stack_ptr as *mut ff_stack;
    (*search_ctx).ffsc_stack_ptr = stack_ptr;
}
unsafe extern "C" fn ff_pop(mut search_ctx: *mut ff_search_ctx_T) -> *mut ff_stack_T {
    let mut sptr: *mut ff_stack_T = (*search_ctx).ffsc_stack_ptr;
    if !(*search_ctx).ffsc_stack_ptr.is_null() {
        (*search_ctx).ffsc_stack_ptr = (*(*search_ctx).ffsc_stack_ptr).ffs_prev as *mut ff_stack_T;
    }
    return sptr;
}
unsafe extern "C" fn ff_free_stack_element(stack_ptr: *mut ff_stack_T) {
    if stack_ptr.is_null() {
        return;
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*stack_ptr).ffs_fix_path.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    *ptr_;
    (*stack_ptr).ffs_fix_path.size = 0 as size_t;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*stack_ptr).ffs_wc_path.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    *ptr__0;
    (*stack_ptr).ffs_wc_path.size = 0 as size_t;
    if !(*stack_ptr).ffs_filearray.is_null() {
        FreeWild((*stack_ptr).ffs_filearray_size, (*stack_ptr).ffs_filearray);
    }
    xfree(stack_ptr as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn ff_clear(mut search_ctx: *mut ff_search_ctx_T) {
    let mut sptr: *mut ff_stack_T = ::core::ptr::null_mut::<ff_stack_T>();
    loop {
        sptr = ff_pop(search_ctx);
        if sptr.is_null() {
            break;
        }
        ff_free_stack_element(sptr);
    }
    if !(*search_ctx).ffsc_stopdirs_v.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !(*(*search_ctx).ffsc_stopdirs_v.offset(i as isize))
            .data
            .is_null()
        {
            xfree(
                (*(*search_ctx).ffsc_stopdirs_v.offset(i as isize)).data
                    as *mut ::core::ffi::c_void,
            );
            i += 1;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*search_ctx).ffsc_stopdirs_v as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
    }
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*search_ctx).ffsc_file_to_search.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    *ptr__0;
    (*search_ctx).ffsc_file_to_search.size = 0 as size_t;
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*search_ctx).ffsc_start_dir.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL;
    *ptr__1;
    (*search_ctx).ffsc_start_dir.size = 0 as size_t;
    let mut ptr__2: *mut *mut ::core::ffi::c_void =
        &raw mut (*search_ctx).ffsc_fix_path.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__2);
    *ptr__2 = NULL;
    *ptr__2;
    (*search_ctx).ffsc_fix_path.size = 0 as size_t;
    let mut ptr__3: *mut *mut ::core::ffi::c_void =
        &raw mut (*search_ctx).ffsc_wc_path.data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__3);
    *ptr__3 = NULL;
    *ptr__3;
    (*search_ctx).ffsc_wc_path.size = 0 as size_t;
    (*search_ctx).ffsc_level = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn ff_path_in_stoplist(
    mut path: *mut ::core::ffi::c_char,
    mut path_len: size_t,
    mut stopdirs_v: *mut String_0,
) -> bool {
    while path_len > 1 as size_t
        && vim_ispathsep(
            *path.offset(path_len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
        ) as ::core::ffi::c_int
            != 0
    {
        path_len = path_len.wrapping_sub(1);
    }
    if path_len == 0 as size_t {
        return true_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !(*stopdirs_v.offset(i as isize)).data.is_null() {
        if path_fnamencmp((*stopdirs_v.offset(i as isize)).data, path, path_len)
            == 0 as ::core::ffi::c_int
            && ((*stopdirs_v.offset(i as isize)).size <= path_len
                || vim_ispathsep(
                    *(*stopdirs_v.offset(i as isize))
                        .data
                        .offset(path_len as isize) as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    != 0)
        {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn find_file_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut first: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut file_to_find: *mut *mut ::core::ffi::c_char,
    mut search_ctx: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return find_file_in_path_option(
        ptr,
        len,
        options,
        first,
        if *(*curbuf).b_p_path as ::core::ffi::c_int == NUL {
            p_path
        } else {
            (*curbuf).b_p_path
        },
        FINDFILE_BOTH as ::core::ffi::c_int,
        rel_fname,
        (*curbuf).b_p_sua,
        file_to_find,
        search_ctx,
    );
}
#[no_mangle]
pub unsafe extern "C" fn find_directory_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut file_to_find: *mut *mut ::core::ffi::c_char,
    mut search_ctx: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return find_file_in_path_option(
        ptr,
        len,
        options,
        true_0,
        p_cdpath,
        FINDFILE_DIR as ::core::ffi::c_int,
        rel_fname,
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        file_to_find,
        search_ctx,
    );
}
#[no_mangle]
pub unsafe extern "C" fn find_file_in_path_option(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut first: ::core::ffi::c_int,
    mut path_option: *mut ::core::ffi::c_char,
    mut find_what: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut suffixes: *mut ::core::ffi::c_char,
    mut file_to_find: *mut *mut ::core::ffi::c_char,
    mut search_ctx_arg: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut search_ctx: *mut *mut ff_search_ctx_T = search_ctx_arg as *mut *mut ff_search_ctx_T;
    static mut dir: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static mut did_findfile_init: bool = false_0 != 0;
    let mut file_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static mut file_to_findlen: size_t = 0 as size_t;
    if !rel_fname.is_null() && path_with_url(rel_fname) != 0 {
        rel_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if first == true_0 {
        if len == 0 as size_t {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut save_char: ::core::ffi::c_char = *ptr.offset(len as isize);
        *ptr.offset(len as isize) = NUL as ::core::ffi::c_char;
        file_to_findlen = expand_env_esc(
            ptr,
            &raw mut NameBuff as *mut ::core::ffi::c_char,
            MAXPATHL,
            false_0 != 0,
            true_0 != 0,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
        *ptr.offset(len as isize) = save_char;
        xfree(*file_to_find as *mut ::core::ffi::c_void);
        *file_to_find = xmemdupz(
            &raw mut NameBuff as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            file_to_findlen,
        ) as *mut ::core::ffi::c_char;
        if options & FNAME_UNESC as ::core::ffi::c_int != 0 {
            ptr = *file_to_find;
            while *ptr as ::core::ffi::c_int != NUL {
                if *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ' ' as ::core::ffi::c_int
                {
                    memmove(
                        ptr as *mut ::core::ffi::c_void,
                        ptr.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                        ((*file_to_find)
                            .offset(file_to_findlen as isize)
                            .offset_from(ptr.offset(1 as ::core::ffi::c_int as isize))
                            as size_t)
                            .wrapping_add(1 as size_t),
                    );
                    file_to_findlen = file_to_findlen.wrapping_sub(1);
                }
                ptr = ptr.offset(1);
            }
        }
    }
    let mut rel_to_curdir: bool = *(*file_to_find).offset(0 as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        && (*(*file_to_find).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            || vim_ispathsep(
                *(*file_to_find).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) as ::core::ffi::c_int
                != 0
            || *(*file_to_find).offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && (*(*file_to_find).offset(2 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    == NUL
                    || vim_ispathsep(*(*file_to_find).offset(2 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int) as ::core::ffi::c_int
                        != 0));
    '_theend: {
        's_300: {
            if vim_isAbsName(*file_to_find) as ::core::ffi::c_int != 0
                || rel_to_curdir as ::core::ffi::c_int != 0
            {
                if first == true_0 {
                    if path_with_url(*file_to_find) != 0 {
                        file_name =
                            xmemdupz(*file_to_find as *const ::core::ffi::c_void, file_to_findlen)
                                as *mut ::core::ffi::c_char;
                        break '_theend;
                    } else {
                        let mut rel_fnamelen: size_t = if !rel_fname.is_null() {
                            strlen(rel_fname)
                        } else {
                            0 as size_t
                        };
                        let mut run: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                        loop {
                            if run > 2 as ::core::ffi::c_int {
                                break 's_300;
                            }
                            let mut l: size_t = file_to_findlen;
                            if run == 1 as ::core::ffi::c_int
                                && rel_to_curdir as ::core::ffi::c_int != 0
                                && options & FNAME_REL as ::core::ffi::c_int != 0
                                && !rel_fname.is_null()
                                && rel_fnamelen.wrapping_add(l) < MAXPATHL as size_t
                            {
                                l = vim_snprintf(
                                    &raw mut NameBuff as *mut ::core::ffi::c_char,
                                    MAXPATHL as size_t,
                                    b"%.*s%s\0".as_ptr() as *const ::core::ffi::c_char,
                                    path_tail(rel_fname).offset_from(rel_fname)
                                        as ::core::ffi::c_int,
                                    rel_fname,
                                    *file_to_find,
                                ) as size_t;
                                '_c2rust_label: {
                                    if l < 4096 as size_t {
                                    } else {
                                        __assert_fail(
                                            b"l < MAXPATHL\0".as_ptr() as *const ::core::ffi::c_char,
                                            b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            1499 as ::core::ffi::c_uint,
                                            b"char *find_file_in_path_option(char *, size_t, int, int, char *, int, char *, char *, char **, char **)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                    }
                                };
                            } else {
                                strcpy(
                                    &raw mut NameBuff as *mut ::core::ffi::c_char,
                                    *file_to_find,
                                );
                                run = 2 as ::core::ffi::c_int;
                            }
                            let mut NameBufflen: size_t = l;
                            let mut suffix: *mut ::core::ffi::c_char = suffixes;
                            loop {
                                if os_path_exists(&raw mut NameBuff as *mut ::core::ffi::c_char)
                                    as ::core::ffi::c_int
                                    != 0
                                    && (find_what == FINDFILE_BOTH as ::core::ffi::c_int
                                        || (find_what == FINDFILE_DIR as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                            == os_isdir(
                                                &raw mut NameBuff as *mut ::core::ffi::c_char,
                                            )
                                                as ::core::ffi::c_int)
                                {
                                    file_name = xmemdupz(
                                        &raw mut NameBuff as *mut ::core::ffi::c_char
                                            as *const ::core::ffi::c_void,
                                        NameBufflen,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    break '_theend;
                                } else {
                                    if *suffix as ::core::ffi::c_int == NUL {
                                        break;
                                    }
                                    '_c2rust_label_0: {
                                        if 4096 as size_t >= l {
                                        } else {
                                            __assert_fail(
                                                b"MAXPATHL >= l\0".as_ptr() as *const ::core::ffi::c_char,
                                                b"/home/overlord/projects/neovim/neovim/src/nvim/file_search.c\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                1518 as ::core::ffi::c_uint,
                                                b"char *find_file_in_path_option(char *, size_t, int, int, char *, int, char *, char *, char **, char **)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    NameBufflen = l.wrapping_add(copy_option_part(
                                        &raw mut suffix,
                                        (&raw mut NameBuff as *mut ::core::ffi::c_char)
                                            .offset(l as isize),
                                        (MAXPATHL as size_t).wrapping_sub(l),
                                        b",\0".as_ptr() as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char,
                                    ));
                                }
                            }
                            run += 1;
                        }
                    }
                }
            } else {
                if first == true_0 {
                    vim_findfile_free_visited(*search_ctx as *mut ::core::ffi::c_void);
                    dir = path_option;
                    did_findfile_init = false_0 != 0;
                }
                loop {
                    if did_findfile_init {
                        file_name = vim_findfile(*search_ctx as *mut ::core::ffi::c_void);
                        if !file_name.is_null() {
                            break;
                        }
                        did_findfile_init = false_0 != 0;
                    } else {
                        let mut r_ptr: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        if dir.is_null() || *dir as ::core::ffi::c_int == NUL {
                            vim_findfile_cleanup(*search_ctx as *mut ::core::ffi::c_void);
                            *search_ctx = ::core::ptr::null_mut::<ff_search_ctx_T>();
                            break;
                        } else {
                            let mut buf: *mut ::core::ffi::c_char =
                                xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
                            *buf.offset(0 as ::core::ffi::c_int as isize) =
                                NUL as ::core::ffi::c_char;
                            copy_option_part(
                                &raw mut dir,
                                buf,
                                MAXPATHL as size_t,
                                b" ,\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            );
                            r_ptr = vim_findfile_stopdir(buf);
                            *search_ctx = vim_findfile_init(
                                buf,
                                *file_to_find,
                                file_to_findlen,
                                r_ptr,
                                100 as ::core::ffi::c_int,
                                false_0,
                                find_what,
                                *search_ctx as *mut ::core::ffi::c_void,
                                false_0,
                                rel_fname,
                            ) as *mut ff_search_ctx_T;
                            if !(*search_ctx).is_null() {
                                did_findfile_init = true_0 != 0;
                            }
                            xfree(buf as *mut ::core::ffi::c_void);
                        }
                    }
                }
            }
        }
        if file_name.is_null() && options & FNAME_MESS as ::core::ffi::c_int != 0 {
            if first == true_0 {
                if find_what == FINDFILE_DIR as ::core::ffi::c_int {
                    semsg(
                        gettext(
                            &raw const e_cant_find_directory_str_in_cdpath
                                as *const ::core::ffi::c_char,
                        ),
                        *file_to_find,
                    );
                } else {
                    semsg(
                        gettext(
                            &raw const e_cant_find_file_str_in_path as *const ::core::ffi::c_char,
                        ),
                        *file_to_find,
                    );
                }
            } else if find_what == FINDFILE_DIR as ::core::ffi::c_int {
                semsg(
                    gettext(
                        &raw const e_no_more_directory_str_found_in_cdpath
                            as *const ::core::ffi::c_char,
                    ),
                    *file_to_find,
                );
            } else {
                semsg(
                    gettext(
                        &raw const e_no_more_file_str_found_in_path as *const ::core::ffi::c_char,
                    ),
                    *file_to_find,
                );
            }
        }
    }
    return file_name;
}
#[no_mangle]
pub unsafe extern "C" fn grab_file_name(
    mut count: ::core::ffi::c_int,
    mut file_lnum: *mut linenr_T,
) -> *mut ::core::ffi::c_char {
    let mut options: ::core::ffi::c_int = FNAME_MESS as ::core::ffi::c_int
        | FNAME_EXP as ::core::ffi::c_int
        | FNAME_REL as ::core::ffi::c_int
        | FNAME_UNESC as ::core::ffi::c_int;
    if VIsual_active {
        let mut len: size_t = 0;
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if get_visual_text(
            ::core::ptr::null_mut::<cmdarg_T>(),
            &raw mut ptr,
            &raw mut len,
        ) as ::core::ffi::c_int
            == FAIL
        {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !file_lnum.is_null()
            && *ptr.offset(len as isize) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *(*__ctype_b_loc()).offset(*ptr.offset(len.wrapping_add(1 as size_t) as isize)
                as uint8_t as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
        {
            let mut p: *mut ::core::ffi::c_char = ptr
                .offset(len as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            *file_lnum = getdigits_int32(&raw mut p, false_0 != 0, 0 as int32_t) as linenr_T;
        }
        return find_file_name_in_path(
            ptr,
            len,
            options,
            count as ::core::ffi::c_long,
            (*curbuf).b_ffname,
        );
    }
    return file_name_at_cursor(options | FNAME_HYP as ::core::ffi::c_int, count, file_lnum);
}
#[no_mangle]
pub unsafe extern "C" fn file_name_at_cursor(
    mut options: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut file_lnum: *mut linenr_T,
) -> *mut ::core::ffi::c_char {
    return file_name_in_line(
        get_cursor_line_ptr(),
        (*curwin).w_cursor.col as ::core::ffi::c_int,
        options,
        count,
        (*curbuf).b_ffname,
        file_lnum,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_name_in_line(
    mut line: *mut ::core::ffi::c_char,
    mut col: ::core::ffi::c_int,
    mut options: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut rel_fname: *mut ::core::ffi::c_char,
    mut file_lnum: *mut linenr_T,
) -> *mut ::core::ffi::c_char {
    let mut ptr: *mut ::core::ffi::c_char = line.offset(col as isize);
    while *ptr as ::core::ffi::c_int != NUL && !vim_isfilec(*ptr as uint8_t as ::core::ffi::c_int) {
        ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
    }
    if *ptr as ::core::ffi::c_int == NUL {
        if options & FNAME_MESS as ::core::ffi::c_int != 0 {
            emsg(gettext(
                b"E446: No file name under cursor\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut len: size_t = 0;
    let mut in_type: bool = true_0 != 0;
    let mut is_url: bool = false_0 != 0;
    while ptr > line {
        len = utf_head_off(line, ptr.offset(-(1 as ::core::ffi::c_int as isize))) as size_t;
        if len > 0 as size_t {
            ptr = ptr.offset(-(len.wrapping_add(1 as size_t) as isize));
        } else {
            if !(vim_isfilec(
                *ptr.offset(-1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            ) as ::core::ffi::c_int
                != 0
                || options & FNAME_HYP as ::core::ffi::c_int != 0
                    && path_is_url(ptr.offset(-(1 as ::core::ffi::c_int as isize))) != 0)
            {
                break;
            }
            ptr = ptr.offset(-1);
        }
    }
    len = (if path_has_drive_letter(ptr, strlen(ptr)) as ::core::ffi::c_int != 0 {
        2 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as size_t;
    while vim_isfilec(*ptr.offset(len as isize) as uint8_t as ::core::ffi::c_int)
        as ::core::ffi::c_int
        != 0
        || *ptr.offset(len as isize) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *ptr.offset(len.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == ' ' as ::core::ffi::c_int
        || options & FNAME_HYP as ::core::ffi::c_int != 0
            && path_is_url(ptr.offset(len as isize)) != 0
        || is_url as ::core::ffi::c_int != 0
            && !vim_strchr(
                b":?&=\0".as_ptr() as *const ::core::ffi::c_char,
                *ptr.offset(len as isize) as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
    {
        if *ptr.offset(len as isize) as ::core::ffi::c_int >= 'A' as ::core::ffi::c_int
            && *ptr.offset(len as isize) as ::core::ffi::c_int <= 'Z' as ::core::ffi::c_int
            || *ptr.offset(len as isize) as ::core::ffi::c_int >= 'a' as ::core::ffi::c_int
                && *ptr.offset(len as isize) as ::core::ffi::c_int <= 'z' as ::core::ffi::c_int
        {
            if in_type as ::core::ffi::c_int != 0
                && path_is_url(
                    ptr.offset(len as isize)
                        .offset(1 as ::core::ffi::c_int as isize),
                ) != 0
            {
                is_url = true_0 != 0;
            }
        } else {
            in_type = false_0 != 0;
        }
        if *ptr.offset(len as isize) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *ptr.offset(len.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == ' ' as ::core::ffi::c_int
        {
            len = len.wrapping_add(1);
        }
        len = len.wrapping_add(utfc_ptr2len(ptr.offset(len as isize)) as size_t);
    }
    if len > 2 as size_t
        && !vim_strchr(
            b".,:;!\0".as_ptr() as *const ::core::ffi::c_char,
            *ptr.offset(len.wrapping_sub(1 as size_t) as isize) as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        && *ptr.offset(len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
            != '.' as ::core::ffi::c_int
    {
        len = len.wrapping_sub(1);
    }
    if !file_lnum.is_null() {
        let mut match_text: *const ::core::ffi::c_char =
            b" line \0".as_ptr() as *const ::core::ffi::c_char;
        let mut match_textlen: size_t = 6 as size_t;
        let mut p: *mut ::core::ffi::c_char = ptr.offset(len as isize);
        if strncmp(p, match_text, match_textlen) == 0 as ::core::ffi::c_int {
            p = p.offset(match_textlen as isize);
        } else {
            match_text = gettext(&raw const line_msg as *const ::core::ffi::c_char);
            match_textlen = strlen(match_text);
            if strncmp(p, match_text, match_textlen) == 0 as ::core::ffi::c_int {
                p = p.offset(match_textlen as isize);
            } else {
                p = skipwhite(p);
            }
        }
        if *p as ::core::ffi::c_int != NUL {
            if *(*__ctype_b_loc()).offset(*p as uint8_t as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            {
                p = p.offset(1);
            }
            p = skipwhite(p);
            if *(*__ctype_b_loc()).offset(*p as uint8_t as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
            {
                *file_lnum =
                    getdigits_long(&raw mut p, false_0 != 0, 0 as ::core::ffi::c_long) as linenr_T;
            }
        }
    }
    return find_file_name_in_path(ptr, len, options, count as ::core::ffi::c_long, rel_fname);
}
unsafe extern "C" fn eval_includeexpr(
    ptr: *const ::core::ffi::c_char,
    len: size_t,
) -> *mut ::core::ffi::c_char {
    let save_sctx: sctx_T = current_sctx;
    set_vim_var_string(VV_FNAME, ptr, len as ptrdiff_t);
    current_sctx = (*curbuf).b_p_script_ctx[kBufOptIncludeexpr as ::core::ffi::c_int as usize];
    let mut res: *mut ::core::ffi::c_char = eval_to_string_safe(
        (*curbuf).b_p_inex,
        was_set_insecurely(curwin, kOptIncludeexpr, OPT_LOCAL as ::core::ffi::c_int) != 0,
        true_0 != 0,
    );
    set_vim_var_string(
        VV_FNAME,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ptrdiff_t,
    );
    current_sctx = save_sctx;
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn find_file_name_in_path(
    mut ptr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut options: ::core::ffi::c_int,
    mut count: ::core::ffi::c_long,
    mut rel_fname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut file_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if len == 0 as size_t {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if options & FNAME_HYP as ::core::ffi::c_int != 0
        && len > 6 as size_t
        && strncmp(
            ptr,
            b"file:/\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
        && !vim_ispathsep(*ptr.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
    {
        let mut off: size_t = (if path_has_drive_letter(
            ptr.offset(6 as ::core::ffi::c_int as isize),
            len.wrapping_sub(6 as size_t),
        ) as ::core::ffi::c_int
            != 0
        {
            6 as ::core::ffi::c_int
        } else {
            5 as ::core::ffi::c_int
        }) as size_t;
        ptr = ptr.offset(off as isize);
        len = len.wrapping_sub(off);
    }
    if options & FNAME_INCL as ::core::ffi::c_int != 0
        && *(*curbuf).b_p_inex as ::core::ffi::c_int != NUL
    {
        tofree = eval_includeexpr(ptr, len);
        if !tofree.is_null() {
            ptr = tofree;
            len = strlen(ptr);
        }
    }
    if options & FNAME_EXP as ::core::ffi::c_int != 0 {
        let mut file_to_find: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut search_ctx: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        file_name = find_file_in_path(
            ptr,
            len,
            options & !(FNAME_MESS as ::core::ffi::c_int),
            true_0,
            rel_fname,
            &raw mut file_to_find,
            &raw mut search_ctx,
        );
        if file_name.is_null()
            && options & FNAME_INCL as ::core::ffi::c_int == 0
            && *(*curbuf).b_p_inex as ::core::ffi::c_int != NUL
        {
            tofree = eval_includeexpr(ptr, len);
            if !tofree.is_null() {
                ptr = tofree;
                len = strlen(ptr);
                file_name = find_file_in_path(
                    ptr,
                    len,
                    options & !(FNAME_MESS as ::core::ffi::c_int),
                    true_0,
                    rel_fname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
            }
        }
        if file_name.is_null() && options & FNAME_MESS as ::core::ffi::c_int != 0 {
            let mut c: ::core::ffi::c_char = *ptr.offset(len as isize);
            *ptr.offset(len as isize) = NUL as ::core::ffi::c_char;
            semsg(
                gettext(b"E447: Can't find file \"%s\" in path\0".as_ptr()
                    as *const ::core::ffi::c_char),
                ptr,
            );
            *ptr.offset(len as isize) = c;
        }
        while !file_name.is_null() && {
            count -= 1;
            count > 0 as ::core::ffi::c_long
        } {
            xfree(file_name as *mut ::core::ffi::c_void);
            file_name = find_file_in_path(
                ptr,
                len,
                options,
                false_0,
                rel_fname,
                &raw mut file_to_find,
                &raw mut search_ctx,
            );
        }
        xfree(file_to_find as *mut ::core::ffi::c_void);
        vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
    } else {
        file_name = xstrnsave(ptr, len);
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    return file_name;
}
#[no_mangle]
pub unsafe extern "C" fn do_autocmd_dirchanged(
    mut new_dir: *mut ::core::ffi::c_char,
    mut scope: CdScope,
    mut cause: CdCause,
    mut pre: bool,
) {
    static mut recursive: bool = false_0 != 0;
    let mut event: event_T = (if pre as ::core::ffi::c_int != 0 {
        EVENT_DIRCHANGEDPRE as ::core::ffi::c_int
    } else {
        EVENT_DIRCHANGED as ::core::ffi::c_int
    }) as event_T;
    if recursive as ::core::ffi::c_int != 0 || !has_event(event) {
        return;
    }
    recursive = true_0 != 0;
    let mut save_v_event: save_v_event_T = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            }; 16],
        },
    };
    let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
    let mut buf: [::core::ffi::c_char; 8] = [0; 8];
    match scope as ::core::ffi::c_int {
        2 => {
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                b"global\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        1 => {
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                b"tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        0 => {
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                b"window\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        -1 => {
            abort();
        }
        _ => {}
    }
    if pre {
        tv_dict_add_str(
            dict,
            b"directory\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            new_dir,
        );
    } else {
        tv_dict_add_str(
            dict,
            b"cwd\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            new_dir,
        );
    }
    tv_dict_add_str(
        dict,
        b"scope\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    tv_dict_add_bool(
        dict,
        b"changed_window\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 15]>().wrapping_sub(1 as size_t),
        (cause as ::core::ffi::c_int == kCdCauseWindow as ::core::ffi::c_int) as ::core::ffi::c_int
            as BoolVarValue,
    );
    tv_dict_set_keys_readonly(dict);
    match cause as ::core::ffi::c_int {
        2 => {
            snprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                b"auto\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        -1 => {
            abort();
        }
        0 | 1 | _ => {}
    }
    apply_autocmds(
        event,
        &raw mut buf as *mut ::core::ffi::c_char,
        new_dir,
        false_0 != 0,
        curbuf,
    );
    restore_v_event(dict, &raw mut save_v_event);
    recursive = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn vim_chdirfile(
    mut fname: *mut ::core::ffi::c_char,
    mut cause: CdCause,
) -> ::core::ffi::c_int {
    let mut dir: [::core::ffi::c_char; 4096] = [0; 4096];
    xstrlcpy(
        &raw mut dir as *mut ::core::ffi::c_char,
        fname,
        MAXPATHL as size_t,
    );
    *path_tail_with_sep(&raw mut dir as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
    if os_dirname(
        &raw mut NameBuff as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
    ) != OK
    {
        NameBuff[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    }
    if pathcmp(
        &raw mut dir as *mut ::core::ffi::c_char,
        &raw mut NameBuff as *mut ::core::ffi::c_char,
        -1 as ::core::ffi::c_int,
    ) == 0 as ::core::ffi::c_int
    {
        return OK;
    }
    if cause as ::core::ffi::c_int != kCdCauseOther as ::core::ffi::c_int {
        do_autocmd_dirchanged(
            &raw mut dir as *mut ::core::ffi::c_char,
            kCdScopeWindow,
            cause,
            true_0 != 0,
        );
    }
    if os_chdir(&raw mut dir as *mut ::core::ffi::c_char) != 0 as ::core::ffi::c_int {
        return FAIL;
    }
    if cause as ::core::ffi::c_int != kCdCauseOther as ::core::ffi::c_int {
        do_autocmd_dirchanged(
            &raw mut dir as *mut ::core::ffi::c_char,
            kCdScopeWindow,
            cause,
            false_0 != 0,
        );
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn vim_chdir(mut new_dir: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut file_to_find: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut search_ctx: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dir_name: *mut ::core::ffi::c_char = find_directory_in_path(
        new_dir,
        strlen(new_dir),
        FNAME_MESS as ::core::ffi::c_int,
        (*curbuf).b_ffname,
        &raw mut file_to_find,
        &raw mut search_ctx,
    );
    xfree(file_to_find as *mut ::core::ffi::c_void);
    vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
    if dir_name.is_null() {
        return -1 as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = os_chdir(dir_name);
    xfree(dir_name as *mut ::core::ffi::c_void);
    return r;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
