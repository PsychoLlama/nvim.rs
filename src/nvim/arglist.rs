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
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut autocmd_no_enter: ::core::ffi::c_int;
    static mut autocmd_no_leave: ::core::ffi::c_int;
    fn is_aucmd_win(win: *mut win_T) -> bool;
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn set_bufref(bufref: *mut bufref_T, buf: *mut buf_T);
    fn bufref_valid(bufref: *mut bufref_T) -> bool;
    fn curbuf_reusable() -> bool;
    fn buflist_findnr(nr: ::core::ffi::c_int) -> *mut buf_T;
    fn buf_set_name(fnum: ::core::ffi::c_int, name: *mut ::core::ffi::c_char);
    fn buflist_add(
        fname: *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn otherfile(ffname: *mut ::core::ffi::c_char) -> bool;
    fn maketitle();
    fn buf_hide(buf: *const buf_T) -> bool;
    fn buf_is_empty(buf: *mut buf_T) -> bool;
    static mut p_ea: ::core::ffi::c_int;
    static mut p_fic: ::core::ffi::c_int;
    static mut p_tpm: OptInt;
    fn skipwhite(p: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn rem_backslash(str: *const ::core::ffi::c_char) -> bool;
    static e_cmdwin: [::core::ffi::c_char; 0];
    static e_invarg: [::core::ffi::c_char; 0];
    static e_invrange: [::core::ffi::c_char; 0];
    static e_nomatch: [::core::ffi::c_char; 0];
    static e_nomatch2: [::core::ffi::c_char; 0];
    static e_cannot_change_arglist_recursively: [::core::ffi::c_char; 0];
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn semsg(fmt: *const ::core::ffi::c_char, ...) -> bool;
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T;
    fn tv_get_number(tv: *const typval_T) -> varnumber_T;
    fn tv_get_number_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T;
    fn find_win_by_nr_or_id(vp: *mut typval_T) -> *mut win_T;
    fn find_tabwin(wvp: *mut typval_T, tvp: *mut typval_T) -> *mut win_T;
    fn autowrite(buf: *mut buf_T, forceit: bool) -> ::core::ffi::c_int;
    fn check_changed(buf: *mut buf_T, flags: ::core::ffi::c_int) -> bool;
    fn do_ecmd(
        fnum: ::core::ffi::c_int,
        ffname: *mut ::core::ffi::c_char,
        sfname: *mut ::core::ffi::c_char,
        eap: *mut exarg_T,
        newlnum: linenr_T,
        flags: ::core::ffi::c_int,
        oldwin: *mut win_T,
    ) -> ::core::ffi::c_int;
    fn gotocmdline(clr: bool);
    fn file_pat_to_reg_pat(
        pat: *const ::core::ffi::c_char,
        pat_end: *const ::core::ffi::c_char,
        allow_dirs: *mut ::core::ffi::c_char,
        no_bslash: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    static mut Columns: ::core::ffi::c_int;
    static mut firstwin: *mut win_T;
    static mut lastwin: *mut win_T;
    static mut curwin: *mut win_T;
    static mut first_tabpage: *mut tabpage_T;
    static mut curtab: *mut tabpage_T;
    static mut lastused_tabpage: *mut tabpage_T;
    static mut curbuf: *mut buf_T;
    static mut global_alist: alist_T;
    static mut max_alist_id: ::core::ffi::c_int;
    static mut arg_had_last: bool;
    static mut cmdmod: cmdmod_T;
    static mut got_int: bool;
    static mut cmdwin_type: ::core::ffi::c_int;
    fn setmark(c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn setpcmark();
    fn reset_VIsual_and_resel();
    fn magic_isset() -> bool;
    fn os_breakcheck();
    fn path_full_compare(
        s1: *mut ::core::ffi::c_char,
        s2: *mut ::core::ffi::c_char,
        checkname: bool,
        expandenv: bool,
    ) -> FileComparison;
    fn path_fnamecmp(
        fname1: *const ::core::ffi::c_char,
        fname2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn FullName_save(fname: *const ::core::ffi::c_char, force: bool) -> *mut ::core::ffi::c_char;
    fn gen_expand_wildcards(
        num_pat: ::core::ffi::c_int,
        pat: *mut *mut ::core::ffi::c_char,
        num_file: *mut ::core::ffi::c_int,
        file: *mut *mut *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn fix_fname(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn expand_wildcards(
        num_pat: ::core::ffi::c_int,
        pat: *mut *mut ::core::ffi::c_char,
        num_files: *mut ::core::ffi::c_int,
        files: *mut *mut *mut ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
    fn bufIsChanged(buf: *mut buf_T) -> bool;
    fn list_in_columns(
        items: *mut *mut ::core::ffi::c_char,
        size: ::core::ffi::c_int,
        current: ::core::ffi::c_int,
    );
    static mut tabpage_move_disallowed: ::core::ffi::c_int;
    fn check_can_set_curbuf_forceit(forceit: ::core::ffi::c_int) -> bool;
    fn win_split(size: ::core::ffi::c_int, flags: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn win_valid(win: *const win_T) -> bool;
    fn win_move_after(win1: *mut win_T, win2: *mut win_T);
    fn win_close(win: *mut win_T, free_buf: bool, force: bool) -> ::core::ffi::c_int;
    fn valid_tabpage(tpc: *mut tabpage_T) -> bool;
    fn tabpage_index(ftp: *mut tabpage_T) -> ::core::ffi::c_int;
    fn goto_tabpage_tp(
        tp: *mut tabpage_T,
        trigger_enter_autocmds: bool,
        trigger_leave_autocmds: bool,
    );
    fn win_enter(wp: *mut win_T, undo_sync: bool);
    fn lastwin_nofloating(tp: *mut tabpage_T) -> *mut win_T;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aentry_T {
    pub ae_fname: *mut ::core::ffi::c_char,
    pub ae_fnum: ::core::ffi::c_int,
}
pub type hash_T = size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
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
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
pub type QUEUE = queue;
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
pub type ssize_t = isize;
pub type time_t = __time_t;
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
pub type disptick_T = uint64_t;
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
    pub cs_pend: C2Rust_Unnamed_12,
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
pub union C2Rust_Unnamed_12 {
    pub csp_rv: [*mut ::core::ffi::c_void; 50],
    pub csp_ex: [*mut ::core::ffi::c_void; 50],
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
pub const BLN_CURBUF: bln_values = 1;
pub const BLN_LISTED: bln_values = 2;
pub const EW_NOTWILD: C2Rust_Unnamed_16 = 1024;
pub const EW_NOTFOUND: C2Rust_Unnamed_16 = 4;
pub const EW_FILE: C2Rust_Unnamed_16 = 2;
pub const AL_SET: C2Rust_Unnamed_18 = 1;
pub type tabpage_T = tabpage_S;
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
pub type diff_T = diffblock_S;
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
pub const kEqualFiles: file_comparison = 1;
pub type FileComparison = file_comparison;
pub type file_comparison = ::core::ffi::c_uint;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const AL_ADD: C2Rust_Unnamed_18 = 2;
pub const EW_ADDSLASH: C2Rust_Unnamed_16 = 8;
pub const EW_DIR: C2Rust_Unnamed_16 = 1;
pub const AL_DEL: C2Rust_Unnamed_18 = 3;
pub const ECMD_FORCEIT: C2Rust_Unnamed_13 = 8;
pub const ECMD_HIDE: C2Rust_Unnamed_13 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_14 = -1;
pub const CCGD_EXCMD: C2Rust_Unnamed_15 = 16;
pub const CCGD_FORCEIT: C2Rust_Unnamed_15 = 4;
pub const CCGD_MULTWIN: C2Rust_Unnamed_15 = 2;
pub const CCGD_AW: C2Rust_Unnamed_15 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct arg_all_state_T {
    pub alist: *mut alist_T,
    pub had_tab: ::core::ffi::c_int,
    pub keep_tabs: bool,
    pub forceit: bool,
    pub use_firstwin: bool,
    pub opened: *mut uint8_t,
    pub opened_len: ::core::ffi::c_int,
    pub new_curwin: *mut win_T,
    pub new_curtab: *mut tabpage_T,
}
pub const ECMD_OLDBUF: C2Rust_Unnamed_13 = 4;
pub const ECMD_ONE: C2Rust_Unnamed_14 = 1;
pub const WSP_BELOW: C2Rust_Unnamed_17 = 64;
pub const WSP_ROOM: C2Rust_Unnamed_17 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufref_T {
    pub br_buf: *mut buf_T,
    pub br_fnum: ::core::ffi::c_int,
    pub br_buf_free_count: ::core::ffi::c_int,
}
pub type bln_values = ::core::ffi::c_uint;
pub const BLN_NOCURWIN: bln_values = 128;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_NEW: bln_values = 8;
pub const BLN_DUMMY: bln_values = 4;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_13 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_13 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_13 = 16;
pub const ECMD_SET_HELP: C2Rust_Unnamed_13 = 2;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const ECMD_LASTL: C2Rust_Unnamed_14 = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const CCGD_ALLBUF: C2Rust_Unnamed_15 = 8;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_16 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_16 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_16 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_16 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_16 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_16 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_16 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_16 = 2048;
pub const EW_NOERROR: C2Rust_Unnamed_16 = 512;
pub const EW_ICASE: C2Rust_Unnamed_16 = 256;
pub const EW_PATH: C2Rust_Unnamed_16 = 128;
pub const EW_EXEC: C2Rust_Unnamed_16 = 64;
pub const EW_SILENT: C2Rust_Unnamed_16 = 32;
pub const EW_KEEPALL: C2Rust_Unnamed_16 = 16;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const WSP_QUICKFIX: C2Rust_Unnamed_17 = 1024;
pub const WSP_NOENTER: C2Rust_Unnamed_17 = 512;
pub const WSP_NEWLOC: C2Rust_Unnamed_17 = 256;
pub const WSP_ABOVE: C2Rust_Unnamed_17 = 128;
pub const WSP_HELP: C2Rust_Unnamed_17 = 32;
pub const WSP_BOT: C2Rust_Unnamed_17 = 16;
pub const WSP_TOP: C2Rust_Unnamed_17 = 8;
pub const WSP_HOR: C2Rust_Unnamed_17 = 4;
pub const WSP_VERT: C2Rust_Unnamed_17 = 2;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
static mut e_window_layout_changed_unexpectedly: [::core::ffi::c_char; 41] = unsafe {
    ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
        *b"E249: Window layout changed unexpectedly\0",
    )
};
static mut arglist_locked: bool = false_0 != 0;
unsafe extern "C" fn check_arglist_locked() -> ::core::ffi::c_int {
    if arglist_locked {
        emsg(gettext(
            &raw const e_cannot_change_arglist_recursively as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn alist_clear(mut al: *mut alist_T) {
    if check_arglist_locked() == FAIL {
        return;
    }
    let mut _gap: *mut garray_T = &raw mut (*al).al_ga;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut aentry_T = ((*_gap).ga_data as *mut aentry_T).offset(i as isize);
            xfree((*_item).ae_fname as *mut ::core::ffi::c_void);
            i += 1;
        }
    }
    ga_clear(_gap);
}
#[no_mangle]
pub unsafe extern "C" fn alist_init(mut al: *mut alist_T) {
    ga_init(
        &raw mut (*al).al_ga,
        ::core::mem::size_of::<aentry_T>() as ::core::ffi::c_int,
        5 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn alist_unlink(mut al: *mut alist_T) {
    if al != &raw mut global_alist && {
        (*al).al_refcount -= 1;
        (*al).al_refcount <= 0 as ::core::ffi::c_int
    } {
        alist_clear(al);
        xfree(al as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn alist_new() {
    (*curwin).w_alist = xmalloc(::core::mem::size_of::<alist_T>()) as *mut alist_T;
    (*(*curwin).w_alist).al_refcount = 1 as ::core::ffi::c_int;
    max_alist_id += 1;
    (*(*curwin).w_alist).id = max_alist_id;
    alist_init((*curwin).w_alist);
}
#[no_mangle]
pub unsafe extern "C" fn alist_set(
    mut al: *mut alist_T,
    mut count: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
    mut use_curbuf: ::core::ffi::c_int,
    mut fnum_list: *mut ::core::ffi::c_int,
    mut fnum_len: ::core::ffi::c_int,
) {
    if check_arglist_locked() == FAIL {
        return;
    }
    alist_clear(al);
    ga_grow(&raw mut (*al).al_ga, count);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count {
        if got_int {
            while i < count {
                let c2rust_fresh0 = i;
                i = i + 1;
                xfree(*files.offset(c2rust_fresh0 as isize) as *mut ::core::ffi::c_void);
            }
            break;
        } else {
            if !fnum_list.is_null() && i < fnum_len {
                arglist_locked = true_0 != 0;
                buf_set_name(*fnum_list.offset(i as isize), *files.offset(i as isize));
                arglist_locked = false_0 != 0;
            }
            alist_add(
                al,
                *files.offset(i as isize),
                if use_curbuf != 0 {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                },
            );
            os_breakcheck();
            i += 1;
        }
    }
    xfree(files as *mut ::core::ffi::c_void);
    if al == &raw mut global_alist {
        arg_had_last = false_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn alist_add(
    mut al: *mut alist_T,
    mut fname: *mut ::core::ffi::c_char,
    mut set_fnum: ::core::ffi::c_int,
) {
    let mut wp: *mut win_T = curwin;
    if fname.is_null() {
        return;
    }
    if check_arglist_locked() == FAIL {
        return;
    }
    arglist_locked = true_0 != 0;
    (*wp).w_locked = true_0 != 0;
    (*((*al).al_ga.ga_data as *mut aentry_T).offset((*al).al_ga.ga_len as isize)).ae_fname = fname;
    if set_fnum > 0 as ::core::ffi::c_int {
        (*((*al).al_ga.ga_data as *mut aentry_T).offset((*al).al_ga.ga_len as isize)).ae_fnum =
            buflist_add(
                fname,
                BLN_LISTED as ::core::ffi::c_int
                    | (if set_fnum == 2 as ::core::ffi::c_int {
                        BLN_CURBUF as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
            );
    }
    (*al).al_ga.ga_len += 1;
    arglist_locked = false_0 != 0;
    (*wp).w_locked = false_0 != 0;
}
unsafe extern "C" fn do_one_arg(mut str: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut inbacktick: bool = false_0 != 0;
    p = str;
    while *str != 0 {
        if rem_backslash(str) {
            let c2rust_fresh1 = str;
            str = str.offset(1);
            let c2rust_fresh2 = p;
            p = p.offset(1);
            *c2rust_fresh2 = *c2rust_fresh1;
            let c2rust_fresh3 = p;
            p = p.offset(1);
            *c2rust_fresh3 = *str;
        } else {
            if !inbacktick && ascii_isspace(*str as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                break;
            }
            if *str as ::core::ffi::c_int == '`' as ::core::ffi::c_int {
                inbacktick = inbacktick as ::core::ffi::c_int ^ true_0 != 0;
            }
            let c2rust_fresh4 = p;
            p = p.offset(1);
            *c2rust_fresh4 = *str;
        }
        str = str.offset(1);
    }
    str = skipwhite(str);
    *p = NUL as ::core::ffi::c_char;
    return str;
}
unsafe extern "C" fn get_arglist(
    mut gap: *mut garray_T,
    mut str: *mut ::core::ffi::c_char,
    mut escaped: bool,
) {
    ga_init(
        gap,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
    );
    while *str as ::core::ffi::c_int != NUL {
        ga_grow(gap, 1 as ::core::ffi::c_int);
        *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset((*gap).ga_len as isize) = str;
        (*gap).ga_len += 1;
        if !escaped {
            return;
        }
        str = do_one_arg(str);
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_arglist_exp(
    mut str: *mut ::core::ffi::c_char,
    mut fcountp: *mut ::core::ffi::c_int,
    mut fnamesp: *mut *mut *mut ::core::ffi::c_char,
    mut wig: bool,
) -> ::core::ffi::c_int {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut i: ::core::ffi::c_int = 0;
    get_arglist(&raw mut ga, str, true_0 != 0);
    if wig {
        i = expand_wildcards(
            ga.ga_len,
            ga.ga_data as *mut *mut ::core::ffi::c_char,
            fcountp,
            fnamesp,
            EW_FILE as ::core::ffi::c_int
                | EW_NOTFOUND as ::core::ffi::c_int
                | EW_NOTWILD as ::core::ffi::c_int,
        );
    } else {
        i = gen_expand_wildcards(
            ga.ga_len,
            ga.ga_data as *mut *mut ::core::ffi::c_char,
            fcountp,
            fnamesp,
            EW_FILE as ::core::ffi::c_int
                | EW_NOTFOUND as ::core::ffi::c_int
                | EW_NOTWILD as ::core::ffi::c_int,
        );
    }
    ga_clear(&raw mut ga);
    return i;
}
unsafe extern "C" fn alist_check_arg_idx() {
    let mut tp: *mut tabpage_T = first_tabpage as *mut tabpage_T;
    while !tp.is_null() {
        let mut win: *mut win_T = if tp == curtab {
            firstwin
        } else {
            (*tp).tp_firstwin
        };
        while !win.is_null() {
            if (*win).w_alist == (*curwin).w_alist {
                check_arg_idx(win);
            }
            win = (*win).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
}
unsafe extern "C" fn alist_add_list(
    mut count: ::core::ffi::c_int,
    mut files: *mut *mut ::core::ffi::c_char,
    mut after: ::core::ffi::c_int,
    mut will_edit: bool,
) {
    let mut old_argcount: ::core::ffi::c_int = (*(*curwin).w_alist).al_ga.ga_len;
    if check_arglist_locked() != FAIL {
        let mut wp: *mut win_T = curwin;
        ga_grow(&raw mut (*(*wp).w_alist).al_ga, count);
        after = if (if after > 0 as ::core::ffi::c_int {
            after
        } else {
            0 as ::core::ffi::c_int
        }) < (*(*curwin).w_alist).al_ga.ga_len
        {
            if after > 0 as ::core::ffi::c_int {
                after
            } else {
                0 as ::core::ffi::c_int
            }
        } else {
            (*(*curwin).w_alist).al_ga.ga_len
        };
        if after < (*(*curwin).w_alist).al_ga.ga_len {
            memmove(
                ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                    .offset((after + count) as isize) as *mut ::core::ffi::c_void,
                ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset(after as isize)
                    as *const ::core::ffi::c_void,
                (((*(*curwin).w_alist).al_ga.ga_len - after) as size_t)
                    .wrapping_mul(::core::mem::size_of::<aentry_T>()),
            );
        }
        arglist_locked = true_0 != 0;
        (*wp).w_locked = true_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < count {
            let flags: ::core::ffi::c_int = BLN_LISTED as ::core::ffi::c_int
                | (if will_edit as ::core::ffi::c_int != 0 {
                    BLN_CURBUF as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                });
            (*((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset((after + i) as isize))
                .ae_fname = *files.offset(i as isize);
            (*((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset((after + i) as isize))
                .ae_fnum = buflist_add(*files.offset(i as isize), flags);
            i += 1;
        }
        arglist_locked = false_0 != 0;
        (*wp).w_locked = false_0 != 0;
        (*(*wp).w_alist).al_ga.ga_len += count;
        if old_argcount > 0 as ::core::ffi::c_int && (*wp).w_arg_idx >= after {
            (*wp).w_arg_idx += count;
        }
        return;
    }
}
unsafe extern "C" fn arglist_del_files(mut alist_ga: *mut garray_T) {
    let mut regmatch: regmatch_T = regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    regmatch.rm_ic = p_fic != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*alist_ga).ga_len && !got_int {
        let mut p: *mut ::core::ffi::c_char =
            *((*alist_ga).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
        p = file_pat_to_reg_pat(
            p,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0,
        );
        if p.is_null() {
            break;
        }
        regmatch.regprog = vim_regcomp(
            p,
            if magic_isset() as ::core::ffi::c_int != 0 {
                RE_MAGIC
            } else {
                0 as ::core::ffi::c_int
            },
        );
        if regmatch.regprog.is_null() {
            xfree(p as *mut ::core::ffi::c_void);
            break;
        } else {
            let mut didone: bool = false_0 != 0;
            let mut match_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while match_0 < (*(*curwin).w_alist).al_ga.ga_len {
                if vim_regexec(
                    &raw mut regmatch,
                    alist_name(
                        ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                            .offset(match_0 as isize),
                    ),
                    0 as colnr_T,
                ) {
                    didone = true_0 != 0;
                    xfree(
                        (*((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                            .offset(match_0 as isize))
                        .ae_fname as *mut ::core::ffi::c_void,
                    );
                    memmove(
                        ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                            .offset(match_0 as isize)
                            as *mut ::core::ffi::c_void,
                        ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                            .offset(match_0 as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        (((*(*curwin).w_alist).al_ga.ga_len - match_0 - 1 as ::core::ffi::c_int)
                            as size_t)
                            .wrapping_mul(::core::mem::size_of::<aentry_T>()),
                    );
                    (*(*curwin).w_alist).al_ga.ga_len -= 1;
                    if (*curwin).w_arg_idx > match_0 {
                        (*curwin).w_arg_idx -= 1;
                    }
                    match_0 -= 1;
                }
                match_0 += 1;
            }
            vim_regfree(regmatch.regprog);
            xfree(p as *mut ::core::ffi::c_void);
            if !didone {
                semsg(
                    gettext(&raw const e_nomatch2 as *const ::core::ffi::c_char),
                    *((*alist_ga).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                );
            }
            i += 1;
        }
    }
    ga_clear(alist_ga);
}
unsafe extern "C" fn do_arglist(
    mut str: *mut ::core::ffi::c_char,
    mut what: ::core::ffi::c_int,
    mut after: ::core::ffi::c_int,
    mut will_edit: bool,
) -> ::core::ffi::c_int {
    let mut new_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut exp_count: ::core::ffi::c_int = 0;
    let mut exp_files: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut arg_escaped: bool = true_0 != 0;
    if check_arglist_locked() == FAIL {
        return FAIL;
    }
    if what == AL_ADD as ::core::ffi::c_int && *str as ::core::ffi::c_int == NUL {
        if (*curbuf).b_ffname.is_null() {
            return FAIL;
        }
        str = (*curbuf).b_fname;
        arg_escaped = false_0 != 0;
    }
    get_arglist(&raw mut new_ga, str, arg_escaped);
    if what == AL_DEL as ::core::ffi::c_int {
        arglist_del_files(&raw mut new_ga);
    } else {
        let mut i: ::core::ffi::c_int = expand_wildcards(
            new_ga.ga_len,
            new_ga.ga_data as *mut *mut ::core::ffi::c_char,
            &raw mut exp_count,
            &raw mut exp_files,
            EW_DIR as ::core::ffi::c_int
                | EW_FILE as ::core::ffi::c_int
                | EW_ADDSLASH as ::core::ffi::c_int
                | EW_NOTFOUND as ::core::ffi::c_int,
        );
        ga_clear(&raw mut new_ga);
        if i == FAIL || exp_count == 0 as ::core::ffi::c_int {
            emsg(gettext(&raw const e_nomatch as *const ::core::ffi::c_char));
            return FAIL;
        }
        if what == AL_ADD as ::core::ffi::c_int {
            alist_add_list(exp_count, exp_files, after, will_edit);
            xfree(exp_files as *mut ::core::ffi::c_void);
        } else {
            '_c2rust_label: {
                if what == AL_SET as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"what == AL_SET\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/arglist.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        471 as ::core::ffi::c_uint,
                        b"int do_arglist(char *, int, int, _Bool)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            alist_set(
                (*curwin).w_alist,
                exp_count,
                exp_files,
                will_edit as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                0 as ::core::ffi::c_int,
            );
        }
    }
    alist_check_arg_idx();
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn set_arglist(mut str: *mut ::core::ffi::c_char) {
    do_arglist(
        str,
        AL_SET as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        true_0 != 0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn editing_arg_idx(mut win: *mut win_T) -> bool {
    return !((*win).w_arg_idx >= (*(*win).w_alist).al_ga.ga_len
        || (*(*win).w_buffer).handle
            != (*((*(*win).w_alist).al_ga.ga_data as *mut aentry_T)
                .offset((*win).w_arg_idx as isize))
            .ae_fnum
            && ((*(*win).w_buffer).b_ffname.is_null()
                || path_full_compare(
                    alist_name(
                        ((*(*win).w_alist).al_ga.ga_data as *mut aentry_T)
                            .offset((*win).w_arg_idx as isize),
                    ),
                    (*(*win).w_buffer).b_ffname,
                    true_0 != 0,
                    true_0 != 0,
                ) as ::core::ffi::c_uint
                    & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                    == 0));
}
#[no_mangle]
pub unsafe extern "C" fn check_arg_idx(mut win: *mut win_T) {
    if (*(*win).w_alist).al_ga.ga_len > 1 as ::core::ffi::c_int && !editing_arg_idx(win) {
        (*win).w_arg_idx_invalid = true_0;
        if (*win).w_arg_idx != (*(*win).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int
            && arg_had_last as ::core::ffi::c_int == false_0
            && (*win).w_alist == &raw mut global_alist
            && global_alist.al_ga.ga_len > 0 as ::core::ffi::c_int
            && (*win).w_arg_idx < global_alist.al_ga.ga_len
            && ((*(*win).w_buffer).handle
                == (*(global_alist.al_ga.ga_data as *mut aentry_T)
                    .offset((global_alist.al_ga.ga_len - 1 as ::core::ffi::c_int) as isize))
                .ae_fnum
                || !(*(*win).w_buffer).b_ffname.is_null()
                    && path_full_compare(
                        alist_name((global_alist.al_ga.ga_data as *mut aentry_T).offset(
                            (global_alist.al_ga.ga_len - 1 as ::core::ffi::c_int) as isize,
                        )),
                        (*(*win).w_buffer).b_ffname,
                        true_0 != 0,
                        true_0 != 0,
                    ) as ::core::ffi::c_uint
                        & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0)
        {
            arg_had_last = true_0 != 0;
        }
    } else {
        (*win).w_arg_idx_invalid = false_0;
        if (*win).w_arg_idx == (*(*win).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int
            && (*win).w_alist == &raw mut global_alist
        {
            arg_had_last = true_0 != 0;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_args(mut eap: *mut exarg_T) {
    if (*eap).cmdidx as ::core::ffi::c_int != CMD_args as ::core::ffi::c_int {
        if check_arglist_locked() == FAIL {
            return;
        }
        alist_unlink((*curwin).w_alist);
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_argglobal as ::core::ffi::c_int {
            (*curwin).w_alist = &raw mut global_alist;
        } else {
            alist_new();
        }
    }
    if *(*eap).arg as ::core::ffi::c_int != NUL {
        if check_arglist_locked() == FAIL {
            return;
        }
        ex_next(eap);
        return;
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_args as ::core::ffi::c_int {
        if (*(*curwin).w_alist).al_ga.ga_len <= 0 as ::core::ffi::c_int {
            return;
        }
        let mut items: *mut *mut ::core::ffi::c_char = xmalloc(
            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                .wrapping_mul((*(*curwin).w_alist).al_ga.ga_len as size_t),
        ) as *mut *mut ::core::ffi::c_char;
        gotocmdline(true_0 != 0);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*(*curwin).w_alist).al_ga.ga_len {
            *items.offset(i as isize) = alist_name(
                ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset(i as isize),
            );
            i += 1;
        }
        list_in_columns(
            items,
            (*(*curwin).w_alist).al_ga.ga_len,
            (*curwin).w_arg_idx,
        );
        xfree(items as *mut ::core::ffi::c_void);
        return;
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_arglocal as ::core::ffi::c_int {
        let mut gap: *mut garray_T = &raw mut (*(*curwin).w_alist).al_ga;
        ga_grow(gap, global_alist.al_ga.ga_len);
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < global_alist.al_ga.ga_len {
            if !(*(global_alist.al_ga.ga_data as *mut aentry_T).offset(i_0 as isize))
                .ae_fname
                .is_null()
            {
                (*((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                    .offset((*gap).ga_len as isize))
                .ae_fname = xstrdup(
                    (*(global_alist.al_ga.ga_data as *mut aentry_T).offset(i_0 as isize)).ae_fname,
                );
                (*((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                    .offset((*gap).ga_len as isize))
                .ae_fnum =
                    (*(global_alist.al_ga.ga_data as *mut aentry_T).offset(i_0 as isize)).ae_fnum;
                (*gap).ga_len += 1;
            }
            i_0 += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_previous(mut eap: *mut exarg_T) {
    if (*curwin).w_arg_idx - (*eap).line2 as ::core::ffi::c_int >= (*(*curwin).w_alist).al_ga.ga_len
    {
        do_argfile(
            eap,
            (*(*curwin).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int,
        );
    } else {
        do_argfile(
            eap,
            (*curwin).w_arg_idx - (*eap).line2 as ::core::ffi::c_int,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn ex_rewind(mut eap: *mut exarg_T) {
    do_argfile(eap, 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn ex_last(mut eap: *mut exarg_T) {
    do_argfile(
        eap,
        (*(*curwin).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ex_argument(mut eap: *mut exarg_T) {
    let mut i: ::core::ffi::c_int = 0;
    if (*eap).addr_count > 0 as ::core::ffi::c_int {
        i = (*eap).line2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    } else {
        i = (*curwin).w_arg_idx;
    }
    do_argfile(eap, i);
}
#[no_mangle]
pub unsafe extern "C" fn do_argfile(mut eap: *mut exarg_T, mut argn: ::core::ffi::c_int) {
    let mut is_split_cmd: bool = *(*eap).cmd as ::core::ffi::c_int == 's' as ::core::ffi::c_int;
    let mut old_arg_idx: ::core::ffi::c_int = (*curwin).w_arg_idx;
    if argn < 0 as ::core::ffi::c_int || argn >= (*(*curwin).w_alist).al_ga.ga_len {
        if (*(*curwin).w_alist).al_ga.ga_len <= 1 as ::core::ffi::c_int {
            emsg(gettext(
                b"E163: There is only one file to edit\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else if argn < 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E164: Cannot go before first file\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            emsg(gettext(
                b"E165: Cannot go beyond last file\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        return;
    }
    if !is_split_cmd
        && (*((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset(argn as isize)).ae_fnum
            != (*curbuf).handle
        && !check_can_set_curbuf_forceit((*eap).forceit)
    {
        return;
    }
    setpcmark();
    if is_split_cmd as ::core::ffi::c_int != 0 || cmdmod.cmod_tab != 0 as ::core::ffi::c_int {
        if win_split(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) == FAIL {
            return;
        }
        (*curwin).w_onebuf_opt.wo_scb = false_0;
        (*curwin).w_onebuf_opt.wo_crb = false_0;
    } else {
        let mut other: ::core::ffi::c_int = true_0;
        if buf_hide(curbuf) {
            let mut p: *mut ::core::ffi::c_char = fix_fname(alist_name(
                ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset(argn as isize),
            ));
            other = otherfile(p) as ::core::ffi::c_int;
            xfree(p as *mut ::core::ffi::c_void);
        }
        if (!buf_hide(curbuf) || other == 0)
            && check_changed(
                curbuf,
                CCGD_AW as ::core::ffi::c_int
                    | (if other != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        CCGD_MULTWIN as ::core::ffi::c_int
                    })
                    | (if (*eap).forceit != 0 {
                        CCGD_FORCEIT as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
                    | CCGD_EXCMD as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
        {
            return;
        }
    }
    (*curwin).w_arg_idx = argn;
    if argn == (*(*curwin).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int
        && (*curwin).w_alist == &raw mut global_alist
    {
        arg_had_last = true_0 != 0;
    }
    if do_ecmd(
        0 as ::core::ffi::c_int,
        alist_name(
            ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                .offset((*curwin).w_arg_idx as isize),
        ),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        eap,
        ECMD_LAST as ::core::ffi::c_int as linenr_T,
        (if buf_hide((*curwin).w_buffer) as ::core::ffi::c_int != 0 {
            ECMD_HIDE as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) + (if (*eap).forceit != 0 {
            ECMD_FORCEIT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }),
        curwin,
    ) == FAIL
    {
        (*curwin).w_arg_idx = old_arg_idx;
    } else if (*eap).cmdidx as ::core::ffi::c_int != CMD_argdo as ::core::ffi::c_int {
        setmark('\'' as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_next(mut eap: *mut exarg_T) {
    if buf_hide(curbuf) as ::core::ffi::c_int != 0
        || (*eap).cmdidx as ::core::ffi::c_int == CMD_snext as ::core::ffi::c_int
        || !check_changed(
            curbuf,
            CCGD_AW as ::core::ffi::c_int
                | (if (*eap).forceit != 0 {
                    CCGD_FORCEIT as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                | CCGD_EXCMD as ::core::ffi::c_int,
        )
    {
        let mut i: ::core::ffi::c_int = 0;
        if *(*eap).arg as ::core::ffi::c_int != NUL {
            if do_arglist(
                (*eap).arg,
                AL_SET as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true_0 != 0,
            ) == FAIL
            {
                return;
            }
            i = 0 as ::core::ffi::c_int;
        } else {
            i = (*curwin).w_arg_idx + (*eap).line2 as ::core::ffi::c_int;
        }
        do_argfile(eap, i);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_argdedupe(mut _eap: *mut exarg_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*(*curwin).w_alist).al_ga.ga_len {
        let mut firstFullname: *mut ::core::ffi::c_char = FullName_save(
            (*((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset(i as isize)).ae_fname,
            false_0 != 0,
        );
        let mut j: ::core::ffi::c_int = i + 1 as ::core::ffi::c_int;
        while j < (*(*curwin).w_alist).al_ga.ga_len {
            let mut secondFullname: *mut ::core::ffi::c_char = FullName_save(
                (*((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset(j as isize))
                    .ae_fname,
                false_0 != 0,
            );
            let mut areNamesDuplicate: bool =
                path_fnamecmp(firstFullname, secondFullname) == 0 as ::core::ffi::c_int;
            xfree(secondFullname as *mut ::core::ffi::c_void);
            if areNamesDuplicate {
                xfree(
                    (*((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset(j as isize))
                        .ae_fname as *mut ::core::ffi::c_void,
                );
                memmove(
                    ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset(j as isize)
                        as *mut ::core::ffi::c_void,
                    ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                        .offset(j as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    (((*(*curwin).w_alist).al_ga.ga_len - j - 1 as ::core::ffi::c_int) as size_t)
                        .wrapping_mul(::core::mem::size_of::<aentry_T>()),
                );
                (*(*curwin).w_alist).al_ga.ga_len -= 1;
                if (*curwin).w_arg_idx == j {
                    (*curwin).w_arg_idx = i;
                } else if (*curwin).w_arg_idx > j {
                    (*curwin).w_arg_idx -= 1;
                }
                j -= 1;
            }
            j += 1;
        }
        xfree(firstFullname as *mut ::core::ffi::c_void);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_argedit(mut eap: *mut exarg_T) {
    let mut i: ::core::ffi::c_int = if (*eap).addr_count != 0 {
        (*eap).line2 as ::core::ffi::c_int
    } else {
        (*curwin).w_arg_idx + 1 as ::core::ffi::c_int
    };
    let mut curbuf_is_reusable: bool = curbuf_reusable();
    if do_arglist((*eap).arg, AL_ADD as ::core::ffi::c_int, i, true_0 != 0) == FAIL {
        return;
    }
    maketitle();
    if (*curwin).w_arg_idx == 0 as ::core::ffi::c_int
        && (*curbuf).b_ml.ml_flags & ML_EMPTY != 0
        && ((*curbuf).b_ffname.is_null() || curbuf_is_reusable as ::core::ffi::c_int != 0)
    {
        i = 0 as ::core::ffi::c_int;
    }
    if i < (*(*curwin).w_alist).al_ga.ga_len {
        do_argfile(eap, i);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ex_argadd(mut eap: *mut exarg_T) {
    do_arglist(
        (*eap).arg,
        AL_ADD as ::core::ffi::c_int,
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            (*eap).line2 as ::core::ffi::c_int
        } else {
            (*curwin).w_arg_idx + 1 as ::core::ffi::c_int
        },
        false_0 != 0,
    );
    maketitle();
}
#[no_mangle]
pub unsafe extern "C" fn ex_argdelete(mut eap: *mut exarg_T) {
    if check_arglist_locked() == FAIL {
        return;
    }
    if (*eap).addr_count > 0 as ::core::ffi::c_int || *(*eap).arg as ::core::ffi::c_int == NUL {
        if (*eap).addr_count == 0 as ::core::ffi::c_int {
            if (*curwin).w_arg_idx >= (*(*curwin).w_alist).al_ga.ga_len {
                emsg(gettext(
                    b"E610: No argument to delete\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return;
            }
            (*eap).line2 = ((*curwin).w_arg_idx + 1 as ::core::ffi::c_int) as linenr_T;
            (*eap).line1 = (*eap).line2;
        } else if (*eap).line2 > (*(*curwin).w_alist).al_ga.ga_len as linenr_T {
            (*eap).line2 = (*(*curwin).w_alist).al_ga.ga_len as linenr_T;
        }
        let mut n: linenr_T = (*eap).line2 - (*eap).line1 + 1 as linenr_T;
        if *(*eap).arg as ::core::ffi::c_int != NUL {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else if n <= 0 as linenr_T {
            if (*eap).line1 != 1 as linenr_T || (*eap).line2 != 0 as linenr_T {
                emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
            }
        } else {
            let mut i: linenr_T = (*eap).line1;
            while i <= (*eap).line2 {
                xfree(
                    (*((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                        .offset((i - 1 as linenr_T) as isize))
                    .ae_fname as *mut ::core::ffi::c_void,
                );
                i += 1;
            }
            memmove(
                ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T)
                    .offset((*eap).line1 as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize))
                    as *mut ::core::ffi::c_void,
                ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset((*eap).line2 as isize)
                    as *const ::core::ffi::c_void,
                (((*(*curwin).w_alist).al_ga.ga_len as linenr_T - (*eap).line2) as size_t)
                    .wrapping_mul(::core::mem::size_of::<aentry_T>()),
            );
            (*(*curwin).w_alist).al_ga.ga_len -= n as ::core::ffi::c_int;
            if (*curwin).w_arg_idx as linenr_T >= (*eap).line2 {
                (*curwin).w_arg_idx -= n as ::core::ffi::c_int;
            } else if (*curwin).w_arg_idx as linenr_T > (*eap).line1 {
                (*curwin).w_arg_idx = (*eap).line1 as ::core::ffi::c_int;
            }
            if (*(*curwin).w_alist).al_ga.ga_len == 0 as ::core::ffi::c_int {
                (*curwin).w_arg_idx = 0 as ::core::ffi::c_int;
            } else if (*curwin).w_arg_idx >= (*(*curwin).w_alist).al_ga.ga_len {
                (*curwin).w_arg_idx = (*(*curwin).w_alist).al_ga.ga_len - 1 as ::core::ffi::c_int;
            }
        }
    } else {
        do_arglist(
            (*eap).arg,
            AL_DEL as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            false_0 != 0,
        );
    }
    maketitle();
}
#[no_mangle]
pub unsafe extern "C" fn get_arglist_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx >= (*(*curwin).w_alist).al_ga.ga_len {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return alist_name(((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset(idx as isize));
}
#[no_mangle]
pub unsafe extern "C" fn alist_name(mut aep: *mut aentry_T) -> *mut ::core::ffi::c_char {
    let mut bp: *mut buf_T = buflist_findnr((*aep).ae_fnum);
    if bp.is_null() || (*bp).b_fname.is_null() {
        return (*aep).ae_fname;
    }
    return (*bp).b_fname;
}
unsafe extern "C" fn arg_all_close_unused_windows(mut aall: *mut arg_all_state_T) {
    let mut old_curwin: *mut win_T = curwin;
    let mut old_curtab: *mut tabpage_T = curtab;
    if (*aall).had_tab > 0 as ::core::ffi::c_int {
        goto_tabpage_tp(first_tabpage, true_0 != 0, true_0 != 0);
    }
    tabpage_move_disallowed += 1;
    loop {
        let mut wpnext: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut tpnext: *mut tabpage_T = (*curtab).tp_next;
        let mut wp: *mut win_T = if (*lastwin).w_floating as ::core::ffi::c_int != 0 {
            lastwin
        } else {
            firstwin
        };
        while !wp.is_null() {
            let mut i: ::core::ffi::c_int = 0;
            wpnext = if (*wp).w_floating as ::core::ffi::c_int != 0 {
                if (*(*wp).w_prev).w_floating as ::core::ffi::c_int != 0 {
                    (*wp).w_prev
                } else {
                    firstwin
                }
            } else if (*wp).w_next.is_null()
                || (*(*wp).w_next).w_floating as ::core::ffi::c_int != 0
            {
                ::core::ptr::null_mut::<win_T>()
            } else {
                (*wp).w_next
            };
            let mut buf: *mut buf_T = (*wp).w_buffer;
            if (*buf).b_ffname.is_null()
                || !(*aall).keep_tabs
                    && ((*buf).b_nwindows > 1 as ::core::ffi::c_int
                        || (*wp).w_width != Columns
                        || (*wp).w_floating as ::core::ffi::c_int != 0 && !is_aucmd_win(wp))
            {
                i = (*aall).opened_len;
            } else {
                i = 0 as ::core::ffi::c_int;
                while i < (*aall).opened_len {
                    if i < (*(*aall).alist).al_ga.ga_len
                        && ((*((*(*aall).alist).al_ga.ga_data as *mut aentry_T).offset(i as isize))
                            .ae_fnum
                            == (*buf).handle
                            || path_full_compare(
                                alist_name(
                                    ((*(*aall).alist).al_ga.ga_data as *mut aentry_T)
                                        .offset(i as isize),
                                ),
                                (*buf).b_ffname,
                                true_0 != 0,
                                true_0 != 0,
                            ) as ::core::ffi::c_uint
                                & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                                != 0)
                    {
                        let mut weight: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                        if old_curtab == curtab {
                            weight += 1;
                            if old_curwin == wp {
                                weight += 1;
                            }
                        }
                        if weight > *(*aall).opened.offset(i as isize) as ::core::ffi::c_int {
                            *(*aall).opened.offset(i as isize) = weight as uint8_t;
                            if i == 0 as ::core::ffi::c_int {
                                if !(*aall).new_curwin.is_null() {
                                    (*(*aall).new_curwin).w_arg_idx = (*aall).opened_len;
                                }
                                (*aall).new_curwin = wp;
                                (*aall).new_curtab = curtab;
                            }
                        } else if (*aall).keep_tabs {
                            i = (*aall).opened_len;
                        }
                        if (*wp).w_alist != (*aall).alist {
                            alist_unlink((*wp).w_alist);
                            (*wp).w_alist = (*aall).alist;
                            (*(*wp).w_alist).al_refcount += 1;
                        }
                        break;
                    } else {
                        i += 1;
                    }
                }
            }
            (*wp).w_arg_idx = i;
            's_31: {
                if i == (*aall).opened_len && !(*aall).keep_tabs {
                    if buf_hide(buf) as ::core::ffi::c_int != 0
                        || (*aall).forceit as ::core::ffi::c_int != 0
                        || (*buf).b_nwindows > 1 as ::core::ffi::c_int
                        || !bufIsChanged(buf)
                    {
                        if !buf_hide(buf)
                            && (*buf).b_nwindows <= 1 as ::core::ffi::c_int
                            && bufIsChanged(buf) as ::core::ffi::c_int != 0
                        {
                            let mut bufref: bufref_T = bufref_T {
                                br_buf: ::core::ptr::null_mut::<buf_T>(),
                                br_fnum: 0,
                                br_buf_free_count: 0,
                            };
                            set_bufref(&raw mut bufref, buf);
                            autowrite(buf, false_0 != 0);
                            if !win_valid(wp) || !bufref_valid(&raw mut bufref) {
                                wpnext = if (*lastwin).w_floating as ::core::ffi::c_int != 0 {
                                    lastwin
                                } else {
                                    firstwin
                                };
                                break 's_31;
                            }
                        }
                        if firstwin == lastwin
                            && ((*first_tabpage).tp_next.is_null() || (*aall).had_tab == 0)
                        {
                            (*aall).use_firstwin = true_0 != 0;
                        } else {
                            win_close(wp, !buf_hide(buf) && !bufIsChanged(buf), false_0 != 0);
                            if !win_valid(wpnext) {
                                wpnext = if (*lastwin).w_floating as ::core::ffi::c_int != 0 {
                                    lastwin
                                } else {
                                    firstwin
                                };
                            }
                        }
                    }
                }
            }
            wp = wpnext;
        }
        if (*aall).had_tab == 0 as ::core::ffi::c_int || tpnext.is_null() {
            break;
        }
        if !valid_tabpage(tpnext) {
            tpnext = first_tabpage;
        }
        goto_tabpage_tp(tpnext, true_0 != 0, true_0 != 0);
    }
    tabpage_move_disallowed -= 1;
}
unsafe extern "C" fn arg_all_open_windows(
    mut aall: *mut arg_all_state_T,
    mut count: ::core::ffi::c_int,
) {
    let mut tab_drop_empty_window: bool = false_0 != 0;
    if (*aall).keep_tabs as ::core::ffi::c_int != 0
        && buf_is_empty(curbuf) as ::core::ffi::c_int != 0
        && (*curbuf).b_nwindows == 1 as ::core::ffi::c_int
        && (*curbuf).b_ffname.is_null()
        && (*curbuf).b_changed == 0
    {
        (*aall).use_firstwin = true_0 != 0;
        tab_drop_empty_window = true_0 != 0;
    }
    let mut split_ret: ::core::ffi::c_int = OK;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < count && !got_int {
        if (*aall).alist == &raw mut global_alist
            && i == global_alist.al_ga.ga_len - 1 as ::core::ffi::c_int
        {
            arg_had_last = true_0 != 0;
        }
        's_23: {
            if *(*aall).opened.offset(i as isize) as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                if (*curwin).w_arg_idx != i {
                    let mut wp: *mut win_T = if curtab == curtab {
                        firstwin
                    } else {
                        (*curtab).tp_firstwin
                    };
                    while !wp.is_null() {
                        if (*wp).w_arg_idx == i {
                            if (*aall).keep_tabs {
                                (*aall).new_curwin = wp;
                                (*aall).new_curtab = curtab;
                                break;
                            } else {
                                if (*wp).w_floating {
                                    break;
                                }
                                if (*(*wp).w_frame).fr_parent != (*(*curwin).w_frame).fr_parent {
                                    emsg(gettext(
                                        &raw const e_window_layout_changed_unexpectedly
                                            as *const ::core::ffi::c_char,
                                    ));
                                    i = count;
                                    break;
                                } else {
                                    win_move_after(wp, curwin);
                                    break;
                                }
                            }
                        } else {
                            wp = (*wp).w_next;
                        }
                    }
                }
            } else if split_ret == OK {
                if tab_drop_empty_window as ::core::ffi::c_int != 0
                    && i == count - 1 as ::core::ffi::c_int
                {
                    autocmd_no_enter -= 1;
                }
                if !(*aall).use_firstwin {
                    let mut p_ea_save: bool = p_ea != 0;
                    p_ea = true_0;
                    split_ret = win_split(
                        0 as ::core::ffi::c_int,
                        WSP_ROOM as ::core::ffi::c_int | WSP_BELOW as ::core::ffi::c_int,
                    );
                    p_ea = p_ea_save as ::core::ffi::c_int;
                    if split_ret == FAIL {
                        break 's_23;
                    }
                } else {
                    autocmd_no_leave -= 1;
                }
                (*curwin).w_arg_idx = i;
                if i == 0 as ::core::ffi::c_int {
                    (*aall).new_curwin = curwin;
                    (*aall).new_curtab = curtab;
                }
                do_ecmd(
                    0 as ::core::ffi::c_int,
                    alist_name(
                        ((*(*aall).alist).al_ga.ga_data as *mut aentry_T).offset(i as isize),
                    ),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<exarg_T>(),
                    ECMD_ONE as ::core::ffi::c_int as linenr_T,
                    (if buf_hide((*curwin).w_buffer) as ::core::ffi::c_int != 0
                        || bufIsChanged((*curwin).w_buffer) as ::core::ffi::c_int != 0
                    {
                        ECMD_HIDE as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) + ECMD_OLDBUF as ::core::ffi::c_int,
                    curwin,
                );
                if tab_drop_empty_window as ::core::ffi::c_int != 0
                    && i == count - 1 as ::core::ffi::c_int
                {
                    autocmd_no_enter += 1;
                }
                if (*aall).use_firstwin {
                    autocmd_no_leave += 1;
                }
                (*aall).use_firstwin = false_0 != 0;
            }
            os_breakcheck();
            if (*aall).had_tab > 0 as ::core::ffi::c_int
                && tabpage_index(::core::ptr::null_mut::<tabpage_T>()) as OptInt <= p_tpm
            {
                cmdmod.cmod_tab = 9999 as ::core::ffi::c_int;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn do_arg_all(
    mut count: ::core::ffi::c_int,
    mut forceit: ::core::ffi::c_int,
    mut keep_tabs: ::core::ffi::c_int,
) {
    let mut last_curwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut last_curtab: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut prev_arglist_locked: bool = arglist_locked;
    '_c2rust_label: {
        if !firstwin.is_null() {
        } else {
            __assert_fail(
                b"firstwin != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/arglist.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1068 as ::core::ffi::c_uint,
                b"void do_arg_all(int, int, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if cmdwin_type != 0 as ::core::ffi::c_int {
        emsg(gettext(&raw const e_cmdwin as *const ::core::ffi::c_char));
        return;
    }
    if (*(*curwin).w_alist).al_ga.ga_len <= 0 as ::core::ffi::c_int {
        return;
    }
    setpcmark();
    let mut aall: arg_all_state_T = arg_all_state_T {
        alist: ::core::ptr::null_mut::<alist_T>(),
        had_tab: cmdmod.cmod_tab,
        keep_tabs: keep_tabs != 0,
        forceit: forceit != 0,
        use_firstwin: false_0 != 0,
        opened: xcalloc((*(*curwin).w_alist).al_ga.ga_len as size_t, 1 as size_t) as *mut uint8_t,
        opened_len: (*(*curwin).w_alist).al_ga.ga_len,
        new_curwin: ::core::ptr::null_mut::<win_T>(),
        new_curtab: ::core::ptr::null_mut::<tabpage_T>(),
    };
    aall.alist = (*curwin).w_alist;
    (*aall.alist).al_refcount += 1;
    arglist_locked = true_0 != 0;
    let new_lu_tp: *mut tabpage_T = curtab;
    reset_VIsual_and_resel();
    arg_all_close_unused_windows(&raw mut aall);
    if count > aall.opened_len || count <= 0 as ::core::ffi::c_int {
        count = aall.opened_len;
    }
    autocmd_no_enter += 1;
    autocmd_no_leave += 1;
    last_curwin = curwin;
    last_curtab = curtab;
    win_enter(
        lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>()),
        false_0 != 0,
    );
    arg_all_open_windows(&raw mut aall, count);
    alist_unlink(aall.alist);
    arglist_locked = prev_arglist_locked;
    autocmd_no_enter -= 1;
    if last_curtab != aall.new_curtab {
        if valid_tabpage(last_curtab) {
            goto_tabpage_tp(last_curtab, true_0 != 0, true_0 != 0);
        }
        if win_valid(last_curwin) {
            win_enter(last_curwin, false_0 != 0);
        }
    }
    if valid_tabpage(aall.new_curtab) {
        goto_tabpage_tp(aall.new_curtab, true_0 != 0, true_0 != 0);
    }
    if valid_tabpage(new_lu_tp) {
        lastused_tabpage = new_lu_tp;
    }
    if win_valid(aall.new_curwin) {
        win_enter(aall.new_curwin, false_0 != 0);
    }
    autocmd_no_leave -= 1;
    xfree(aall.opened as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ex_all(mut eap: *mut exarg_T) {
    if (*eap).addr_count == 0 as ::core::ffi::c_int {
        (*eap).line2 = 9999 as ::core::ffi::c_int as linenr_T;
    }
    do_arg_all(
        (*eap).line2 as ::core::ffi::c_int,
        (*eap).forceit,
        ((*eap).cmdidx as ::core::ffi::c_int == CMD_drop as ::core::ffi::c_int)
            as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn arg_all() -> *mut ::core::ffi::c_char {
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < (*(*curwin).w_alist).al_ga.ga_len {
            let mut p: *mut ::core::ffi::c_char = alist_name(
                ((*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T).offset(idx as isize),
            );
            if !p.is_null() {
                if len > 0 as ::core::ffi::c_int {
                    if !retval.is_null() {
                        *retval.offset(len as isize) = ' ' as ::core::ffi::c_char;
                    }
                    len += 1;
                }
                while *p as ::core::ffi::c_int != NUL {
                    if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '`' as ::core::ffi::c_int
                    {
                        if !retval.is_null() {
                            *retval.offset(len as isize) = '\\' as ::core::ffi::c_char;
                        }
                        len += 1;
                    }
                    if !retval.is_null() {
                        *retval.offset(len as isize) = *p;
                    }
                    len += 1;
                    p = p.offset(1);
                }
            }
            idx += 1;
        }
        if !retval.is_null() {
            *retval.offset(len as isize) = NUL as ::core::ffi::c_char;
            break;
        } else {
            retval = xmalloc((len as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        }
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn f_argc(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*rettv).vval.v_number = (*(*curwin).w_alist).al_ga.ga_len as varnumber_T;
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) == -1 as varnumber_T
    {
        (*rettv).vval.v_number = global_alist.al_ga.ga_len as varnumber_T;
    } else {
        let mut wp: *mut win_T =
            find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
        if !wp.is_null() {
            (*rettv).vval.v_number = (*(*wp).w_alist).al_ga.ga_len as varnumber_T;
        } else {
            (*rettv).vval.v_number = -1 as varnumber_T;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_argidx(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (*curwin).w_arg_idx as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_arglistid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut wp: *mut win_T = find_tabwin(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        argvars.offset(1 as ::core::ffi::c_int as isize),
    );
    if !wp.is_null() {
        (*rettv).vval.v_number = (*(*wp).w_alist).id as varnumber_T;
    }
}
unsafe extern "C" fn get_arglist_as_rettv(
    mut arglist: *mut aentry_T,
    mut argcount: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
) {
    tv_list_alloc_ret(rettv, argcount as ptrdiff_t);
    if !arglist.is_null() {
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < argcount {
            tv_list_append_string(
                (*rettv).vval.v_list,
                alist_name(arglist.offset(idx as isize)),
                -1 as ssize_t,
            );
            idx += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_argv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut arglist: *mut aentry_T = ::core::ptr::null_mut::<aentry_T>();
    let mut argcount: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        get_arglist_as_rettv(
            (*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T,
            (*(*curwin).w_alist).al_ga.ga_len,
            rettv,
        );
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        arglist = (*(*curwin).w_alist).al_ga.ga_data as *mut aentry_T;
        argcount = (*(*curwin).w_alist).al_ga.ga_len;
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) == -1 as varnumber_T
    {
        arglist = global_alist.al_ga.ga_data as *mut aentry_T;
        argcount = global_alist.al_ga.ga_len;
    } else {
        let mut wp: *mut win_T =
            find_win_by_nr_or_id(argvars.offset(1 as ::core::ffi::c_int as isize));
        if !wp.is_null() {
            arglist = (*(*wp).w_alist).al_ga.ga_data as *mut aentry_T;
            argcount = (*(*wp).w_alist).al_ga.ga_len;
        }
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) as ::core::ffi::c_int;
    if !arglist.is_null() && idx >= 0 as ::core::ffi::c_int && idx < argcount {
        (*rettv).vval.v_string = xstrdup(alist_name(arglist.offset(idx as isize)));
    } else if idx == -1 as ::core::ffi::c_int {
        get_arglist_as_rettv(arglist, argcount, rettv);
    }
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn ascii_isspace(mut c: ::core::ffi::c_int) -> bool {
    return c >= 9 as ::core::ffi::c_int && c <= 13 as ::core::ffi::c_int
        || c == ' ' as ::core::ffi::c_int;
}
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
