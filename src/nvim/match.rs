use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    pub type MsgpackRpcRequestHandler;
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
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xmemdupz(data: *const ::core::ffi::c_void, len: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static p_cpo: GlobalCell<*mut ::core::ffi::c_char>;
    static p_rdt: GlobalCell<OptInt>;
    fn vim_strchr(
        string: *const ::core::ffi::c_char,
        c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn skiptowhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static search_hl_has_cursor_lnum: GlobalCell<linenr_T>;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn redraw_later(wp: *mut win_T, type_0: ::core::ffi::c_int);
    fn redraw_win_range_later(wp: *mut win_T, first: linenr_T, last: linenr_T);
    static e_invarg2: [::core::ffi::c_char; 0];
    static e_invcmd: [::core::ffi::c_char; 0];
    static e_dictreq: [::core::ffi::c_char; 0];
    static e_listreq: [::core::ffi::c_char; 0];
    static e_trailing_arg: [::core::ffi::c_char; 0];
    static e_listarg: [::core::ffi::c_char; 0];
    static e_invalwindow: [::core::ffi::c_char; 0];
    fn get_optional_window(argvars: *mut typval_T, idx: ::core::ffi::c_int) -> *mut win_T;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_unref(l: *mut list_T);
    fn tv_list_append_tv(l: *mut list_T, tv: *mut typval_T);
    fn tv_list_append_dict(l: *mut list_T, dict: *mut dict_T);
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn tv_list_append_number(l: *mut list_T, n: varnumber_T);
    fn tv_list_idx_of_item(l: *const list_T, item: *const listitem_T) -> ::core::ffi::c_int;
    fn tv_dict_alloc() -> *mut dict_T;
    fn tv_dict_find(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        len: ptrdiff_t,
    ) -> *mut dictitem_T;
    fn tv_dict_get_number(d: *const dict_T, key: *const ::core::ffi::c_char) -> varnumber_T;
    fn tv_dict_get_string(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        save: bool,
    ) -> *mut ::core::ffi::c_char;
    fn tv_dict_get_string_buf(
        d: *const dict_T,
        key: *const ::core::ffi::c_char,
        numbuf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn tv_dict_add_list(
        d: *mut dict_T,
        key: *const ::core::ffi::c_char,
        key_len: size_t,
        list: *mut list_T,
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
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn tv_get_string_buf_chk(
        tv: *const typval_T,
        buf: *mut ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char;
    fn find_win_by_nr_or_id(vp: *mut typval_T) -> *mut win_T;
    fn ex_errmsg(
        msg: *const ::core::ffi::c_char,
        arg: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn ends_excmd(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn find_nextcmd(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn set_no_hlsearch(flag: bool);
    fn hasFolding(
        win: *mut win_T,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> bool;
    static called_emsg: GlobalCell<::core::ffi::c_int>;
    static search_first_line: GlobalCell<linenr_T>;
    static search_last_line: GlobalCell<linenr_T>;
    static curwin: GlobalCell<*mut win_T>;
    static got_int: GlobalCell<bool>;
    static ns_hl_fast: GlobalCell<NS>;
    static hl_attr_active: GlobalCell<*mut ::core::ffi::c_int>;
    fn syn_name2id(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn syn_id2name(id: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn syn_check_group(name: *const ::core::ffi::c_char, len: size_t) -> ::core::ffi::c_int;
    fn syn_id2attr(hl_id: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn utf_ptr2char(p_in: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utfc_ptr2len(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn utf_char2bytes(c: ::core::ffi::c_int, buf: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char;
    fn profile_setlimit(msec: int64_t) -> proftime_T;
    fn profile_passed_limit(tm: proftime_T) -> bool;
    fn re_multiline(prog: *const regprog_T) -> ::core::ffi::c_int;
    fn skip_regexp(
        startp: *mut ::core::ffi::c_char,
        delim: ::core::ffi::c_int,
        magic: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec_multi(
        rmp: *mut regmmatch_T,
        win: *mut win_T,
        buf: *mut buf_T,
        lnum: linenr_T,
        col: colnr_T,
        tm: *mut proftime_T,
        timed_out: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
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
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed = 2147483647;
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
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const UPD_CLEAR: C2Rust_Unnamed_14 = 50;
pub const UPD_NOT_VALID: C2Rust_Unnamed_14 = 40;
pub const UPD_SOME_VALID: C2Rust_Unnamed_14 = 35;
pub const UPD_REDRAW_TOP: C2Rust_Unnamed_14 = 30;
pub const UPD_INVERTED_ALL: C2Rust_Unnamed_14 = 25;
pub const UPD_INVERTED: C2Rust_Unnamed_14 = 20;
pub const UPD_VALID: C2Rust_Unnamed_14 = 10;
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
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 56] = unsafe {
    ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
        *b"void f_getmatches(typval_T *, typval_T *, EvalFuncData)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_iswhite(mut c: ::core::ffi::c_int) -> bool {
    return c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int;
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_SEARCH: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn tv_list_ref(l: *mut list_T) {
    if l.is_null() {
        return;
    }
    (*l).lv_refcount += 1;
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
unsafe extern "C" fn win_hl_attr(
    mut wp: *mut win_T,
    mut hlf: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return *if !(*wp).w_ns_hl_attr.is_null() && ns_hl_fast.get() < 0 as ::core::ffi::c_int {
        (*wp).w_ns_hl_attr
    } else {
        hl_attr_active.get()
    }
    .offset(hlf as isize);
}
pub const SEARCH_HL_PRIORITY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn match_add(
    mut wp: *mut win_T,
    grp: *const ::core::ffi::c_char,
    pat: *const ::core::ffi::c_char,
    mut prio: ::core::ffi::c_int,
    mut id: ::core::ffi::c_int,
    mut pos_list: *mut list_T,
    conceal_char: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut cur_0: *mut matchitem_T = ::core::ptr::null_mut::<matchitem_T>();
    let mut prev: *mut matchitem_T = ::core::ptr::null_mut::<matchitem_T>();
    let mut hlg_id: ::core::ffi::c_int = 0;
    let mut regprog: *mut regprog_T = ::core::ptr::null_mut::<regprog_T>();
    let mut rtype: ::core::ffi::c_int = UPD_SOME_VALID as ::core::ffi::c_int;
    if *grp as ::core::ffi::c_int == NUL || !pat.is_null() && *pat as ::core::ffi::c_int == NUL {
        return -1 as ::core::ffi::c_int;
    }
    if id < -1 as ::core::ffi::c_int || id == 0 as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E799: Invalid ID: %ld (must be greater than or equal to 1)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            id as int64_t,
        );
        return -1 as ::core::ffi::c_int;
    }
    if id == -1 as ::core::ffi::c_int {
        let c2rust_fresh0 = (*wp).w_next_match_id;
        (*wp).w_next_match_id = (*wp).w_next_match_id + 1;
        id = c2rust_fresh0;
    } else {
        let mut cur: *mut matchitem_T = (*wp).w_match_head;
        while !cur.is_null() {
            if (*cur).mit_id == id {
                semsg(
                    gettext(b"E801: ID already taken: %ld\0".as_ptr() as *const ::core::ffi::c_char),
                    id as int64_t,
                );
                return -1 as ::core::ffi::c_int;
            }
            cur = (*cur).mit_next;
        }
        if (*wp).w_next_match_id < id + 100 as ::core::ffi::c_int {
            (*wp).w_next_match_id = id + 100 as ::core::ffi::c_int;
        }
    }
    hlg_id = syn_check_group(grp, strlen(grp));
    if hlg_id == 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if !pat.is_null() && {
        regprog = vim_regcomp(pat, RE_MAGIC);
        regprog.is_null()
    } {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            pat,
        );
        return -1 as ::core::ffi::c_int;
    }
    let mut m: *mut matchitem_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<matchitem_T>()) as *mut matchitem_T;
    if tv_list_len(pos_list) > 0 as ::core::ffi::c_int {
        (*m).mit_pos_array = xcalloc(
            tv_list_len(pos_list) as size_t,
            ::core::mem::size_of::<llpos_T>(),
        ) as *mut llpos_T;
        (*m).mit_pos_count = tv_list_len(pos_list);
    }
    (*m).mit_id = id;
    (*m).mit_priority = prio;
    (*m).mit_pattern = if pat.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup(pat)
    };
    (*m).mit_hlg_id = hlg_id;
    (*m).mit_match.regprog = regprog;
    (*m).mit_match.rmm_ic = false_0;
    (*m).mit_match.rmm_maxcol = 0 as ::core::ffi::c_int as colnr_T;
    (*m).mit_conceal_char = 0 as ::core::ffi::c_int;
    if !conceal_char.is_null() {
        (*m).mit_conceal_char = utf_ptr2char(conceal_char);
    }
    if !pos_list.is_null() {
        let mut toplnum: linenr_T = 0 as linenr_T;
        let mut botlnum: linenr_T = 0 as linenr_T;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let l_: *mut list_T = pos_list;
        's_369: {
            if !l_.is_null() {
                let mut li: *mut listitem_T = (*l_).lv_first;
                '_fail: loop {
                    if li.is_null() {
                        break 's_369;
                    }
                    let mut lnum: linenr_T = 0 as linenr_T;
                    let mut col: colnr_T = 0 as colnr_T;
                    let mut len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    let mut error: bool = false;
                    's_183: {
                        if (*li).li_tv.v_type as ::core::ffi::c_uint
                            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            let subl: *const list_T = (*li).li_tv.vval.v_list;
                            let mut subli: *const listitem_T = tv_list_first(subl);
                            if subli.is_null() {
                                semsg(
                                    gettext(b"E5030: Empty list at position %d\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    tv_list_idx_of_item(pos_list, li),
                                );
                                break '_fail;
                            } else {
                                lnum = tv_get_number_chk(&raw const (*subli).li_tv, &raw mut error)
                                    as linenr_T;
                                if error {
                                    break '_fail;
                                }
                                if lnum <= 0 as linenr_T {
                                    break 's_183;
                                } else {
                                    (*(*m).mit_pos_array.offset(i as isize)).lnum = lnum;
                                    subli = (*subli).li_next;
                                    if !subli.is_null() {
                                        col = tv_get_number_chk(
                                            &raw const (*subli).li_tv,
                                            &raw mut error,
                                        ) as colnr_T;
                                        if error {
                                            break '_fail;
                                        }
                                        if col < 0 as ::core::ffi::c_int {
                                            break 's_183;
                                        } else {
                                            subli = (*subli).li_next;
                                            if !subli.is_null() {
                                                len = tv_get_number_chk(
                                                    &raw const (*subli).li_tv,
                                                    &raw mut error,
                                                )
                                                    as colnr_T
                                                    as ::core::ffi::c_int;
                                                if len < 0 as ::core::ffi::c_int {
                                                    break 's_183;
                                                } else if error {
                                                    break '_fail;
                                                }
                                            }
                                        }
                                    }
                                    (*(*m).mit_pos_array.offset(i as isize)).col = col;
                                    (*(*m).mit_pos_array.offset(i as isize)).len = len;
                                }
                            }
                        } else if (*li).li_tv.v_type as ::core::ffi::c_uint
                            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if (*li).li_tv.vval.v_number <= 0 as varnumber_T {
                                break 's_183;
                            } else {
                                (*(*m).mit_pos_array.offset(i as isize)).lnum =
                                    (*li).li_tv.vval.v_number as linenr_T;
                                (*(*m).mit_pos_array.offset(i as isize)).col =
                                    0 as ::core::ffi::c_int as colnr_T;
                                (*(*m).mit_pos_array.offset(i as isize)).len =
                                    0 as ::core::ffi::c_int;
                            }
                        } else {
                            semsg(
                                gettext(
                                    b"E5031: List or number required at position %d\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ),
                                tv_list_idx_of_item(pos_list, li),
                            );
                            break '_fail;
                        }
                        if toplnum == 0 as linenr_T || lnum < toplnum {
                            toplnum = lnum;
                        }
                        if botlnum == 0 as linenr_T || lnum >= botlnum {
                            botlnum = lnum + 1 as linenr_T;
                        }
                        i += 1;
                    }
                    li = (*li).li_next;
                }
                vim_regfree(regprog);
                xfree((*m).mit_pattern as *mut ::core::ffi::c_void);
                xfree((*m).mit_pos_array as *mut ::core::ffi::c_void);
                xfree(m as *mut ::core::ffi::c_void);
                return -1 as ::core::ffi::c_int;
            }
        }
        if toplnum != 0 as linenr_T {
            redraw_win_range_later(wp, toplnum, botlnum);
            (*m).mit_toplnum = toplnum;
            (*m).mit_botlnum = botlnum;
            rtype = UPD_VALID as ::core::ffi::c_int;
        }
    }
    cur_0 = (*wp).w_match_head;
    prev = cur_0;
    while !cur_0.is_null() && prio >= (*cur_0).mit_priority {
        prev = cur_0;
        cur_0 = (*cur_0).mit_next;
    }
    if cur_0 == prev {
        (*wp).w_match_head = m;
    } else {
        (*prev).mit_next = m;
    }
    (*m).mit_next = cur_0;
    redraw_later(wp, rtype);
    return id;
}
unsafe extern "C" fn match_delete(
    mut wp: *mut win_T,
    mut id: ::core::ffi::c_int,
    mut perr: bool,
) -> ::core::ffi::c_int {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut prev: *mut matchitem_T = cur;
    let mut rtype: ::core::ffi::c_int = UPD_SOME_VALID as ::core::ffi::c_int;
    if id < 1 as ::core::ffi::c_int {
        if perr {
            semsg(
                gettext(
                    b"E802: Invalid ID: %ld (must be greater than or equal to 1)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                id as int64_t,
            );
        }
        return -1 as ::core::ffi::c_int;
    }
    while !cur.is_null() && (*cur).mit_id != id {
        prev = cur;
        cur = (*cur).mit_next;
    }
    if cur.is_null() {
        if perr {
            semsg(
                gettext(b"E803: ID not found: %ld\0".as_ptr() as *const ::core::ffi::c_char),
                id as int64_t,
            );
        }
        return -1 as ::core::ffi::c_int;
    }
    if cur == prev {
        (*wp).w_match_head = (*cur).mit_next;
    } else {
        (*prev).mit_next = (*cur).mit_next;
    }
    vim_regfree((*cur).mit_match.regprog);
    xfree((*cur).mit_pattern as *mut ::core::ffi::c_void);
    if (*cur).mit_toplnum != 0 as linenr_T {
        redraw_win_range_later(wp, (*cur).mit_toplnum, (*cur).mit_botlnum);
        rtype = UPD_VALID as ::core::ffi::c_int;
    }
    xfree((*cur).mit_pos_array as *mut ::core::ffi::c_void);
    xfree(cur as *mut ::core::ffi::c_void);
    redraw_later(wp, rtype);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn clear_matches(mut wp: *mut win_T) {
    while !(*wp).w_match_head.is_null() {
        let mut m: *mut matchitem_T = (*(*wp).w_match_head).mit_next;
        vim_regfree((*(*wp).w_match_head).mit_match.regprog);
        xfree((*(*wp).w_match_head).mit_pattern as *mut ::core::ffi::c_void);
        xfree((*(*wp).w_match_head).mit_pos_array as *mut ::core::ffi::c_void);
        xfree((*wp).w_match_head as *mut ::core::ffi::c_void);
        (*wp).w_match_head = m;
    }
    redraw_later(wp, UPD_SOME_VALID as ::core::ffi::c_int);
}
unsafe extern "C" fn get_match(mut wp: *mut win_T, mut id: ::core::ffi::c_int) -> *mut matchitem_T {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    while !cur.is_null() && (*cur).mit_id != id {
        cur = (*cur).mit_next;
    }
    return cur;
}
#[no_mangle]
pub unsafe extern "C" fn init_search_hl(mut wp: *mut win_T, mut search_hl: *mut match_T) {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    while !cur.is_null() {
        (*cur).mit_hl.rm = (*cur).mit_match;
        if (*cur).mit_hlg_id == 0 as ::core::ffi::c_int {
            (*cur).mit_hl.attr = 0 as ::core::ffi::c_int;
        } else {
            (*cur).mit_hl.attr = syn_id2attr((*cur).mit_hlg_id);
        }
        (*cur).mit_hl.buf = (*wp).w_buffer;
        (*cur).mit_hl.lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*cur).mit_hl.first_lnum = 0 as ::core::ffi::c_int as linenr_T;
        (*cur).mit_hl.tm = profile_setlimit(p_rdt.get() as int64_t);
        cur = (*cur).mit_next;
    }
    (*search_hl).buf = (*wp).w_buffer;
    (*search_hl).lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*search_hl).first_lnum = 0 as ::core::ffi::c_int as linenr_T;
    (*search_hl).attr = win_hl_attr(wp, HLF_L as ::core::ffi::c_int);
}
unsafe extern "C" fn next_search_hl_pos(
    mut shl: *mut match_T,
    mut lnum: linenr_T,
    mut match_0: *mut matchitem_T,
    mut mincol: colnr_T,
) -> ::core::ffi::c_int {
    let mut found: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
    let mut i: ::core::ffi::c_int = (*match_0).mit_pos_cur;
    while i < (*match_0).mit_pos_count {
        let mut pos: *mut llpos_T = (*match_0).mit_pos_array.offset(i as isize);
        if (*pos).lnum == 0 as linenr_T {
            break;
        }
        if !((*pos).len == 0 as ::core::ffi::c_int && (*pos).col < mincol) {
            if (*pos).lnum == lnum {
                if found >= 0 as ::core::ffi::c_int {
                    if (*pos).col < (*(*match_0).mit_pos_array.offset(found as isize)).col {
                        let mut tmp: llpos_T = *pos;
                        *pos = *(*match_0).mit_pos_array.offset(found as isize);
                        *(*match_0).mit_pos_array.offset(found as isize) = tmp;
                    }
                } else {
                    found = i;
                }
            }
        }
        i += 1;
    }
    (*match_0).mit_pos_cur = 0 as ::core::ffi::c_int;
    if found >= 0 as ::core::ffi::c_int {
        let mut start: colnr_T =
            if (*(*match_0).mit_pos_array.offset(found as isize)).col == 0 as ::core::ffi::c_int {
                0 as colnr_T
            } else {
                (*(*match_0).mit_pos_array.offset(found as isize)).col - 1 as colnr_T
            };
        let mut end: colnr_T =
            if (*(*match_0).mit_pos_array.offset(found as isize)).col == 0 as ::core::ffi::c_int {
                MAXCOL as ::core::ffi::c_int
            } else {
                start + (*(*match_0).mit_pos_array.offset(found as isize)).len as colnr_T
            };
        (*shl).lnum = lnum;
        (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum =
            0 as ::core::ffi::c_int as linenr_T;
        (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col = start;
        (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum =
            0 as ::core::ffi::c_int as linenr_T;
        (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col = end;
        (*shl).is_addpos = true_0 != 0;
        (*shl).has_cursor = false_0 != 0;
        (*match_0).mit_pos_cur = found + 1 as ::core::ffi::c_int;
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn next_search_hl(
    mut win: *mut win_T,
    mut search_hl: *mut match_T,
    mut shl: *mut match_T,
    mut lnum: linenr_T,
    mut mincol: colnr_T,
    mut cur: *mut matchitem_T,
) {
    let mut matchcol: colnr_T = 0;
    let mut nmatched: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    if (lnum < search_first_line.get() || lnum > search_last_line.get()) && cur.is_null() {
        (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
        return;
    }
    if (*shl).lnum != 0 as linenr_T {
        let mut l: linenr_T = (*shl).lnum + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
            - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
        if lnum > l {
            (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
        } else if lnum < l || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col > mincol {
            return;
        }
    }
    loop {
        if profile_passed_limit((*shl).tm) {
            (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
            break;
        } else {
            if (*shl).lnum == 0 as linenr_T {
                matchcol = 0 as ::core::ffi::c_int as colnr_T;
            } else if vim_strchr(p_cpo.get(), CPO_SEARCH).is_null()
                || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum == 0 as linenr_T
                    && (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col
                        <= (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col
            {
                matchcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
                let mut ml: *mut ::core::ffi::c_char =
                    ml_get_buf((*shl).buf, lnum).offset(matchcol as isize);
                if *ml as ::core::ffi::c_int == NUL {
                    matchcol += 1;
                    (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                    break;
                } else {
                    matchcol += utfc_ptr2len(ml);
                }
            } else {
                matchcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
            }
            (*shl).lnum = lnum;
            if !(*shl).rm.regprog.is_null() {
                let mut regprog_is_copy: bool = shl != search_hl
                    && !cur.is_null()
                    && shl == &raw mut (*cur).mit_hl
                    && (*cur).mit_match.regprog == (*cur).mit_hl.rm.regprog;
                let mut timed_out: ::core::ffi::c_int = false_0;
                nmatched = vim_regexec_multi(
                    &raw mut (*shl).rm,
                    win,
                    (*shl).buf,
                    lnum,
                    matchcol,
                    &raw mut (*shl).tm,
                    &raw mut timed_out,
                );
                if regprog_is_copy {
                    (*cur).mit_match.regprog = (*cur).mit_hl.rm.regprog;
                }
                if called_emsg.get() > called_emsg_before
                    || got_int.get() as ::core::ffi::c_int != 0
                    || timed_out != 0
                {
                    if shl == search_hl {
                        vim_regfree((*shl).rm.regprog);
                        set_no_hlsearch(true_0 != 0);
                    }
                    (*shl).rm.regprog = ::core::ptr::null_mut::<regprog_T>();
                    (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                    got_int.set(false_0 != 0);
                    break;
                }
            } else if !cur.is_null() {
                nmatched = next_search_hl_pos(shl, lnum, cur, matchcol);
            }
            if nmatched == 0 as ::core::ffi::c_int {
                (*shl).lnum = 0 as ::core::ffi::c_int as linenr_T;
                break;
            } else {
                if !((*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum > 0 as linenr_T
                    || (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col >= mincol
                    || nmatched > 1 as ::core::ffi::c_int
                    || (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col > mincol)
                {
                    continue;
                }
                (*shl).lnum += (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
                break;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn prepare_search_hl(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut lnum: linenr_T,
) {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    while !cur.is_null() || shl_flag as ::core::ffi::c_int == false_0 {
        if shl_flag as ::core::ffi::c_int == false_0 {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if !(*shl).rm.regprog.is_null()
            && (*shl).lnum == 0 as linenr_T
            && re_multiline((*shl).rm.regprog) != 0
        {
            if (*shl).first_lnum == 0 as linenr_T {
                (*shl).first_lnum = lnum;
                while (*shl).first_lnum > (*wp).w_topline {
                    if hasFolding(
                        wp,
                        (*shl).first_lnum - 1 as linenr_T,
                        ::core::ptr::null_mut::<linenr_T>(),
                        ::core::ptr::null_mut::<linenr_T>(),
                    ) {
                        break;
                    }
                    (*shl).first_lnum -= 1;
                }
            }
            if !cur.is_null() {
                (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
            }
            let mut pos_inprogress: bool = true_0 != 0;
            let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while (*shl).first_lnum < lnum
                && (!(*shl).rm.regprog.is_null()
                    || !cur.is_null() && pos_inprogress as ::core::ffi::c_int != 0)
            {
                next_search_hl(
                    wp,
                    search_hl,
                    shl,
                    (*shl).first_lnum,
                    n,
                    if shl == search_hl {
                        ::core::ptr::null_mut::<matchitem_T>()
                    } else {
                        cur
                    },
                );
                pos_inprogress = !(cur.is_null() || (*cur).mit_pos_cur == 0 as ::core::ffi::c_int);
                if (*shl).lnum != 0 as linenr_T {
                    (*shl).first_lnum = (*shl).lnum
                        + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
                        - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
                    n = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col
                        as ::core::ffi::c_int;
                } else {
                    (*shl).first_lnum += 1;
                    n = 0 as ::core::ffi::c_int;
                }
            }
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
}
unsafe extern "C" fn check_cur_search_hl(mut wp: *mut win_T, mut shl: *mut match_T) {
    let mut linecount: linenr_T = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
        - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum;
    if (*wp).w_cursor.lnum >= (*shl).lnum
        && (*wp).w_cursor.lnum <= (*shl).lnum + linecount
        && ((*wp).w_cursor.lnum > (*shl).lnum
            || (*wp).w_cursor.col >= (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col)
        && ((*wp).w_cursor.lnum < (*shl).lnum + linecount
            || (*wp).w_cursor.col < (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col)
    {
        (*shl).has_cursor = true_0 != 0;
    } else {
        (*shl).has_cursor = false_0 != 0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn prepare_search_hl_line(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut mincol: colnr_T,
    mut line: *mut *mut ::core::ffi::c_char,
    mut search_hl: *mut match_T,
    mut search_attr: *mut ::core::ffi::c_int,
    mut search_attr_from_match: *mut bool,
) -> bool {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    let mut area_highlighting: bool = false_0 != 0;
    while !cur.is_null() || !shl_flag {
        if !shl_flag {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        (*shl).startcol = MAXCOL as ::core::ffi::c_int as colnr_T;
        (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
        (*shl).attr_cur = 0 as ::core::ffi::c_int;
        (*shl).is_addpos = false_0 != 0;
        (*shl).has_cursor = false_0 != 0;
        if !cur.is_null() {
            (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
        }
        next_search_hl(
            wp,
            search_hl,
            shl,
            lnum,
            mincol,
            if shl == search_hl {
                ::core::ptr::null_mut::<matchitem_T>()
            } else {
                cur
            },
        );
        *line = ml_get_buf((*wp).w_buffer, lnum);
        if (*shl).lnum != 0 as linenr_T && (*shl).lnum <= lnum {
            if (*shl).lnum == lnum {
                (*shl).startcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
            } else {
                (*shl).startcol = 0 as ::core::ffi::c_int as colnr_T;
            }
            if lnum
                == (*shl).lnum + (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum
                    - (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].lnum
            {
                (*shl).endcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
            } else {
                (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
            }
            if shl == search_hl {
                check_cur_search_hl(wp, shl);
            }
            if (*shl).startcol == (*shl).endcol {
                if *(*line).offset((*shl).endcol as isize) as ::core::ffi::c_int != NUL {
                    (*shl).endcol += utfc_ptr2len((*line).offset((*shl).endcol as isize));
                } else {
                    (*shl).endcol += 1;
                }
            }
            if (*shl).startcol < mincol {
                (*shl).attr_cur = (*shl).attr;
                *search_attr = (*shl).attr;
                *search_attr_from_match = shl != search_hl;
            }
            area_highlighting = true_0 != 0;
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
    return area_highlighting;
}
#[no_mangle]
pub unsafe extern "C" fn update_search_hl(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut line: *mut *mut ::core::ffi::c_char,
    mut search_hl: *mut match_T,
    mut has_match_conc: *mut ::core::ffi::c_int,
    mut match_conc: *mut ::core::ffi::c_int,
    mut lcs_eol_todo: bool,
    mut on_last_col: *mut bool,
    mut search_attr_from_match: *mut bool,
) -> ::core::ffi::c_int {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    let mut search_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !cur.is_null() || !shl_flag {
        if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if !cur.is_null() {
            (*cur).mit_pos_cur = 0 as ::core::ffi::c_int;
        }
        let mut pos_inprogress: bool = true_0 != 0;
        while !(*shl).rm.regprog.is_null()
            || !cur.is_null() && pos_inprogress as ::core::ffi::c_int != 0
        {
            if (*shl).startcol != MAXCOL as ::core::ffi::c_int
                && col >= (*shl).startcol
                && col < (*shl).endcol
            {
                let mut next_col: ::core::ffi::c_int =
                    col as ::core::ffi::c_int + utfc_ptr2len((*line).offset(col as isize));
                if (*shl).endcol < next_col {
                    (*shl).endcol = next_col as colnr_T;
                }
                if shl == search_hl && (*shl).has_cursor as ::core::ffi::c_int != 0 {
                    (*shl).attr_cur = win_hl_attr(wp, HLF_LC as ::core::ffi::c_int);
                    if (*shl).attr_cur != (*shl).attr {
                        search_hl_has_cursor_lnum.set(lnum);
                    }
                } else {
                    (*shl).attr_cur = (*shl).attr;
                }
                if !cur.is_null()
                    && shl != search_hl
                    && syn_name2id(b"Conceal\0".as_ptr() as *const ::core::ffi::c_char)
                        == (*cur).mit_hlg_id
                {
                    *has_match_conc = if col == (*shl).startcol {
                        2 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    };
                    *match_conc = (*cur).mit_conceal_char;
                } else {
                    *has_match_conc = 0 as ::core::ffi::c_int;
                }
                break;
            } else {
                if col != (*shl).endcol {
                    break;
                }
                (*shl).attr_cur = 0 as ::core::ffi::c_int;
                next_search_hl(
                    wp,
                    search_hl,
                    shl,
                    lnum,
                    col,
                    if shl == search_hl {
                        ::core::ptr::null_mut::<matchitem_T>()
                    } else {
                        cur
                    },
                );
                pos_inprogress = !(cur.is_null() || (*cur).mit_pos_cur == 0 as ::core::ffi::c_int);
                *line = ml_get_buf((*wp).w_buffer, lnum);
                if (*shl).lnum != lnum {
                    break;
                }
                (*shl).startcol = (*shl).rm.startpos[0 as ::core::ffi::c_int as usize].col;
                if (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].lnum == 0 as linenr_T {
                    (*shl).endcol = (*shl).rm.endpos[0 as ::core::ffi::c_int as usize].col;
                } else {
                    (*shl).endcol = MAXCOL as ::core::ffi::c_int as colnr_T;
                }
                if shl == search_hl {
                    check_cur_search_hl(wp, shl);
                }
                if (*shl).startcol == (*shl).endcol {
                    let mut p: *mut ::core::ffi::c_char = (*line).offset((*shl).endcol as isize);
                    if *p as ::core::ffi::c_int == NUL {
                        (*shl).endcol += 1;
                    } else {
                        (*shl).endcol += utfc_ptr2len(p);
                    }
                }
            }
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
    *search_attr_from_match = false_0 != 0;
    search_attr = (*search_hl).attr_cur;
    cur = (*wp).w_match_head;
    shl_flag = false_0 != 0;
    while !cur.is_null() || !shl_flag {
        if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if (*shl).attr_cur != 0 as ::core::ffi::c_int {
            search_attr = (*shl).attr_cur;
            *on_last_col = col as ::core::ffi::c_int + 1 as ::core::ffi::c_int >= (*shl).endcol;
            *search_attr_from_match = shl != search_hl;
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
    if *(*line).offset(col as isize) as ::core::ffi::c_int == NUL
        && ((*wp).w_onebuf_opt.wo_list != 0 && !lcs_eol_todo)
    {
        search_attr = 0 as ::core::ffi::c_int;
    }
    return search_attr;
}
#[no_mangle]
pub unsafe extern "C" fn get_prevcol_hl_flag(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut curcol: colnr_T,
) -> bool {
    let mut prevcol: colnr_T = curcol;
    if (if (*wp).w_onebuf_opt.wo_wrap != 0 {
        (*wp).w_skipcol
    } else {
        (*wp).w_leftcol
    }) > prevcol
    {
        prevcol += 1;
    }
    if !(*search_hl).is_addpos
        && (prevcol == (*search_hl).startcol
            || prevcol > (*search_hl).startcol
                && (*search_hl).endcol == MAXCOL as ::core::ffi::c_int)
    {
        return true_0 != 0;
    }
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    while !cur.is_null() {
        if !(*cur).mit_hl.is_addpos
            && (prevcol == (*cur).mit_hl.startcol
                || prevcol > (*cur).mit_hl.startcol
                    && (*cur).mit_hl.endcol == MAXCOL as ::core::ffi::c_int)
        {
            return true_0 != 0;
        }
        cur = (*cur).mit_next;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn get_search_match_hl(
    mut wp: *mut win_T,
    mut search_hl: *mut match_T,
    mut col: colnr_T,
    mut char_attr: *mut ::core::ffi::c_int,
) {
    let mut cur: *mut matchitem_T = (*wp).w_match_head;
    let mut shl: *mut match_T = ::core::ptr::null_mut::<match_T>();
    let mut shl_flag: bool = false_0 != 0;
    while !cur.is_null() || !shl_flag {
        if !shl_flag && (cur.is_null() || (*cur).mit_priority > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true_0 != 0;
        } else {
            shl = &raw mut (*cur).mit_hl;
        }
        if col as ::core::ffi::c_int - 1 as ::core::ffi::c_int == (*shl).startcol
            && (shl == search_hl || !(*shl).is_addpos)
        {
            *char_attr = (*shl).attr;
        }
        if shl != search_hl && !cur.is_null() {
            cur = (*cur).mit_next;
        }
    }
}
unsafe extern "C" fn matchadd_dict_arg(
    mut tv: *mut typval_T,
    mut conceal_char: *mut *const ::core::ffi::c_char,
    mut win: *mut *mut win_T,
) -> ::core::ffi::c_int {
    let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    if (*tv).v_type as ::core::ffi::c_uint != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
        return FAIL;
    }
    di = tv_dict_find(
        (*tv).vval.v_dict,
        b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        *conceal_char = tv_get_string(&raw mut (*di).di_tv);
    }
    di = tv_dict_find(
        (*tv).vval.v_dict,
        b"window\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if di.is_null() {
        return OK;
    }
    *win = find_win_by_nr_or_id(&raw mut (*di).di_tv);
    if (*win).is_null() {
        emsg(gettext(
            &raw const e_invalwindow as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn f_clearmatches(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut win: *mut win_T = get_optional_window(argvars, 0 as ::core::ffi::c_int);
    if !win.is_null() {
        clear_matches(win);
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_getmatches(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut win: *mut win_T = get_optional_window(argvars, 0 as ::core::ffi::c_int);
    tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
    if win.is_null() {
        return;
    }
    let mut cur: *mut matchitem_T = (*win).w_match_head;
    while !cur.is_null() {
        let mut dict: *mut dict_T = tv_dict_alloc();
        if (*cur).mit_match.regprog.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*cur).mit_pos_count {
                let mut llpos: *mut llpos_T = ::core::ptr::null_mut::<llpos_T>();
                let mut buf: [::core::ffi::c_char; 30] = [0; 30];
                llpos = (*cur).mit_pos_array.offset(i as isize);
                if (*llpos).lnum == 0 as linenr_T {
                    break;
                }
                let l: *mut list_T = tv_list_alloc(
                    (1 as ::core::ffi::c_int
                        + (if (*llpos).col > 0 as ::core::ffi::c_int {
                            2 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        })) as ptrdiff_t,
                );
                tv_list_append_number(l, (*llpos).lnum as varnumber_T);
                if (*llpos).col > 0 as ::core::ffi::c_int {
                    tv_list_append_number(l, (*llpos).col as varnumber_T);
                    tv_list_append_number(l, (*llpos).len as varnumber_T);
                }
                let mut len: ::core::ffi::c_int = snprintf(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                    b"pos%d\0".as_ptr() as *const ::core::ffi::c_char,
                    i + 1 as ::core::ffi::c_int,
                );
                '_c2rust_label: {
                    if (len as size_t) < ::core::mem::size_of::<[::core::ffi::c_char; 30]>() {
                    } else {
                        __assert_fail(
                            b"(size_t)len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/match.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            898 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                tv_dict_add_list(
                    dict,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    len as size_t,
                    l,
                );
                i += 1;
            }
        } else {
            tv_dict_add_str(
                dict,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                (*cur).mit_pattern,
            );
        }
        tv_dict_add_str(
            dict,
            b"group\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            syn_id2name((*cur).mit_hlg_id),
        );
        tv_dict_add_nr(
            dict,
            b"priority\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            (*cur).mit_priority as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            (*cur).mit_id as varnumber_T,
        );
        if (*cur).mit_conceal_char != 0 {
            let mut buf_0: [::core::ffi::c_char; 7] = [0; 7];
            buf_0[utf_char2bytes(
                (*cur).mit_conceal_char,
                &raw mut buf_0 as *mut ::core::ffi::c_char,
            ) as usize] = NUL as ::core::ffi::c_char;
            tv_dict_add_str(
                dict,
                b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                &raw mut buf_0 as *mut ::core::ffi::c_char,
            );
        }
        tv_list_append_dict((*rettv).vval.v_list, dict);
        cur = (*cur).mit_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_setmatches(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut s: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut win: *mut win_T = get_optional_window(argvars, 1 as ::core::ffi::c_int);
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
        return;
    }
    if win.is_null() {
        return;
    }
    let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let mut li_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as ::core::ffi::c_uint
                != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                || {
                    d = (*li).li_tv.vval.v_dict;
                    d.is_null()
                }
            {
                semsg(
                    gettext(
                        b"E474: List item %d is either not a dictionary or an empty one\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    li_idx,
                );
                return;
            }
            if !(!tv_dict_find(
                d,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            )
            .is_null()
                && (!tv_dict_find(
                    d,
                    b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                )
                .is_null()
                    || !tv_dict_find(
                        d,
                        b"pos1\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                            as ptrdiff_t,
                    )
                    .is_null())
                && !tv_dict_find(
                    d,
                    b"priority\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                )
                .is_null()
                && !tv_dict_find(
                    d,
                    b"id\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                )
                .is_null())
            {
                semsg(
                    gettext(
                        b"E474: List item %d is missing one of the required keys\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    li_idx,
                );
                return;
            }
            li_idx += 1;
            li = (*li).li_next;
        }
    }
    clear_matches(win);
    let mut match_add_failed: bool = false_0 != 0;
    let l__0: *const list_T = l;
    if !l__0.is_null() {
        let mut li_0: *const listitem_T = (*l__0).lv_first;
        while !li_0.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            d = (*li_0).li_tv.vval.v_dict;
            let di: *mut dictitem_T = tv_dict_find(
                d,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            if di.is_null() {
                if s.is_null() {
                    s = tv_list_alloc(9 as ptrdiff_t);
                }
                i = 1 as ::core::ffi::c_int;
                while i < 9 as ::core::ffi::c_int {
                    let mut buf: [::core::ffi::c_char; 30] = [0; 30];
                    snprintf(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                        b"pos%d\0".as_ptr() as *const ::core::ffi::c_char,
                        i,
                    );
                    let pos_di: *mut dictitem_T =
                        tv_dict_find(d, &raw mut buf as *mut ::core::ffi::c_char, -1 as ptrdiff_t);
                    if pos_di.is_null() {
                        break;
                    }
                    if (*pos_di).di_tv.v_type as ::core::ffi::c_uint
                        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        return;
                    }
                    tv_list_append_tv(s, &raw mut (*pos_di).di_tv);
                    tv_list_ref(s);
                    i += 1;
                }
            }
            let mut group_buf: [::core::ffi::c_char; 65] = [0; 65];
            let group: *const ::core::ffi::c_char = tv_dict_get_string_buf(
                d,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut group_buf as *mut ::core::ffi::c_char,
            );
            let priority: ::core::ffi::c_int =
                tv_dict_get_number(d, b"priority\0".as_ptr() as *const ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            let id: ::core::ffi::c_int =
                tv_dict_get_number(d, b"id\0".as_ptr() as *const ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            let conceal_di: *mut dictitem_T = tv_dict_find(
                d,
                b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            let conceal: *const ::core::ffi::c_char = if !conceal_di.is_null() {
                tv_get_string(&raw mut (*conceal_di).di_tv)
            } else {
                ::core::ptr::null::<::core::ffi::c_char>()
            };
            if i == 0 as ::core::ffi::c_int {
                if match_add(
                    win,
                    group,
                    tv_dict_get_string(
                        d,
                        b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                        false,
                    ),
                    priority,
                    id,
                    ::core::ptr::null_mut::<list_T>(),
                    conceal,
                ) != id
                {
                    match_add_failed = true;
                }
            } else {
                if match_add(
                    win,
                    group,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    priority,
                    id,
                    s,
                    conceal,
                ) != id
                {
                    match_add_failed = true;
                }
                tv_list_unref(s);
                s = ::core::ptr::null_mut::<list_T>();
            }
            li_0 = (*li_0).li_next;
        }
    }
    if !match_add_failed {
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_matchadd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut grpbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
    let grp: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut grpbuf as *mut ::core::ffi::c_char,
    );
    let pat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut patbuf as *mut ::core::ffi::c_char,
    );
    let mut prio: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    let mut conceal_char: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut win: *mut win_T = curwin.get();
    (*rettv).vval.v_number = -1 as varnumber_T;
    if grp.is_null() || pat.is_null() {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        prio = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            id = tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && matchadd_dict_arg(
                    argvars.offset(4 as ::core::ffi::c_int as isize),
                    &raw mut conceal_char,
                    &raw mut win,
                ) == FAIL
            {
                return;
            }
        }
    }
    if error {
        return;
    }
    if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E798: ID is reserved for \":match\": %d\0".as_ptr() as *const ::core::ffi::c_char
            ),
            id,
        );
        return;
    }
    (*rettv).vval.v_number = match_add(
        win,
        grp,
        pat,
        prio,
        id,
        ::core::ptr::null_mut::<list_T>(),
        conceal_char,
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_matchaddpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    let group: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    if group.is_null() {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_listarg as *const ::core::ffi::c_char),
            b"matchaddpos()\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
    l = (*argvars.offset(1 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    if tv_list_len(l) == 0 as ::core::ffi::c_int {
        return;
    }
    let mut error: bool = false_0 != 0;
    let mut prio: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut conceal_char: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut win: *mut win_T = curwin.get();
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        prio = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as ::core::ffi::c_int;
        if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            id = tv_get_number_chk(
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && matchadd_dict_arg(
                    argvars.offset(4 as ::core::ffi::c_int as isize),
                    &raw mut conceal_char,
                    &raw mut win,
                ) == FAIL
            {
                return;
            }
        }
    }
    if error as ::core::ffi::c_int == true_0 {
        return;
    }
    if id == 1 as ::core::ffi::c_int || id == 2 as ::core::ffi::c_int {
        semsg(
            gettext(
                b"E798: ID is reserved for \"match\": %d\0".as_ptr() as *const ::core::ffi::c_char
            ),
            id,
        );
        return;
    }
    (*rettv).vval.v_number = match_add(
        win,
        group,
        ::core::ptr::null::<::core::ffi::c_char>(),
        prio,
        id,
        l,
        conceal_char,
    ) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_matcharg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let id: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    tv_list_alloc_ret(
        rettv,
        (if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
            2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as ptrdiff_t,
    );
    if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
        let m: *mut matchitem_T = get_match(curwin.get(), id);
        if !m.is_null() {
            tv_list_append_string(
                (*rettv).vval.v_list,
                syn_id2name((*m).mit_hlg_id),
                -1 as ssize_t,
            );
            tv_list_append_string((*rettv).vval.v_list, (*m).mit_pattern, -1 as ssize_t);
        } else {
            tv_list_append_string(
                (*rettv).vval.v_list,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ssize_t,
            );
            tv_list_append_string(
                (*rettv).vval.v_list,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ssize_t,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_matchdelete(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut win: *mut win_T = get_optional_window(argvars, 1 as ::core::ffi::c_int);
    if win.is_null() {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = match_delete(
            win,
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
            true_0 != 0,
        ) as varnumber_T;
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_match(mut eap: *mut exarg_T) {
    let mut g: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = 0;
    if (*eap).line2 <= 3 as linenr_T {
        id = (*eap).line2 as ::core::ffi::c_int;
    } else {
        emsg(&raw const e_invcmd as *const ::core::ffi::c_char);
        return;
    }
    if (*eap).skip == 0 {
        match_delete(curwin.get(), id, false_0 != 0);
    }
    if ends_excmd(*(*eap).arg as ::core::ffi::c_int) != 0 {
        end = (*eap).arg;
    } else if strncasecmp(
        (*eap).arg,
        b"none\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        4 as ::core::ffi::c_int as size_t,
    ) == 0 as ::core::ffi::c_int
        && (ascii_iswhite(*(*eap).arg.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
            || ends_excmd(
                *(*eap).arg.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) != 0)
    {
        end = (*eap).arg.offset(4 as ::core::ffi::c_int as isize);
    } else {
        let mut p: *mut ::core::ffi::c_char = skiptowhite((*eap).arg);
        if (*eap).skip == 0 {
            g = xmemdupz(
                (*eap).arg as *const ::core::ffi::c_void,
                p.offset_from((*eap).arg) as size_t,
            ) as *mut ::core::ffi::c_char;
        }
        p = skipwhite(p);
        if *p as ::core::ffi::c_int == NUL {
            xfree(g as *mut ::core::ffi::c_void);
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
            return;
        }
        end = skip_regexp(
            p.offset(1 as ::core::ffi::c_int as isize),
            *p as ::core::ffi::c_int,
            true_0,
        );
        if (*eap).skip == 0 {
            if *end as ::core::ffi::c_int != NUL
                && ends_excmd(
                    *skipwhite(end.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
                ) == 0
            {
                xfree(g as *mut ::core::ffi::c_void);
                (*eap).errmsg =
                    ex_errmsg(&raw const e_trailing_arg as *const ::core::ffi::c_char, end);
                return;
            }
            if *end as ::core::ffi::c_int != *p as ::core::ffi::c_int {
                xfree(g as *mut ::core::ffi::c_void);
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    p,
                );
                return;
            }
            let mut c: ::core::ffi::c_int = *end as uint8_t as ::core::ffi::c_int;
            *end = NUL as ::core::ffi::c_char;
            match_add(
                curwin.get(),
                g,
                p.offset(1 as ::core::ffi::c_int as isize),
                10 as ::core::ffi::c_int,
                id,
                ::core::ptr::null_mut::<list_T>(),
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            xfree(g as *mut ::core::ffi::c_void);
            *end = c as ::core::ffi::c_char;
        }
    }
    (*eap).nextcmd = find_nextcmd(end);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
