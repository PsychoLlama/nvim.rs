use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type MsgpackRpcRequestHandler;
    pub type terminal;
    pub type regprog;
    pub type undo_object;
    pub type qf_info_S;
    fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xmallocz(size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn is_aucmd_win(win: *mut win_T) -> bool;
    fn block_autocmds();
    fn unblock_autocmds();
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn do_autochdir();
    fn bt_quickfix(buf: *const buf_T) -> bool;
    fn bt_terminal(buf: *const buf_T) -> bool;
    fn check_pos(buf: *mut buf_T, pos: *mut pos_T);
    fn check_cursor(wp: *mut win_T);
    static e_invexpr2: [::core::ffi::c_char; 0];
    static e_auabort: [::core::ffi::c_char; 0];
    static e_invalwindow: [::core::ffi::c_char; 0];
    fn execute_common(argvars: *mut typval_T, rettv: *mut typval_T, arg_off: ::core::ffi::c_int);
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_list(l: *mut list_T, itemlist: *mut list_T);
    fn tv_list_append_dict(l: *mut list_T, dict: *mut dict_T);
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn tv_list_append_number(l: *mut list_T, n: varnumber_T);
    fn tv_dict_alloc() -> *mut dict_T;
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_get_number(d: *const dict_T, key: *const ::core::ffi::c_char) -> varnumber_T;
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
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_dict_alloc_ret(ret_tv: *mut typval_T);
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_check_for_nonnull_dict_arg(
        args: *const typval_T,
        idx: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn tv_get_string_chk(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
    fn text_or_buf_locked() -> bool;
    static firstwin: GlobalCell<*mut win_T>;
    static lastwin: GlobalCell<*mut win_T>;
    static prevwin: GlobalCell<*mut win_T>;
    static curwin: GlobalCell<*mut win_T>;
    static first_tabpage: GlobalCell<*mut tabpage_T>;
    static curtab: GlobalCell<*mut tabpage_T>;
    static lastused_tabpage: GlobalCell<*mut tabpage_T>;
    static curbuf: GlobalCell<*mut buf_T>;
    static VIsual: GlobalCell<pos_T>;
    static VIsual_active: GlobalCell<bool>;
    static cmdwin_type: GlobalCell<::core::ffi::c_int>;
    static cmdwin_win: GlobalCell<*mut win_T>;
    fn update_curswant();
    fn changed_window_setting(wp: *mut win_T);
    fn set_topline(wp: *mut win_T, lnum: linenr_T);
    fn validate_botline_win(wp: *mut win_T);
    fn validate_cursor(wp: *mut win_T);
    fn win_col_off(wp: *mut win_T) -> ::core::ffi::c_int;
    fn check_topfill(wp: *mut win_T, down: bool);
    static p_acd: GlobalCell<::core::ffi::c_int>;
    fn end_visual_mode();
    fn vim_snprintf_safelen(
        str: *mut ::core::ffi::c_char,
        str_m: size_t,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> size_t;
    fn os_chdir(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_dirname(buf: *mut ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn check_split_disallowed(wp: *const win_T) -> ::core::ffi::c_int;
    fn win_valid(win: *const win_T) -> bool;
    fn win_splitmove(
        wp: *mut win_T,
        size: ::core::ffi::c_int,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn unuse_tabpage(tp: *mut tabpage_T);
    fn use_tabpage(tp: *mut tabpage_T);
    fn valid_tabpage(tpc: *mut tabpage_T) -> bool;
    fn find_tabpage(n: ::core::ffi::c_int) -> *mut tabpage_T;
    fn tabpage_index(ftp: *mut tabpage_T) -> ::core::ffi::c_int;
    fn goto_tabpage_tp(
        tp: *mut tabpage_T,
        trigger_enter_autocmds: bool,
        trigger_leave_autocmds: bool,
    );
    fn goto_tabpage_win(tp: *mut tabpage_T, wp: *mut win_T);
    fn win_goto(wp: *mut win_T);
    fn win_vert_neighbor(
        tp: *mut tabpage_T,
        wp: *mut win_T,
        up: bool,
        count: ::core::ffi::c_int,
    ) -> *mut win_T;
    fn win_horz_neighbor(
        tp: *mut tabpage_T,
        wp: *mut win_T,
        left: bool,
        count: ::core::ffi::c_int,
    ) -> *mut win_T;
    fn win_drag_status_line(dragwin: *mut win_T, offset: ::core::ffi::c_int);
    fn win_drag_vsep_line(dragwin: *mut win_T, offset: ::core::ffi::c_int);
    fn win_new_height(wp: *mut win_T, height: ::core::ffi::c_int);
    fn win_new_width(wp: *mut win_T, width: ::core::ffi::c_int);
    fn win_get_tabwin(id: handle_T, tabnr: *mut ::core::ffi::c_int, winnr: *mut ::core::ffi::c_int);
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
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type ptrdiff_t = isize;
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
pub type handle_T = ::core::ffi::c_int;
pub type LuaRef = ::core::ffi::c_int;
pub type float_T = ::core::ffi::c_double;
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
    pub b_wininfo: C2Rust_Unnamed_10,
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
    pub b_signcols: C2Rust_Unnamed_2,
    pub terminal: *mut Terminal,
    pub additional_data: *mut AdditionalData,
    pub b_mapped_ctrl_c: ::core::ffi::c_int,
    pub b_marktree: [MarkTree; 1],
    pub b_extmark_ns: [Map_uint32_t_uint32_t; 1],
    pub b_prev_line_count: ::core::ffi::c_int,
    pub update_channels: C2Rust_Unnamed_0,
    pub update_callbacks: C2Rust_Unnamed,
    pub update_need_codepoints: bool,
    pub deleted_bytes: size_t,
    pub deleted_bytes2: size_t,
    pub deleted_codepoints: size_t,
    pub deleted_codeunits: size_t,
    pub flush_count: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
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
pub struct C2Rust_Unnamed_0 {
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
    pub data: C2Rust_Unnamed_1,
    pub next: *mut DecorVirtText,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_1 {
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
pub struct C2Rust_Unnamed_2 {
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
    pub sst_union: C2Rust_Unnamed_3,
    pub sst_next_flags: ::core::ffi::c_int,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
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
    pub data: C2Rust_Unnamed_4,
    pub type_0: CallbackType,
}
pub type CallbackType = ::core::ffi::c_uint;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
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
    pub fc_fixvar: [C2Rust_Unnamed_5; 12],
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
pub struct C2Rust_Unnamed_5 {
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
    pub uh_next: C2Rust_Unnamed_9,
    pub uh_prev: C2Rust_Unnamed_8,
    pub uh_alt_next: C2Rust_Unnamed_7,
    pub uh_alt_prev: C2Rust_Unnamed_6,
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
pub union C2Rust_Unnamed_6 {
    pub ptr: *mut u_header_T,
    pub seq: ::core::ffi::c_int,
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
pub struct C2Rust_Unnamed_10 {
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
    pub type_0: C2Rust_Unnamed_11,
    pub tabnr: ::core::ffi::c_int,
    pub func: *mut ::core::ffi::c_char,
}
pub type C2Rust_Unnamed_11 = ::core::ffi::c_uint;
pub const kStlClickFuncRun: C2Rust_Unnamed_11 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_11 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_11 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_11 = 0;
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
pub struct switchwin_T {
    pub sw_curwin: *mut win_T,
    pub sw_curtab: *mut tabpage_T,
    pub sw_same_win: bool,
    pub sw_visual_active: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct win_execute_T {
    pub wp: *mut win_T,
    pub curpos: pos_T,
    pub cwd: [::core::ffi::c_char; 4096],
    pub cwd_status: ::core::ffi::c_int,
    pub apply_acd: bool,
    pub save_sfname: *mut ::core::ffi::c_char,
    pub switchwin: switchwin_T,
}
pub const LOWEST_WIN_ID: C2Rust_Unnamed_13 = 1000;
pub const WSP_ABOVE: C2Rust_Unnamed_12 = 128;
pub const WSP_BELOW: C2Rust_Unnamed_12 = 64;
pub const WSP_VERT: C2Rust_Unnamed_12 = 2;
pub type C2Rust_Unnamed_12 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_12 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_12 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_12 = 256;
pub const WSP_HELP: C2Rust_Unnamed_12 = 32;
pub const WSP_BOT: C2Rust_Unnamed_12 = 16;
pub const WSP_TOP: C2Rust_Unnamed_12 = 8;
pub const WSP_HOR: C2Rust_Unnamed_12 = 4;
pub const WSP_ROOM: C2Rust_Unnamed_12 = 1;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn equalpos(mut a: pos_T, mut b: pos_T) -> bool {
    return a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd;
}
static e_cannot_resize_window_in_another_tab_page: GlobalCell<[::core::ffi::c_char; 50]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 50], [::core::ffi::c_char; 50]>(
            *b"E1308: Cannot resize a window in another tab page\0",
        )
    });
#[no_mangle]
pub unsafe extern "C" fn win_has_winnr(mut wp: *mut win_T, mut tp: *mut tabpage_T) -> bool {
    return wp
        == (if tp == curtab.get() {
            curwin.get()
        } else {
            (*tp).tp_curwin
        })
        || !(*wp).w_config.hide && (*wp).w_config.focusable as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn win_getid(mut argvars: *mut typval_T) -> ::core::ffi::c_int {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (*curwin.get()).handle as ::core::ffi::c_int;
    }
    let mut winnr: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if winnr <= 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tp = curtab.get();
        wp = firstwin.get();
    } else {
        let mut tabnr: ::core::ffi::c_int =
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        let mut tp2: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp2.is_null() {
            tabnr -= 1;
            if tabnr == 0 as ::core::ffi::c_int {
                tp = tp2 as *mut tabpage_T;
                break;
            } else {
                tp2 = (*tp2).tp_next as *mut tabpage_T;
            }
        }
        if tp.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        if tp == curtab.get() {
            wp = firstwin.get();
        } else {
            wp = (*tp).tp_firstwin;
        }
    }
    while !wp.is_null() {
        winnr -= win_has_winnr(wp, tp) as ::core::ffi::c_int;
        if winnr == 0 as ::core::ffi::c_int {
            return (*wp).handle as ::core::ffi::c_int;
        }
        wp = (*wp).w_next;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn win_id2tabwin(argvars: *mut typval_T, rettv: *mut typval_T) {
    let mut id: handle_T =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as handle_T;
    let mut winnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut tabnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    win_get_tabwin(id, &raw mut tabnr, &raw mut winnr);
    let list: *mut list_T = tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    tv_list_append_number(list, tabnr as varnumber_T);
    tv_list_append_number(list, winnr as varnumber_T);
}
#[no_mangle]
pub unsafe extern "C" fn win_id2wp(mut id: ::core::ffi::c_int) -> *mut win_T {
    return win_id2wp_tp(id, ::core::ptr::null_mut::<*mut tabpage_T>());
}
#[no_mangle]
pub unsafe extern "C" fn win_id2wp_tp(
    mut id: ::core::ffi::c_int,
    mut tpp: *mut *mut tabpage_T,
) -> *mut win_T {
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).handle == id {
                if !tpp.is_null() {
                    *tpp = tp as *mut tabpage_T;
                }
                return wp;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    return ::core::ptr::null_mut::<win_T>();
}
unsafe extern "C" fn win_id2win(mut argvars: *mut typval_T) -> ::core::ffi::c_int {
    let mut nr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        if (*wp).handle == id {
            return if win_has_winnr(wp, curtab.get()) as ::core::ffi::c_int != 0 {
                nr
            } else {
                0 as ::core::ffi::c_int
            };
        }
        nr += win_has_winnr(wp, curtab.get()) as ::core::ffi::c_int;
        wp = (*wp).w_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn win_findbuf(mut argvars: *mut typval_T, mut list: *mut list_T) {
    let mut bufnr: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*(*wp).w_buffer).handle == bufnr {
                tv_list_append_number(list, (*wp).handle as varnumber_T);
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn find_win_by_nr(
    mut vp: *mut typval_T,
    mut tp: *mut tabpage_T,
) -> *mut win_T {
    let mut nr: ::core::ffi::c_int =
        tv_get_number_chk(vp, ::core::ptr::null_mut::<bool>()) as ::core::ffi::c_int;
    if nr < 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<win_T>();
    }
    if nr == 0 as ::core::ffi::c_int {
        return curwin.get();
    }
    if tp.is_null() {
        tp = curtab.get();
    }
    let mut wp: *mut win_T = if tp == curtab.get() {
        firstwin.get()
    } else {
        (*tp).tp_firstwin
    };
    while !wp.is_null() {
        if nr >= LOWEST_WIN_ID as ::core::ffi::c_int {
            if (*wp).handle == nr {
                return wp;
            }
        } else {
            nr -= 1;
            if nr <= 0 as ::core::ffi::c_int {
                return wp;
            }
        }
        wp = (*wp).w_next;
    }
    return ::core::ptr::null_mut::<win_T>();
}
#[no_mangle]
pub unsafe extern "C" fn find_win_by_nr_or_id(mut vp: *mut typval_T) -> *mut win_T {
    let mut nr: ::core::ffi::c_int =
        tv_get_number_chk(vp, ::core::ptr::null_mut::<bool>()) as ::core::ffi::c_int;
    if nr >= LOWEST_WIN_ID as ::core::ffi::c_int {
        return win_id2wp(tv_get_number(vp) as ::core::ffi::c_int);
    }
    return find_win_by_nr(vp, ::core::ptr::null_mut::<tabpage_T>());
}
#[no_mangle]
pub unsafe extern "C" fn find_tabwin(mut wvp: *mut typval_T, mut tvp: *mut typval_T) -> *mut win_T {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    if (*wvp).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*tvp).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut n: ::core::ffi::c_int = tv_get_number(tvp) as ::core::ffi::c_int;
            if n >= 0 as ::core::ffi::c_int {
                tp = find_tabpage(n);
            }
        } else {
            tp = curtab.get();
        }
        if !tp.is_null() {
            wp = find_win_by_nr(wvp, tp);
        }
    } else {
        wp = curwin.get();
    }
    return wp;
}
unsafe extern "C" fn get_framelayout(mut fr: *const frame_T, mut l: *mut list_T, mut outer: bool) {
    if fr.is_null() {
        return;
    }
    let mut fr_list: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if outer {
        fr_list = l;
    } else {
        fr_list = tv_list_alloc(2 as ptrdiff_t);
        tv_list_append_list(l, fr_list);
    }
    if (*fr).fr_layout as ::core::ffi::c_int == FR_LEAF {
        if !(*fr).fr_win.is_null() {
            tv_list_append_string(
                fr_list,
                b"leaf\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
            tv_list_append_number(fr_list, (*(*fr).fr_win).handle as varnumber_T);
        }
    } else {
        if (*fr).fr_layout as ::core::ffi::c_int == FR_ROW {
            tv_list_append_string(
                fr_list,
                b"row\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        } else {
            tv_list_append_string(
                fr_list,
                b"col\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                    as ssize_t,
            );
        }
        let win_list: *mut list_T =
            tv_list_alloc(kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
        tv_list_append_list(fr_list, win_list);
        let mut child: *const frame_T = (*fr).fr_child;
        while !child.is_null() {
            get_framelayout(child, win_list, false_0 != 0);
            child = (*child).fr_next;
        }
    };
}
unsafe extern "C" fn get_winnr(
    mut tp: *mut tabpage_T,
    mut argvar: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut nr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut twin: *mut win_T = if tp == curtab.get() {
        curwin.get()
    } else {
        (*tp).tp_curwin
    };
    if (*argvar).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut invalid_arg: bool = false_0 != 0;
        let arg: *const ::core::ffi::c_char = tv_get_string_chk(argvar);
        if arg.is_null() {
            nr = 0 as ::core::ffi::c_int;
        } else if strcmp(arg, b"$\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            twin = if tp == curtab.get() {
                lastwin.get()
            } else {
                (*tp).tp_lastwin
            };
        } else if strcmp(arg, b"#\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            twin = if tp == curtab.get() {
                prevwin.get()
            } else {
                (*tp).tp_prevwin
            };
            if twin.is_null() {
                nr = 0 as ::core::ffi::c_int;
            }
        } else {
            let mut endp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut count: ::core::ffi::c_int =
                strtol(arg, &raw mut endp, 10 as ::core::ffi::c_int) as ::core::ffi::c_int;
            if count <= 0 as ::core::ffi::c_int {
                count = 1 as ::core::ffi::c_int;
            }
            if !endp.is_null() && *endp as ::core::ffi::c_int != NUL {
                if strequal(endp, b"j\0".as_ptr() as *const ::core::ffi::c_char) {
                    twin = win_vert_neighbor(tp, twin, false_0 != 0, count);
                } else if strequal(endp, b"k\0".as_ptr() as *const ::core::ffi::c_char) {
                    twin = win_vert_neighbor(tp, twin, true_0 != 0, count);
                } else if strequal(endp, b"h\0".as_ptr() as *const ::core::ffi::c_char) {
                    twin = win_horz_neighbor(tp, twin, true_0 != 0, count);
                } else if strequal(endp, b"l\0".as_ptr() as *const ::core::ffi::c_char) {
                    twin = win_horz_neighbor(tp, twin, false_0 != 0, count);
                } else {
                    invalid_arg = true_0 != 0;
                }
            } else {
                invalid_arg = true_0 != 0;
            }
        }
        if invalid_arg {
            semsg(
                gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                arg,
            );
            nr = 0 as ::core::ffi::c_int;
        }
    } else if !win_has_winnr(twin, tp) {
        nr = 0 as ::core::ffi::c_int;
    }
    if nr <= 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    nr = 0 as ::core::ffi::c_int;
    let mut wp: *mut win_T = if tp == curtab.get() {
        firstwin.get()
    } else {
        (*tp).tp_firstwin
    };
    while !wp.is_null() {
        nr += win_has_winnr(wp, tp) as ::core::ffi::c_int;
        if wp == twin {
            break;
        }
        wp = (*wp).w_next;
    }
    if wp.is_null() {
        nr = 0 as ::core::ffi::c_int;
    }
    return nr;
}
unsafe extern "C" fn get_win_info(
    mut wp: *mut win_T,
    mut tpnr: int16_t,
    mut winnr: int16_t,
) -> *mut dict_T {
    let dict: *mut dict_T = tv_dict_alloc();
    validate_botline_win(wp);
    tv_dict_add_nr(
        dict,
        b"tabnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        tpnr as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"winnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        winnr as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"winid\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*wp).handle as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"height\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*wp).w_view_height as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"status_height\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
        (*wp).w_status_height as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"winrow\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        ((*wp).w_winrow + 1 as ::core::ffi::c_int) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"topline\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*wp).w_topline as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"botline\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        ((*wp).w_botline - 1 as linenr_T) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"leftcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*wp).w_leftcol as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"winbar\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*wp).w_winbar_height as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"width\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*wp).w_view_width as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        (*(*wp).w_buffer).handle as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"wincol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        ((*wp).w_wincol + 1 as ::core::ffi::c_int) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"textoff\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        win_col_off(wp) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"terminal\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        bt_terminal((*wp).w_buffer) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"quickfix\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        bt_quickfix((*wp).w_buffer) as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"loclist\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && !(*wp).w_llist_ref.is_null())
            as ::core::ffi::c_int as varnumber_T,
    );
    tv_dict_add_dict(
        dict,
        b"variables\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (*wp).w_vars,
    );
    return dict;
}
unsafe extern "C" fn get_tabpage_info(
    mut tp: *mut tabpage_T,
    mut tp_idx: ::core::ffi::c_int,
) -> *mut dict_T {
    let dict: *mut dict_T = tv_dict_alloc();
    tv_dict_add_nr(
        dict,
        b"tabnr\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        tp_idx as varnumber_T,
    );
    let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    let mut wp: *mut win_T = if tp == curtab.get() {
        firstwin.get()
    } else {
        (*tp).tp_firstwin
    };
    while !wp.is_null() {
        tv_list_append_number(l, (*wp).handle as varnumber_T);
        wp = (*wp).w_next;
    }
    tv_dict_add_list(
        dict,
        b"windows\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        l,
    );
    tv_dict_add_dict(
        dict,
        b"variables\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (*tp).tp_vars,
    );
    return dict;
}
#[no_mangle]
pub unsafe extern "C" fn f_gettabinfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut tparg: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    tv_list_alloc_ret(
        rettv,
        (if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            1 as ::core::ffi::c_int
        } else {
            kListLenMayKnow as ::core::ffi::c_int
        }) as ptrdiff_t,
    );
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tparg = find_tabpage(tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) as ::core::ffi::c_int);
        if tparg.is_null() {
            return;
        }
    }
    let mut tpnr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        tpnr += 1;
        if !(!tparg.is_null() && tp != tparg) {
            let d: *mut dict_T = get_tabpage_info(tp as *mut tabpage_T, tpnr);
            tv_list_append_dict((*rettv).vval.v_list, d);
            if !tparg.is_null() {
                return;
            }
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_getwininfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wparg: *mut win_T = ::core::ptr::null_mut::<win_T>();
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        wparg = win_id2wp(
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
        );
        if wparg.is_null() {
            return;
        }
    }
    let mut tabnr: int16_t = 0 as int16_t;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        tabnr += 1;
        let mut winnr: int16_t = 0 as int16_t;
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            winnr = (winnr as ::core::ffi::c_int
                + win_has_winnr(wp, tp as *mut tabpage_T) as ::core::ffi::c_int)
                as int16_t;
            if !(!wparg.is_null() && wp != wparg) {
                let d: *mut dict_T = get_win_info(
                    wp,
                    tabnr,
                    (if win_has_winnr(wp, tp as *mut tabpage_T) as ::core::ffi::c_int != 0 {
                        winnr as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as int16_t,
                );
                tv_list_append_dict((*rettv).vval.v_list, d);
                if !wparg.is_null() {
                    return;
                }
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_getwinpos(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    tv_list_append_number((*rettv).vval.v_list, -1 as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, -1 as varnumber_T);
}
#[no_mangle]
pub unsafe extern "C" fn f_getwinposx(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_getwinposy(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_tabpagenr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut nr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let arg: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        nr = 0 as ::core::ffi::c_int;
        if !arg.is_null() {
            if strcmp(arg, b"$\0".as_ptr() as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
            {
                nr = tabpage_index(::core::ptr::null_mut::<tabpage_T>()) - 1 as ::core::ffi::c_int;
            } else if strcmp(arg, b"#\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                nr = if valid_tabpage(lastused_tabpage.get()) as ::core::ffi::c_int != 0 {
                    tabpage_index(lastused_tabpage.get())
                } else {
                    0 as ::core::ffi::c_int
                };
            } else {
                semsg(
                    gettext(&raw const e_invexpr2 as *const ::core::ffi::c_char),
                    arg,
                );
            }
        }
    } else {
        nr = tabpage_index(curtab.get());
    }
    (*rettv).vval.v_number = nr as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_tabpagewinnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut nr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let tp: *mut tabpage_T = find_tabpage(tv_get_number(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ) as ::core::ffi::c_int);
    if tp.is_null() {
        nr = 0 as ::core::ffi::c_int;
    } else {
        nr = get_winnr(tp, argvars.offset(1 as ::core::ffi::c_int as isize));
    }
    (*rettv).vval.v_number = nr as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn win_execute_before(
    mut args: *mut win_execute_T,
    mut wp: *mut win_T,
    mut tp: *mut tabpage_T,
) -> bool {
    (*args).wp = wp;
    (*args).curpos = (*wp).w_cursor;
    (*args).cwd_status = FAIL;
    (*args).apply_acd = false_0 != 0;
    (*args).save_sfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if curwin.get() != wp
        && (!(*curwin.get()).w_localdir.is_null()
            || !(*wp).w_localdir.is_null()
            || curtab.get() != tp
                && (!(*curtab.get()).tp_localdir.is_null() || !(*tp).tp_localdir.is_null())
            || p_acd.get() != 0)
    {
        (*args).cwd_status = os_dirname(
            &raw mut (*args).cwd as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
        );
    }
    if (*args).cwd_status == OK && p_acd.get() != 0 {
        if !(*curbuf.get()).b_sfname.is_null()
            && (*curbuf.get()).b_fname == (*curbuf.get()).b_sfname
        {
            (*args).save_sfname = xstrdup((*curbuf.get()).b_sfname);
        }
        do_autochdir();
        let mut autocwd: [::core::ffi::c_char; 4096] = [0; 4096];
        if os_dirname(
            &raw mut autocwd as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
        ) == OK
        {
            (*args).apply_acd = strcmp(
                &raw mut (*args).cwd as *mut ::core::ffi::c_char,
                &raw mut autocwd as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int;
        }
    }
    if switch_win_noblock(&raw mut (*args).switchwin, wp, tp, true_0 != 0) == OK {
        check_cursor(curwin.get());
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn win_execute_after(mut args: *mut win_execute_T) {
    restore_win_noblock(&raw mut (*args).switchwin, true_0 != 0);
    if (*args).apply_acd {
        xfree((*args).save_sfname as *mut ::core::ffi::c_void);
        do_autochdir();
    } else if (*args).cwd_status == OK {
        os_chdir(&raw mut (*args).cwd as *mut ::core::ffi::c_char);
        if !(*args).save_sfname.is_null() {
            xfree((*curbuf.get()).b_sfname as *mut ::core::ffi::c_void);
            (*curbuf.get()).b_sfname = (*args).save_sfname;
            (*curbuf.get()).b_fname = (*curbuf.get()).b_sfname;
        }
    }
    if win_valid((*args).wp) as ::core::ffi::c_int != 0
        && !equalpos((*args).curpos, (*(*args).wp).w_cursor)
    {
        (*(*args).wp).w_redr_status = true_0 != 0;
    }
    check_cursor(curwin.get());
    if VIsual_active.get() {
        check_pos(curbuf.get(), VIsual.ptr());
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_win_execute(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = tv_get_number(argvars) as ::core::ffi::c_int;
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut wp: *mut win_T = win_id2wp_tp(id, &raw mut tp);
    if wp.is_null() || tp.is_null() {
        return;
    }
    let mut win_execute_args: win_execute_T = win_execute_T {
        wp: ::core::ptr::null_mut::<win_T>(),
        curpos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        cwd: [0; 4096],
        cwd_status: 0,
        apply_acd: false,
        save_sfname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        switchwin: switchwin_T {
            sw_curwin: ::core::ptr::null_mut::<win_T>(),
            sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
            sw_same_win: false,
            sw_visual_active: false,
        },
    };
    if win_execute_before(&raw mut win_execute_args, wp, tp) {
        execute_common(argvars, rettv, 1 as ::core::ffi::c_int);
    }
    win_execute_after(&raw mut win_execute_args);
}
#[no_mangle]
pub unsafe extern "C" fn f_win_findbuf(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    win_findbuf(argvars, (*rettv).vval.v_list);
}
#[no_mangle]
pub unsafe extern "C" fn f_win_getid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = win_getid(argvars) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_win_gotoid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    if (*curwin.get()).handle == id {
        (*rettv).vval.v_number = 1 as varnumber_T;
        return;
    }
    if text_or_buf_locked() {
        return;
    }
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).handle == id {
                if VIsual_active.get() as ::core::ffi::c_int != 0 && (*wp).w_buffer != curbuf.get()
                {
                    end_visual_mode();
                }
                goto_tabpage_win(tp as *mut tabpage_T, wp);
                (*rettv).vval.v_number = 1 as varnumber_T;
                return;
            }
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_win_id2tabwin(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    win_id2tabwin(argvars, rettv);
}
#[no_mangle]
pub unsafe extern "C" fn f_win_id2win(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = win_id2win(argvars) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_win_move_separator(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = false_0 as varnumber_T;
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() || (*wp).w_floating as ::core::ffi::c_int != 0 {
        return;
    }
    if !win_valid(wp) {
        emsg(gettext(
            (e_cannot_resize_window_in_another_tab_page.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut offset: ::core::ffi::c_int =
        tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    win_drag_vsep_line(wp, offset);
    (*rettv).vval.v_number = true_0 as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_win_move_statusline(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut offset: ::core::ffi::c_int = 0;
    (*rettv).vval.v_number = false_0 as varnumber_T;
    wp = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() || (*wp).w_floating as ::core::ffi::c_int != 0 {
        return;
    }
    if !win_valid(wp) {
        emsg(gettext(
            (e_cannot_resize_window_in_another_tab_page.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
        return;
    }
    offset = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    win_drag_status_line(wp, offset);
    (*rettv).vval.v_number = true_0 as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_win_screenpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    let wp: *const win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    tv_list_append_number(
        (*rettv).vval.v_list,
        (if wp.is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*wp).w_winrow + 1 as ::core::ffi::c_int
        }) as varnumber_T,
    );
    tv_list_append_number(
        (*rettv).vval.v_list,
        (if wp.is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*wp).w_wincol + 1 as ::core::ffi::c_int
        }) as varnumber_T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn f_win_splitmove(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut targetwin: *mut win_T =
        find_win_by_nr_or_id(argvars.offset(1 as ::core::ffi::c_int as isize));
    let mut oldwin: *mut win_T = curwin.get();
    (*rettv).vval.v_number = -1 as varnumber_T;
    if wp.is_null()
        || targetwin.is_null()
        || wp == targetwin
        || !win_valid(wp)
        || !win_valid(targetwin)
        || (*targetwin).w_floating as ::core::ffi::c_int != 0
    {
        emsg(gettext(
            &raw const e_invalwindow as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        if tv_check_for_nonnull_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return;
        }
        d = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        if tv_dict_get_number(d, b"vertical\0".as_ptr() as *const ::core::ffi::c_char) != 0 {
            flags |= WSP_VERT as ::core::ffi::c_int;
        }
        di = tv_dict_find(
            d,
            b"rightbelow\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        );
        if !di.is_null() {
            flags |= if tv_get_number(&raw mut (*di).di_tv) != 0 {
                WSP_BELOW as ::core::ffi::c_int
            } else {
                WSP_ABOVE as ::core::ffi::c_int
            };
        }
        size = tv_dict_get_number(d, b"size\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int;
    }
    if is_aucmd_win(wp) as ::core::ffi::c_int != 0
        || text_or_buf_locked() as ::core::ffi::c_int != 0
        || check_split_disallowed(wp) == FAIL
    {
        return;
    }
    if curwin.get() != targetwin {
        win_goto(targetwin);
    }
    if curwin.get() == targetwin && win_valid(wp) as ::core::ffi::c_int != 0 {
        if win_splitmove(wp, size, flags) == OK {
            (*rettv).vval.v_number = 0 as varnumber_T;
        }
    } else {
        emsg(gettext(&raw const e_auabort as *const ::core::ffi::c_char));
    }
    if oldwin != curwin.get() && win_valid(oldwin) as ::core::ffi::c_int != 0 {
        win_goto(oldwin);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_win_gettype(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = curwin.get();
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        wp = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
        if wp.is_null() {
            (*rettv).vval.v_string = xstrdup(b"unknown\0".as_ptr() as *const ::core::ffi::c_char);
            return;
        }
    }
    if is_aucmd_win(wp) {
        (*rettv).vval.v_string = xstrdup(b"autocmd\0".as_ptr() as *const ::core::ffi::c_char);
    } else if (*wp).w_onebuf_opt.wo_pvw != 0 {
        (*rettv).vval.v_string = xstrdup(b"preview\0".as_ptr() as *const ::core::ffi::c_char);
    } else if (*wp).w_floating {
        (*rettv).vval.v_string = xstrdup(b"popup\0".as_ptr() as *const ::core::ffi::c_char);
    } else if wp == cmdwin_win.get() {
        (*rettv).vval.v_string = xstrdup(b"command\0".as_ptr() as *const ::core::ffi::c_char);
    } else if bt_quickfix((*wp).w_buffer) {
        (*rettv).vval.v_string = xstrdup(if !(*wp).w_llist_ref.is_null() {
            b"loclist\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"quickfix\0".as_ptr() as *const ::core::ffi::c_char
        });
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_getcmdwintype(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*rettv).vval.v_string = xmallocz(1 as size_t) as *mut ::core::ffi::c_char;
    *(*rettv)
        .vval
        .v_string
        .offset(0 as ::core::ffi::c_int as isize) = cmdwin_type.get() as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn f_winbufnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = (*(*wp).w_buffer).handle as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_wincol(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    validate_cursor(curwin.get());
    (*rettv).vval.v_number = ((*curwin.get()).w_wcol + 1 as ::core::ffi::c_int) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_winheight(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = (*wp).w_view_height as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_winlayout(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tp = curtab.get();
    } else {
        tp = find_tabpage(
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
        );
        if tp.is_null() {
            return;
        }
    }
    get_framelayout((*tp).tp_topframe, (*rettv).vval.v_list, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_winline(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    validate_cursor(curwin.get());
    (*rettv).vval.v_number = ((*curwin.get()).w_wrow + 1 as ::core::ffi::c_int) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_winnr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = get_winnr(
        curtab.get(),
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_winrestcmd(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut buf: [::core::ffi::c_char; 50] = [0; 50];
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
        70 as ::core::ffi::c_int,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 2 as ::core::ffi::c_int {
        let mut winnr: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if win_has_winnr(wp, curtab.get()) {
                let mut buflen: size_t = vim_snprintf_safelen(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 50]>(),
                    b"%dresize %d|\0".as_ptr() as *const ::core::ffi::c_char,
                    winnr,
                    (*wp).w_height,
                );
                ga_concat_len(
                    &raw mut ga,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    buflen,
                );
                buflen = vim_snprintf_safelen(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 50]>(),
                    b"vert %dresize %d|\0".as_ptr() as *const ::core::ffi::c_char,
                    winnr,
                    (*wp).w_width,
                );
                ga_concat_len(
                    &raw mut ga,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    buflen,
                );
                winnr += 1;
            }
            wp = (*wp).w_next;
        }
        i += 1;
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
    (*rettv).v_type = VAR_STRING;
}
#[no_mangle]
pub unsafe extern "C" fn f_winrestview(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_nonnull_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut dict: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    di = tv_dict_find(
        dict,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_cursor.lnum = tv_get_number(&raw mut (*di).di_tv) as linenr_T;
    }
    di = tv_dict_find(
        dict,
        b"col\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_cursor.col = tv_get_number(&raw mut (*di).di_tv) as colnr_T;
    }
    di = tv_dict_find(
        dict,
        b"coladd\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_cursor.coladd = tv_get_number(&raw mut (*di).di_tv) as colnr_T;
    }
    di = tv_dict_find(
        dict,
        b"curswant\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_curswant = tv_get_number(&raw mut (*di).di_tv) as colnr_T;
        (*curwin.get()).w_set_curswant = false_0;
    }
    di = tv_dict_find(
        dict,
        b"topline\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        set_topline(
            curwin.get(),
            tv_get_number(&raw mut (*di).di_tv) as linenr_T,
        );
    }
    di = tv_dict_find(
        dict,
        b"topfill\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_topfill = tv_get_number(&raw mut (*di).di_tv) as ::core::ffi::c_int;
    }
    di = tv_dict_find(
        dict,
        b"leftcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_leftcol = tv_get_number(&raw mut (*di).di_tv) as colnr_T;
    }
    di = tv_dict_find(
        dict,
        b"skipcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        (*curwin.get()).w_skipcol = tv_get_number(&raw mut (*di).di_tv) as colnr_T;
    }
    check_cursor(curwin.get());
    win_new_height(curwin.get(), (*curwin.get()).w_height);
    win_new_width(curwin.get(), (*curwin.get()).w_width);
    changed_window_setting(curwin.get());
    if (*curwin.get()).w_topline <= 0 as linenr_T {
        (*curwin.get()).w_topline = 1 as ::core::ffi::c_int as linenr_T;
    }
    if (*curwin.get()).w_topline > (*curbuf.get()).b_ml.ml_line_count {
        (*curwin.get()).w_topline = (*curbuf.get()).b_ml.ml_line_count;
    }
    check_topfill(curwin.get(), true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn f_winsaveview(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let mut dict: *mut dict_T = (*rettv).vval.v_dict;
    tv_dict_add_nr(
        dict,
        b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_cursor.lnum as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"col\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_cursor.col as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"coladd\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_cursor.coladd as varnumber_T,
    );
    update_curswant();
    tv_dict_add_nr(
        dict,
        b"curswant\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_curswant as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"topline\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_topline as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"topfill\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_topfill as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"leftcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_leftcol as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"skipcol\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*curwin.get()).w_skipcol as varnumber_T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn f_winwidth(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
    if wp.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = (*wp).w_view_width as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn switch_win(
    mut switchwin: *mut switchwin_T,
    mut win: *mut win_T,
    mut tp: *mut tabpage_T,
    mut no_display: bool,
) -> ::core::ffi::c_int {
    block_autocmds();
    return switch_win_noblock(switchwin, win, tp, no_display);
}
#[no_mangle]
pub unsafe extern "C" fn switch_win_noblock(
    mut switchwin: *mut switchwin_T,
    mut win: *mut win_T,
    mut tp: *mut tabpage_T,
    mut no_display: bool,
) -> ::core::ffi::c_int {
    memset(
        switchwin as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<switchwin_T>(),
    );
    (*switchwin).sw_curwin = curwin.get();
    if win == curwin.get() {
        (*switchwin).sw_same_win = true_0 != 0;
    } else {
        (*switchwin).sw_visual_active = VIsual_active.get();
        VIsual_active.set(false_0 != 0);
    }
    if !tp.is_null() {
        (*switchwin).sw_curtab = curtab.get();
        if no_display {
            unuse_tabpage(curtab.get());
            use_tabpage(tp);
        } else {
            goto_tabpage_tp(tp, false_0 != 0, false_0 != 0);
        }
    }
    if !win_valid(win) {
        return FAIL;
    }
    curwin.set(win);
    curbuf.set((*curwin.get()).w_buffer);
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn restore_win(mut switchwin: *mut switchwin_T, mut no_display: bool) {
    restore_win_noblock(switchwin, no_display);
    unblock_autocmds();
}
#[no_mangle]
pub unsafe extern "C" fn restore_win_noblock(
    mut switchwin: *mut switchwin_T,
    mut no_display: bool,
) {
    if !(*switchwin).sw_curtab.is_null()
        && valid_tabpage((*switchwin).sw_curtab) as ::core::ffi::c_int != 0
    {
        if no_display {
            let old_tp_curwin: *mut win_T = (*curtab.get()).tp_curwin;
            unuse_tabpage(curtab.get());
            (*curtab.get()).tp_curwin = old_tp_curwin;
            use_tabpage((*switchwin).sw_curtab);
        } else {
            goto_tabpage_tp((*switchwin).sw_curtab, false_0 != 0, false_0 != 0);
        }
    }
    if !(*switchwin).sw_same_win {
        VIsual_active.set((*switchwin).sw_visual_active);
    }
    if win_valid((*switchwin).sw_curwin) {
        curwin.set((*switchwin).sw_curwin);
        curbuf.set((*curwin.get()).w_buffer);
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
